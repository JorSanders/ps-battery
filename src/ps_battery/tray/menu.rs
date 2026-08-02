use crate::{log_err, log_info};
use std::cell::RefCell;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::InvalidateRect;
use windows::Win32::UI::Shell::{NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW, ShellExecuteW};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, DefWindowProcW, DestroyMenu, GetCursorPos, GetMenuItemCount,
    HMENU, KillTimer, MF_BYPOSITION, MF_CHECKED, MF_GRAYED, MF_SEPARATOR, MF_STRING, MF_UNCHECKED,
    MSGF_MENU, PostQuitMessage, RemoveMenu, SW_SHOWNORMAL, SetForegroundWindow, SetTimer,
    TPM_RIGHTBUTTON, TrackPopupMenu, WM_COMMAND, WM_ENTERIDLE, WM_LBUTTONUP, WM_RBUTTONUP,
};
use windows::core::{PCWSTR, w};

use super::{TRAY_ICON_ID, WM_TRAYICON, autostart};
use crate::ps_battery::controller_status_to_string::controller_status_to_string;
use crate::ps_battery::controller_store::{get_controllers, get_generation};
use crate::ps_battery::logger::get_log_path;
use crate::ps_battery::poll_controllers::{is_polling, request_poll};

const MENU_ID_AUTOSTART: u16 = 1001;
const MENU_ID_OPEN_LOG: u16 = 1003;
const MENU_ID_EXIT: u16 = 1;

/// Ticks while the tray menu is open, so the menu keeps refreshing even when
/// the mouse isn't moving. A `TIMERPROC` callback (rather than a plain
/// `WM_TIMER` message) is required here: `TrackPopupMenu`'s internal modal
/// loop only dispatches ordinary posted messages in response to real user
/// input, but the OS invokes a timer callback directly from its
/// `GetMessage`/`PeekMessage` internals regardless of that loop's filtering.
const MENU_REFRESH_TIMER_ID: usize = 1;
const MENU_REFRESH_INTERVAL_MS: u32 = 250;

/// Scan progress for the currently open menu. `Completed` (rather than
/// clearing the line entirely) keeps a fixed-size placeholder in the menu
/// once a scan finishes, so the menu doesn't resize/jump unless the
/// controller list itself actually changed.
#[derive(Clone, Copy, PartialEq)]
enum ScanStatus {
    NotStarted,
    Scanning,
    Completed,
}

impl ScanStatus {
    fn initial() -> Self {
        if is_polling() { Self::Scanning } else { Self::NotStarted }
    }

    fn next(self, currently_polling: bool) -> Self {
        match (self, currently_polling) {
            (_, true) => Self::Scanning,
            (Self::Scanning, false) => Self::Completed,
            (other, false) => other,
        }
    }
}

struct ActiveMenu {
    menu: HMENU,
    /// The popup's own window handle, learned from the first `WM_ENTERIDLE`
    /// we receive for it — needed to `InvalidateRect` it after a refresh.
    menu_hwnd: Option<HWND>,
    last_generation: u64,
    scan_status: ScanStatus,
}

thread_local! {
    static ACTIVE_MENU: RefCell<Option<ActiveMenu>> = const { RefCell::new(None) };
}

fn populate_menu(menu: HMENU, scan_status: ScanStatus) {
    let status_text = match scan_status {
        ScanStatus::NotStarted => None,
        ScanStatus::Scanning => Some("Scanning for controllers…"),
        ScanStatus::Completed => Some("Scan completed"),
    };
    if let Some(status_text) = status_text {
        let res = unsafe {
            let status_text: Vec<u16> = status_text.encode_utf16().chain(Some(0)).collect();
            AppendMenuW(menu, MF_STRING | MF_GRAYED, 0, PCWSTR(status_text.as_ptr()))
        };
        if res.is_err() {
            log_err!("AppendMenuW failed");
        }
    }

    let controllers = get_controllers();

    for controller in &controllers {
        let formatted = controller_status_to_string(controller);
        let utf16: Vec<u16> = formatted.encode_utf16().chain(Some(0)).collect();
        let res = unsafe { AppendMenuW(menu, MF_STRING | MF_GRAYED, 0, PCWSTR(utf16.as_ptr())) };
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
    let res = unsafe { AppendMenuW(menu, log_flags, MENU_ID_OPEN_LOG as usize, w!("Open log")) };
    if res.is_err() {
        log_err!("AppendMenuW open log failed");
    }

    let res = unsafe { AppendMenuW(menu, MF_STRING, MENU_ID_EXIT as usize, w!("Exit")) };
    if res.is_err() {
        log_err!("AppendMenuW exit failed");
    }
}

/// Rebuilds `menu` in place from the latest controller data and forces the
/// still-open popup window (`menu_hwnd`) to repaint with it.
fn refresh_menu(menu: HMENU, menu_hwnd: HWND, scan_status: ScanStatus) {
    unsafe {
        while GetMenuItemCount(Some(menu)) > 0 {
            if RemoveMenu(menu, 0, MF_BYPOSITION).is_err() {
                log_err!("RemoveMenu failed");
                break;
            }
        }
    }

    populate_menu(menu, scan_status);

    if !unsafe { InvalidateRect(Some(menu_hwnd), None, true) }.as_bool() {
        log_err!("InvalidateRect failed");
    }
}

/// Refreshes the currently tracked menu if the controller data or scan
/// status has moved on since it was last built. Called both from the
/// per-tick timer and from `WM_ENTERIDLE`.
fn try_refresh_active_menu() {
    ACTIVE_MENU.with_borrow_mut(|active| {
        let Some(state) = active.as_mut() else { return };
        let Some(menu_hwnd) = state.menu_hwnd else { return };

        let current_generation = get_generation();
        let new_status = state.scan_status.next(is_polling());
        if current_generation != state.last_generation || new_status != state.scan_status {
            refresh_menu(state.menu, menu_hwnd, new_status);
            state.last_generation = current_generation;
            state.scan_status = new_status;
        }
    });
}

unsafe extern "system" fn menu_refresh_timer_proc(_hwnd: HWND, _msg: u32, _timer_id: usize, _time: u32) {
    try_refresh_active_menu();
}

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
            log_info!("Tray menu opened");

            // Kicks the polling thread awake via a condvar notify; the actual
            // HID scan runs there, not on this (UI) thread, so it never
            // blocks the menu from opening.
            request_poll();

            let menu = match unsafe { CreatePopupMenu() } {
                Ok(m) => m,
                Err(e) => {
                    log_err!("CreatePopupMenu failed: {e}");
                    return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
                }
            };

            let scan_status = ScanStatus::initial();
            populate_menu(menu, scan_status);
            ACTIVE_MENU.with_borrow_mut(|active| {
                *active = Some(ActiveMenu {
                    menu,
                    menu_hwnd: None,
                    last_generation: get_generation(),
                    scan_status,
                });
            });

            let mut cursor = POINT::default();
            if let Err(e) = unsafe { GetCursorPos(&raw mut cursor) } {
                log_err!("GetCursorPos failed: {e}");
                ACTIVE_MENU.with_borrow_mut(|active| *active = None);
                let _ = unsafe { DestroyMenu(menu) };
                return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
            }

            // Called for its side effect: forces the hidden window to receive
            // WM_COMMAND when the user selects a menu item. The return value
            // is FALSE when the window isn't already in the foreground, which
            // is the normal case for a background tray app — not an error.
            let _ = unsafe { SetForegroundWindow(hwnd) };

            let timer_id = unsafe {
                SetTimer(
                    Some(hwnd),
                    MENU_REFRESH_TIMER_ID,
                    MENU_REFRESH_INTERVAL_MS,
                    Some(menu_refresh_timer_proc),
                )
            };
            if timer_id == 0 {
                log_err!("SetTimer failed");
            }

            // Blocks (running a nested message loop on this thread) until the
            // menu is dismissed. menu_refresh_timer_proc above still fires
            // during that loop, which is how the menu gets live-refreshed.
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

            if unsafe { KillTimer(Some(hwnd), MENU_REFRESH_TIMER_ID) }.is_err() {
                log_err!("KillTimer failed");
            }

            ACTIVE_MENU.with_borrow_mut(|active| *active = None);

            let res = unsafe { DestroyMenu(menu) };
            if res.is_err() {
                log_err!("DestroyMenu failed");
            }
        }
    } else if msg == WM_ENTERIDLE {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        if wparam.0 as u32 == MSGF_MENU {
            ACTIVE_MENU.with_borrow_mut(|active| {
                if let Some(state) = active.as_mut() {
                    state
                        .menu_hwnd
                        .get_or_insert(HWND(lparam.0 as *mut core::ffi::c_void));
                }
            });
            try_refresh_active_menu();
        }
    } else if msg == WM_COMMAND {
        #[allow(clippy::cast_possible_truncation)]
        match wparam.0 as u16 {
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
