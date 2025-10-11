mod hid;
mod ps5_bt;
mod sound;
mod toast;
mod tray;

use hid::*;
use std::{thread, time::Duration};
use tray::*;

use crate::ps5_bt::list_paired_bluetooth_devices;

const POLL_INTERVAL_SECS: u64 = 300;

fn main() {
    let class_name = windows::core::HSTRING::from("PSBatteryHiddenWindow");
    let hwnd = unsafe { create_hidden_window(&class_name) };
    let mut nid = unsafe { add_tray_icon(hwnd) };

    list_paired_bluetooth_devices();

    loop {
        check_controllers(&mut nid);
        thread::sleep(Duration::from_secs(POLL_INTERVAL_SECS));
    }
}
