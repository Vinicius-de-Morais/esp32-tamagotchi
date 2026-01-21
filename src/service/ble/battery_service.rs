use trouble_host::prelude::gatt_service;
use trouble_host::prelude::service::BATTERY;
use trouble_host::prelude::descriptors::{VALID_RANGE, MEASUREMENT_DESCRIPTION};
use trouble_host::prelude::characteristic::BATTERY_LEVEL;


#[gatt_service(uuid = BATTERY)]
struct BatteryService {
    /// Battery Level
    #[descriptor(uuid = VALID_RANGE, read, value = [0, 100])]
    #[descriptor(uuid = MEASUREMENT_DESCRIPTION, name = "hello", read, value = "Battery Level")]
    #[characteristic(uuid = BATTERY_LEVEL, read, notify, value = 10)]
    level: u8,
    #[characteristic(uuid = "408813df-5dd4-1f87-ec11-cdb001100000", write, read, notify)]
    status: bool,
}