use hidapi::HidDevice;

use crate::ps_battery::get_playstation_controllers::{
    DUALSENSE_EDGE_PRODUCT_ID, DUALSENSE_PRODUCT_ID,
};

const DUALSENSE_REPORT_SIZE: usize = 78;
const DUALSHOCK_REPORT_SIZE: usize = 64;

const DUALSENSE_REPORT_FEATURE_ID: u8 = 0x05;
const DUALSHOCK_REPORT_FEATURE_ID: u8 = 0x02;

pub fn send_controller_feature_report(hid_device: &HidDevice, product_id: u16) {
    let (buffer_size, report_feature_id) =
        if product_id == DUALSENSE_PRODUCT_ID || product_id == DUALSENSE_EDGE_PRODUCT_ID {
            (DUALSENSE_REPORT_SIZE, DUALSENSE_REPORT_FEATURE_ID)
        } else {
            (DUALSHOCK_REPORT_SIZE, DUALSHOCK_REPORT_FEATURE_ID)
        };

    let mut report_buffer = vec![0u8; buffer_size];
    report_buffer[0] = report_feature_id;

    match hid_device.get_feature_report(&mut report_buffer) {
        Ok(n) => {
            println!(
                "-> send_controller_feature_report response {} bytes: {:02X?}",
                n,
                &report_buffer[..n]
            );
        }
        Err(err) => {
            eprintln!(" !! Failed to read feature report: '{}'", err);
        }
    }
}
