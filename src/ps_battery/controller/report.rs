use crate::ps_battery::log::log_error_with;
use hidapi::HidDevice;

const BLUETOOTH_REPORT_FEATURE_ID: u8 = 0x05;
const BLUETOOTH_REPORT_SIZE: usize = 78;

pub struct MaybeSendBluetoothCalibrationArgs<'a> {
    pub device: &'a HidDevice,
}

pub fn send_feature_report(args: &MaybeSendBluetoothCalibrationArgs) {
    let mut report_buffer = vec![0u8; BLUETOOTH_REPORT_SIZE];
    report_buffer[0] = BLUETOOTH_REPORT_FEATURE_ID;

    let report_resullt = args.device.get_feature_report(&mut report_buffer);
    if let Err(err) = report_resullt {
        log_error_with("Failed to read feature report", err);
    }
}
