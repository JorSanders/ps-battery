use crate::ps_battery::get_playstation_controllers::{
    DUALSENSE_EDGE_PRODUCT_ID, DUALSENSE_PRODUCT_ID, DUALSHOCK_GEN_1_PRODUCT_ID,
    DUALSHOCK_GEN_2_PRODUCT_ID,
};
use crate::{log_err, log_info};

/// Offsets of the status byte, counting the report id as byte 0. Each family
/// puts it at a fixed place inside its report body: 52 bytes in for the
/// DualSense and 29 for the DualShock. USB prefixes that body with a single id
/// byte, while Bluetooth prefixes it with the id plus one more byte on the
/// DualSense and the id plus two more on the DualShock. Taken from the layouts
/// in the Linux `hid-playstation` driver.
const USB_BATTERY_BYTE_INDEX: usize = 53;
const BLUETOOTH_BATTERY_BYTE_INDEX: usize = 54;
const DUALSHOCK_USB_BATTERY_BYTE_INDEX: usize = 30;
const DUALSHOCK_BLUETOOTH_BATTERY_BYTE_INDEX: usize = 32;

const MASK_LOW_NIBBLE: u8 = 0b0000_1111;
const MASK_HIGH_NIBBLE: u8 = 0b1111_0000;

/// One state, not a set of flags: the DualSense also reports too hot, too cold
/// and out of voltage range, which share bits with these three.
const BATTERY_STATE_DISCHARGING: u8 = 0x0;
const BATTERY_STATE_CHARGING: u8 = 0x1;
const BATTERY_STATE_FULLY_CHARGED: u8 = 0x2;

/// The DualShock spends the same nibble differently: only the low bit means
/// anything, and a full battery is a level of 11 rather than a state.
const DUALSHOCK_CABLE_CONNECTED_FLAG: u8 = 0b0001;
const DUALSHOCK_FULLY_CHARGED_LEVEL: u8 = 11;

pub struct BatteryAndChargingResult {
    pub battery_percent: u8,
    pub is_charging: bool,
    pub is_fully_charged: bool,
    pub unexpected_battery_data: bool,
}

#[derive(Clone, Copy)]
enum ControllerFamily {
    DualSense,
    DualShock,
}

pub fn parse_battery_and_charging(
    buffer: &[u8],
    is_bluetooth: bool,
    product_id: u16,
) -> Option<BatteryAndChargingResult> {
    let (family, battery_byte_index) = match product_id {
        DUALSENSE_PRODUCT_ID | DUALSENSE_EDGE_PRODUCT_ID => (
            ControllerFamily::DualSense,
            if is_bluetooth {
                BLUETOOTH_BATTERY_BYTE_INDEX
            } else {
                USB_BATTERY_BYTE_INDEX
            },
        ),
        DUALSHOCK_GEN_1_PRODUCT_ID | DUALSHOCK_GEN_2_PRODUCT_ID => (
            ControllerFamily::DualShock,
            if is_bluetooth {
                DUALSHOCK_BLUETOOTH_BATTERY_BYTE_INDEX
            } else {
                DUALSHOCK_USB_BATTERY_BYTE_INDEX
            },
        ),
        _ => {
            log_err!("Product_id 0x{:04X} not known", product_id);
            return None;
        }
    };

    if battery_byte_index >= buffer.len() {
        log_err!(
            "Battery index {} out of buffer bounds (len={})",
            battery_byte_index,
            buffer.len()
        );
        return None;
    }

    let battery_byte = buffer[battery_byte_index];
    let battery_level_nibble = battery_byte & MASK_LOW_NIBBLE;
    let battery_state_nibble = (battery_byte & MASK_HIGH_NIBBLE) >> 4;

    let (is_charging, is_fully_charged, unexpected_battery_data) = match family {
        ControllerFamily::DualSense => {
            let unknown_state = !matches!(
                battery_state_nibble,
                BATTERY_STATE_DISCHARGING | BATTERY_STATE_CHARGING | BATTERY_STATE_FULLY_CHARGED
            );
            if unknown_state {
                log_err!(
                    "battery_state_nibble 0x{:X} is not discharging, charging or fully charged (battery_byte=0b{:08b})",
                    battery_state_nibble,
                    battery_byte
                );
            }
            if battery_level_nibble > 10 {
                log_err!(
                    "battery_level_nibble {} out of expected 0-10 range (battery_byte=0b{:08b})",
                    battery_level_nibble,
                    battery_byte
                );
            }
            (
                battery_state_nibble == BATTERY_STATE_CHARGING,
                battery_state_nibble == BATTERY_STATE_FULLY_CHARGED,
                battery_level_nibble > 10 || unknown_state,
            )
        }
        ControllerFamily::DualShock => {
            let cable_connected = (battery_state_nibble & DUALSHOCK_CABLE_CONNECTED_FLAG) != 0;
            let fully_charged =
                cable_connected && battery_level_nibble >= DUALSHOCK_FULLY_CHARGED_LEVEL;
            if battery_level_nibble > DUALSHOCK_FULLY_CHARGED_LEVEL {
                log_err!(
                    "battery_level_nibble {} out of expected 0-11 range (battery_byte=0b{:08b})",
                    battery_level_nibble,
                    battery_byte
                );
            }
            (
                cable_connected && !fully_charged,
                fully_charged,
                battery_level_nibble > DUALSHOCK_FULLY_CHARGED_LEVEL,
            )
        }
    };

    let battery_percent = if is_fully_charged {
        100
    } else {
        (battery_level_nibble * 10).min(100)
    };

    log_info!(
        "battery_byte_index={}, battery_byte=0b{:08b}, battery_level_nibble=0b{:04b}, battery_state_nibble=0b{:04b}, battery_raw={}, battery_percent={}, is_charging={}, is_fully_charged={}",
        battery_byte_index,
        battery_byte,
        battery_level_nibble,
        battery_state_nibble,
        battery_level_nibble,
        battery_percent,
        is_charging,
        is_fully_charged
    );

    Some(BatteryAndChargingResult {
        battery_percent,
        is_charging,
        is_fully_charged,
        unexpected_battery_data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report_with_battery_byte(index: usize, battery_byte: u8) -> Vec<u8> {
        let mut buffer = vec![0u8; index + 1];
        buffer[index] = battery_byte;
        buffer
    }

    fn dualsense_bluetooth(battery_byte: u8) -> Option<BatteryAndChargingResult> {
        let buffer = report_with_battery_byte(BLUETOOTH_BATTERY_BYTE_INDEX, battery_byte);
        parse_battery_and_charging(&buffer, true, DUALSENSE_PRODUCT_ID)
    }

    #[test]
    fn reads_the_level_as_tens_of_a_percent() {
        let result = dualsense_bluetooth(0b0000_0101).expect("should parse");
        assert_eq!(result.battery_percent, 50);
        assert!(!result.is_charging);
        assert!(!result.is_fully_charged);
        assert!(!result.unexpected_battery_data);
    }

    #[test]
    fn reports_charging() {
        let result = dualsense_bluetooth(0b0001_0011).expect("should parse");
        assert_eq!(result.battery_percent, 30);
        assert!(result.is_charging);
        assert!(!result.is_fully_charged);
    }

    #[test]
    fn reports_fully_charged_as_one_hundred_percent() {
        let result = dualsense_bluetooth(0b0010_0000).expect("should parse");
        assert_eq!(result.battery_percent, 100);
        assert!(result.is_fully_charged);
        assert!(!result.is_charging);
    }

    /// 0xB is one of the temperature and voltage states. Read as bit flags it
    /// looks like charging and fully charged at once, which pins the reading
    /// to 100% and stops the controller from ever being reported as low.
    #[test]
    fn an_error_state_is_neither_charging_nor_full() {
        let result = dualsense_bluetooth(0b1011_0001).expect("should parse");
        assert!(!result.is_charging);
        assert!(!result.is_fully_charged);
        assert!(result.unexpected_battery_data);
        assert_eq!(result.battery_percent, 10);
    }

    #[test]
    fn a_level_above_ten_is_unexpected() {
        let result = dualsense_bluetooth(0b0000_1100).expect("should parse");
        assert!(result.unexpected_battery_data);
        assert_eq!(result.battery_percent, 100);
    }

    #[test]
    fn usb_and_bluetooth_read_different_bytes() {
        let mut buffer = vec![0u8; BLUETOOTH_BATTERY_BYTE_INDEX + 1];
        buffer[USB_BATTERY_BYTE_INDEX] = 0b0000_0010;
        buffer[BLUETOOTH_BATTERY_BYTE_INDEX] = 0b0000_1000;

        let over_usb =
            parse_battery_and_charging(&buffer, false, DUALSENSE_PRODUCT_ID).expect("should parse");
        let over_bluetooth =
            parse_battery_and_charging(&buffer, true, DUALSENSE_PRODUCT_ID).expect("should parse");

        assert_eq!(over_usb.battery_percent, 20);
        assert_eq!(over_bluetooth.battery_percent, 80);
    }

    /// The offsets come from the report layouts in the Linux `hid-playstation`
    /// driver: the status byte sits 29 bytes into a body that USB prefixes with
    /// one id byte and Bluetooth with three.
    #[test]
    fn the_dualshock_status_byte_is_where_the_report_layout_puts_it() {
        assert_eq!(DUALSHOCK_USB_BATTERY_BYTE_INDEX, 1 + 29);
        assert_eq!(DUALSHOCK_BLUETOOTH_BATTERY_BYTE_INDEX, 3 + 29);
        assert_eq!(USB_BATTERY_BYTE_INDEX, 1 + 52);
        assert_eq!(BLUETOOTH_BATTERY_BYTE_INDEX, 2 + 52);
    }

    #[test]
    fn the_dualshock_shifts_by_two_over_bluetooth() {
        let mut buffer = vec![0u8; DUALSHOCK_BLUETOOTH_BATTERY_BYTE_INDEX + 1];
        buffer[DUALSHOCK_USB_BATTERY_BYTE_INDEX] = 0b0000_0010;
        buffer[DUALSHOCK_BLUETOOTH_BATTERY_BYTE_INDEX] = 0b0000_1000;

        let over_usb = parse_battery_and_charging(&buffer, false, DUALSHOCK_GEN_1_PRODUCT_ID)
            .expect("should parse");
        let over_bluetooth = parse_battery_and_charging(&buffer, true, DUALSHOCK_GEN_1_PRODUCT_ID)
            .expect("should parse");

        assert_eq!(over_usb.battery_percent, 20);
        assert_eq!(over_bluetooth.battery_percent, 80);
    }

    fn dualshock_usb(battery_byte: u8) -> BatteryAndChargingResult {
        let buffer = report_with_battery_byte(DUALSHOCK_USB_BATTERY_BYTE_INDEX, battery_byte);
        parse_battery_and_charging(&buffer, false, DUALSHOCK_GEN_1_PRODUCT_ID)
            .expect("should parse")
    }

    #[test]
    fn a_dualshock_off_the_cable_is_discharging() {
        let result = dualshock_usb(0b0000_0110);
        assert_eq!(result.battery_percent, 60);
        assert!(!result.is_charging);
        assert!(!result.is_fully_charged);
        assert!(!result.unexpected_battery_data);
    }

    #[test]
    fn a_dualshock_on_the_cable_is_charging() {
        let result = dualshock_usb(0b0001_0110);
        assert_eq!(result.battery_percent, 60);
        assert!(result.is_charging);
        assert!(!result.is_fully_charged);
    }

    /// The DualShock signals a full battery with a level of 11 rather than a
    /// state, so it must not be read as a level out of range.
    #[test]
    fn a_dualshock_reports_full_as_level_eleven() {
        let result = dualshock_usb(0b0001_1011);
        assert_eq!(result.battery_percent, 100);
        assert!(result.is_fully_charged);
        assert!(!result.is_charging);
        assert!(!result.unexpected_battery_data);
    }

    #[test]
    fn a_dualshock_level_above_eleven_is_unexpected() {
        assert!(dualshock_usb(0b0001_1100).unexpected_battery_data);
    }

    #[test]
    fn a_report_that_ends_before_the_battery_byte_is_rejected() {
        let buffer = vec![0u8; BLUETOOTH_BATTERY_BYTE_INDEX];
        assert!(parse_battery_and_charging(&buffer, true, DUALSENSE_PRODUCT_ID).is_none());
    }

    #[test]
    fn an_unknown_controller_is_rejected() {
        let buffer = report_with_battery_byte(BLUETOOTH_BATTERY_BYTE_INDEX, 0b0000_0101);
        assert!(parse_battery_and_charging(&buffer, true, 0x0000).is_none());
    }

    #[test]
    fn the_edge_parses_like_the_dualsense() {
        let buffer = report_with_battery_byte(BLUETOOTH_BATTERY_BYTE_INDEX, 0b0000_0101);
        let result = parse_battery_and_charging(&buffer, true, DUALSENSE_EDGE_PRODUCT_ID)
            .expect("should parse");
        assert_eq!(result.battery_percent, 50);
    }
}
