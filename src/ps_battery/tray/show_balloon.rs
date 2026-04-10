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

pub fn show_balloon(
    notify: &mut NOTIFYICONDATAW,
    title: &str,
    message: &str,
    icon: BalloonIcon,
) {
    let title_utf16: Vec<u16> = title.encode_utf16().chain(Some(0)).collect();
    let msg_utf16: Vec<u16> = message.encode_utf16().chain(Some(0)).collect();

    notify.uFlags |= NIF_INFO;

    let msg_len = msg_utf16.len().min(notify.szInfo.len());
    notify.szInfo[..msg_len].copy_from_slice(&msg_utf16[..msg_len]);
    if msg_len == notify.szInfo.len() {
        notify.szInfo[msg_len - 1] = 0;
    }

    let title_len = title_utf16.len().min(notify.szInfoTitle.len());
    notify.szInfoTitle[..title_len].copy_from_slice(&title_utf16[..title_len]);
    if title_len == notify.szInfoTitle.len() {
        notify.szInfoTitle[title_len - 1] = 0;
    }

    notify.dwInfoFlags = match icon {
        BalloonIcon::Info => NIIF_INFO,
        BalloonIcon::Warning => NIIF_WARNING,
        BalloonIcon::Error => NIIF_ERROR,
    };

    unsafe {
        let res = Shell_NotifyIconW(NIM_MODIFY, notify);
        if res.as_bool() {
            log_info!("Balloon sent. Title: '{}' Message: '{}'", title, message);
        } else {
            log_err!("Shell_NotifyIconW NIM_MODIFY failed");
        }
    }
}
