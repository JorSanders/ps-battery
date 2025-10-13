use crate::ps_battery::controller::transport::USB_REPORT_SIZE;
use crate::ps_battery::log::{log_error_with, log_info_with};
use hidapi::HidDevice;

const USB_BATTERY_OFFSET: usize = 53;
const BLUETOOTH_BATTERY_OFFSET: usize = 54;
const BLUETOOTH_CHARGE_FLAG_INDEX: usize = 55;
const FEATURE_IDS_USB: &[u8] = &[0x02, 0x05, 0x09];

const MASK_LOW_NIBBLE: u8 = 0b0000_1111;
const MASK_HIGH_NIBBLE: u8 = 0b1111_0000;
const MASK_CHARGING_FLAG: u8 = 0b0001_0000;

pub struct ParseBatteryAndChargingArgs<'a> {
    pub device: &'a HidDevice,
    pub buffer: &'a [u8],
    pub is_bluetooth: bool,
    pub should_log: bool,
}

pub fn parse_battery_and_charging(args: &ParseBatteryAndChargingArgs) -> (u8, bool) {
    let report_id = args.buffer.get(0).copied().unwrap_or(0);

    if args.should_log {
        log_info_with(
            "Controller buffer dump",
            format!(
                "len={} bytes={}",
                args.buffer.len(),
                args.buffer
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        );

        log_info_with(
            "Detected report type",
            format!(
                "report_id=0x{:02X} ({})",
                report_id,
                if report_id == 0x31 {
                    "DualSense new format"
                } else {
                    "Legacy format"
                }
            ),
        );
    }

    if !args.is_bluetooth {
        if USB_BATTERY_OFFSET >= args.buffer.len() {
            return (0, false);
        }

        let raw = args.buffer[USB_BATTERY_OFFSET];
        let level = (raw & MASK_LOW_NIBBLE).min(0x0A);
        let pct = level.saturating_mul(10);

        let mut is_charging = false;
        if let Some(flag) = get_usb_charging_flag(args.device, args.should_log) {
            is_charging = flag;
        }

        if args.should_log {
            log_info_with(
                "USB battery decode",
                format!(
                    "idx={} raw=0x{:02X} level(low)={} -> pct={} charging={}",
                    USB_BATTERY_OFFSET, raw, level, pct, is_charging
                ),
            );
        }

        return (pct, is_charging);
    }

    if BLUETOOTH_BATTERY_OFFSET >= args.buffer.len() {
        return (0, false);
    }

    let raw_battery = args.buffer[BLUETOOTH_BATTERY_OFFSET];
    let raw_charge = args
        .buffer
        .get(BLUETOOTH_CHARGE_FLAG_INDEX)
        .copied()
        .unwrap_or(0);

    let level = ((raw_battery & MASK_HIGH_NIBBLE) >> 4).min(0x0A);
    let pct = level.saturating_mul(10);
    let is_charging = (raw_charge & MASK_CHARGING_FLAG) != 0;

    if args.should_log {
        log_info_with(
            "BT battery decode",
            format!(
                "idx={} raw_battery=0x{:02X} level(high)={} -> pct={} raw_charge=0x{:02X} charging={}",
                BLUETOOTH_BATTERY_OFFSET, raw_battery, level, pct, raw_charge, is_charging
            ),
        );
    }

    (pct, is_charging)
}

fn get_usb_charging_flag(device: &HidDevice, should_log: bool) -> Option<bool> {
    let candidate_lengths = [USB_REPORT_SIZE, USB_REPORT_SIZE + 1];

    for &len in &candidate_lengths {
        for &rid in FEATURE_IDS_USB {
            let mut buf = vec![0u8; len];
            buf[0] = rid;

            match device.get_feature_report(&mut buf) {
                Ok(_) => {
                    let b4 = *buf.get(4).unwrap_or(&0);
                    let b5 = *buf.get(5).unwrap_or(&0);
                    let charging_flag =
                        (b4 & MASK_CHARGING_FLAG) != 0 || (b5 & MASK_CHARGING_FLAG) != 0;
                    return Some(charging_flag);
                }
                Err(e) => {
                    if should_log {
                        log_error_with(
                            "USB get_feature_report failed",
                            format!("id=0x{:02X}, len={}, err={}", rid, len, e),
                        );
                    }
                }
            }
        }
    }

    None
}
