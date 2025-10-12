use crate::ps_battery::log::log_info_with;
use hidapi::HidDevice;
use std::thread;
use std::time::Duration;

pub const TRUNCATED_BLUETOOTH_HEADER: u8 = 0x01;
const BLUETOOTH_CALIBRATION_FEATURE_ID: u8 = 0x05;
const BLUETOOTH_CALIBRATION_SETTLE_MS: u64 = 500;
const BLUETOOTH_REPORT_SIZE: usize = 78;

pub struct MaybeSendBluetoothCalibrationArgs<'a> {
    pub device: &'a HidDevice,
    pub device_name: &'a str,
    pub first_byte: u8,
    pub should_log: bool,
}

pub fn maybe_send_bluetooth_calibration(args: &MaybeSendBluetoothCalibrationArgs) {
    if args.first_byte != TRUNCATED_BLUETOOTH_HEADER {
        return;
    }

    if args.should_log {
        log_info_with(
            "Truncated Bluetooth report detected; calibrating",
            args.device_name,
        );
    }

    let mut calibration_buffer = vec![0u8; BLUETOOTH_REPORT_SIZE];
    calibration_buffer[0] = BLUETOOTH_CALIBRATION_FEATURE_ID;

    let calibration_result = args.device.get_feature_report(&mut calibration_buffer);
    if let Err(err) = calibration_result {
        eprintln!("Failed to read feature report: {err}");
    }

    thread::sleep(Duration::from_millis(BLUETOOTH_CALIBRATION_SETTLE_MS));
}
