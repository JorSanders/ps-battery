use hidapi::HidApi;
use std::{mem::zeroed, thread, time::Duration};
use windows::{
    Data::Xml::Dom::*, UI::Notifications::*, Win32::Foundation::*, Win32::System::Threading::*,
    Win32::UI::WindowsAndMessaging::*, core::*,
};

use windows::{
    Win32::System::Com::*, Win32::System::Variant::*, Win32::UI::Shell::*,
    Win32::UI::WindowsAndMessaging::*, core::*,
};

const SONY_VID: u16 = 0x054C;
const SONY_PIDS: [u16; 2] = [0x0CE6, 0x0DF2];
const BUFFER_SIZE: usize = 64;
const BATTERY_OFFSET: usize = 53;
const BATTERY_MASK: u8 = 0b00001111;
const POLL_INTERVAL_SECS: u64 = 60;

fn show_toast(title: &str, message: &str) {
    let toast_xml =
        ToastNotificationManager::GetTemplateContent(ToastTemplateType::ToastText02).unwrap();
    let nodes = toast_xml
        .GetElementsByTagName(&HSTRING::from("text"))
        .unwrap();
    nodes
        .Item(0)
        .unwrap()
        .AppendChild(&toast_xml.CreateTextNode(&HSTRING::from(title)).unwrap())
        .unwrap();
    nodes
        .Item(1)
        .unwrap()
        .AppendChild(&toast_xml.CreateTextNode(&HSTRING::from(message)).unwrap())
        .unwrap();

    let toast = ToastNotification::CreateToastNotification(&toast_xml).unwrap();
    let notifier =
        ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from("ps-battery")).unwrap();
    notifier.Show(&toast).unwrap();
}

fn show_balloon(title: &str, message: &str) {
    unsafe {
        let mut nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: HWND(std::ptr::null_mut()),
            uID: 1,
            uFlags: NIF_INFO,
            szInfo: {
                let mut buf = [0u16; 256];
                let s: Vec<u16> = message.encode_utf16().collect();
                buf[..s.len()].copy_from_slice(&s);
                buf
            },
            szInfoTitle: {
                let mut buf = [0u16; 64];
                let s: Vec<u16> = title.encode_utf16().collect();
                buf[..s.len()].copy_from_slice(&s);
                buf
            },
            dwInfoFlags: NIIF_INFO,
            ..zeroed()
        };

        let success = Shell_NotifyIconW(NIM_MODIFY, &mut nid);
        if !success.as_bool() {
            eprintln!("Failed to show balloon");
        }
    }
}

fn main() {
    show_balloon("balloon", "message");
    show_toast("toast", "message");

    loop {
        let api = HidApi::new().expect("Failed to create HID API instance");
        let device_list: Vec<_> = api.device_list().collect();

        device_list.iter().for_each(|device| {
            println!(
                "VID: {:04X}, PID: {:04X}, Product: {:?}, Manufacturer: {:?}, Serial: {:?}",
                device.vendor_id(),
                device.product_id(),
                device.product_string(),
                device.manufacturer_string(),
                device.serial_number()
            );
        });

        device_list
            .iter()
            .filter(|d| d.vendor_id() == SONY_VID && SONY_PIDS.contains(&d.product_id()))
            .for_each(|device_info| {
                let device = device_info
                    .open_device(&api)
                    .expect("Failed to open device");
                let mut buf = [0u8; BUFFER_SIZE];

                if device.read_timeout(&mut buf, 200).is_err() {
                    println!(
                        "Failed to read controller PID {:04X}",
                        device_info.product_id()
                    );
                    return;
                }

                let battery = buf[BATTERY_OFFSET] & BATTERY_MASK;
                let percentage = battery * 10;
                println!(
                    "Controller PID {:04X} battery: {}%",
                    device_info.product_id(),
                    percentage
                );

                if battery <= 2 {
                    show_toast(
                        "Controller Battery Low",
                        &format!(
                            "Controller {:04X} battery at {}%",
                            device_info.product_id(),
                            percentage
                        ),
                    );
                    show_balloon(
                        "Controller Battery Low",
                        &format!(
                            "Controller {:04X} battery at {}%",
                            device_info.product_id(),
                            percentage
                        ),
                    );
                }
            });

        thread::sleep(Duration::from_secs(POLL_INTERVAL_SECS));
    }
}
