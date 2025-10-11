use std::ptr;
use windows::Win32::Devices::Bluetooth::*;
use windows::Win32::Foundation::*;

use btleplug::api::{Central, Manager as _, Manager, Peripheral as _, ScanFilter};
use btleplug::platform::Manager as PlatformManager;
use futures::stream::StreamExt;
use tokio::time::{Duration, sleep};

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

        // --- BLE: Try reading battery levels ---
        if controllers.is_empty() {
            println!("No PS5 controllers detected via Classic Bluetooth.");
            return;
        }

        let controllers_clone = controllers.clone();
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
                let manager = PlatformManager::new().await.unwrap();
                let adapters = manager.adapters().await.unwrap();

                if adapters.is_empty() {
                    println!("No BLE adapters found!");
                    return;
                }

                let central = adapters.into_iter().nth(0).unwrap();
                central.start_scan(ScanFilter::default()).await.unwrap();
                println!("Scanning for BLE devices for 5 seconds...");
                sleep(Duration::from_secs(5)).await;

                let peripherals = central.peripherals().await.unwrap();
                println!("Found {} BLE peripherals", peripherals.len());

                for p in peripherals {
                    let properties = match p.properties().await {
                        Ok(Some(props)) => props,
                        _ => continue,
                    };

                    let name = properties
                        .local_name
                        .unwrap_or_else(|| "<unknown>".to_string());
                    let connected = p.is_connected().await.unwrap_or(false);

                    if connected && controllers_clone.iter().any(|c| name.contains(c)) {
                        println!("BLE: Connected PS5 controller detected: {name}");

                        p.discover_services().await.unwrap();
                        for service in p.services() {
                            for characteristic in &service.characteristics {
                                if characteristic.uuid.to_string()
                                    == "00002a19-0000-1000-8000-00805f9b34fb"
                                {
                                    let battery = match p.read(characteristic).await {
                                        Ok(data) => data,
                                        Err(_) => continue,
                                    };
                                    println!("Battery: {}%", battery[0]);
                                }
                            }
                        }
                    }
                }
            });
    }
}
