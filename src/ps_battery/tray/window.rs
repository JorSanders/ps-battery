use windows::Win32::Foundation::{HINSTANCE, HWND};
use windows::Win32::UI::WindowsAndMessaging::{
    CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateWindowExW, RegisterClassW, WINDOW_EX_STYLE,
    WNDCLASSW, WS_OVERLAPPEDWINDOW,
};
use windows::core::{HSTRING, PCWSTR};

use super::menu::window_proc;

const WINDOW_CLASS_NAME: &str = "ps_batteryHiddenWindow";

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
        let res = RegisterClassW(&window_class);
        if res == 0 {
            eprintln!("RegisterClassW failed");
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
        .expect("create hidden window failed")
    }
}
