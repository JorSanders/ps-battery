use hidapi::HidApi;
use std::{thread, time::Duration};

const SONY_VID: u16 = 0x054C;
const SONY_PIDS: [u16; 2] = [0x0CE6, 0x0DF2];

const BATTERY_INDEX: usize = 53;
const BATTERY_MASK: u8 = 0b00001111;
const MAX_BATTERY: u8 = 10;

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
                let mut buf = [0u8; 64];

                if device.read_timeout(&mut buf, 200).is_ok() {
                    let battery = buf[BATTERY_INDEX] & BATTERY_MASK;
                    let battery_percent = (battery as f32 / MAX_BATTERY as f32) * 100.0;
                    println!(
                        "Controller PID {:04X} battery: {:.0}%",
                        device_info.product_id(),
                        battery_percent
                    );
                } else {
                    println!(
                        "Failed to read controller PID {:04X}",
                        device_info.product_id()
                    );
                }
            });

        thread::sleep(Duration::from_secs(60));
    }
}
