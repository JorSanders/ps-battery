#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ps_battery;
use crate::ps_battery::poll_controllers::{ALERT_INTERVAL, poll_controllers};
use crate::ps_battery::tray::{add_tray_icon, create_hidden_window};
use hidapi::HidApi;
use tokio::task::LocalSet;
use tokio::time::{Duration, Instant, sleep};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage, WM_QUIT,
};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let local = LocalSet::new();

    local
        .run_until(async {
            let hidden_window = create_hidden_window();
            let mut tray_icon = add_tray_icon(hidden_window);

            tokio::task::spawn_local(async move {
                println!(" -> tokio thread");

                let mut last_alert_sent = Instant::now()
                    .checked_sub(ALERT_INTERVAL)
                    .unwrap_or_else(Instant::now);

                println!(" -> initializing hidapi");
                let mut hid_api = HidApi::new().expect("Failed to initialize hidapi");
                println!(" -> initialized hidapi");
                println!(" -> hid device count {}", hid_api.device_list().count());

                loop {
                    println!(" -> poll");
                    poll_controllers(&mut hid_api, &mut tray_icon, &mut last_alert_sent);
                    sleep(Duration::from_millis(3000)).await;
                }
            });

            loop {
                println!(" -> loop");
                let mut msg = MSG::default();
                while unsafe { PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() } {
                    if msg.message == WM_QUIT {
                        return;
                    }
                    let _translated = unsafe { TranslateMessage(&msg) };
                    unsafe { DispatchMessageW(&msg) };
                }
                sleep(Duration::from_millis(100)).await;
            }
        })
        .await;
}
