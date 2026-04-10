#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ps_battery;
use crate::ps_battery::poll_controllers::{poll_controllers, wait_for_next_poll};
use crate::ps_battery::send_controller_alerts;
use crate::ps_battery::tray::{add_tray_icon, create_hidden_window};
use crate::send_controller_alerts::send_controller_alerts;
use hidapi::HidApi;
use std::time::{Duration, Instant};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage, WM_QUIT,
};

const ALERT_INTERVAL: Duration = Duration::from_secs(300);

fn main() {
    ps_battery::logger::init();

    let hidden_window = create_hidden_window();
    let mut tray_icon = add_tray_icon(hidden_window);

    std::thread::spawn(move || {
        log_info!("initializing hidapi");
        let mut hid_api = HidApi::new().expect("Failed to initialize hidapi");
        log_info!("initialized hidapi");
        log_info!("hid device count {}", hid_api.device_list().count());

        loop {
            poll_controllers(&mut hid_api);
            wait_for_next_poll();
        }
    });

    let mut last_alert = Instant::now()
        .checked_sub(ALERT_INTERVAL)
        .unwrap_or_else(Instant::now);

    loop {
        let mut msg = MSG::default();
        while unsafe { PeekMessageW(&raw mut msg, None, 0, 0, PM_REMOVE).as_bool() } {
            if msg.message == WM_QUIT {
                return;
            }
            let _translated = unsafe { TranslateMessage(&raw const msg) };
            unsafe { DispatchMessageW(&raw const msg) };
        }

        if last_alert.elapsed() >= ALERT_INTERVAL {
            let alerts_sent = send_controller_alerts(&mut tray_icon);
            if alerts_sent > 0 {
                last_alert = Instant::now();
            }
        }

        std::thread::sleep(Duration::from_millis(100));
    }
}
