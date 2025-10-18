use crate::ps_battery::controller_status_to_string::controller_status_to_string;
use crate::ps_battery::controller_store::{ControllerStatus, get_controllers, set_controllers};
use crate::ps_battery::get_controller_info::get_controller_info;
use crate::ps_battery::get_playstation_controllers::get_playstation_controllers;
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
use std::time::{Duration, Instant};
use windows::Win32::UI::Shell::NOTIFYICONDATAW;

pub const ALERT_INTERVAL: Duration = Duration::from_secs(300);

pub fn poll_controllers(
    hid_api: &mut HidApi,
    tray_icon: &mut NOTIFYICONDATAW,
    last_alert_sent: &mut Instant,
) {
    let now = Instant::now();

    let controllers = get_playstation_controllers(hid_api);

    let mut status_list: Vec<ControllerStatus> = Vec::new();
    let previous_controllers = get_controllers();

    println!("-------------------------------\n");

    for controller_info in controllers {
        let parsed_info = get_controller_info(&controller_info);

        println!(
            " -> get_controller_info: name={}, connection_type={}, product_id=0x{:02X}",
            parsed_info.name, parsed_info.connection_type, parsed_info.product_id,
        );

        println!(" -> path='{}'", parsed_info.path);

        let hid_device = match open_device(&OpenDeviceArgs {
            hid_api: &hid_api,
            info: &controller_info,
        }) {
            Some(d) => d,
            None => continue,
        };

        let mut read_args = ReadControllerInputReportArgs {
            hid_device: &hid_device,
            device_name: &parsed_info.name,
            connection_type: parsed_info.connection_type,
            product_id: parsed_info.product_id,
        };

        let buffer = read_controller_input_report(&mut read_args);

        if buffer.len() == 0 || buffer[0] == 0b0 {
            let previous_controller = if let Some(c) = previous_controllers
                .iter()
                .find(|c| c.path == parsed_info.path)
            {
                c
            } else {
                eprintln!(" !! Buffer is empty and device not found in previous results");
                continue;
            };

            if previous_controller.last_read_failed {
                eprintln!(" !! Buffer is empty and last read also failed");
                continue;
            }

            eprintln!(" !! Buffer is empty using last result");

            status_list.push(ControllerStatus {
                name: previous_controller.name.clone(),
                battery_percent: previous_controller.battery_percent,
                is_charging: previous_controller.is_charging,
                connection_type: previous_controller.connection_type,
                path: previous_controller.path.clone(),
                last_read_failed: true,
            });

            continue;
        }

        let (battery_percent, is_charging) =
            parse_battery_and_charging(&ParseBatteryAndChargingArgs {
                buffer: &buffer,
                connection_type: parsed_info.connection_type,
                product_id: parsed_info.product_id,
            });

        status_list.push(ControllerStatus {
            name: parsed_info.name.clone(),
            battery_percent,
            is_charging,
            connection_type: parsed_info.connection_type,
            path: parsed_info.path,
            last_read_failed: false,
        });

        println!();
        println!();
    }

    if status_list.len() == 0 {
        println!();
        println!(" -> No controllers connected");
        println!();
        return;
    }

    // status_list.push(ControllerStatus {
    //     name: "DualSense Edge Wireless Controller".to_string(),
    //     battery_percent: 60,
    //     is_charging: false,
    //     connection_type: crate::ps_battery::get_controller_info::ConnectionType::Bluetooth,
    // });

    // status_list.push(ControllerStatus {
    //     name: "DualSense Wireless Controller".to_string(),
    //     battery_percent: 40,
    //     is_charging: false,
    //     connection_type: crate::ps_battery::get_controller_info::ConnectionType::Bluetooth,
    // });

    // status_list.push(ControllerStatus {
    //     name: "DualSense Edge Wireless Controller".to_string(),
    //     battery_percent: 30,
    //     is_charging: false,
    //     connection_type: crate::ps_battery::get_controller_info::ConnectionType::Bluetooth,
    // });

    // status_list.push(ControllerStatus {
    //     name: "DualSense Wireless Controller".to_string(),
    //     battery_percent: 20,
    //     is_charging: false,
    //     connection_type: crate::ps_battery::get_controller_info::ConnectionType::Bluetooth,
    // });

    // status_list.push(ControllerStatus {
    //     name: "DualSense Wireless Controller".to_string(),
    //     battery_percent: 10,
    //     is_charging: false,
    //     connection_type: crate::ps_battery::get_controller_info::ConnectionType::Bluetooth,
    // });

    if now.duration_since(*last_alert_sent) >= ALERT_INTERVAL {
        let mut alert_sent = false;
        for controller_status in &status_list {
            if controller_status.battery_percent > 30 {
                continue;
            }
            alert_sent = true;

            *last_alert_sent = now;

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
        if !alert_sent {
            println!(" -> No alerts sent");
        }
    }

    set_controllers(status_list);
}
