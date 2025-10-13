use crate::ps_battery::log::{log_error_with, log_info_with};
use hidapi::HidDevice;
use std::thread;
use std::time::Duration;

pub const TRUNCATED_BLUETOOTH_HEADER: u8 = 0x01;
const BLUETOOTH_REPORT_FEATURE_ID: u8 = 0x05;
const BLUETOOTH_REPORT_SETTLE_MS: u64 = 500;
const BLUETOOTH_REPORT_SIZE: usize = 78;

pub struct MaybeSendBluetoothCalibrationArgs<'a> {
    pub device: &'a HidDevice,
    pub device_name: &'a str,
    pub first_byte: u8,
    pub should_log: bool,
}

pub fn maybe_send_feature_report(args: &MaybeSendBluetoothCalibrationArgs) {
    if args.first_byte != TRUNCATED_BLUETOOTH_HEADER {
        return;
    }

    if args.should_log {
        log_info_with(
            "Truncated Bluetooth report detected; calibrating",
            args.device_name,
        );
    }

    let mut report_buffer = vec![0u8; BLUETOOTH_REPORT_SIZE];
    report_buffer[0] = BLUETOOTH_REPORT_FEATURE_ID;

    let report_resullt = args.device.get_feature_report(&mut report_buffer);
    if let Err(err) = report_resullt {
        log_error_with("Failed to read feature report", err);
    }

    thread::sleep(Duration::from_millis(BLUETOOTH_REPORT_SETTLE_MS));
}
