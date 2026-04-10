use crate::{log_err, log_info};
use crate::ps_battery::controller_store::{ControllerStatus, get_controllers, set_controllers};
use crate::ps_battery::get_playstation_controllers::get_playstation_controllers;
use crate::ps_battery::parse_battery_and_charging::parse_battery_and_charging;
use crate::ps_battery::read_controller_input_report::{open_device, read_controller_input_report};
use hidapi::HidApi;
use std::sync::{Condvar, Mutex, OnceLock};
use std::time::Duration;

pub const POLL_INTERVAL: Duration = Duration::from_secs(60);

const BLUETOOTH_GUID_SUBSTRING: &str = "00001124-0000-1000-8000-00805F9B34FB";

static POLL_SIGNAL: OnceLock<(Mutex<bool>, Condvar)> = OnceLock::new();

fn poll_signal() -> &'static (Mutex<bool>, Condvar) {
    POLL_SIGNAL.get_or_init(|| (Mutex::new(false), Condvar::new()))
}

/// Wakes the polling thread early, triggering an immediate poll.
pub fn request_poll() {
    let (lock, cvar) = poll_signal();
    *lock.lock().expect("poll signal poisoned") = true;
    cvar.notify_one();
}

/// Blocks until either `POLL_INTERVAL` elapses or `request_poll` is called.
pub fn wait_for_next_poll() {
    let (lock, cvar) = poll_signal();
    let guard = lock.lock().expect("poll signal poisoned");
    let (mut guard, _) = cvar
        .wait_timeout(guard, POLL_INTERVAL)
        .expect("condvar wait failed");
    *guard = false;
}

pub fn poll_controllers(hid_api: &mut HidApi) {
    let controllers = get_playstation_controllers(hid_api);

    if controllers.is_empty() {
        set_controllers(Vec::new());
        log_info!("No controllers connected");
        return;
    }

    let mut status_list: Vec<ControllerStatus> = Vec::new();
    let previous_controllers = get_controllers();

    log_info!("-------------------------------");

    for controller_info in controllers {
        let path = controller_info.path().to_string_lossy().into_owned();
        let is_bluetooth = path.to_ascii_uppercase().contains(BLUETOOTH_GUID_SUBSTRING);
        let name = controller_info.product_string().unwrap_or("Unknown").to_string();
        let product_id = controller_info.product_id();

        log_info!(
            "controller: name={}, is_bluetooth={}, product_id=0x{:02X}",
            name,
            is_bluetooth,
            product_id,
        );
        log_info!("path='{}'", path);

        let Some(hid_device) = open_device(hid_api, &controller_info) else { continue };

        let buffer = read_controller_input_report(&hid_device, &name, is_bluetooth, product_id);

        if buffer.is_empty() || buffer[0] == 0 {
            let Some(previous_controller) =
                previous_controllers.iter().find(|c| c.path == path)
            else {
                log_err!("Buffer is empty and device not found in previous results");
                continue;
            };

            if previous_controller.last_read_failed {
                log_err!("Buffer is empty and last read also failed");
                continue;
            }

            log_err!("Buffer is empty, using last result");

            status_list.push(ControllerStatus {
                name: previous_controller.name.clone(),
                battery_percent: previous_controller.battery_percent,
                is_charging: previous_controller.is_charging,
                is_fully_charged: previous_controller.is_fully_charged,
                is_bluetooth: previous_controller.is_bluetooth,
                path: previous_controller.path.clone(),
                last_read_failed: true,
            });

            continue;
        }

        let Some(battery_result) =
            parse_battery_and_charging(&buffer, is_bluetooth, product_id)
        else {
            log_err!("Failed to parse battery data for '{}'", name);
            continue;
        };

        status_list.push(ControllerStatus {
            name,
            battery_percent: battery_result.battery_percent,
            is_charging: battery_result.is_charging,
            is_fully_charged: battery_result.is_fully_charged,
            is_bluetooth,
            path,
            last_read_failed: false,
        });
    }

    if !status_list.is_empty() {
        set_controllers(status_list);
    }
    // else: HID found controllers but all reads failed — keep old state
}
