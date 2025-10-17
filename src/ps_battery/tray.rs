pub mod autostart;
pub mod create_hidden_window;
pub mod menu;
pub mod show_balloon;

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIS_HIDDEN, NOTIFYICONDATAW, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{IDI_APPLICATION, LoadIconW};

pub const WM_TRAYICON: u32 = 0x8000 + 1;
const TRAY_ICON_ID: u32 = 100;
const TRAY_TIP_TEXT: &str = "PS Battery";

pub fn add_tray_icon(hwnd: HWND) -> NOTIFYICONDATAW {
    let mut sz_tip = [0u16; 128];
    let tip_utf16 = TRAY_TIP_TEXT.encode_utf16().collect::<Vec<_>>();
    sz_tip[..tip_utf16.len()].copy_from_slice(&tip_utf16);

    let h_icon = unsafe { LoadIconW(None, IDI_APPLICATION).expect("load icon failed") };
    let notify = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ICON_ID,
        uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        uCallbackMessage: WM_TRAYICON,
        dwState: NIS_HIDDEN,
        szTip: sz_tip,
        hIcon: h_icon,
        ..Default::default()
    };

    let res = unsafe { Shell_NotifyIconW(NIM_ADD, &notify) };
    if !res.as_bool() {
        eprintln!(" !! Shell_NotifyIconW failed");
    }

    notify
}

pub use create_hidden_window::create_hidden_window;
pub use show_balloon::{ShowBalloonArgs, show_balloon};
