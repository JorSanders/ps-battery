use crate::ps_battery::controller_store::ControllerStatus;

pub fn controller_status_to_string(status: &ControllerStatus) -> String {
    format!(
        "{} [{}] — {}% — {}",
        status.name,
        status.transport_label,
        status.battery_percent,
        if status.is_charging {
            "Charging"
        } else {
            "Not Charging"
        }
    )
}
