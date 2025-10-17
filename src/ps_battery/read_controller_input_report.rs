use crate::ps_battery::{
    get_controller_info::ConnectionType,
    send_bluetooth_feature_report::send_bluetooth_feature_report,
};
use hidapi::{DeviceInfo, HidApi, HidDevice};

const HID_REFRESH_TIMEOUT_MS: i32 = 400;
const TRUNCATED_BLUETOOTH_HEADER: u8 = 0x01;

pub struct OpenDeviceArgs<'a> {
    pub hid_api: &'a HidApi,
    pub info: &'a DeviceInfo,
}

pub fn open_device(args: &OpenDeviceArgs) -> Option<HidDevice> {
    match args.info.open_device(args.hid_api) {
        Ok(d) => {
            if let Err(err) = d.set_blocking_mode(false) {
                eprintln!(" !! Failed to set non-blocking mode: '{}'", err);
            }
            Some(d)
        }
        Err(err) => {
            eprintln!(" !! Failed to open HID device: '{}'", err);
            None
        }
    }
}

pub struct ReadControllerInputReportArgs<'a> {
    pub hid_device: &'a HidDevice,
    pub device_name: &'a str,
    pub buffer: &'a mut [u8],
    pub connection_type: ConnectionType,
}

pub fn read_controller_input_report(args: &mut ReadControllerInputReportArgs) {
    let buffer_length = match args
        .hid_device
        .read_timeout(args.buffer, HID_REFRESH_TIMEOUT_MS)
    {
        Ok(n) => {
            println!(" -> read_controller_input_report {} bytes", n);
            n
        }
        Err(e) => {
            eprintln!(" !! read_controller_input_report error: '{}'", e);
            0
        }
    };

    if buffer_length > 0 && args.connection_type == ConnectionType::Bluetooth {
        let first_byte = args.buffer[0];

        if first_byte == TRUNCATED_BLUETOOTH_HEADER {
            println!(
                "Sending Bluetooth report, to start receiving battery and charging info. For controller : '{}'",
                args.device_name,
            );

            send_bluetooth_feature_report(args.hid_device);

            std::thread::sleep(std::time::Duration::from_millis(500));

            match args
                .hid_device
                .read_timeout(args.buffer, HID_REFRESH_TIMEOUT_MS)
            {
                Ok(0) => eprintln!(" !! read_controller_input_report timeout"),
                Ok(n) => println!(" -> read_controller_input_report {} bytes", n),
                Err(e) => eprintln!(" !! read_controller_input_report error: '{}'", e),
            };
        }
    }
}
