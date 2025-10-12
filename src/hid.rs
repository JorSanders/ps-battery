use crate::{controller::*, sound::*, tray::*};
use hidapi::HidApi;
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};
use windows::Win32::UI::Shell::NOTIFYICONDATAW;

const SONY_VID: u16 = 0x054C;
const SONY_PIDS: [u16; 4] = [0x0CE6, 0x0DF2, 0x05C4, 0x09CC];
const USB_BATTERY_OFFSET: usize = 53;
const BT_BATTERY_OFFSET: usize = 54;

static HID: OnceLock<Mutex<HidApi>> = OnceLock::new();
static LAST_ALERTS: OnceLock<Mutex<HashMap<String, Instant>>> = OnceLock::new();
static LAST_SEEN: OnceLock<Mutex<HashMap<String, (u8, bool)>>> = OnceLock::new();
static LOG_TIMER: OnceLock<Mutex<Instant>> = OnceLock::new();

const ALERT_INTERVAL: Duration = Duration::from_secs(300);
const LOG_INTERVAL: Duration = Duration::from_secs(10);

pub fn check_controllers(nid: &mut NOTIFYICONDATAW) {
    let now = Instant::now();
    let log_timer = LOG_TIMER.get_or_init(|| Mutex::new(now - LOG_INTERVAL));
    let mut last_log = log_timer.lock().unwrap();
    let should_log = now.duration_since(*last_log) >= LOG_INTERVAL;

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

        let name = device_info
            .product_string()
            .unwrap_or("Unknown")
            .to_string();
        let path_str = device_info.path().to_string_lossy();
        let is_bt = path_str
            .to_ascii_uppercase()
            .contains("00001124-0000-1000-8000-00805F9B34FB");

        let offset = if is_bt {
            BT_BATTERY_OFFSET
        } else {
            USB_BATTERY_OFFSET
        };
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
        let mut n = device.read_timeout(&mut buf, 200).unwrap_or(0);

        if n > 0 && buf[0] == 0x01 && is_bt {
            if should_log {
                println!(
                    "{}: truncated BT report detected, sending calibration (0x05)",
                    name
                );
            }
            let mut cal = vec![0u8; buf_size];
            cal[0] = 0x05;
            let _ = device.get_feature_report(&mut cal);
            thread::sleep(Duration::from_millis(500));
            n = device.read_timeout(&mut buf, 200).unwrap_or(0);
        }

        if n == 0 {
            continue;
        }

        if should_log {
            println!("{}: read {} bytes", name, n);
            println!(
                "{} buffer: {:02X?}",
                name,
                &buf[..std::cmp::min(buf.len(), 64)]
            );
        }

        let (percentage, charging) = if offset >= buf.len() {
            match cache_lock.get(&name) {
                Some((b, c)) => (*b, *c),
                None => continue,
            }
        } else {
            let raw = buf[offset];
            let percentage = if is_bt {
                let level = raw & 0x0F;
                let state = (raw & 0xF0) >> 4;
                if state == 0x02 {
                    100
                } else {
                    (level as u32 * 100 / 0x0A) as u8
                }
            } else {
                (raw & 0x0F) * 10
            };
            let charging = if is_bt {
                let cond = buf.len() > 55 && (buf[55] & 0x10) != 0;
                if should_log {
                    println!(
                        "{}: BT raw[55]={:02X} charging={}",
                        name,
                        if buf.len() > 55 { buf[55] } else { 0 },
                        cond
                    );
                }
                cond
            } else {
                let mut feat = [0u8; 64];
                let result = device.get_feature_report(&mut feat);
                match result {
                    Ok(_) => {
                        let cond = (feat[4] & 0x10) != 0 || (feat[5] & 0x10) != 0;
                        if should_log {
                            println!(
                                "{}: USB feat[4]={:02X} feat[5]={:02X} charging={}",
                                name, feat[4], feat[5], cond
                            );
                        }
                        cond
                    }
                    Err(e) => {
                        if should_log {
                            println!("{}: get_feature_report failed: {}", name, e);
                        }
                        false
                    }
                }
            };
            cache_lock.insert(name.clone(), (percentage, charging));
            (percentage, charging)
        };

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
