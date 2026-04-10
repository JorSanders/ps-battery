use crate::{log_err, log_info};
use crate::ps_battery::send_controller_feature_report::send_controller_feature_report;
use hidapi::{DeviceInfo, HidApi, HidDevice};

const HID_REFRESH_TIMEOUT_MS: i32 = 1000;
const TRUNCATED_BLUETOOTH_HEADER: u8 = 0x01;
const INITIAL_BUFFER_SIZE: usize = 100;

pub fn open_device(hid_api: &HidApi, info: &DeviceInfo) -> Option<HidDevice> {
    match info.open_device(hid_api) {
        Ok(d) => {
            if let Err(err) = d.set_blocking_mode(false) {
                log_err!("Failed to set non-blocking mode: {err}");
            }
            Some(d)
        }
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

    let mut buffer_length = match hid_device.read_timeout(&mut buffer, HID_REFRESH_TIMEOUT_MS) {
        Ok(n) => n,
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

    if buffer_length > 0 {
        let report_header = buffer[0];
        if report_header == TRUNCATED_BLUETOOTH_HEADER && is_bluetooth {
            log_info!(
                "Truncated header detected. Sending feature report for controller: '{}'",
                device_name,
            );
            send_controller_feature_report(hid_device, product_id);
            std::thread::sleep(std::time::Duration::from_millis(500));

            buffer_length = match hid_device.read_timeout(&mut buffer, HID_REFRESH_TIMEOUT_MS) {
                Ok(0) => {
                    log_err!("read_controller_input_report timeout");
                    0
                }
                Ok(n) => n,
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
        }
    }

    buffer.truncate(buffer_length);
    buffer
}
