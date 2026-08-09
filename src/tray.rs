use crate::log_err;
use crate::show_error_message_box::show_error_message_box;
use crate::tray::copy_str_to_utf16_buffer::copy_str_to_utf16_buffer;

pub mod copy_str_to_utf16_buffer;
pub mod create_hidden_window;
pub mod menu;
pub mod show_balloon;

use windows::Win32::Foundation::HWND;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NOTIFYICONDATAW, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{HICON, IDI_APPLICATION, LoadIconW};
use windows::core::PCWSTR;

pub const WM_TRAYICON: u32 = 0x8000 + 1;
pub const TRAY_ICON_ID: u32 = 100;
const TRAY_TIP_TEXT: &str = concat!("PS Battery: v", env!("CARGO_PKG_VERSION"));

/// Resource id `build.rs` embeds `Assets/app.ico` under.
const APP_ICON_RESOURCE_ID: u16 = 1;

/// Falls back to the stock Windows icon if the embedded one can't be loaded,
/// since a missing icon isn't worth refusing to start over.
fn load_app_icon() -> Option<HICON> {
    let module = match unsafe { GetModuleHandleW(PCWSTR::null()) } {
        Ok(module) => module,
        Err(e) => {
            log_err!("GetModuleHandleW failed: {e}");
            return None;
        }
    };
    match unsafe {
        LoadIconW(
            Some(module.into()),
            PCWSTR(APP_ICON_RESOURCE_ID as *const u16),
        )
    } {
        Ok(icon) => Some(icon),
        Err(e) => {
            log_err!("Loading the embedded app icon failed: {e}");
            None
        }
    }
}

/// Returns `None` when the icon could not be added, so the caller decides
/// whether that is fatal (startup) or retryable (the taskbar was recreated).
pub fn try_add_tray_icon(hwnd: HWND) -> Option<NOTIFYICONDATAW> {
    let mut sz_tip = [0u16; 128];
    copy_str_to_utf16_buffer(TRAY_TIP_TEXT, &mut sz_tip);

    let h_icon = match load_app_icon() {
        Some(icon) => icon,
        None => match unsafe { LoadIconW(None, IDI_APPLICATION) } {
            Ok(icon) => icon,
            Err(e) => {
                log_err!("LoadIconW failed: {e}");
                return None;
            }
        },
    };
    let notify = NOTIFYICONDATAW {
        #[allow(clippy::cast_possible_truncation)]
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ICON_ID,
        uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        uCallbackMessage: WM_TRAYICON,
        szTip: sz_tip,
        hIcon: h_icon,
        ..Default::default()
    };

    let result = unsafe { Shell_NotifyIconW(NIM_ADD, &raw const notify) };
    if !result.as_bool() {
        log_err!("Shell_NotifyIconW NIM_ADD failed, tray icon could not be created");
        return None;
    }

    Some(notify)
}

pub fn add_tray_icon(hwnd: HWND) -> NOTIFYICONDATAW {
    match try_add_tray_icon(hwnd) {
        Some(notify) => notify,
        None => {
            show_error_message_box("PS Battery could not set up its tray icon and will now close.");
            std::process::exit(1);
        }
    }
}

pub use create_hidden_window::create_hidden_window;
pub use show_balloon::{BalloonIcon, show_balloon};
