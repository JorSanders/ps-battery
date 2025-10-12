use crate::{controller::*, sound::*, tray::*};
use hidapi::HidApi;
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use windows::Win32::UI::Shell::NOTIFYICONDATAW;

const SONY_VID: u16 = 0x054C;
const SONY_PIDS: [u16; 4] = [0x0CE6, 0x0DF2, 0x05C4, 0x09CC];

static HID: OnceLock<Mutex<HidApi>> = OnceLock::new();
static LAST_ALERTS: OnceLock<Mutex<HashMap<String, Instant>>> = OnceLock::new();
static LAST_SEEN: OnceLock<Mutex<HashMap<String, (u8, bool)>>> = OnceLock::new();
static LOG_TIMER: OnceLock<Mutex<Instant>> = OnceLock::new();
static POLL_TIMER: OnceLock<Mutex<Instant>> = OnceLock::new();

const ALERT_INTERVAL: Duration = Duration::from_secs(300);
const LOG_INTERVAL: Duration = Duration::from_secs(10);
const POLL_INTERVAL: Duration = Duration::from_secs(1);

pub fn check_controllers(nid: &mut NOTIFYICONDATAW) {
    let now = Instant::now();
    let log_timer = LOG_TIMER.get_or_init(|| Mutex::new(now - LOG_INTERVAL));
    let mut last_log = log_timer.lock().unwrap();
    let should_log = now.duration_since(*last_log) >= LOG_INTERVAL;

    let poll_timer = POLL_TIMER.get_or_init(|| Mutex::new(now - POLL_INTERVAL));
    let mut last_poll = poll_timer.lock().unwrap();
    if now.duration_since(*last_poll) < POLL_INTERVAL {
        return;
    }
    *last_poll = now;

    let hid = HID.get_or_init(|| {
        let api = HidApi::new().expect("hidapi init failed");
        Mutex::new(api)
    });
    let mut api = hid.lock().unwrap();
    let _ = api.refresh_devices();

    let alerts = LAST_ALERTS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut alerts_lock = alerts.lock().unwrap();
    let cache = LAST_SEEN.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache_lock = cache.lock().unwrap();

    let mut statuses = Vec::new();

    for device_info in api.device_list() {
        if device_info.vendor_id() != SONY_VID || !SONY_PIDS.contains(&device_info.product_id()) {
            continue;
        }

        let pid = device_info.product_id();
        let name = device_info
            .product_string()
            .unwrap_or("Unknown")
            .to_string();
        let path_str = device_info.path().to_string_lossy();
        let is_bt = path_str
            .to_ascii_uppercase()
            .contains("00001124-0000-1000-8000-00805F9B34FB");

        let buf_size = if is_bt { 78 } else { 64 };

        let device = match device_info.open_device(&api) {
            Ok(d) => d,
            Err(_) => {
                if let Some((b, c)) = cache_lock.get(&name) {
                    statuses.push(ControllerStatus {
                        name: name.clone(),
                        battery: *b,
                        charging: *c,
                    });
                }
                continue;
            }
        };

        let _ = device.set_blocking_mode(false);

        let mut buf = vec![0u8; buf_size];
        let n = device.read_timeout(&mut buf, 20).unwrap_or(0);

        if n == 0 {
            if let Some((prev_batt, prev_chg)) = cache_lock.get(&name) {
                statuses.push(ControllerStatus {
                    name: name.clone(),
                    battery: *prev_batt,
                    charging: *prev_chg,
                });
            }
            continue;
        }

        let (mut percentage, mut charging) = (0u8, false);

        if n > 55 && is_bt {
            let raw = buf[54];
            let state = buf[55];
            let level = raw & 0x0F;

            percentage = match level {
                0 => 0,
                1 => 10,
                2 => 20,
                3 => 30,
                4 => 40,
                5 => 50,
                6 => 60,
                7 => 70,
                8 => 80,
                9 | 0x0A..=0x0F => 90,
                _ => 0,
            };

            if pid == 0x0CE6 {
                charging = (raw & 0x10) != 0 || (state & 0x10) != 0;
            } else if pid == 0x0DF2 {
                charging = (state & 0x08) != 0 || (state & 0x10) != 0 || (raw & 0x20) != 0;
            }
        } else {
            let mut feat = [0u8; 64];
            if let Ok(_nf) = device.get_feature_report(&mut feat) {
                let raw = feat[4];
                let level = raw & 0x0F;
                percentage = level * 10;
                charging = (feat[4] & 0x10) != 0 || (feat[5] & 0x10) != 0;
            }
        }

        cache_lock.insert(name.clone(), (percentage, charging));

        statuses.push(ControllerStatus {
            name: name.clone(),
            battery: percentage,
            charging,
        });

        if !charging {
            let due = match alerts_lock.get(&name) {
                Some(last) => now.duration_since(*last) >= ALERT_INTERVAL,
                None => true,
            };
            if due {
                let alert = if percentage <= 10 {
                    Some(AlertSound::Critical)
                } else if percentage <= 20 {
                    Some(AlertSound::Exclamation)
                } else if percentage <= 30 {
                    Some(AlertSound::Notify)
                } else {
                    None
                };
                if let Some(sound) = alert {
                    play_sound(sound);
                    unsafe {
                        show_balloon(nid, &name, &format!("Battery at {}%", percentage));
                    }
                    alerts_lock.insert(name.clone(), now);
                }
            }
        }
    }

    if should_log {
        for c in &statuses {
            println!(
                "{}: {}% ({})",
                c.name,
                c.battery,
                if c.charging {
                    "charging"
                } else {
                    "not charging"
                }
            );
        }
        let _ = io::stdout().flush();
        *last_log = now;
    }

    set_controllers(statuses);
}
