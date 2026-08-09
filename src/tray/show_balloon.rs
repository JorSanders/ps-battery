use crate::tray::copy_str_to_utf16_buffer::copy_str_to_utf16_buffer;
use crate::{log_err, log_info};
use windows::Win32::UI::Shell::{
    NIF_INFO, NIIF_ERROR, NIIF_INFO, NIIF_WARNING, NIM_MODIFY, NOTIFYICONDATAW, Shell_NotifyIconW,
};

#[derive(Clone, Copy)]
pub enum BalloonIcon {
    Info,
    Warning,
    Error,
}

/// Works on a local copy of the icon data, so the caller's struct keeps only
/// the icon's identity and never accumulates balloon state between calls.
pub fn show_balloon(notify: &NOTIFYICONDATAW, title: &str, message: &str, icon: BalloonIcon) {
    let mut balloon = *notify;
    balloon.uFlags |= NIF_INFO;

    copy_str_to_utf16_buffer(message, &mut balloon.szInfo);
    copy_str_to_utf16_buffer(title, &mut balloon.szInfoTitle);

    balloon.dwInfoFlags = match icon {
        BalloonIcon::Info => NIIF_INFO,
        BalloonIcon::Warning => NIIF_WARNING,
        BalloonIcon::Error => NIIF_ERROR,
    };

    unsafe {
        let result = Shell_NotifyIconW(NIM_MODIFY, &raw const balloon);
        if result.as_bool() {
            log_info!("Balloon sent. Title: '{}' Message: '{}'", title, message);
        } else {
            log_err!(
                "Shell_NotifyIconW NIM_MODIFY failed. Title: '{}' Message: '{}'",
                title,
                message
            );
        }
    }
}
