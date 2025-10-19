use hidapi::DeviceInfo;

const BLUETOOTH_GUID_SUBSTRING: &str = "00001124-0000-1000-8000-00805F9B34FB";

pub struct ControllerInfo {
    pub name: String,
    pub is_bluetooth: bool,
    pub product_id: u16,
    pub path: String,
}

pub fn get_controller_info(info: &DeviceInfo) -> ControllerInfo {
    let path_string = info.path().to_string_lossy().to_ascii_uppercase();
    let is_bluetooth = path_string.contains(BLUETOOTH_GUID_SUBSTRING);

    let name = info.product_string().unwrap_or("Unknown").to_string();
    let product_id = info.product_id();
    let path = info.path().to_string_lossy().into_owned();

    ControllerInfo {
        is_bluetooth,
        name,
        product_id,
        path,
    }
}
