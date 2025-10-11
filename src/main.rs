#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod controller;
mod hid;
mod sound;
mod toast;
mod tray;

use hid::*;
use std::time::Duration;
use tray::*;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage, WM_QUIT,
};

fn main() {
    let class_name = windows::core::HSTRING::from("PSBatteryHiddenWindow");
    let hwnd = unsafe { create_hidden_window(&class_name) };
    let mut nid = unsafe { add_tray_icon(hwnd) };

    loop {
        unsafe {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                if msg.message == WM_QUIT {
                    return;
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        check_controllers(&mut nid);
        std::thread::sleep(Duration::from_millis(100));
    }
}
