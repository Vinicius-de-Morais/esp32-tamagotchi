use trouble_host::prelude::gatt_service;
use trouble_host::prelude::service::HUMAN_INTERFACE_DEVICE;

static DESC: [u8; 67] = [
    5u8, 1u8, 9u8, 6u8, 161u8, 1u8, 5u8, 7u8, 25u8, 224u8, 41u8, 231u8, 21u8, 0u8, 37u8, 1u8, 117u8, 1u8, 149u8, 8u8,
    129u8, 2u8, 21u8, 0u8, 38u8, 255u8, 0u8, 117u8, 8u8, 149u8, 1u8, 129u8, 3u8, 5u8, 8u8, 25u8, 1u8, 41u8, 5u8, 37u8,
    1u8, 117u8, 1u8, 149u8, 5u8, 145u8, 2u8, 149u8, 3u8, 145u8, 3u8, 5u8, 7u8, 25u8, 0u8, 41u8, 221u8, 38u8, 255u8,
    0u8, 117u8, 8u8, 149u8, 6u8, 129u8, 0u8, 192u8,
];

#[gatt_service(uuid = HUMAN_INTERFACE_DEVICE)]
pub(crate) struct HidService {
    #[characteristic(uuid = "2a4a", read, value = [0x01, 0x01, 0x00, 0x03])]
    pub(crate) hid_info: [u8; 4],
    #[characteristic(uuid = "2a4b", read, value = DESC)]
    pub(crate) report_map: [u8; 67],
    #[characteristic(uuid = "2a4c", write_without_response)]
    pub(crate) hid_control_point: u8,
    #[characteristic(uuid = "2a4e", read, write_without_response, value = 1)]
    pub(crate) protocol_mode: u8,
    #[descriptor(uuid = "2908", read, value = [0u8, 1u8])]
    #[characteristic(uuid = "2a4d", read, notify)]
    pub(crate) input_keyboard: [u8; 8],
    #[descriptor(uuid = "2908", read, value = [0u8, 2u8])]
    #[characteristic(uuid = "2a4d", read, write, write_without_response)]
    pub(crate) output_keyboard: [u8; 1],
}