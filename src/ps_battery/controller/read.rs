use crate::ps_battery::controller::calibration::{
    MaybeSendBluetoothCalibrationArgs, TRUNCATED_BLUETOOTH_HEADER, maybe_send_bluetooth_calibration,
};
use hidapi::{DeviceInfo, HidApi, HidDevice};

const HID_REFRESH_TIMEOUT_MS: i32 = 200;

pub struct OpenDeviceArgs<'a> {
    pub api: &'a HidApi,
    pub info: &'a DeviceInfo,
}

pub fn open_device(args: &OpenDeviceArgs) -> Option<HidDevice> {
    match args.info.open_device(args.api) {
        Ok(d) => {
            if let Err(err) = d.set_blocking_mode(false) {
                eprintln!("Failed to set non-blocking mode: {err}");
            }
            Some(d)
        }
        Err(err) => {
            eprintln!("Failed to open HID device: {err}");
            None
        }
    }
}

pub struct ReadReportWithCalibrationArgs<'a> {
    pub device: &'a HidDevice,
    pub device_name: &'a str,
    pub is_bluetooth: bool,
    pub buffer: &'a mut [u8],
    pub should_log: bool,
}

pub fn read_report_with_calibration(args: &mut ReadReportWithCalibrationArgs) -> usize {
    let mut count = args
        .device
        .read_timeout(args.buffer, HID_REFRESH_TIMEOUT_MS)
        .unwrap_or(0);

    if count > 0 && args.is_bluetooth {
        let first_byte = args.buffer[0];
        if first_byte == TRUNCATED_BLUETOOTH_HEADER {
            let calib_args = MaybeSendBluetoothCalibrationArgs {
                device: args.device,
                device_name: args.device_name,
                first_byte,
                should_log: args.should_log,
            };
            maybe_send_bluetooth_calibration(&calib_args);
            count = args
                .device
                .read_timeout(args.buffer, HID_REFRESH_TIMEOUT_MS)
                .unwrap_or(0);
        }
    }

    count
}
