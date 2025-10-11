use crate::{sound::*, toast::*, tray::*};
use hidapi::HidApi;

const SONY_VID: u16 = 0x054C;
const SONY_PIDS: [u16; 2] = [0x0CE6, 0x0DF2];
const USB_BATTERY_OFFSET: usize = 53;
const BT_BATTERY_OFFSET: usize = 54;

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
            _ => continue,
        }

        if offset >= buf.len() {
            continue;
        }

        let raw = buf[offset];

        // Battery percentage (proven logic)
        let (percentage, _) = if is_bt {
            let level = raw & 0x0F;
            let state = (raw & 0xF0) >> 4;
            let percent = match state {
                0x02 => 100,
                _ => (level as u32 * 100 / 0x0A),
            };
            (percent as u8, state == 0x01)
        } else {
            let level = raw & 0x0F;
            (level * 10, (raw & 0x10) != 0)
        };

        // Charging bool (robust logic)
        let charging = if is_bt {
            if buf.len() > 55 && buf[0] == 0x31 {
                let state = buf[55];
                (state & 0x10) != 0
            } else {
                false
            }
        } else {
            let mut feat = [0u8; 64];
            match device.get_feature_report(&mut feat) {
                Ok(n) => {
                    println!("Feature report ({} bytes): {:?}", n, &feat[..n]);
                    (feat[4] & 0x10) != 0 || (feat[5] & 0x10) != 0
                }
                Err(_) => false,
            }
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
