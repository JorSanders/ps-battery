use crate::ps_battery::controller::transport::USB_REPORT_SIZE;
use crate::ps_battery::log::{log_error_with, log_info_with};
use hidapi::HidDevice;

const USB_BATTERY_OFFSET: usize = 53;
const BLUETOOTH_BATTERY_OFFSET: usize = 54;
const BLUETOOTH_CHARGE_FLAG_INDEX: usize = 55;

const BATTERY_LEVEL_MASK_LOW_NIBBLE: u8 = 0x0F;
const BATTERY_STATE_MASK_HIGH_NIBBLE: u8 = 0xF0;
const BATTERY_STATE_FULLY_CHARGED: u8 = 0x02;

const PERCENT_100: u8 = 100;
const PERCENT_STEP_USB: u8 = 10;
const PERCENT_MAX_LEVEL_BLUETOOTH: u8 = 0x0A;

const FEATURE_IDS_USB: &[u8] = &[0x02, 0x05, 0x09];

pub struct ParseBatteryAndChargingArgs<'a> {
    pub device: &'a HidDevice,
    pub buffer: &'a [u8],
    pub is_bluetooth: bool,
    pub should_log: bool,
}

pub fn parse_battery_and_charging(args: &ParseBatteryAndChargingArgs) -> (u8, bool) {
    let battery_offset: usize = if args.is_bluetooth {
        BLUETOOTH_BATTERY_OFFSET
    } else {
        USB_BATTERY_OFFSET
    };

    if battery_offset >= args.buffer.len() {
        return (0, false);
    }

    let raw = args.buffer[battery_offset];
    let battery_percent: u8;
    let mut is_charging: bool = false;

    if args.is_bluetooth {
        let level = raw & BATTERY_LEVEL_MASK_LOW_NIBBLE;
        let state = (raw & BATTERY_STATE_MASK_HIGH_NIBBLE) >> 4;

        if state == BATTERY_STATE_FULLY_CHARGED {
            battery_percent = PERCENT_100;
        } else {
            let calc = (level as u32 * 100 / PERCENT_MAX_LEVEL_BLUETOOTH as u32) as u8;
            battery_percent = calc.min(PERCENT_100);
        }

        if let Some(byte) = args.buffer.get(BLUETOOTH_CHARGE_FLAG_INDEX) {
            is_charging = (byte & 0x10) != 0;
        }
    } else {
        battery_percent = (raw & BATTERY_LEVEL_MASK_LOW_NIBBLE) * PERCENT_STEP_USB;

        if let Some(flag) = get_usb_charging_flag(args.device, args.should_log) {
            is_charging = flag;
        }
    }

    (battery_percent, is_charging)
}

fn get_usb_charging_flag(device: &HidDevice, should_log: bool) -> Option<bool> {
    // Windows/hidapi sometimes expects 64 (with ID in [0]) or 65 bytes.
    let candidate_lengths = [USB_REPORT_SIZE, USB_REPORT_SIZE + 1];

    for &len in &candidate_lengths {
        for &rid in FEATURE_IDS_USB {
            let mut buf = vec![0u8; len];
            buf[0] = rid;

            match device.get_feature_report(&mut buf) {
                Ok(n) => {
                    if should_log {
                        log_info_with(
                            "USB feature report ok",
                            format!("id=0x{:02X}, len_req={}, len_got={}", rid, len, n),
                        );
                    }

                    let b4 = *buf.get(4).unwrap_or(&0);
                    let b5 = *buf.get(5).unwrap_or(&0);
                    let charging_flag = (b4 & 0x10) != 0 || (b5 & 0x10) != 0;

                    if should_log {
                        log_info_with(
                            "USB feature bytes [4],[5]",
                            format!("{:02X},{:02X}", b4, b5),
                        );
                        log_info_with("USB charging", charging_flag);
                    }

                    return Some(charging_flag);
                }
                Err(e) => {
                    if should_log {
                        log_error_with(
                            "USB get_feature_report failed",
                            format!("id=0x{:02X}, len={}, err={}", rid, len, e),
                        );
                    }
                    continue;
                }
            }
        }
    }

    None
}
