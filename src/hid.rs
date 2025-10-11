use crate::{sound::*, toast::*, tray::*};
use hidapi::HidApi;

const SONY_VID: u16 = 0x054C;
const SONY_PIDS: [u16; 2] = [0x0CE6, 0x0DF2];
const BATTERY_OFFSET: usize = 53;

pub fn check_controllers(nid: &mut windows::Win32::UI::Shell::NOTIFYICONDATAW) {
    let api = HidApi::new().expect("Failed to create HID API instance");
    let device_list: Vec<_> = api.device_list().collect();

    for device_info in device_list {
        if device_info.vendor_id() != SONY_VID || !SONY_PIDS.contains(&device_info.product_id()) {
            continue;
        }

        let device = device_info
            .open_device(&api)
            .expect("Failed to open device");
        let mut buf = [0u8; 64];
        if device.read_timeout(&mut buf, 200).is_err() {
            return;
        }

        let battery = buf[BATTERY_OFFSET] & 0b00001111;
        let percentage = battery * 10;

        let name = device_info.product_string().unwrap_or("Unknown");

        println!(
            "Controller {:04X} ({}) battery: {}%",
            device_info.product_id(),
            name,
            percentage
        );

        if battery == 3 {
            play_sound(AlertSound::Notify);
        } else if battery == 2 {
            play_sound(AlertSound::Exclamation);
        } else if battery == 1 {
            play_sound(AlertSound::Critical);
        }

        if battery <= 3 {
            show_toast(
                "Controller Battery Low",
                &format!(
                    "Controller {:04X} battery at {}%",
                    device_info.product_id(),
                    percentage
                ),
            );
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
