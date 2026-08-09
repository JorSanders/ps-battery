#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ps_battery;
use crate::ps_battery::controller_store::get_generation;
use crate::ps_battery::poll_controllers::{poll_controllers, wait_for_next_poll};
use crate::ps_battery::send_controller_alerts::send_controller_alerts;
use crate::ps_battery::show_error_message_box::show_error_message_box;
use crate::ps_battery::tray::{add_tray_icon, create_hidden_window};
use hidapi::HidApi;
use std::time::{Duration, Instant};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage, WM_QUIT,
};

const ALERT_INTERVAL: Duration = Duration::from_secs(300);

fn main() {
    ps_battery::logger::init();
    ps_battery::tray::autostart::init();

    let hidden_window = create_hidden_window();
    let tray_icon = add_tray_icon(hidden_window);

    // HidApi::new can take a long time on some machines, so it runs off the
    // main thread to keep the tray responsive while it initializes.
    std::thread::spawn(move || {
        log_info!("initializing hidapi");
        let mut hid_api = match HidApi::new() {
            Ok(api) => api,
            Err(e) => {
                log_err!("Failed to initialize hidapi: {e}");
                show_error_message_box(
                    "PS Battery could not access HID devices and will now close.",
                );
                std::process::exit(1);
            }
        };
        log_info!("initialized hidapi");
        log_info!("hid device count {}", hid_api.device_list().count());

        loop {
            poll_controllers(&mut hid_api);
            wait_for_next_poll();
        }
    });

    // Not a backdated Instant: on Windows it counts from boot, so subtracting
    // the interval underflows during the first five minutes of uptime.
    let mut last_alert: Option<Instant> = None;
    let mut last_alerted_generation = 0;

    loop {
        let mut msg = MSG::default();
        while unsafe { PeekMessageW(&raw mut msg, None, 0, 0, PM_REMOVE).as_bool() } {
            if msg.message == WM_QUIT {
                return;
            }
            let _translated = unsafe { TranslateMessage(&raw const msg) };
            unsafe { DispatchMessageW(&raw const msg) };
        }

        // Sending nothing leaves last_alert alone so a newly low controller
        // alerts at once, so gate on fresh data or this runs every 100ms.
        let generation = get_generation();
        let interval_passed = last_alert.is_none_or(|sent| sent.elapsed() >= ALERT_INTERVAL);
        if generation != last_alerted_generation && interval_passed {
            last_alerted_generation = generation;
            if send_controller_alerts(&tray_icon) {
                last_alert = Some(Instant::now());
            }
        }

        std::thread::sleep(Duration::from_millis(100));
    }
}
