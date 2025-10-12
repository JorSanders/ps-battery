use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr;

use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WIN32_ERROR, WPARAM};
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SAM_FLAGS, REG_SZ, RRF_RT_REG_SZ,
    RegCloseKey, RegDeleteValueW, RegGetValueW, RegOpenKeyExW, RegSetValueExW,
};
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY, NIS_HIDDEN, NOTIFYICONDATAW,
    Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreatePopupMenu, CreateWindowExW,
    DefWindowProcW, DestroyMenu, GetCursorPos, IDI_APPLICATION, LoadIconW, MF_CHECKED, MF_GRAYED,
    MF_SEPARATOR, MF_STRING, MF_UNCHECKED, PostQuitMessage, RegisterClassW, SetForegroundWindow,
    TPM_RIGHTBUTTON, TrackPopupMenu, WINDOW_EX_STYLE, WNDCLASSW, WS_OVERLAPPEDWINDOW,
};
use windows::core::{HSTRING, PCWSTR, w};

use crate::ps_battery::controller_store::get_controllers;

const MENU_ID_AUTOSTART: u16 = 1001;
const MENU_ID_EXIT: u16 = 1;
const TRAY_ICON_ID: u32 = 100;
const TRAY_TIP_TEXT: &str = "PS Battery Monitor";
const WM_TRAYICON: u32 = 0x8000 + 1;
const WINDOW_CLASS_NAME: &str = "ps_batteryHiddenWindow";
const APP_NAME: &str = "PS Battery";
const RUN_SUBKEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

fn is_autostart_enabled() -> bool {
    let subkey = to_wide(RUN_SUBKEY);
    let value_name = to_wide(APP_NAME);
    let mut buf: [u16; 260] = [0; 260];
    let mut cb = (buf.len() * 2) as u32;
    unsafe {
        let res: WIN32_ERROR = RegGetValueW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            PCWSTR(value_name.as_ptr()),
            RRF_RT_REG_SZ,
            None,
            Some(buf.as_mut_ptr() as _),
            Some(&mut cb),
        );
        if res.is_ok() {
            println!("[autostart] RegGetValueW OK ({} bytes)", cb);
            true
        } else {
            println!("[autostart] RegGetValueW failed: {:?}", res);
            false
        }
    }
}

fn enable_autostart() -> bool {
    let subkey = to_wide(RUN_SUBKEY);
    let mut hkey = HKEY(ptr::null_mut());
    unsafe {
        let open = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            Some(0),
            KEY_SET_VALUE | KEY_QUERY_VALUE,
            &mut hkey,
        );
        if open.is_err() {
            eprintln!("[autostart] RegOpenKeyExW (enable) failed: {:?}", open);
            return false;
        }
        println!("[autostart] RegOpenKeyExW (enable): OK");

        let exe = std::env::current_exe().unwrap_or_default();
        let exe_str = exe.to_string_lossy();
        let data_u16 = to_wide(&exe_str);
        let bytes = std::slice::from_raw_parts(data_u16.as_ptr() as *const u8, data_u16.len() * 2);
        let name = to_wide(APP_NAME);

        let set = RegSetValueExW(hkey, PCWSTR(name.as_ptr()), Some(0), REG_SZ, Some(bytes));
        let _ = RegCloseKey(hkey);
        if set.is_ok() {
            println!("[autostart] RegSetValueExW OK -> {}", exe_str);
            true
        } else {
            eprintln!("[autostart] RegSetValueExW failed: {:?}", set);
            false
        }
    }
}

fn disable_autostart() -> bool {
    let subkey = to_wide(RUN_SUBKEY);
    let mut hkey = HKEY(ptr::null_mut());
    unsafe {
        let open = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            Some(0),
            KEY_SET_VALUE | KEY_QUERY_VALUE,
            &mut hkey,
        );
        if open.is_err() {
            eprintln!("[autostart] RegOpenKeyExW (disable) failed: {:?}", open);
            return false;
        }
        println!("[autostart] RegOpenKeyExW (disable): OK");

        let name = to_wide(APP_NAME);
        let del = RegDeleteValueW(hkey, PCWSTR(name.as_ptr()));
        let _ = RegCloseKey(hkey);
        if del.is_ok() {
            println!("[autostart] RegDeleteValueW OK");
            true
        } else {
            eprintln!("[autostart] RegDeleteValueW failed: {:?}", del);
            false
        }
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_TRAYICON {
        if lparam.0 as u32 == windows::Win32::UI::WindowsAndMessaging::WM_RBUTTONUP {
            let menu = unsafe { CreatePopupMenu() }.expect("Failed to create menu");

            for status in get_controllers() {
                let text = format!(
                    "{}: {}% {}",
                    status.name,
                    status.battery_percent,
                    if status.is_charging {
                        "Charging"
                    } else {
                        "Discharging"
                    }
                );
                let text_utf16: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
                unsafe {
                    AppendMenuW(menu, MF_STRING | MF_GRAYED, 0, PCWSTR(text_utf16.as_ptr()))
                        .expect("Failed to append menu item");
                }
            }

            let auto_enabled = is_autostart_enabled();
            println!(
                "[tray] building menu: autostart currently = {}",
                auto_enabled
            );
            let auto_text = to_wide("Start with Windows");
            let auto_state = if auto_enabled {
                MF_CHECKED
            } else {
                MF_UNCHECKED
            };
            unsafe {
                AppendMenuW(
                    menu,
                    MF_STRING | auto_state,
                    MENU_ID_AUTOSTART as usize,
                    PCWSTR(auto_text.as_ptr()),
                )
                .expect("Failed to add autostart toggle");
            }

            unsafe {
                AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null())
                    .expect("Failed to append separator");
                AppendMenuW(menu, MF_STRING, MENU_ID_EXIT as usize, w!("Exit"))
                    .expect("Failed to append Exit item");
            }

            let mut cursor = POINT::default();
            unsafe {
                let _ = GetCursorPos(&mut cursor);
                let _ = SetForegroundWindow(hwnd);
                let _ = TrackPopupMenu(
                    menu,
                    TPM_RIGHTBUTTON,
                    cursor.x,
                    cursor.y,
                    Some(0),
                    hwnd,
                    None,
                );
                DestroyMenu(menu).expect("Failed to destroy menu");
            }
        } else if lparam.0 as u32 == windows::Win32::UI::WindowsAndMessaging::WM_LBUTTONUP {
            unsafe {
                let mut notify = NOTIFYICONDATAW::default();
                let _ = Shell_NotifyIconW(NIM_DELETE, &mut notify);
                PostQuitMessage(0);
            }
        }
    } else if msg == windows::Win32::UI::WindowsAndMessaging::WM_COMMAND {
        match wparam.0 as u16 {
            MENU_ID_AUTOSTART => {
                let before = is_autostart_enabled();
                println!("[tray] toggle clicked; before = {}", before);
                let ok = if before {
                    disable_autostart()
                } else {
                    enable_autostart()
                };
                let after = is_autostart_enabled();
                println!("[tray] toggle result ok = {}, after = {}", ok, after);
                let _ = SetForegroundWindow(hwnd);
            }
            MENU_ID_EXIT => {
                let mut notify = NOTIFYICONDATAW::default();
                let _ = Shell_NotifyIconW(NIM_DELETE, &mut notify);
                PostQuitMessage(0);
            }
            _ => {}
        }
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

pub unsafe fn create_hidden_window() -> HWND {
    let class_name = HSTRING::from(WINDOW_CLASS_NAME);
    let window_class = WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        hInstance: HINSTANCE::default(),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        style: CS_HREDRAW | CS_VREDRAW,
        ..Default::default()
    };

    unsafe {
        let result = RegisterClassW(&window_class);
        if result == 0 {
            eprintln!("Failed to register window class");
        }

        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            &class_name,
            &HSTRING::from(""),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            None,
            None,
        )
        .expect("Failed to create hidden window")
    }
}

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
        notify.hIcon = LoadIconW(None, IDI_APPLICATION).expect("Failed to load icon");
        let _ = Shell_NotifyIconW(NIM_ADD, &mut notify);
    }

    notify
}

pub struct ShowBalloonArgs<'a> {
    pub notify: &'a mut NOTIFYICONDATAW,
    pub title: &'a str,
    pub message: &'a str,
}

pub unsafe fn show_balloon(args: &mut ShowBalloonArgs) {
    let title_utf16: Vec<u16> = args.title.encode_utf16().chain(Some(0)).collect();
    let message_utf16: Vec<u16> = args.message.encode_utf16().chain(Some(0)).collect();

    args.notify.uFlags = NIF_TIP;
    args.notify.szInfo[..message_utf16.len()].copy_from_slice(&message_utf16);
    args.notify.szInfoTitle[..title_utf16.len()].copy_from_slice(&title_utf16);

    unsafe {
        let _ = Shell_NotifyIconW(NIM_MODIFY, args.notify);
    }
}
