use embassy_sync::blocking_mutex::{CriticalSectionMutex, raw::{CriticalSectionRawMutex, NoopRawMutex}};
use trouble_host::{Controller, advertise, gatt::GattConnection, prelude::{AdStructure, AdvertisementParameters, AttributeServer, DefaultPacketPool, Peripheral, TxPower}};


pub struct AdvertiseService {
    advertise_data: [u8; 31],
    len: usize,
}

impl AdvertiseService {

    pub async fn new(name: &str) -> Self {
        let mut advertise_data = [0u8; 31];
        let len = AdStructure::encode_slice(
            &[
                AdStructure::Flags(trouble_host::prelude::LE_GENERAL_DISCOVERABLE | trouble_host::prelude::BR_EDR_NOT_SUPPORTED),
                AdStructure::ServiceUuids16(&[
                trouble_host::prelude::service::BATTERY.to_le_bytes(),
                trouble_host::prelude::service::HUMAN_INTERFACE_DEVICE.to_le_bytes(),
                ]),
                AdStructure::CompleteLocalName(name.as_bytes()),
            ],
            &mut advertise_data,
        )
        .unwrap();

        Self {
            advertise_data,
            len,
        }
    } 

    pub async fn advertise<'a, 'server, C: Controller>(
        &mut self,
        peripheral: &mut Peripheral<'a, C, DefaultPacketPool>,
        server: &'server mut AttributeServer<'a, CriticalSectionRawMutex, DefaultPacketPool, 1, 1, 1>,

    ) -> GattConnection<'a, 'server, DefaultPacketPool> {

        let adv_params = AdvertisementParameters {
                interval_min: embassy_time::Duration::from_millis(100),
                interval_max: embassy_time::Duration::from_millis(200),
                tx_power: TxPower::ZerodBm, // 0 dBm
                ..Default::default()
            };

        let advertise = peripheral.advertise(
            &adv_params, 
            advertise::Advertisement::ConnectableScannableUndirected {
                adv_data: &self.advertise_data[..self.len],
                scan_data: &[],
            }
        ).await;

        let advertise = match advertise {
            Ok(advertise) => advertise,
            Err(e) => panic!("Failed to start advertising: {:?}", e),
        };

        let conn = advertise.accept().await;

        match conn {
            Ok(conn) => {
                match conn.with_attribute_server(server) {
                    Ok(att_server) => att_server,
                    Err(e) => panic!("Failed to attach attribute server: {:?}", e),
                }
            },
            Err(e) => panic!("Failed to accept connection: {:?}", e),
        }
    }
}