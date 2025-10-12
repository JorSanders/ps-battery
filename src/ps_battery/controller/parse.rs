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

pub struct ParseBatteryAndChargingArgs<'a> {
    pub device: &'a HidDevice,
    pub buffer: &'a [u8],
    pub is_bluetooth: bool,
    pub should_log: bool,
}

pub fn parse_battery_and_charging(args: &ParseBatteryAndChargingArgs) -> (u8, bool) {
    let battery_offset: usize;

    if args.is_bluetooth {
        battery_offset = BLUETOOTH_BATTERY_OFFSET;
    } else {
        battery_offset = USB_BATTERY_OFFSET;
    }

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

        let mut feature = [0u8; USB_REPORT_SIZE];
        match args.device.get_feature_report(&mut feature) {
            Ok(_) => {
                let charging_flag = (feature[4] & 0x10) != 0 || (feature[5] & 0x10) != 0;
                if args.should_log {
                    log_info_with(
                        "USB feature bytes [4],[5]",
                        format!("{:02X},{:02X}", feature[4], feature[5]),
                    );
                    log_info_with("USB charging", charging_flag);
                }
                is_charging = charging_flag;
            }
            Err(e) => {
                if args.should_log {
                    log_error_with("USB get_feature_report failed", e);
                }
            }
        }
    }

    (battery_percent, is_charging)
}
