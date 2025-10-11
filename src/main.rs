mod hid;
mod sound;
mod toast;
mod tray;

use hid::*;
use std::{thread, time::Duration};
use tray::*;

const POLL_INTERVAL_SECS: u64 = 300;

fn main() {
    let class_name = windows::core::HSTRING::from("PSBatteryHiddenWindow");
    let hwnd = unsafe { create_hidden_window(&class_name) };
    let mut nid = unsafe { add_tray_icon(hwnd) };

    loop {
        check_controllers(&mut nid);
        thread::sleep(Duration::from_secs(POLL_INTERVAL_SECS));
    }
}
