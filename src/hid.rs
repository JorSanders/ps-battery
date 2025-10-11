use crate::{sound::*, toast::*, tray::*};
use hidapi::HidApi;

const SONY_VID: u16 = 0x054C;
const SONY_PIDS: [u16; 2] = [0x0CE6, 0x0DF2];
const USB_BATTERY_OFFSET: usize = 53;

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
            Ok(_) => continue,
            Err(e) => {
                println!("Failed to read HID report: {:?}", e);
                continue;
            }
        }

        let (percentage, charging) = if is_bt && buf.len() > 56 && buf[0] == 0x31 {
            let level = buf[54].min(10);
            let state = buf[55];
            let pct = match level {
                0 => 0,
                1 => 10,
                2 => 20,
                3 => 30,
                4 => 40,
                5 => 50,
                6 => 60,
                7 => 70,
                8 => 80,
                9 => 90,
                _ => 100,
            };
            let is_charging = (state & 0x10) != 0;
            println!("Parsed from BT report: level={}, state={}", level, state);
            (pct, is_charging)
        } else {
            let raw = buf[USB_BATTERY_OFFSET];
            let level = raw & 0x0F;
            let is_charging = (raw & 0x10) != 0;
            (level * 10, is_charging)
        };

        println!(
            "Battery percentage: {}%, charging: {}",
            percentage, charging
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
