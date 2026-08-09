use crate::ps_battery::request_controller_feature_report::request_controller_feature_report;
use crate::{log_err, log_info};
use hidapi::{DeviceInfo, HidApi, HidDevice};
use std::time::Duration;

const HID_REFRESH_TIMEOUT_MS: i32 = 1000;
const TRUNCATED_BLUETOOTH_HEADER: u8 = 0x01;
const INITIAL_BUFFER_SIZE: usize = 100;

/// Reports that arrived before the mode switch stay queued behind it, so the
/// re-read may need to discard a few stale truncated reports first.
const REPORT_MODE_SWITCH_READ_ATTEMPTS: usize = 5;
const REPORT_MODE_SWITCH_DELAY: Duration = Duration::from_millis(500);

pub fn open_device(hid_api: &HidApi, info: &DeviceInfo) -> Option<HidDevice> {
    match info.open_device(hid_api) {
        Ok(device) => Some(device),
        Err(err) => {
            log_err!("Failed to open HID device: {err}");
            None
        }
    }
}

pub fn read_controller_input_report(
    hid_device: &HidDevice,
    device_name: &str,
    is_bluetooth: bool,
    product_id: u16,
) -> Vec<u8> {
    let mut buffer = vec![0u8; INITIAL_BUFFER_SIZE];

    let mut buffer_length = read_input_report(hid_device, &mut buffer);

    if buffer_length == 0 {
        log_info!("No input report available for '{}'", device_name);
    } else if buffer[0] == TRUNCATED_BLUETOOTH_HEADER && is_bluetooth {
        log_info!(
            "Truncated header detected. Sending feature report for controller: '{}'",
            device_name,
        );
        request_controller_feature_report(hid_device, product_id);
        std::thread::sleep(REPORT_MODE_SWITCH_DELAY);

        for _attempt in 0..REPORT_MODE_SWITCH_READ_ATTEMPTS {
            buffer_length = read_input_report(hid_device, &mut buffer);

            if buffer_length == 0 {
                log_err!(
                    "No input report after sending the feature report for '{}'",
                    device_name
                );
                break;
            }
            if buffer[0] != TRUNCATED_BLUETOOTH_HEADER {
                break;
            }
            log_info!("Discarding stale truncated report");
        }
    }

    buffer.truncate(buffer_length);
    buffer
}

fn read_input_report(hid_device: &HidDevice, buffer: &mut [u8]) -> usize {
    let buffer_length = match hid_device.read_timeout(buffer, HID_REFRESH_TIMEOUT_MS) {
        Ok(bytes_read) => bytes_read,
        Err(e) => {
            log_err!("read_controller_input_report error: {e}");
            0
        }
    };

    log_info!(
        "read_controller_input_report response {} bytes: {:02X?}",
        buffer_length,
        &buffer[..buffer_length]
    );

    buffer_length
}
