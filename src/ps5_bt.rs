use std::ptr;
use windows::Win32::Devices::Bluetooth::*;
use windows::Win32::Foundation::*;

pub fn list_paired_bluetooth_devices() {
    unsafe {
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
            Err(_) => return,
        };

        let mut device_info = device_info;
        let mut h_find = h_find;

        loop {
            if device_info.fConnected.as_bool() {
                let name = String::from_utf16_lossy(&device_info.szName)
                    .trim_end_matches('\0')
                    .to_string();
                if name.contains("Wireless Controller") || name.contains("PS5 Edge") {
                    println!("Connected PS5 controller: {name}");
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
    }
}
