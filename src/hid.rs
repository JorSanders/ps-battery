use crate::{sound::*, toast::*, tray::*};
use hidapi::HidApi;

const SONY_VID: u16 = 0x054C;
const SONY_PIDS: [u16; 2] = [0x0CE6, 0x0DF2];
const USB_BATTERY_OFFSET: usize = 53; // wired
const BT_BATTERY_OFFSET: usize = 54; // wireless Classic BT

pub fn check_controllers(nid: &mut windows::Win32::UI::Shell::NOTIFYICONDATAW) {
    let api = HidApi::new().expect("Failed to create HID API instance");

    for device_info in api.device_list() {
        if device_info.vendor_id() != SONY_VID || !SONY_PIDS.contains(&device_info.product_id()) {
            continue;
        }

        let name = device_info.product_string().unwrap_or("Unknown");
        let path_str = device_info.path().to_string_lossy();
        let is_bt = path_str
            .to_ascii_uppercase()
            .contains("00001124-0000-1000-8000-00805F9B34FB");

        println!("--- Found device ---");
        println!("Name: {}", name);
        println!("PID: {:04X}", device_info.product_id());
        println!("Path: {}", path_str);
        println!("Detected as Bluetooth? {}", is_bt);

        let offset = if is_bt {
            BT_BATTERY_OFFSET
        } else {
            USB_BATTERY_OFFSET
        };
        let buf_size = if is_bt { 78 } else { 64 };

        let device = match device_info.open_device(&api) {
            Ok(d) => d,
            Err(e) => {
                println!("Failed to open device: {:?}", e);
                continue;
            }
        };

        let mut buf = vec![0u8; buf_size];
        match device.read_timeout(&mut buf, 500) {
            Ok(n) if n > 0 => {
                println!("Read {} bytes from device", n);
                println!("Full report: {:?}", &buf[..n]);
            }
            Ok(_) => {
                println!("Read 0 bytes from device");
                continue;
            }
            Err(e) => {
                println!("Failed to read HID report: {:?}", e);
                continue;
            }
        }

        if offset >= buf.len() {
            println!(
                "Offset {} out of bounds for report length {}",
                offset,
                buf.len()
            );
            continue;
        }

        let raw = buf[offset];

        // Go logic: extract power level and state
        let (percentage, charging) = if is_bt {
            match device_info.product_id() {
                0x0CE6 => {
                    // Regular DualSense
                    let level = raw & 0x0F;
                    let state = (raw & 0xF0) >> 4;
                    let pct = match state {
                        0x02 => 100,             // Full
                        _ => (level as u8 * 10), // 0–10 scale → 0–100
                    };
                    let is_charging = state == 0x01; // 1 = charging
                    (pct, is_charging)
                }
                0x0DF2 => {
                    // DualSense Edge (Bluetooth)
                    let level = raw & 0x0F;
                    let state = (raw & 0xF0) >> 4;
                    let percent = match state {
                        0x02 => 100,                      // Complete
                        _ => (level as u32 * 100 / 0x0A), // scale 0–0x0A to 0–100%
                    };
                    let is_charging = state == 0x01;
                    (percent as u8, is_charging)
                }
                _ => (0, false),
            }
        } else {
            // Wired USB
            let level = raw & 0x0F;
            let is_charging = (raw & 0x10) != 0;
            (level * 10, is_charging)
        };

        println!(
            "Battery percentage: {}%, charging: {} (raw byte: {})",
            percentage, charging, raw
        );

        if percentage <= 30 && !charging && false {
            play_sound(AlertSound::Notify);
            unsafe {
                show_balloon(
                    nid,
                    "Controller Battery Low",
                    &format!(
                        "Controller {:04X} battery at {}%",
                        device_info.product_id(),
                        percentage
                    ),
                );
            }
        }
    }
}

//Full report: [1, 128, 128, 129, 135, 8, 0, 36, 0, 0, 0, 0, 0, 12, 189, 176, 45, 0, 0, 253, 255, 253, 255, 246, 255, 88, 31, 203, 4, 248, 13, 39, 85, 16, 128, 0, 0, 0, 128, 0, 0, 0, 0, 9, 9, 0, 0, 0, 0, 0, 164, 23, 39, 85, 7, 0, 0, 6, 82, 31, 5, 148, 93, 126, 30, 0, 99, 0, 0, 0, 0, 0, 0, 0, 102, 192, 63, 198]
//Full report: [1, 128, 128, 129, 135, 8, 0, 0, 0, 0, 0, 0, 0, 12, 189, 176, 45, 0, 0, 253, 255, 253, 255, 246, 255, 88, 31, 203, 4, 248, 13, 39, 85, 16, 128, 0, 0, 0, 128, 0, 0, 0, 0, 9, 9, 0, 0, 0, 0, 0, 164, 23, 39, 85, 7, 0, 0, 6, 82, 31, 5, 148, 93, 126, 30, 0, 99, 0, 0, 0, 0, 0, 0, 0, 102, 192, 63, 198]
