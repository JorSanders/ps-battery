use crate::{log_err, log_info};
use hidapi::HidDevice;

use crate::ps_battery::get_playstation_controllers::{
    DUALSENSE_EDGE_PRODUCT_ID, DUALSENSE_PRODUCT_ID, DUALSHOCK_GEN_1_PRODUCT_ID,
    DUALSHOCK_GEN_2_PRODUCT_ID,
};

const DUALSENSE_REPORT_SIZE: usize = 78;
const DUALSHOCK_REPORT_SIZE: usize = 64;

const DUALSENSE_REPORT_FEATURE_ID: u8 = 0x05;
const DUALSHOCK_REPORT_FEATURE_ID: u8 = 0x02;

/// Requesting the calibration feature report has a side effect this app needs:
/// it switches a Bluetooth controller from truncated to full input reports,
/// which are the ones that carry the battery byte.
pub fn request_controller_feature_report(hid_device: &HidDevice, product_id: u16) {
    let (buffer_size, report_feature_id) = match product_id {
        DUALSENSE_PRODUCT_ID | DUALSENSE_EDGE_PRODUCT_ID => {
            (DUALSENSE_REPORT_SIZE, DUALSENSE_REPORT_FEATURE_ID)
        }
        DUALSHOCK_GEN_1_PRODUCT_ID | DUALSHOCK_GEN_2_PRODUCT_ID => {
            (DUALSHOCK_REPORT_SIZE, DUALSHOCK_REPORT_FEATURE_ID)
        }
        _ => {
            log_err!(
                "Product_id 0x{:04X} not known, not requesting a feature report",
                product_id
            );
            return;
        }
    };

    let mut report_buffer = vec![0u8; buffer_size];
    report_buffer[0] = report_feature_id;

    match hid_device.get_feature_report(&mut report_buffer) {
        Ok(bytes_read) => {
            log_info!(
                "request_controller_feature_report response {} bytes: {:02X?}",
                bytes_read,
                &report_buffer[..bytes_read]
            );
        }
        Err(err) => {
            log_err!("Failed to read feature report: {err}");
        }
    }
}
