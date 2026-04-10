use crate::{log_err, log_info};
use windows::Win32::Media::Audio::{PlaySoundW, SND_ALIAS, SND_ASYNC};
use windows::core::w;

#[derive(Clone, Copy)]
pub enum AlertSound {
    Notify,
    Exclamation,
    Critical,
}

pub fn play_sound(alert: AlertSound) {
    let (alias, name) = match alert {
        AlertSound::Notify => (w!("SystemNotification"), "SystemNotification"),
        AlertSound::Exclamation => (w!("SystemExclamation"), "SystemExclamation"),
        AlertSound::Critical => (w!("SystemHand"), "SystemHand"),
    };

    let result = unsafe { PlaySoundW(alias, None, SND_ALIAS | SND_ASYNC) };

    if result.as_bool() {
        log_info!("Sound played. Alias: '{}'", name);
    } else {
        log_err!("Failed to play sound alias '{}'", name);
    }
}
