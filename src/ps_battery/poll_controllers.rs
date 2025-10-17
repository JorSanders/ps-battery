use crate::ps_battery::controller_status_to_string::controller_status_to_string;
use crate::ps_battery::controller_store::{ControllerStatus, set_controllers};
use crate::ps_battery::get_controller_info::get_controller_info;
use crate::ps_battery::get_playstation_controllers::{
    GetPlaystationControllerArgs, get_playstation_controllers,
};
use crate::ps_battery::parse_battery_and_charging::{
    ParseBatteryAndChargingArgs, parse_battery_and_charging,
};
use crate::ps_battery::play_sound::{AlertSound, PlaySoundArgs, play_sound};
use crate::ps_battery::read_controller_input_report::{
    OpenDeviceArgs, ReadControllerInputReportArgs, open_device, read_controller_input_report,
};
use crate::ps_battery::tray::show_balloon::BalloonIcon;
use crate::ps_battery::tray::{ShowBalloonArgs, show_balloon};
use hidapi::HidApi;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use windows::Win32::UI::Shell::NOTIFYICONDATAW;

static LAST_ALERT_LOCK: OnceLock<Mutex<Instant>> = OnceLock::new();

const ALERT_INTERVAL: Duration = Duration::from_secs(300);

pub fn poll_controllers(tray_icon: &mut NOTIFYICONDATAW) {
    let now = Instant::now();

    let mut hid_api = HidApi::new().expect("Failed to initialize hidapi");

    let mut enumerate_args = GetPlaystationControllerArgs {
        hid_api: &mut hid_api,
    };
    let controllers = get_playstation_controllers(&mut enumerate_args);

    let last_alert_mutex = LAST_ALERT_LOCK.get_or_init(|| Mutex::new(now - ALERT_INTERVAL));
    let mut last_alert = last_alert_mutex.lock().unwrap();

    let mut status_list: Vec<ControllerStatus> = Vec::new();

    println!("-------------------------------\n");

    for controller_info in controllers {
        let parsed_info = get_controller_info(&controller_info);

        println!(
            "Parsed controller info: name={}, transport_label={}, report_size={}",
            parsed_info.name, parsed_info.transport_label, parsed_info.report_size
        );

        let hid_device = match open_device(&OpenDeviceArgs {
            hid_api: &hid_api,
            info: &controller_info,
        }) {
            Some(d) => d,
            None => continue,
        };

        let mut buffer = vec![0u8; parsed_info.report_size];
        let mut read_args = ReadControllerInputReportArgs {
            hid_device: &hid_device,
            device_name: &parsed_info.name,
            transport_label: parsed_info.transport_label,
            buffer: &mut buffer,
        };

        read_controller_input_report(&mut read_args);

        if buffer[0] == 0b0 {
            eprintln!("Buffer is empty");
            continue;
        }

        let (battery_percent, is_charging) =
            parse_battery_and_charging(&ParseBatteryAndChargingArgs {
                buffer: &buffer,
                transport_label: parsed_info.transport_label,
            });

        status_list.push(ControllerStatus {
            name: parsed_info.name.clone(),
            battery_percent,
            is_charging,
            transport_label: parsed_info.transport_label,
        });

        println!();
        println!();
    }

    if now.duration_since(*last_alert) >= ALERT_INTERVAL {
        for controller_status in &status_list {
            if controller_status.battery_percent < 30 {
                continue;
            }

            let (sound, icon) = if controller_status.battery_percent <= 10 {
                (AlertSound::Critical, BalloonIcon::Error)
            } else if controller_status.battery_percent <= 20 {
                (AlertSound::Exclamation, BalloonIcon::Warning)
            } else {
                (AlertSound::Notify, BalloonIcon::Info)
            };

            play_sound(&PlaySoundArgs { alert: sound });

            let mut show_args = ShowBalloonArgs {
                notify: tray_icon,
                message: &controller_status_to_string(controller_status),
                title: &format!(
                    "PS controller {}% battery",
                    controller_status.battery_percent
                ),
                icon,
            };
            show_balloon(&mut show_args);
        }
        *last_alert = now;
    } else {
        println!();
        println!("No controllers require alerting");
        println!();
    }

    set_controllers(status_list);
}
