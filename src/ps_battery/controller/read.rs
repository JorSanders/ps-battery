use crate::ps_battery::{
    controller::{
        info::TransportLabel,
        report::{MaybeSendBluetoothCalibrationArgs, send_feature_report},
    },
    log::log_error_with,
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
                log_error_with("Failed to set non-blocking mode", err);
            }
            Some(d)
        }
        Err(err) => {
            log_error_with("Failed to open HID device", err);
            None
        }
    }
}

pub struct ReadControllerBufferArgs<'a> {
    pub device: &'a HidDevice,
    pub device_name: &'a str,
    pub buffer: &'a mut [u8],
    pub transport_label: TransportLabel,
}

pub fn read_controller_buffer(args: &mut ReadControllerBufferArgs) {
    let buffer_length = args
        .device
        .read_timeout(args.buffer, HID_REFRESH_TIMEOUT_MS)
        .unwrap_or(0);

    if buffer_length > 0 && args.transport_label == TransportLabel::Bluetooth {
        let first_byte = args.buffer[0];

        if first_byte == TRUNCATED_BLUETOOTH_HEADER {
            log_error_with(
                "Sending Bluetooth report, to start receiving battery and charging info",
                args.device_name,
            );

            let calib_args = MaybeSendBluetoothCalibrationArgs {
                device: args.device,
            };

            send_feature_report(&calib_args);

            args.device
                .read_timeout(args.buffer, HID_REFRESH_TIMEOUT_MS)
                .unwrap_or(0);
        }
    }
}
