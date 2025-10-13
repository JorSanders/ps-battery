use hidapi::HidApi;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

pub(super) static HID_API: OnceLock<Mutex<HidApi>> = OnceLock::new();
pub(super) static LAST_ALERT_TIMES: OnceLock<Mutex<HashMap<String, Instant>>> = OnceLock::new();
pub(super) static LOG_TIMER: OnceLock<Mutex<Instant>> = OnceLock::new();

pub fn should_log(now: Instant, interval: Duration) -> bool {
    let timer = LOG_TIMER.get_or_init(|| Mutex::new(now - interval));
    let mut last = timer.lock().expect("log timer poisoned");
    if now.duration_since(*last) >= interval {
        *last = now;
        true
    } else {
        false
    }
}
