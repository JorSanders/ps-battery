#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod controller;
mod hid;
mod sound;
mod toast;
mod tray;

use hid::*;
use std::time::{Duration, Instant};
use tray::*;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage, WM_QUIT,
};

const POLL_INTERVAL: Duration = Duration::from_secs(300);

fn main() {
    let class_name = windows::core::HSTRING::from("PSBatteryHiddenWindow");
    let hwnd = unsafe { create_hidden_window(&class_name) };
    let mut nid = unsafe { add_tray_icon(hwnd) };

    let mut last_check = Instant::now() - POLL_INTERVAL;

    loop {
        unsafe {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                if msg.message == WM_QUIT {
                    return;
                }
                let result = TranslateMessage(&msg);
                if !result.as_bool() {
                    eprintln!("Failed to translate message")
                }
                DispatchMessageW(&msg);
            }
        }

        if last_check.elapsed() >= POLL_INTERVAL {
            check_controllers(&mut nid);
            last_check = Instant::now();
        }

        std::thread::sleep(Duration::from_millis(100));
    }
}
