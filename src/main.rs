#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ps_battery;
use crate::ps_battery::poll_controllers::poll_controllers;
use crate::ps_battery::send_controller_alerts;
use crate::ps_battery::tray::{add_tray_icon, create_hidden_window};
use crate::send_controller_alerts::send_controller_alerts;
use hidapi::HidApi;
use tokio::time::{Duration, Instant, sleep};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage, WM_QUIT,
};

const ALERT_INTERVAL: Duration = Duration::from_secs(300);
const POLL_INTERVAL: Duration = Duration::from_secs(10);

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let hidden_window = create_hidden_window();
    let mut tray_icon = add_tray_icon(hidden_window);

    tokio::spawn(async move {
        println!(" -> initializing hidapi");
        let mut hid_api = HidApi::new().expect("Failed to initialize hidapi");
        println!(" -> initialized hidapi");
        println!(" -> hid device count {}", hid_api.device_list().count());

        loop {
            poll_controllers(&mut hid_api);
            sleep(POLL_INTERVAL).await;
        }
    });

    let mut last_alert = Instant::now()
        .checked_sub(ALERT_INTERVAL)
        .unwrap_or_else(Instant::now);

    loop {
        let mut msg = MSG::default();
        while unsafe { PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() } {
            if msg.message == WM_QUIT {
                return;
            }
            let _translated = unsafe { TranslateMessage(&msg) };
            unsafe { DispatchMessageW(&msg) };
        }

        if last_alert.elapsed() >= ALERT_INTERVAL {
            let alerts_sent = send_controller_alerts(&mut tray_icon);
            if alerts_sent > 0 {
                last_alert = Instant::now();
            }
        }

        sleep(Duration::from_millis(100)).await;
    }
}
