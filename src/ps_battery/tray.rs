use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY, NIS_HIDDEN, NOTIFYICONDATAW,
    Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreatePopupMenu, CreateWindowExW,
    DefWindowProcW, DestroyMenu, GetCursorPos, IDI_APPLICATION, LoadIconW, MF_GRAYED, MF_SEPARATOR,
    MF_STRING, PostQuitMessage, RegisterClassW, SetForegroundWindow, TPM_RIGHTBUTTON,
    TrackPopupMenu, WINDOW_EX_STYLE, WNDCLASSW, WS_OVERLAPPEDWINDOW,
};
use windows::core::{HSTRING, PCWSTR, w};

use crate::ps_battery::controller_store::get_controllers;

const MENU_ID_EXIT: u16 = 1;
const TRAY_ICON_ID: u32 = 100;
const TRAY_TIP_TEXT: &str = "PS Battery Monitor";
const WM_TRAYICON: u32 = 0x8000 + 1;
const WINDOW_CLASS_NAME: &str = "ps_batteryHiddenWindow";

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

            unsafe {
                AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null())
                    .expect("Failed to append separator");
                AppendMenuW(menu, MF_STRING, MENU_ID_EXIT as usize, w!("Exit"))
                    .expect("Failed to append Exit item");
            }

            let mut cursor = POINT::default();
            unsafe {
                let cursor_result = GetCursorPos(&mut cursor);
                if !cursor_result.is_err() {
                    eprintln!("Failed to get cursor position");
                }

                let fg_result = SetForegroundWindow(hwnd);
                if !fg_result.as_bool() {
                    eprintln!("Failed to set foreground window");
                }

                let popup_result = TrackPopupMenu(
                    menu,
                    TPM_RIGHTBUTTON,
                    cursor.x,
                    cursor.y,
                    Some(0),
                    hwnd,
                    None,
                );
                if !popup_result.as_bool() {
                    eprintln!("Failed to track popup menu");
                }

                DestroyMenu(menu).expect("Failed to destroy menu");
            }
        } else if lparam.0 as u32 == windows::Win32::UI::WindowsAndMessaging::WM_LBUTTONUP {
            unsafe {
                let mut notify = NOTIFYICONDATAW::default();
                let delete_result = Shell_NotifyIconW(NIM_DELETE, &mut notify);
                if !delete_result.as_bool() {
                    eprintln!("Failed to delete tray icon");
                }
                PostQuitMessage(0);
            }
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

        let add_result = Shell_NotifyIconW(NIM_ADD, &mut notify);
        if !add_result.as_bool() {
            eprintln!("Failed to add tray icon");
        }
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
        let modify_result = Shell_NotifyIconW(NIM_MODIFY, args.notify);
        if !modify_result.as_bool() {
            eprintln!("Failed to show balloon");
        }
    }
}
