use crate::ps_battery::get_controller_info::TransportLabel;

const USB_BATTERY_INDEX: usize = 53;
const BLUETOOTH_BATTERY_INDEX: usize = 54;
const BLUETOOTH_CHARGE_FLAG_INDEX: usize = 55;

const MASK_LOW_NIBBLE: u8 = 0b0000_1111;
const MASK_HIGH_NIBBLE: u8 = 0b1111_0000;
const MASK_CHARGING_FLAG: u8 = 0b0001_0000;
const MASK_FULLY_CHARGED: u8 = 0b0000_0010;

pub struct ParseBatteryAndChargingArgs<'a> {
    pub buffer: &'a [u8],
    pub transport_label: TransportLabel,
}

pub fn parse_battery_and_charging(args: &ParseBatteryAndChargingArgs) -> (u8, bool) {
    let battery_offset: usize = if args.transport_label == TransportLabel::Bluetooth {
        BLUETOOTH_BATTERY_INDEX
    } else {
        USB_BATTERY_INDEX
    };

    if battery_offset >= args.buffer.len() {
        return (0, false);
    }

    let battery_byte = args.buffer[battery_offset];
    let battery_level_binary = battery_byte & MASK_LOW_NIBBLE;
    let battery_state_binary = (battery_byte & MASK_HIGH_NIBBLE) >> 4;
    let charging_byte = args.buffer[BLUETOOTH_CHARGE_FLAG_INDEX];
    let is_fully_charged = (battery_state_binary & MASK_FULLY_CHARGED) != 0;

    let battery_percent = if is_fully_charged {
        100
    } else {
        battery_level_binary * 10
    };

    let is_charging = if args.transport_label == TransportLabel::Bluetooth {
        (charging_byte & MASK_CHARGING_FLAG) != 0
    } else {
        true
    };

    println!("Buffer: {:02X?}", args.buffer);
    println!(
        "battery_byte=0x{:02X}, battery_level_binary={}, battery_state_binary={}, battery_percent={}, is_fully_charged={}",
        battery_byte, battery_level_binary, battery_state_binary, battery_percent, is_fully_charged
    );
    println!(
        "charging_byte=0x{:02X}, is_charging={}",
        charging_byte, is_charging
    );

    (battery_percent, is_charging)
}
