pub mod autostart;
pub mod balloon;
pub mod menu;
pub mod window;

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIS_HIDDEN, NOTIFYICONDATAW, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{IDI_APPLICATION, LoadIconW};

pub const WM_TRAYICON: u32 = 0x8000 + 1;
const TRAY_ICON_ID: u32 = 100;
const TRAY_TIP_TEXT: &str = "PS Battery Monitor";

pub unsafe fn add_tray_icon(hwnd: HWND) -> NOTIFYICONDATAW {
    let mut notify = NOTIFYICONDATAW::default();
    notify.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    notify.hWnd = hwnd;
    notify.uID = TRAY_ICON_ID;
    notify.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    notify.uCallbackMessage = WM_TRAYICON;
    notify.dwState = NIS_HIDDEN;
    notify.szTip[..TRAY_TIP_TEXT.len()]
        .copy_from_slice(&TRAY_TIP_TEXT.encode_utf16().collect::<Vec<_>>()[..]);
    unsafe {
        notify.hIcon = LoadIconW(None, IDI_APPLICATION).expect("load icon failed");
        let _ = Shell_NotifyIconW(NIM_ADD, &mut notify);
    }
    notify
}

pub use balloon::{ShowBalloonArgs, show_balloon};
pub use window::create_hidden_window;
