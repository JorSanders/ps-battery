use crate::{sound::*, tray::*};
use hidapi::HidApi;
use windows::Win32::UI::Shell::NOTIFYICONDATAW;

const SONY_VID: u16 = 0x054C;
const SONY_PIDS: [u16; 2] = [0x0CE6, 0x0DF2];
const USB_BATTERY_OFFSET: usize = 53; // wired
const BT_BATTERY_OFFSET: usize = 54; // wireless Classic BT

pub fn check_controllers(nid: &mut NOTIFYICONDATAW) {
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

        // Determine battery and charging
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
                    let level = raw & 0x0F; // 0–4
                    let pct = (level as u8) * 20; // map to 0,20,40,60,80,100
                    (pct, false)
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

        if percentage <= 30 {
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
