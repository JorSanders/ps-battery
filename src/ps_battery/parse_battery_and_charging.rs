use crate::ps_battery::{
    get_controller_info::ConnectionType,
    get_playstation_controllers::{
        DUALSENSE_EDGE_PRODUCT_ID, DUALSENSE_PRODUCT_ID, DUALSHOCK_GEN_1_PRODUCT_ID,
        DUALSHOCK_GEN_2_PRODUCT_ID,
    },
};

const USB_BATTERY_BYTE_INDEX: usize = 53;
const BLUETOOTH_BATTERY_BYTE_INDEX: usize = 54;
const BLUETOOTH_CHARGE_BYTE_INDEX: usize = 55;
const DUALSHOCK_BATTERY_BYTE_INDEX: usize = 29;

const MASK_LOW_NIBBLE: u8 = 0b0000_1111;
const MASK_HIGH_NIBBLE: u8 = 0b1111_0000;
const MASK_CHARGING_FLAG: u8 = 0b0001_0000;
const MASK_FULLY_CHARGED: u8 = 0b0000_0010;

pub struct ParseBatteryAndChargingArgs<'a> {
    pub buffer: &'a [u8],
    pub connection_type: ConnectionType,
    pub product_id: u16,
}

pub fn parse_battery_and_charging(args: &ParseBatteryAndChargingArgs) -> (u8, bool) {
    let (battery_byte_index, charge_byte_index) = match args.product_id {
        DUALSENSE_PRODUCT_ID | DUALSENSE_EDGE_PRODUCT_ID => {
            if args.connection_type == ConnectionType::Bluetooth {
                (BLUETOOTH_BATTERY_BYTE_INDEX, BLUETOOTH_CHARGE_BYTE_INDEX)
            } else {
                (USB_BATTERY_BYTE_INDEX, USB_BATTERY_BYTE_INDEX)
            }
        }
        DUALSHOCK_GEN_1_PRODUCT_ID | DUALSHOCK_GEN_2_PRODUCT_ID => {
            (DUALSHOCK_BATTERY_BYTE_INDEX, DUALSHOCK_BATTERY_BYTE_INDEX)
        }
        _ => return (0, false),
    };

    if battery_byte_index >= args.buffer.len()
        || charge_byte_index >= args.buffer.len()
        || battery_byte_index == 0
    {
        return (u8::MAX, false);
    }

    let battery_byte = args.buffer[battery_byte_index];
    let charging_byte = args.buffer[charge_byte_index];
    let battery_level_nibble = battery_byte & MASK_LOW_NIBBLE;
    let battery_state_nibble = (battery_byte & MASK_HIGH_NIBBLE) >> 4;

    let is_fully_charged = (battery_state_nibble & MASK_FULLY_CHARGED) != 0;
    let is_charging = if args.connection_type == ConnectionType::Usb {
        true
    } else {
        (charging_byte & MASK_CHARGING_FLAG) != 0
    };

    let battery_percent = if is_fully_charged {
        100
    } else {
        battery_level_nibble * 10
    };

    println!(
        " -> battery_byte_index={}, battery_byte=0b{:08b}, battery_level_nibble=0b{:04b}, battery_state_nibble=0b{:04b}, battery_raw={}, battery_percent={}, is_fully_charged={}",
        battery_byte_index,
        battery_byte,
        battery_level_nibble,
        battery_state_nibble,
        battery_level_nibble,
        battery_percent,
        is_fully_charged
    );
    println!(
        " -> charge_byte_index={}, charging_byte=0b{:08b}, is_charging={}",
        charge_byte_index, charging_byte, is_charging
    );

    (battery_percent, is_charging)
}
