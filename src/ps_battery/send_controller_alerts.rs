use windows::Win32::UI::Shell::NOTIFYICONDATAW;

use crate::{
    log_info,
    ps_battery::{
        controller_status_to_string::controller_status_to_string,
        controller_store::get_controllers,
        is_dnd_active::is_dnd_active,
        play_sound::{AlertSound, play_sound},
        tray::{BalloonIcon, show_balloon},
    },
};

pub fn send_controller_alerts(tray_icon: &mut NOTIFYICONDATAW) -> u8 {
    let controllers = get_controllers();
    let dnd = is_dnd_active();

    let mut alerts_sent: u8 = 0;
    for controller_status in controllers {
        if controller_status.battery_percent > 30
            || !controller_status.is_bluetooth
            || controller_status.is_fully_charged
            || controller_status.is_charging
        {
            continue;
        }
        alerts_sent += 1;

        let (sound, icon) = if controller_status.battery_percent <= 10 {
            (AlertSound::Critical, BalloonIcon::Error)
        } else if controller_status.battery_percent <= 20 {
            (AlertSound::Exclamation, BalloonIcon::Warning)
        } else {
            (AlertSound::Notify, BalloonIcon::Info)
        };

        log_info!("Sending alert. DND active: {}", dnd);

        if dnd {
            play_sound(sound);
        }

        show_balloon(
            tray_icon,
            &format!("PS controller {}% battery", controller_status.battery_percent),
            &controller_status_to_string(&controller_status),
            icon,
        );
    }

    alerts_sent
}
