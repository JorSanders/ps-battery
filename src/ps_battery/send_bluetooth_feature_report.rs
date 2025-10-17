use hidapi::HidDevice;

const BLUETOOTH_REPORT_FEATURE_ID: u8 = 0x05;
const BLUETOOTH_REPORT_SIZE: usize = 78;

pub fn send_bluetooth_feature_report(hid_device: &HidDevice) {
    let mut report_buffer = vec![0u8; BLUETOOTH_REPORT_SIZE];
    report_buffer[0] = BLUETOOTH_REPORT_FEATURE_ID;

    let report_resullt = hid_device.get_feature_report(&mut report_buffer);
    if let Err(err) = report_resullt {
        eprintln!("Failed to read feature report: '{}'", err);
    }
}
