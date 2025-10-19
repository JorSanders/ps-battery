use windows::Win32::UI::Shell::NOTIFYICONDATAW;

use crate::ps_battery::{
    controller_status_to_string::controller_status_to_string,
    controller_store::get_controllers,
    play_sound::{AlertSound, PlaySoundArgs, play_sound},
    tray::{ShowBalloonArgs, show_balloon, show_balloon::BalloonIcon},
};

pub fn send_controller_alerts(tray_icon: &mut NOTIFYICONDATAW) -> u8 {
    let controllers = get_controllers();

    let mut alerts_sent: u8 = 0;
    for controller_status in controllers {
        if controller_status.battery_percent > 30 {
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

        play_sound(&PlaySoundArgs { alert: sound });

        let mut show_args = ShowBalloonArgs {
            notify: tray_icon,
            message: &controller_status_to_string(&controller_status),
            title: &format!(
                "PS controller {}% battery",
                controller_status.battery_percent
            ),
            icon,
        };
        show_balloon(&mut show_args);
    }

    alerts_sent
}
