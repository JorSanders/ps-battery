use crate::{log_err, log_info};
use std::cell::RefCell;
use std::sync::LazyLock;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::InvalidateRect;
use windows::Win32::UI::Shell::{NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW, ShellExecuteW};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, DefWindowProcW, DestroyMenu, GetCursorPos, GetMenuItemCount,
    HMENU, KillTimer, MENU_ITEM_FLAGS, MF_BYPOSITION, MF_CHECKED, MF_GRAYED, MF_SEPARATOR,
    MF_STRING, MF_UNCHECKED, MSGF_MENU, PostQuitMessage, RegisterWindowMessageW, RemoveMenu,
    SW_SHOWNORMAL, SetForegroundWindow, SetTimer, TPM_RIGHTBUTTON, TrackPopupMenu, WM_COMMAND,
    WM_ENTERIDLE, WM_LBUTTONUP, WM_RBUTTONUP,
};
use windows::core::{PCWSTR, w};

use super::{TRAY_ICON_ID, WM_TRAYICON, try_add_tray_icon};
use crate::autostart;
use crate::controllers::controller_status_to_string::controller_status_to_string;
use crate::controllers::controller_store::{get_controllers, get_generation};
use crate::controllers::poll_controllers::{is_polling, request_poll};
use crate::logger::get_log_path;

const MENU_ID_AUTOSTART: u16 = 1001;
const MENU_ID_OPEN_LOG: u16 = 1002;
const MENU_ID_EXIT: u16 = 1003;

/// Explorer broadcasts this registered message when the taskbar is recreated
/// after a crash or restart; every tray icon is gone then and must be
/// re-added. Registration returns 0 on failure, which the window proc guards
/// against because 0 is also `WM_NULL`.
static TASKBAR_CREATED_MESSAGE: LazyLock<u32> =
    LazyLock::new(|| unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) });

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
        if is_polling() {
            Self::Scanning
        } else {
            Self::NotStarted
        }
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
    /// we receive for it, needed to `InvalidateRect` it after a refresh.
    menu_hwnd: Option<HWND>,
    last_generation: u64,
    scan_status: ScanStatus,
    /// Packaged builds resolve these on a worker thread, so they can change
    /// while the menu is open.
    last_autostart_enabled: bool,
    last_autostart_available: bool,
}

thread_local! {
    static ACTIVE_MENU: RefCell<Option<ActiveMenu>> = const { RefCell::new(None) };
}

fn append_menu_item(menu: HMENU, flags: MENU_ITEM_FLAGS, item_id: u16, text: &str) {
    let text_utf16: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
    let result = unsafe {
        AppendMenuW(
            menu,
            MF_STRING | flags,
            item_id as usize,
            PCWSTR(text_utf16.as_ptr()),
        )
    };
    if let Err(e) = result {
        log_err!("AppendMenuW failed for '{text}': {e}");
    }
}

fn append_menu_separator(menu: HMENU) {
    if let Err(e) = unsafe { AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null()) } {
        log_err!("AppendMenuW separator failed: {e}");
    }
}

fn populate_menu(menu: HMENU, scan_status: ScanStatus) {
    let status_text = match scan_status {
        ScanStatus::NotStarted => None,
        ScanStatus::Scanning => Some("Scanning for controllers…"),
        ScanStatus::Completed => Some("Scan completed"),
    };
    if let Some(status_text) = status_text {
        append_menu_item(menu, MF_GRAYED, 0, status_text);
    }

    let controllers = get_controllers();

    for controller in &controllers {
        append_menu_item(menu, MF_GRAYED, 0, &controller_status_to_string(controller));
    }

    if controllers.is_empty() {
        append_menu_item(menu, MF_GRAYED, 0, "No controllers connected");
    }

    append_menu_separator(menu);

    let mut autostart_flags = if autostart::is_enabled() {
        MF_CHECKED
    } else {
        MF_UNCHECKED
    };
    // A grayed item cannot be selected, so a toggle whose backend is gone
    // never sends WM_COMMAND into the void.
    if !autostart::is_available() {
        autostart_flags |= MF_GRAYED;
    }
    append_menu_item(menu, autostart_flags, MENU_ID_AUTOSTART, "Run on startup");

    append_menu_separator(menu);

    let log_flags = if get_log_path().is_some() {
        MENU_ITEM_FLAGS(0)
    } else {
        MF_GRAYED
    };
    append_menu_item(menu, log_flags, MENU_ID_OPEN_LOG, "Open log");

    append_menu_item(menu, MENU_ITEM_FLAGS(0), MENU_ID_EXIT, "Exit");
}

fn refresh_menu(menu: HMENU, menu_hwnd: HWND, scan_status: ScanStatus) {
    while unsafe { GetMenuItemCount(Some(menu)) } > 0 {
        if let Err(e) = unsafe { RemoveMenu(menu, 0, MF_BYPOSITION) } {
            log_err!("RemoveMenu failed: {e}");
            break;
        }
    }

    populate_menu(menu, scan_status);

    if !unsafe { InvalidateRect(Some(menu_hwnd), None, true) }.as_bool() {
        log_err!("InvalidateRect failed");
    }
}

fn try_refresh_active_menu() {
    ACTIVE_MENU.with_borrow_mut(|active| {
        let Some(state) = active.as_mut() else { return };
        let Some(menu_hwnd) = state.menu_hwnd else {
            return;
        };

        let current_generation = get_generation();
        let new_status = state.scan_status.next(is_polling());
        let autostart_enabled = autostart::is_enabled();
        let autostart_available = autostart::is_available();
        if current_generation != state.last_generation
            || new_status != state.scan_status
            || autostart_enabled != state.last_autostart_enabled
            || autostart_available != state.last_autostart_available
        {
            refresh_menu(state.menu, menu_hwnd, new_status);
            state.last_generation = current_generation;
            state.scan_status = new_status;
            state.last_autostart_enabled = autostart_enabled;
            state.last_autostart_available = autostart_available;
        }
    });
}

extern "system" fn menu_refresh_timer_proc(_hwnd: HWND, _msg: u32, _timer_id: usize, _time: u32) {
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

            request_poll();
            autostart::request_refresh();

            let menu = match unsafe { CreatePopupMenu() } {
                Ok(menu) => menu,
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
                    last_autostart_enabled: autostart::is_enabled(),
                    last_autostart_available: autostart::is_available(),
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
            // is the normal case for a background tray app, not an error.
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
                log_err!("SetTimer failed: {}", windows::core::Error::from_thread());
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
                log_err!(
                    "TrackPopupMenu failed: {}",
                    windows::core::Error::from_thread()
                );
            }

            if let Err(e) = unsafe { KillTimer(Some(hwnd), MENU_REFRESH_TIMER_ID) } {
                log_err!("KillTimer failed: {e}");
            }

            ACTIVE_MENU.with_borrow_mut(|active| *active = None);

            if let Err(e) = unsafe { DestroyMenu(menu) } {
                log_err!("DestroyMenu failed: {e}");
            }
        }
    } else if msg == *TASKBAR_CREATED_MESSAGE && msg != 0 {
        log_info!("Taskbar recreated, re-adding the tray icon");
        if try_add_tray_icon(hwnd).is_none() {
            log_err!("Re-adding the tray icon after a taskbar restart failed");
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
                    log_info!("Opening the log file");
                    let path_utf16: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
                    let result = unsafe {
                        ShellExecuteW(
                            None,
                            w!("open"),
                            PCWSTR(path_utf16.as_ptr()),
                            None,
                            None,
                            SW_SHOWNORMAL,
                        )
                    };
                    // ShellExecuteW reports success as a value above 32.
                    if result.0 as isize <= 32 {
                        log_err!(
                            "ShellExecuteW failed to open the log (code {})",
                            result.0 as isize
                        );
                    }
                }
            }
            MENU_ID_AUTOSTART => {
                let result = if autostart::is_enabled() {
                    log_info!("Disabling autostart");
                    autostart::disable()
                } else {
                    log_info!("Enabling autostart");
                    autostart::enable()
                };
                if !result {
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
                let result = unsafe { Shell_NotifyIconW(NIM_DELETE, &raw const notify) };
                if !result.as_bool() {
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
