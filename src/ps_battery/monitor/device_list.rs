use hidapi::{DeviceInfo, HidApi};

const SONY_VENDOR_ID: u16 = 0x054C;
const SONY_PRODUCT_IDS: [u16; 4] = [0x0CE6, 0x0DF2, 0x05C4, 0x09CC];

pub struct EnumerateControllersArgs<'a> {
    pub api: &'a mut HidApi,
}

pub fn enumerate_sony_controllers(args: &mut EnumerateControllersArgs) -> Vec<DeviceInfo> {
    if let Err(err) = args.api.refresh_devices() {
        eprintln!("Failed to refresh HID devices: {err}");
    }

    args.api
        .device_list()
        .filter(|d| d.vendor_id() == SONY_VENDOR_ID && SONY_PRODUCT_IDS.contains(&d.product_id()))
        .cloned()
        .collect()
}
