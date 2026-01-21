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
use esp_bootloader_esp_idf::partitions::FlashRegion;
use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_hal::peripherals::TIMG0;
use esp_hal::time::{Duration, Instant};
use esp_hal::timer::timg::TimerGroup;
use esp_radio::ble;
use esp_radio::ble::controller::BleConnector;
use esp_storage::FlashStorage;
use esp32_tamagotchi::factory::factory::Factory;
use esp32_tamagotchi::peripherals::timer::TimerPeripherals;
use esp32_tamagotchi::service::ble::advertise_service::AdvertiseService;
use log::info;
use trouble_host::Address;
use trouble_host::prelude::{BdAddr, EventHandler, ExternalController};
use embassy_executor::Spawner;
use core::cell::RefCell;
use heapless::Deque;
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
const L2CAP_CHANNELS_MAX: usize = 1;


#[esp_rtos::main]
async fn main(_s: embassy_executor::Spawner) {
    // generator version: 1.1.0

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

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
    let controller: ExternalController<BleConnector<'_>, 20> = ExternalController::new(ble);    
    let address = Address::random([0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01]);
    info!("Our address = {:?}", address);

    // Set BLE Config
    let mut resources: trouble_host::HostResources<trouble_host::prelude::DefaultPacketPool,CONNECTIONS_MAX,L2CAP_CHANNELS_MAX>  = trouble_host::HostResources::new();
    let stack = trouble_host::new(controller, &mut resources).set_random_address(address);
    let stack = &stack;

    let trouble_host::Host {
        central, 
        mut runner, 
        mut peripheral,
        ..
    } = stack.build();

    // WAITING CONECTION PART
    let mut advertise_service = AdvertiseService::new("Tamagotchi").await;
    let attribute_table: AttributeTable<'_, CriticalSectionRawMutex, CONNECTIONS_MAX> = AttributeTable::new();
    let mut server = AttributeServer::new(
        attribute_table
    );
    let bond_stored = false;

    let _ = join(runner.run(), async {
        loop {
            info!("Advertising, waiting for connection...");
            let conn = advertise_service.advertise(&mut peripheral, &mut server).await;

            let raw = conn.raw();
            raw.set_bondable(!bond_stored).unwrap();

            info!("Connection established");
        }
    })
    .await;

    // SCANING PART
    // let printer = Printer {
    //     seen: RefCell::new(Deque::new()),
    // };
    // let mut scanner = Scanner::new(central);
    // let _ = join(runner.run_with_handler(&printer), async {
    //     let mut config = ScanConfig::default();
    //     config.active = true;
    //     config.phys = PhySet::M1;
    //     config.interval = embassy_time::Duration::from_secs(1);
    //     config.window = embassy_time::Duration::from_secs(1);
    //     let mut _session = scanner.scan(&config).await.unwrap();
        
        
    //     // Scan forever
    //     loop {
    //         embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    //     }
    // })
    // .await;
}

struct Printer {
    seen: RefCell<Deque<BdAddr, 128>>,
}

impl EventHandler for Printer {
    fn on_adv_reports(&self, mut it: LeAdvReportsIter<'_>) {
        let mut seen = self.seen.borrow_mut();
        while let Some(Ok(report)) = it.next() {
            if seen.iter().find(|b| b.raw() == report.addr.raw()).is_none() {
                info!("discovered: {:?}", report.addr);
                if seen.is_full() {
                    seen.pop_front();
                }
                seen.push_back(report.addr).unwrap();
            }
        }
    }
}