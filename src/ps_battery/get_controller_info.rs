use hidapi::DeviceInfo;

const BLUETOOTH_GUID_SUBSTRING: &str = "00001124-0000-1000-8000-00805F9B34FB";
pub const USB_REPORT_SIZE: usize = 64;
pub const BLUETOOTH_REPORT_SIZE: usize = 78;

#[derive(Clone, Copy, PartialEq)]
pub enum TransportLabel {
    Usb,
    Bluetooth,
}
use std::fmt;

impl fmt::Display for TransportLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportLabel::Usb => write!(f, "USB"),
            TransportLabel::Bluetooth => write!(f, "Bluetooth"),
        }
    }
}

pub struct ControllerInfo {
    pub report_size: usize,
    pub name: String,
    pub transport_label: TransportLabel,
}

pub fn get_controller_info(info: &DeviceInfo) -> ControllerInfo {
    let path_string = info.path().to_string_lossy().to_ascii_uppercase();
    let is_bluetooth = path_string.contains(BLUETOOTH_GUID_SUBSTRING);
    let report_size = if is_bluetooth {
        BLUETOOTH_REPORT_SIZE
    } else {
        USB_REPORT_SIZE
    };
    let transport_label = if is_bluetooth {
        TransportLabel::Bluetooth
    } else {
        TransportLabel::Usb
    };
    let name = info.product_string().unwrap_or("Unknown").to_string();

    ControllerInfo {
        transport_label,
        report_size,
        name,
    }
}
