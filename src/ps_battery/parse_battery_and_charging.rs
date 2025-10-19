use std::u8;

use crate::ps_battery::get_playstation_controllers::{
    DUALSENSE_EDGE_PRODUCT_ID, DUALSENSE_PRODUCT_ID, DUALSHOCK_GEN_1_PRODUCT_ID,
    DUALSHOCK_GEN_2_PRODUCT_ID,
};

const USB_BATTERY_BYTE_INDEX: usize = 53;
const BLUETOOTH_BATTERY_BYTE_INDEX: usize = 54;
const DUALSHOCK_BATTERY_BYTE_INDEX: usize = 29;

const MASK_LOW_NIBBLE: u8 = 0b0000_1111;
const MASK_HIGH_NIBBLE: u8 = 0b1111_0000;
const MASK_CHARGING_FLAG: u8 = 0b0000_0001;
const MASK_FULLY_CHARGED: u8 = 0b0000_0010;

pub struct ParseBatteryAndChargingArgs<'a> {
    pub buffer: &'a [u8],
    pub is_bluetooth: bool,
    pub product_id: u16,
}

pub struct BatteryAndChargingResult {
    pub battery_percent: u8,
    pub is_charging: bool,
    pub is_fully_charged: bool,
}

pub fn parse_battery_and_charging(args: &ParseBatteryAndChargingArgs) -> BatteryAndChargingResult {
    let battery_byte_index = match args.product_id {
        DUALSENSE_PRODUCT_ID | DUALSENSE_EDGE_PRODUCT_ID => {
            if args.is_bluetooth {
                BLUETOOTH_BATTERY_BYTE_INDEX
            } else {
                USB_BATTERY_BYTE_INDEX
            }
        }
        DUALSHOCK_GEN_1_PRODUCT_ID | DUALSHOCK_GEN_2_PRODUCT_ID => DUALSHOCK_BATTERY_BYTE_INDEX,
        _ => {
            eprintln!(" !! Product_id not known");

            return BatteryAndChargingResult {
                battery_percent: u8::MAX,
                is_charging: false,
                is_fully_charged: false,
            };
        }
    };

    if battery_byte_index >= args.buffer.len() {
        eprintln!(" !! Battery index out of buffer bounds");
        return BatteryAndChargingResult {
            battery_percent: u8::MAX,
            is_charging: false,
            is_fully_charged: false,
        };
    }

    let battery_byte = args.buffer[battery_byte_index];
    let battery_level_nibble = battery_byte & MASK_LOW_NIBBLE;
    let battery_state_nibble = (battery_byte & MASK_HIGH_NIBBLE) >> 4;

    let is_charging = (battery_state_nibble & MASK_CHARGING_FLAG) != 0;
    let is_fully_charged = (battery_state_nibble & MASK_FULLY_CHARGED) != 0;

    let battery_percent = if is_fully_charged {
        100
    } else {
        battery_level_nibble * 10
    };

    println!(
        " -> battery_byte_index={}, battery_byte=0b{:08b}, battery_level_nibble=0b{:04b}, battery_state_nibble=0b{:04b}, battery_raw={}, battery_percent={}, is_charging={}, is_fully_charged={}",
        battery_byte_index,
        battery_byte,
        battery_level_nibble,
        battery_state_nibble,
        battery_level_nibble,
        battery_percent,
        is_charging,
        is_fully_charged
    );

    BatteryAndChargingResult {
        battery_percent,
        is_charging,
        is_fully_charged,
    }
}
