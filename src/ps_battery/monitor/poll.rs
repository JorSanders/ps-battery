use crate::ps_battery::audio::{AlertSound, PlaySoundArgs, play_sound};
use crate::ps_battery::controller::parse::{
    ParseBatteryAndChargingArgs, parse_battery_and_charging,
};
use crate::ps_battery::controller::read::{
    OpenDeviceArgs, ReadReportWithCalibrationArgs, open_device, read_report_with_calibration,
};
use crate::ps_battery::controller::transport::{DetectTransportArgs, detect_transport};
use crate::ps_battery::controller_store::{ControllerStatus, set_controllers};
use crate::ps_battery::log::log_info_with;
use crate::ps_battery::monitor::cache::{HID_API, LAST_ALERT_TIMES, LAST_SEEN_CACHE, should_log};
use crate::ps_battery::monitor::device_list::{
    EnumerateControllersArgs, enumerate_sony_controllers,
};
use crate::ps_battery::tray::balloon::BalloonIcon;
use crate::ps_battery::tray::{ShowBalloonArgs, show_balloon};

use hidapi::HidApi;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use windows::Win32::UI::Shell::NOTIFYICONDATAW;

const ALERT_INTERVAL_SECONDS: u64 = 300;
const LOG_INTERVAL_SECONDS: u64 = 10;

const ALERT_LVL_CRITICAL_MAX: u8 = 10;
const ALERT_LVL_EXCLAMATION_MAX: u8 = 20;
const ALERT_LVL_NOTIFY_MAX: u8 = 30;

pub struct PollControllersArgs<'a> {
    pub tray_icon: &'a mut NOTIFYICONDATAW,
}

pub fn poll_controllers(args: &mut PollControllersArgs) {
    let now = Instant::now();
    let should_log_now = should_log(now, Duration::from_secs(LOG_INTERVAL_SECONDS));
    let alert_interval = Duration::from_secs(ALERT_INTERVAL_SECONDS);

    let hid_api = HID_API.get_or_init(|| {
        let api = HidApi::new().unwrap_or_else(|e| {
            eprintln!("hidapi init failed: {e}");
            HidApi::new().expect("hidapi init retry failed")
        });
        Mutex::new(api)
    });
    let mut api = hid_api.lock().expect("hid api poisoned");

    let mut enumerate_args = EnumerateControllersArgs { api: &mut api };
    let controllers = enumerate_sony_controllers(&mut enumerate_args);

    let alerts = LAST_ALERT_TIMES.get_or_init(|| Mutex::new(HashMap::new()));
    let mut alert_times = alerts.lock().expect("alert map poisoned");

    let cache = LAST_SEEN_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut last_seen = cache.lock().expect("cache map poisoned");

    let mut status_list: Vec<ControllerStatus> = Vec::new();

    for info in controllers {
        let name = info.product_string().unwrap_or("Unknown").to_string();
        let transport = detect_transport(&DetectTransportArgs { info: &info });
        let transport_label = if transport.is_bluetooth {
            "Bluetooth"
        } else {
            "USB"
        };

        let device = match open_device(&OpenDeviceArgs {
            api: &api,
            info: &info,
        }) {
            Some(d) => d,
            None => {
                if let Some((battery, charging)) = last_seen.get(&name).copied() {
                    status_list.push(ControllerStatus {
                        name: name.clone(),
                        battery_percent: battery,
                        is_charging: charging,
                        is_bluetooth: transport.is_bluetooth,
                    });
                }
                continue;
            }
        };

        let mut buffer = vec![0u8; transport.report_size];
        let mut read_args = ReadReportWithCalibrationArgs {
            device: &device,
            device_name: &name,
            is_bluetooth: transport.is_bluetooth,
            buffer: &mut buffer,
            should_log: should_log_now,
        };
        let count = read_report_with_calibration(&mut read_args);
        if count == 0 {
            continue;
        }

        let (battery_percent, is_charging) =
            parse_battery_and_charging(&ParseBatteryAndChargingArgs {
                device: &device,
                buffer: &buffer,
                is_bluetooth: transport.is_bluetooth,
                should_log: should_log_now,
            });

        last_seen.insert(name.clone(), (battery_percent, is_charging));

        status_list.push(ControllerStatus {
            name: name.clone(),
            battery_percent,
            is_charging,
            is_bluetooth: transport.is_bluetooth,
        });

        if !is_charging {
            let due = match alert_times.get(&name) {
                Some(last) => now.duration_since(*last) >= alert_interval,
                None => true,
            };

            if due {
                let alert_level = if battery_percent <= ALERT_LVL_CRITICAL_MAX {
                    Some((AlertSound::Critical, BalloonIcon::Error))
                } else if battery_percent <= ALERT_LVL_EXCLAMATION_MAX {
                    Some((AlertSound::Exclamation, BalloonIcon::Warning))
                } else if battery_percent <= ALERT_LVL_NOTIFY_MAX {
                    Some((AlertSound::Notify, BalloonIcon::Info))
                } else {
                    None
                };

                if let Some((sound_level, balloon_icon)) = alert_level {
                    play_sound(&PlaySoundArgs { alert: sound_level });

                    unsafe {
                        let mut show_args = ShowBalloonArgs {
                            notify: args.tray_icon,
                            title: &format!(
                                "{} [{}] — {}% — {}",
                                name,
                                transport_label,
                                battery_percent,
                                if is_charging {
                                    "Charging"
                                } else {
                                    "Not Charging"
                                }
                            ),
                            message: &format!("Battery at {}%", battery_percent),
                            icon: balloon_icon,
                        };
                        show_balloon(&mut show_args);
                    }

                    alert_times.insert(name.clone(), now);
                }
            }
        }

        if should_log_now {
            log_info_with(
                "Controller",
                format!(
                    "{} [{}] — {}% — {}",
                    name,
                    transport_label,
                    battery_percent,
                    if is_charging {
                        "Charging"
                    } else {
                        "Not Charging"
                    }
                ),
            );
        }
    }

    set_controllers(status_list);
}
