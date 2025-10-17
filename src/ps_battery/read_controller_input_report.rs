use crate::ps_battery::{
    get_controller_info::TransportLabel,
    send_bluetooth_feature_report::send_bluetooth_feature_report,
};
use hidapi::{DeviceInfo, HidApi, HidDevice};

const HID_REFRESH_TIMEOUT_MS: i32 = 200;
const TRUNCATED_BLUETOOTH_HEADER: u8 = 0x01;

pub struct OpenDeviceArgs<'a> {
    pub hid_api: &'a HidApi,
    pub info: &'a DeviceInfo,
}

pub fn open_device(args: &OpenDeviceArgs) -> Option<HidDevice> {
    match args.info.open_device(args.hid_api) {
        Ok(d) => {
            if let Err(err) = d.set_blocking_mode(false) {
                println!("Failed to set non-blocking mode: '{}'", err);
            }
            Some(d)
        }
        Err(err) => {
            println!("Failed to open HID device: '{}'", err);
            None
        }
    }
}

pub struct ReadControllerInputReportArgs<'a> {
    pub hid_device: &'a HidDevice,
    pub device_name: &'a str,
    pub buffer: &'a mut [u8],
    pub transport_label: TransportLabel,
}

pub fn read_controller_input_report(args: &mut ReadControllerInputReportArgs) {
    let buffer_length = args
        .hid_device
        .read_timeout(args.buffer, HID_REFRESH_TIMEOUT_MS)
        .unwrap_or(0);

    if buffer_length > 0 && args.transport_label == TransportLabel::Bluetooth {
        let first_byte = args.buffer[0];

        if first_byte == TRUNCATED_BLUETOOTH_HEADER {
            println!(
                "Sending Bluetooth report, to start receiving battery and charging info. For controller : '{}'",
                args.device_name,
            );

            send_bluetooth_feature_report(args.hid_device);

            args.hid_device
                .read_timeout(args.buffer, HID_REFRESH_TIMEOUT_MS)
                .unwrap_or(0);
        }
    }
}
