use crate::{log_err, log_info};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::UI::Shell::{NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW, ShellExecuteW};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, DefWindowProcW, DestroyMenu, GetCursorPos, MF_CHECKED, MF_GRAYED,
    MF_SEPARATOR, MF_STRING, MF_UNCHECKED, PostQuitMessage, SW_SHOWNORMAL, SetForegroundWindow,
    TPM_RIGHTBUTTON, TrackPopupMenu, WM_COMMAND, WM_LBUTTONUP, WM_RBUTTONUP,
};
use windows::core::{PCWSTR, w};

use super::{TRAY_ICON_ID, WM_TRAYICON, autostart};
use crate::ps_battery::controller_status_to_string::controller_status_to_string;
use crate::ps_battery::controller_store::get_controllers;
use crate::ps_battery::logger::get_log_path;
use crate::ps_battery::poll_controllers::request_poll;

const MENU_ID_AUTOSTART: u16 = 1001;
const MENU_ID_FORCE_POLL: u16 = 1002;
const MENU_ID_OPEN_LOG: u16 = 1003;
const MENU_ID_EXIT: u16 = 1;

#[allow(clippy::too_many_lines)]
pub extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_TRAYICON {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        if lparam.0 as u32 == WM_RBUTTONUP || lparam.0 as u32 == WM_LBUTTONUP {
            let menu = match unsafe { CreatePopupMenu() } {
                Ok(m) => m,
                Err(e) => {
                    log_err!("CreatePopupMenu failed: {e}");
                    return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
                }
            };

            let controllers = get_controllers();

            for controller in &controllers {
                let formatted = controller_status_to_string(controller);
                let utf16: Vec<u16> = formatted.encode_utf16().chain(Some(0)).collect();
                let res =
                    unsafe { AppendMenuW(menu, MF_STRING | MF_GRAYED, 0, PCWSTR(utf16.as_ptr())) };
                if res.is_err() {
                    log_err!("AppendMenuW failed");
                }
            }

            if controllers.is_empty() {
                let res = unsafe {
                    let no_controllers_text: Vec<u16> = "No controllers connected"
                        .encode_utf16()
                        .chain(Some(0))
                        .collect();
                    AppendMenuW(
                        menu,
                        MF_STRING | MF_GRAYED,
                        0,
                        PCWSTR(no_controllers_text.as_ptr()),
                    )
                };
                if res.is_err() {
                    log_err!("AppendMenuW failed");
                }
            }

            let res = unsafe { AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null()) };
            if res.is_err() {
                log_err!("AppendMenuW separator failed");
            }

            let res = unsafe {
                AppendMenuW(
                    menu,
                    MF_STRING,
                    MENU_ID_FORCE_POLL as usize,
                    w!("Scan for controllers now"),
                )
            };
            if res.is_err() {
                log_err!("AppendMenuW refresh failed");
            }

            let res = unsafe { AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null()) };
            if res.is_err() {
                log_err!("AppendMenuW separator failed");
            }

            let autostart_enabled = autostart::is_enabled();
            let autostart_text: Vec<u16> = "Run on startup".encode_utf16().chain(Some(0)).collect();
            let autostart_state = if autostart_enabled {
                MF_CHECKED
            } else {
                MF_UNCHECKED
            };
            let res = unsafe {
                AppendMenuW(
                    menu,
                    MF_STRING | autostart_state,
                    MENU_ID_AUTOSTART as usize,
                    PCWSTR(autostart_text.as_ptr()),
                )
            };
            if res.is_err() {
                log_err!("AppendMenuW autostart failed");
            }

            let res = unsafe { AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null()) };
            if res.is_err() {
                log_err!("AppendMenuW separator failed");
            }

            let log_flags = if get_log_path().is_some() {
                MF_STRING
            } else {
                MF_STRING | MF_GRAYED
            };
            let res =
                unsafe { AppendMenuW(menu, log_flags, MENU_ID_OPEN_LOG as usize, w!("Open log")) };
            if res.is_err() {
                log_err!("AppendMenuW open log failed");
            }

            let res = unsafe { AppendMenuW(menu, MF_STRING, MENU_ID_EXIT as usize, w!("Exit")) };
            if res.is_err() {
                log_err!("AppendMenuW exit failed");
            }

            let mut cursor = POINT::default();
            if let Err(e) = unsafe { GetCursorPos(&raw mut cursor) } {
                log_err!("GetCursorPos failed: {e}");
                let _ = unsafe { DestroyMenu(menu) };
                return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
            }

            // Called for its side effect: forces the hidden window to receive
            // WM_COMMAND when the user selects a menu item. The return value
            // is FALSE when the window isn't already in the foreground, which
            // is the normal case for a background tray app — not an error.
            let _ = unsafe { SetForegroundWindow(hwnd) };

            let popup = unsafe {
                TrackPopupMenu(
                    menu,
                    TPM_RIGHTBUTTON,
                    cursor.x,
                    cursor.y,
                    Some(0),
                    hwnd,
                    None,
                )
            };
            if !popup.as_bool() {
                log_err!("TrackPopupMenu failed");
            }

            let res = unsafe { DestroyMenu(menu) };
            if res.is_err() {
                log_err!("DestroyMenu failed");
            }
        }
    } else if msg == WM_COMMAND {
        #[allow(clippy::cast_possible_truncation)]
        match wparam.0 as u16 {
            MENU_ID_FORCE_POLL => {
                log_info!("Refresh requested via tray menu");
                request_poll();
            }
            MENU_ID_OPEN_LOG => {
                if let Some(path) = get_log_path() {
                    let path_utf16: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
                    unsafe {
                        ShellExecuteW(
                            None,
                            w!("open"),
                            PCWSTR(path_utf16.as_ptr()),
                            None,
                            None,
                            SW_SHOWNORMAL,
                        )
                    };
                }
            }
            MENU_ID_AUTOSTART => {
                let res = if autostart::is_enabled() {
                    log_info!("Autostart disabled");
                    autostart::disable()
                } else {
                    log_info!("Autostart enabled");
                    autostart::enable()
                };
                if !res {
                    log_err!("autostart toggle failed");
                }
            }
            MENU_ID_EXIT => {
                #[allow(clippy::cast_possible_truncation)]
                let notify = NOTIFYICONDATAW {
                    cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                    hWnd: hwnd,
                    uID: TRAY_ICON_ID,
                    ..Default::default()
                };
                let res = unsafe { Shell_NotifyIconW(NIM_DELETE, &raw const notify) };
                if !res.as_bool() {
                    log_err!("Shell_NotifyIconW NIM_DELETE failed");
                }
                log_info!("Closing app via tray menu");
                unsafe { PostQuitMessage(0) };
            }
            _ => {}
        }
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
