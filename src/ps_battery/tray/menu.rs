use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::UI::Shell::{NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, DefWindowProcW, DestroyMenu, GetCursorPos, MF_CHECKED, MF_GRAYED,
    MF_SEPARATOR, MF_STRING, MF_UNCHECKED, PostQuitMessage, SetForegroundWindow, TPM_RIGHTBUTTON,
    TrackPopupMenu, WM_COMMAND, WM_LBUTTONUP, WM_RBUTTONUP,
};
use windows::core::{PCWSTR, w};

use super::{WM_TRAYICON, autostart};
use crate::ps_battery::controller_status_to_string::controller_status_to_string;
use crate::ps_battery::controller_store::get_controllers;

const MENU_ID_AUTOSTART: u16 = 1001;
const MENU_ID_EXIT: u16 = 1;

pub extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_TRAYICON {
        if lparam.0 as u32 == WM_RBUTTONUP || lparam.0 as u32 == WM_LBUTTONUP {
            let menu = unsafe { CreatePopupMenu() }.expect("create menu failed");

            let controllers = get_controllers();

            for controller in &controllers {
                let formatted = controller_status_to_string(controller);
                let utf16: Vec<u16> = formatted.encode_utf16().chain(Some(0)).collect();
                let res =
                    unsafe { AppendMenuW(menu, MF_STRING | MF_GRAYED, 0, PCWSTR(utf16.as_ptr())) };
                if res.is_err() {
                    eprintln!(" !! AppendMenuW failed");
                }
            }

            if controllers.len() == 0 {
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
                    eprintln!(" !! AppendMenuW failed");
                }
            }

            let autostart_enabled = autostart::is_enabled();
            let autostart_text: Vec<u16> = "Run on Startup".encode_utf16().chain(Some(0)).collect();
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
                eprintln!(" !! AppendMenuW autostart failed");
            }
            let res = unsafe { AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null()) };
            if res.is_err() {
                eprintln!(" !! AppendMenuW sep failed");
            }
            let res = unsafe { AppendMenuW(menu, MF_STRING, MENU_ID_EXIT as usize, w!("Exit")) };
            if res.is_err() {
                eprintln!(" !! AppendMenuW exit failed");
            }

            let mut cursor = POINT::default();
            let cursor_pos = unsafe { GetCursorPos(&mut cursor) };
            if cursor_pos.is_err() {
                eprintln!(" !! GetCursorPos failed");
            }

            let set_foreground_result = unsafe { SetForegroundWindow(hwnd) };
            if !set_foreground_result.as_bool() {
                eprintln!(" !! SetForegroundWindow failed");
            }

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
                eprintln!(" !! TrackPopupMenu failed");
            }

            let res = unsafe { DestroyMenu(menu) };
            if res.is_err() {
                eprintln!(" !! DestroyMenu failed");
            }
        }
    } else if msg == WM_COMMAND {
        match wparam.0 as u16 {
            MENU_ID_AUTOSTART => {
                let res = if autostart::is_enabled() {
                    println!(" -> Autostart disabled");
                    autostart::disable()
                } else {
                    println!(" -> Autostart enabled");
                    autostart::enable()
                };
                if !res {
                    eprintln!(" !! autostart toggle failed");
                }
            }
            MENU_ID_EXIT => {
                let notify = NOTIFYICONDATAW::default();
                let res = unsafe { Shell_NotifyIconW(NIM_DELETE, &notify) };
                if !res.as_bool() {
                    eprintln!(" !! Shell_NotifyIconW delete failed");
                }
                println!(" -> Closing app via tray menu");
                unsafe {
                    PostQuitMessage(0);
                }
            }
            _ => {}
        }
    }

    println!(" -> window proc");

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
