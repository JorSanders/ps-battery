#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ps_battery;

use crate::ps_battery::monitor::poll::{PollControllersArgs, poll_controllers};
use crate::ps_battery::tray::{add_tray_icon, create_hidden_window};
use std::thread::sleep;
use std::time::Duration;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage, WM_QUIT,
};

fn main() {
    unsafe {
        let hwnd = create_hidden_window();
        let mut tray_icon = add_tray_icon(hwnd);

        loop {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                if msg.message == WM_QUIT {
                    return;
                }

                let _translated = TranslateMessage(&msg);

                DispatchMessageW(&msg);
            }

            poll_controllers(&mut PollControllersArgs {
                tray_icon: &mut tray_icon,
            });

            sleep(Duration::from_millis(100));
        }
    }
}
