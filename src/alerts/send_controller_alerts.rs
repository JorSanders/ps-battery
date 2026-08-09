use windows::Win32::UI::Shell::NOTIFYICONDATAW;

use crate::alerts::is_balloon_suppressed::is_balloon_suppressed;
use crate::alerts::play_sound::{AlertSound, play_sound};
use crate::controllers::controller_status_to_string::controller_status_to_string;
use crate::controllers::controller_store::{ControllerStatus, get_controllers};
use crate::log_info;
use crate::tray::{BalloonIcon, show_balloon};

const LOW_BATTERY_PERCENT: u8 = 20;
const URGENT_BATTERY_PERCENT: u8 = 10;
const EMPTY_BATTERY_PERCENT: u8 = 0;

fn is_low_on_battery(controller_status: &ControllerStatus) -> bool {
    controller_status.battery_percent <= LOW_BATTERY_PERCENT
        && controller_status.is_bluetooth
        && !controller_status.is_fully_charged
        && !controller_status.is_charging
        // A reading the parser could not make sense of says nothing about the
        // battery, and the level that came with it is as likely to be noise.
        && !controller_status.unexpected_battery_data
}

pub fn send_controller_alerts(tray_icon: &NOTIFYICONDATAW) -> bool {
    // Every balloon replaces the previous one on the same tray icon, so
    // alerting per controller would leave only the last one readable.
    let Some(controller_status) = get_controllers()
        .into_iter()
        .filter(is_low_on_battery)
        .min_by_key(|controller_status| controller_status.battery_percent)
    else {
        return false;
    };

    let (sound, icon) = if controller_status.battery_percent == EMPTY_BATTERY_PERCENT {
        (AlertSound::Critical, BalloonIcon::Error)
    } else if controller_status.battery_percent <= URGENT_BATTERY_PERCENT {
        (AlertSound::Exclamation, BalloonIcon::Warning)
    } else {
        (AlertSound::Notify, BalloonIcon::Info)
    };

    let balloon_suppressed = is_balloon_suppressed();
    log_info!("Sending alert. Balloon suppressed: {}", balloon_suppressed);

    if balloon_suppressed {
        play_sound(sound);
    }

    show_balloon(
        tray_icon,
        &format!(
            "PS controller {}% battery",
            controller_status.battery_percent
        ),
        &controller_status_to_string(&controller_status),
        icon,
    );

    true
}
