use crate::ps_battery::{
    get_controller_info::ConnectionType,
    send_controller_feature_report::send_controller_feature_report,
};
use hidapi::{DeviceInfo, HidApi, HidDevice};

const HID_REFRESH_TIMEOUT_MS: i32 = 400;
const TRUNCATED_BLUETOOTH_HEADER: u8 = 0x01;
const INITIAL_BUFFER_SIZE: usize = 100;

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
    pub connection_type: ConnectionType,
    pub product_id: u16,
}

pub fn read_controller_input_report(args: &ReadControllerInputReportArgs) -> Vec<u8> {
    let mut buffer = vec![0u8; INITIAL_BUFFER_SIZE];

    let mut buffer_length = match args
        .hid_device
        .read_timeout(&mut buffer, HID_REFRESH_TIMEOUT_MS)
    {
        Ok(n) => n,
        Err(e) => {
            eprintln!(" !! read_controller_input_report error: '{}'", e);
            0
        }
    };

    println!(
        " -> read_controller_input_report response {} bytes: {:02X?}",
        buffer_length,
        &buffer[..buffer_length]
    );

    if buffer_length > 0 {
        let report_header = buffer[0];

        if report_header == TRUNCATED_BLUETOOTH_HEADER
            && args.connection_type == ConnectionType::Bluetooth
        {
            println!(
                " -> Truncated header detected. Sending feature report for controller : '{}'",
                args.device_name,
            );

            send_controller_feature_report(args.hid_device, args.product_id);
            std::thread::sleep(std::time::Duration::from_millis(500));

            buffer_length = match args
                .hid_device
                .read_timeout(&mut buffer, HID_REFRESH_TIMEOUT_MS)
            {
                Ok(0) => {
                    eprintln!(" !! read_controller_input_report timeout");
                    0
                }
                Ok(n) => n,
                Err(e) => {
                    eprintln!(" !! read_controller_input_report error: '{}'", e);
                    0
                }
            };
            println!(
                " -> read_controller_input_report response {} bytes: {:02X?}",
                buffer_length,
                &buffer[..buffer_length]
            );
        }
    }

    buffer.truncate(buffer_length);
    buffer
}
