use hidapi::{DeviceInfo, HidApi};

const SONY_VENDOR_ID: u16 = 0x054C;

pub const DUALSENSE_PRODUCT_ID: u16 = 0x0CE6;
pub const DUALSENSE_EDGE_PRODUCT_ID: u16 = 0x0DF2;
pub const DUALSHOCK_GEN_1_PRODUCT_ID: u16 = 0x05C4; // Untested
pub const DUALSHOCK_GEN_2_PRODUCT_ID: u16 = 0x09CC; // Untested

const SONY_PRODUCT_IDS: [u16; 4] = [
    DUALSENSE_PRODUCT_ID,
    DUALSENSE_EDGE_PRODUCT_ID,
    DUALSHOCK_GEN_1_PRODUCT_ID,
    DUALSHOCK_GEN_2_PRODUCT_ID,
];

pub fn get_playstation_controllers(hid_api: &mut HidApi) -> Vec<DeviceInfo> {
    if let Err(err) = hid_api.refresh_devices() {
        eprintln!("Failed to refresh HID devices: {err}");
    }

    hid_api
        .device_list()
        .filter(|d| d.vendor_id() == SONY_VENDOR_ID && SONY_PRODUCT_IDS.contains(&d.product_id()))
        .cloned()
        .collect()
}
