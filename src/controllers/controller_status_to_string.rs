use crate::controllers::controller_store::ControllerStatus;

pub fn controller_status_to_string(status: &ControllerStatus) -> String {
    let mut line = format!(
        "{} [{}] · {}% · {}",
        status.name,
        if status.is_bluetooth {
            "Bluetooth"
        } else {
            "USB"
        },
        if status.is_fully_charged {
            100
        } else {
            status.battery_percent
        },
        if status.is_fully_charged {
            "Fully charged"
        } else if status.is_charging {
            "Charging"
        } else {
            "Not charging"
        }
    );

    if status.unexpected_battery_data {
        line.push_str(" (unexpected data)");
    }

    line
}
