use hidapi::DeviceInfo;

const BLUETOOTH_GUID_SUBSTRING: &str = "00001124-0000-1000-8000-00805F9B34FB";
pub const USB_REPORT_SIZE: usize = 64;
pub const BLUETOOTH_REPORT_SIZE: usize = 78;

pub struct TransportInfo {
    pub is_bluetooth: bool,
    pub report_size: usize,
}

pub struct DetectTransportArgs<'a> {
    pub info: &'a DeviceInfo,
}

pub fn detect_transport(args: &DetectTransportArgs) -> TransportInfo {
    let path_string = args.info.path().to_string_lossy().to_ascii_uppercase();
    let is_bluetooth = path_string.contains(BLUETOOTH_GUID_SUBSTRING);
    let report_size = if is_bluetooth {
        BLUETOOTH_REPORT_SIZE
    } else {
        USB_REPORT_SIZE
    };
    TransportInfo {
        is_bluetooth,
        report_size,
    }
}
