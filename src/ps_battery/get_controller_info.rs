use hidapi::DeviceInfo;

const BLUETOOTH_GUID_SUBSTRING: &str = "00001124-0000-1000-8000-00805F9B34FB";
pub const USB_REPORT_SIZE: usize = 64;
pub const BLUETOOTH_REPORT_SIZE: usize = 78;

#[derive(Clone, Copy, PartialEq)]
pub enum ConnectionType {
    Usb,
    Bluetooth,
}
use std::fmt;

impl fmt::Display for ConnectionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionType::Usb => write!(f, "USB"),
            ConnectionType::Bluetooth => write!(f, "Bluetooth"),
        }
    }
}

pub struct ControllerInfo {
    pub report_size: usize,
    pub name: String,
    pub connection_type: ConnectionType,
    pub product_id: u16,
    pub path: String,
}

pub fn get_controller_info(info: &DeviceInfo) -> ControllerInfo {
    let path_string = info.path().to_string_lossy().to_ascii_uppercase();
    let is_bluetooth = path_string.contains(BLUETOOTH_GUID_SUBSTRING);
    let report_size = if is_bluetooth {
        BLUETOOTH_REPORT_SIZE
    } else {
        USB_REPORT_SIZE
    };
    let connection_type = if is_bluetooth {
        ConnectionType::Bluetooth
    } else {
        ConnectionType::Usb
    };
    let name = info.product_string().unwrap_or("Unknown").to_string();
    let product_id = info.product_id();
    let path = info.path().to_string_lossy().into_owned();

    ControllerInfo {
        connection_type,
        report_size,
        name,
        product_id,
        path,
    }
}
