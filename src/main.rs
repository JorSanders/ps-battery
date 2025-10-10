use hidapi::HidApi;
use std::{thread, time::Duration};
use windows::UI::Notifications::*;

const SONY_VID: u16 = 0x054C;
const SONY_PIDS: [u16; 2] = [0x0CE6, 0x0DF2];
const BUFFER_SIZE: usize = 64;
const BATTERY_OFFSET: usize = 53;
const BATTERY_MASK: u8 = 0b00001111;
const POLL_INTERVAL_SECS: u64 = 60;

fn show_notification(title: &str, message: &str) {
    let toast_xml =
        ToastNotificationManager::GetTemplateContent(ToastTemplateType::ToastText02).unwrap();
    let nodes = toast_xml
        .GetElementsByTagName(&windows::core::HSTRING::from("text"))
        .unwrap();
    nodes
        .Item(0)
        .unwrap()
        .AppendChild(
            &toast_xml
                .CreateTextNode(&windows::core::HSTRING::from(title))
                .unwrap(),
        )
        .unwrap();
    nodes
        .Item(1)
        .unwrap()
        .AppendChild(
            &toast_xml
                .CreateTextNode(&windows::core::HSTRING::from(message))
                .unwrap(),
        )
        .unwrap();

    let toast = ToastNotification::CreateToastNotification(&toast_xml).unwrap();
    let notifier = ToastNotificationManager::CreateToastNotifierWithId(
        &windows::core::HSTRING::from("ps-battery"),
    )
    .unwrap();
    notifier.Show(&toast).unwrap();
}

fn main() {
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
                    show_notification(
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
