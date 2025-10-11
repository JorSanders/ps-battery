use hidapi::HidApi;
use std::ptr;
use windows::Win32::Devices::Bluetooth::*;
use windows::Win32::Foundation::*;

const SONY_VID: u16 = 0x054C;
const SONY_PIDS: [u16; 2] = [0x0CE6, 0x0DF2];

const BUFFER_SIZE_USB: usize = 64;
const BUFFER_SIZE_BT: usize = 78;

const OFFSET_USB: usize = 53;
const OFFSET_BT: usize = 54;

const MASK_POWER_LEVEL: u8 = 0x0F;
const MASK_POWER_STATE: u8 = 0xF0;

const STATE_DISCHARGING: u8 = 0x00;
const STATE_CHARGING: u8 = 0x01;
const STATE_COMPLETE: u8 = 0x02;

const MAX_POWER_LEVEL: u8 = 0x0A;

const BT_REPORT_TRUNCATED: u8 = 0x01;
const CALIBRATION_FR: u8 = 0x05;

pub fn list_connected_ps5_controllers() {
    unsafe {
        // --- Classic Bluetooth: Detect connected controllers ---
        let mut search_params = BLUETOOTH_DEVICE_SEARCH_PARAMS {
            dwSize: std::mem::size_of::<BLUETOOTH_DEVICE_SEARCH_PARAMS>() as u32,
            fReturnAuthenticated: TRUE,
            fReturnRemembered: TRUE,
            fReturnUnknown: FALSE,
            fReturnConnected: TRUE,
            fIssueInquiry: FALSE,
            cTimeoutMultiplier: 0,
            hRadio: HANDLE(ptr::null_mut()),
        };

        let mut device_info = BLUETOOTH_DEVICE_INFO {
            dwSize: std::mem::size_of::<BLUETOOTH_DEVICE_INFO>() as u32,
            ..Default::default()
        };

        let h_find = match BluetoothFindFirstDevice(&search_params, &mut device_info) {
            Ok(h) => h,
            Err(_) => {
                println!("No classic Bluetooth devices found.");
                return;
            }
        };

        let mut device_info = device_info;
        let mut h_find = h_find;
        let mut controllers = Vec::new();

        loop {
            if device_info.fConnected.as_bool() {
                let name = String::from_utf16_lossy(&device_info.szName)
                    .trim_end_matches('\0')
                    .to_string();
                if name.contains("Wireless Controller") || name.contains("PS5 Edge") {
                    println!("Classic BT: Connected PS5 controller: {name}");
                    controllers.push(name);
                }
            }

            let mut next_device = BLUETOOTH_DEVICE_INFO {
                dwSize: std::mem::size_of::<BLUETOOTH_DEVICE_INFO>() as u32,
                ..Default::default()
            };

            if BluetoothFindNextDevice(h_find, &mut next_device).is_ok() {
                device_info = next_device;
            } else {
                break;
            }
        }

        let _ = BluetoothFindDeviceClose(h_find);

        if controllers.is_empty() {
            println!("No PS5 controllers detected via Classic Bluetooth.");
            return;
        }

        // --- Read HID battery levels (USB or Bluetooth) ---
        let api = HidApi::new().expect("Failed to create HID API instance");

        for device_info in api.device_list() {
            if device_info.vendor_id() != SONY_VID || !SONY_PIDS.contains(&device_info.product_id())
            {
                continue;
            }

            let device = match device_info.open_device(&api) {
                Ok(d) => d,
                Err(_) => continue,
            };

            let path_str = device_info.path().to_string_lossy();
            let bus_type = if path_str.contains("BTH") {
                "BT"
            } else {
                "USB"
            };
            let (offset, buf_size) = if bus_type == "BT" {
                (OFFSET_BT, BUFFER_SIZE_BT)
            } else {
                (OFFSET_USB, BUFFER_SIZE_USB)
            };

            let mut buf = vec![0u8; buf_size];
            if device.read_timeout(&mut buf, 200).is_err() {
                println!(
                    "Failed to read controller PID {:04X}",
                    device_info.product_id()
                );
                continue;
            }

            // BT calibration (Go logic)
            if bus_type == "BT" && buf[0] == BT_REPORT_TRUNCATED {
                let mut calib_buf = vec![0u8; buf_size];
                calib_buf[0] = CALIBRATION_FR;
                let _ = device.send_feature_report(&calib_buf);
                continue;
            }

            let mut battery = buf[offset] & MASK_POWER_LEVEL;
            let state = (buf[offset] & MASK_POWER_STATE) >> 4;

            if state == STATE_COMPLETE {
                battery = MAX_POWER_LEVEL;
            }

            let percentage = battery * 10;
            println!(
                "Controller PID {:04X} ({bus_type}) battery: {}%, state: {:#X}",
                device_info.product_id(),
                percentage,
                state
            );
        }
    }
}
