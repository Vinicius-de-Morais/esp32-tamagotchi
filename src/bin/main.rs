#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use embassy_embedded_hal::adapter::BlockingAsync;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use esp_hal::clock::CpuClock;
use esp_radio::ble;
use esp_radio::ble::controller::BleConnector;
use esp_storage::FlashStorage;
use esp32_tamagotchi::factory::factory::Factory;
use esp32_tamagotchi::peripherals::timer::TimerPeripherals;
use esp32_tamagotchi::service::ble::advertise_service::AdvertiseService;
use esp32_tamagotchi::service::ble::gatt_service::{GattService};
use esp32_tamagotchi::service::ble::storage_service::{get_first_bonded};
use log::info;
use trouble_host::Address;
use trouble_host::prelude::{ExternalController};
use trouble_host::prelude::*;
use embassy_futures::join::join;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]

const CONNECTIONS_MAX: usize = 1;
const DESCRIPTORS_MAX: usize = 3;
const L2CAP_CHANNELS_MAX: usize = 4;
const BLE_STACK_RESOURCES_MAX: usize = 20;


#[esp_rtos::main]
async fn main(_spawner: embassy_executor::Spawner) {
    // generator version: 1.1.0

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

    // Init Timer
    let timer_peripherals = TimerPeripherals::new(peripherals.TIMG0);
    let timg0 = Factory::create_timer_group0(timer_peripherals);
    esp_rtos::start(timg0.timer0);

    // Init RNG
    let _trng_source = esp_hal::rng::TrngSource::new(peripherals.RNG, peripherals.ADC1);
    let mut trng = esp_hal::rng::Trng::try_new().unwrap();

    // Init Flash and Storage
    let flash = BlockingAsync::new(FlashStorage::new(peripherals.FLASH));
    let mut storage = esp32_tamagotchi::service::ble::storage_service::init_storage(flash);
    
    // Init BLE
    let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");
    let device_ble = peripherals.BT;
    let ble_config = ble::Config::default();
    let ble = match BleConnector::new(&radio_init, device_ble, ble_config) {
        Ok(ble) => ble,
        Err(e) => panic!("Failed to initialize BLE: {:?}", e),
    };
    let controller: ExternalController<BleConnector<'_>, BLE_STACK_RESOURCES_MAX> = ExternalController::new(ble);    
    let address = Address::random([0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01]);
    info!("Our address = {:?}", address);

    info!("Set BLE Config");
    let mut resources: trouble_host::HostResources<DefaultPacketPool,CONNECTIONS_MAX,L2CAP_CHANNELS_MAX>  = trouble_host::HostResources::new();
    let stack = trouble_host::new(controller, &mut resources).set_random_address(address)
    .set_random_generator_seed(&mut trng);
    //let stack = &stack;

    info!("Loading bonded devices from storage");
    let mut bond_stored = false;
    let bond = get_first_bonded(&mut storage).await;
    match &bond {
        Ok(Some(bond2)) => {
            info!("Found bonded device: {:?}", bond2.identity.bd_addr);
            let _ = stack.add_bond_information(bond2.clone());
            bond_stored = true;
        }
        Ok(None) => {
            info!("No bonded devices found in storage");
        }
        Err(e) => {
            info!("Error retrieving bonded devices: {:?}. Continuing without bonds.", e);
        }
    }


    info!("Init Host");
    // let trouble_host::Host {
    //     mut runner, 
    //     mut peripheral,
    //     ..
    // } = stack.build();
    let host = stack.build();
    
    info!("Init peripheral");
    let mut peripheral = host.peripheral;
    info!("Init runner");
    let mut runner = host.runner;
    

    info!("Starting advertising loop...");
    let _ = join(runner.run(), async {
        loop {
            let mut advertise_service = AdvertiseService::new("Tamagotchi").await;
            let attribute_table: AttributeTable<'_, CriticalSectionRawMutex, L2CAP_CHANNELS_MAX> = AttributeTable::new();
            let mut server = AttributeServer::new(
                attribute_table
            );
            

            info!("Advertising, waiting for connection...");
            let conn = 
                advertise_service.advertise::
                    <
                        ExternalController<_, BLE_STACK_RESOURCES_MAX>, 
                        L2CAP_CHANNELS_MAX, 
                        DESCRIPTORS_MAX, 
                        CONNECTIONS_MAX
                    >
                (&mut peripheral, &mut server)
                .await;

            let raw: &Connection<'_, DefaultPacketPool> = conn.raw();
            raw.set_bondable(!bond_stored).unwrap();

            let gatt_service = GattService::new();
            let gatt_task = gatt_service.handle_gatt_events(&mut storage, &conn, &mut bond_stored);
            
            // Keep connection alive without needing stack reference
            let keep_alive_task = esp32_tamagotchi::service::ble::advertise_service::keep_connection_alive(&conn, &stack);

            embassy_futures::select::select(gatt_task, keep_alive_task).await;

            info!("Connection dropped, restarting advertising...");
        }
    })
    .await;
}