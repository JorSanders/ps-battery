#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ps_battery;

use crate::ps_battery::poll_controllers::poll_controllers;
use crate::ps_battery::tray::{add_tray_icon, create_hidden_window};
use std::thread::sleep;
use std::time::{Duration, Instant};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage, WM_QUIT,
};

const CONTROLLER_POLL_INTERVAL: Duration = Duration::from_secs(30);

fn main() {
    let hidden_window = create_hidden_window();
    let mut tray_icon = add_tray_icon(hidden_window);
    let mut last_controler_poll = Instant::now() - CONTROLLER_POLL_INTERVAL;

    loop {
        let mut msg = MSG::default();
        while unsafe { PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() } {
            if msg.message == WM_QUIT {
                return;
            }
            let _translated = unsafe { TranslateMessage(&msg) };
            unsafe { DispatchMessageW(&msg) };
        }

        if last_controler_poll.elapsed() >= CONTROLLER_POLL_INTERVAL {
            poll_controllers(&mut tray_icon);
            last_controler_poll = Instant::now();
        }

        sleep(Duration::from_millis(100));
    }
}
