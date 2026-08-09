use crate::show_error_message_box::show_error_message_box;
use crate::{log_err, log_info};
use windows::Win32::Foundation::{HINSTANCE, HWND};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateWindowExW, RegisterClassW, WINDOW_EX_STYLE,
    WNDCLASSW, WS_OVERLAPPEDWINDOW,
};
use windows::core::{HSTRING, PCWSTR};

use super::menu::window_proc;

const WINDOW_CLASS_NAME: &str = "ps_batteryHiddenWindow";

pub fn create_hidden_window() -> HWND {
    log_info!("Creating hidden window");
    // A null handle also resolves to the exe's module, so failing to fetch
    // the real one is not worth refusing to start over.
    let module = match unsafe { GetModuleHandleW(PCWSTR::null()) } {
        Ok(module) => module.into(),
        Err(e) => {
            log_err!("GetModuleHandleW failed: {e}");
            HINSTANCE::default()
        }
    };
    let class_name = HSTRING::from(WINDOW_CLASS_NAME);
    let window_class = WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        hInstance: module,
        lpszClassName: PCWSTR(class_name.as_ptr()),
        style: CS_HREDRAW | CS_VREDRAW,
        ..Default::default()
    };
    unsafe {
        let result = RegisterClassW(&raw const window_class);
        if result == 0 {
            log_err!("RegisterClassW failed");
            show_error_message_box(
                "PS Battery could not register its window class and will now close.",
            );
            std::process::exit(1);
        }
        let hidden_window = match CreateWindowExW(
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
            Some(module),
            None,
        ) {
            Ok(window) => window,
            Err(e) => {
                log_err!("CreateWindowExW failed: {e}");
                show_error_message_box(
                    "PS Battery could not create its message window and will now close.",
                );
                std::process::exit(1);
            }
        };
        log_info!("Created hidden window");
        hidden_window
    }
}
