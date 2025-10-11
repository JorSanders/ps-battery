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

        let offset = if is_bt {
            BT_BATTERY_OFFSET
        } else {
            USB_BATTERY_OFFSET
        };
        let buf_size = if is_bt { 78 } else { 64 };

        let device = match device_info.open_device(&api) {
            Ok(d) => d,
            Err(_) => continue,
        };

        let mut buf = vec![0u8; buf_size];
        if device.read_timeout(&mut buf, 500).unwrap_or(0) == 0 {
            continue;
        }

        if offset >= buf.len() {
            continue;
        }

        let raw = buf[offset];

        let percentage = if is_bt {
            let level = raw & 0x0F;
            let state = (raw & 0xF0) >> 4;
            match state {
                0x02 => 100,
                _ => (level as u32 * 100 / 0x0A) as u8,
            }
        } else {
            (raw & 0x0F) * 10
        };

        let charging = if is_bt {
            buf.len() > 55 && buf[0] == 0x31 && (buf[55] & 0x10) != 0
        } else {
            let mut feat = [0u8; 64];
            device
                .get_feature_report(&mut feat)
                .map(|_| (feat[4] & 0x10) != 0 || (feat[5] & 0x10) != 0)
                .unwrap_or(false)
        };

        println!(
            "{}: {}% ({})",
            name,
            percentage,
            if charging { "charging" } else { "not charging" }
        );

        if !charging {
            let alert = if percentage <= 10 {
                Some(AlertSound::Critical)
            } else if percentage <= 20 {
                Some(AlertSound::Exclamation)
            } else if percentage <= 30 {
                Some(AlertSound::Notify)
            } else {
                None
            };

            if let Some(sound) = alert {
                play_sound(sound);

                if false {
                    show_toast(&name, &format!("Battery at {}%", percentage));
                }

                unsafe {
                    show_balloon(nid, &name, &format!("Battery at {}%", percentage));
                }
            }
        }
    }
}
