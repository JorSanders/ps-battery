use crate::ps_battery::controller_store::{ControllerStatus, get_controllers, set_controllers};
use crate::ps_battery::get_controller_info::get_controller_info;
use crate::ps_battery::get_playstation_controllers::get_playstation_controllers;
use crate::ps_battery::parse_battery_and_charging::{
    ParseBatteryAndChargingArgs, parse_battery_and_charging,
};
use crate::ps_battery::read_controller_input_report::{
    OpenDeviceArgs, ReadControllerInputReportArgs, open_device, read_controller_input_report,
};
use hidapi::HidApi;

pub fn poll_controllers(hid_api: &mut HidApi) {
    let controllers = get_playstation_controllers(hid_api);

    let mut status_list: Vec<ControllerStatus> = Vec::new();
    let previous_controllers = get_controllers();

    println!("-------------------------------\n");

    for controller_info in controllers {
        let parsed_info = get_controller_info(&controller_info);

        println!(
            " -> get_controller_info: name={}, is_bluetooth={}, product_id=0x{:02X}",
            parsed_info.name, parsed_info.is_bluetooth, parsed_info.product_id,
        );

        println!(" -> path='{}'", parsed_info.path);

        let hid_device = match open_device(&OpenDeviceArgs {
            hid_api,
            info: &controller_info,
        }) {
            Some(d) => d,
            None => continue,
        };

        let read_args = ReadControllerInputReportArgs {
            hid_device: &hid_device,
            device_name: &parsed_info.name,
            is_bluetooth: parsed_info.is_bluetooth,
            product_id: parsed_info.product_id,
        };

        let buffer = read_controller_input_report(&read_args);

        if buffer.is_empty() || buffer[0] == 0b0 {
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
                is_fully_charged: previous_controller.is_fully_charged,
                is_bluetooth: previous_controller.is_bluetooth,
                path: previous_controller.path.clone(),
                last_read_failed: true,
            });

            continue;
        }

        let battery_and_charging_result =
            parse_battery_and_charging(&ParseBatteryAndChargingArgs {
                buffer: &buffer,
                is_bluetooth: parsed_info.is_bluetooth,
                product_id: parsed_info.product_id,
            });

        status_list.push(ControllerStatus {
            name: parsed_info.name.clone(),
            battery_percent: battery_and_charging_result.battery_percent,
            is_charging: battery_and_charging_result.is_charging,
            is_fully_charged: battery_and_charging_result.is_fully_charged,
            is_bluetooth: parsed_info.is_bluetooth,
            path: parsed_info.path,
            last_read_failed: false,
        });

        println!();
        println!();
    }

    if status_list.is_empty() {
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

    set_controllers(status_list);
}
