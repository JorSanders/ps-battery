use hidapi::HidApi;
use std::{thread, time::Duration};
use windows::{
    UI::Notifications::*, Win32::Foundation::*, Win32::UI::Shell::*,
    Win32::UI::WindowsAndMessaging::*, core::*,
};

const SONY_VID: u16 = 0x054C;
const SONY_PIDS: [u16; 2] = [0x0CE6, 0x0DF2];
const BUFFER_SIZE: usize = 64;
const BATTERY_OFFSET: usize = 53;
const BATTERY_MASK: u8 = 0b00001111;
const POLL_INTERVAL_SECS: u64 = 60;

fn show_toast(title: &str, message: &str) {
    let toast_xml =
        ToastNotificationManager::GetTemplateContent(ToastTemplateType::ToastText02).unwrap();
    let nodes = toast_xml
        .GetElementsByTagName(&HSTRING::from("text"))
        .unwrap();
    nodes
        .Item(0)
        .unwrap()
        .AppendChild(&toast_xml.CreateTextNode(&HSTRING::from(title)).unwrap())
        .unwrap();
    nodes
        .Item(1)
        .unwrap()
        .AppendChild(&toast_xml.CreateTextNode(&HSTRING::from(message)).unwrap())
        .unwrap();

    let toast = ToastNotification::CreateToastNotification(&toast_xml).unwrap();
    let notifier =
        ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from("ps-battery")).unwrap();
    notifier.Show(&toast).unwrap();
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

unsafe fn create_hidden_window(class_name: &HSTRING) -> HWND {
    let wnd_class = WNDCLASSW {
        lpfnWndProc: Some(wnd_proc),
        hInstance: HINSTANCE::default(),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };

    let atom = unsafe { RegisterClassW(&wnd_class) };
    if atom == 0 {
        panic!("Failed to register window class");
    }

    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class_name,
            &HSTRING::from(""),
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            None,
            None,
            Some(HINSTANCE::default()),
            None,
        )
        .expect("Failed to create hidden window")
    };
    hwnd
}

unsafe fn add_tray_icon(hwnd: HWND) -> NOTIFYICONDATAW {
    let mut nid = NOTIFYICONDATAW::default();
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_ICON | NIF_TIP | NIF_MESSAGE;
    nid.uCallbackMessage = WM_USER + 1;
    nid.hIcon = unsafe { LoadIconW(None, IDI_APPLICATION).expect("Failed to load icon") };

    let tip = "PS Battery";
    let tip_u16: Vec<u16> = tip.encode_utf16().collect();
    nid.szTip[..tip_u16.len()].copy_from_slice(&tip_u16);

    let success = unsafe { Shell_NotifyIconW(NIM_ADD, &mut nid) };
    if !success.as_bool() {
        eprintln!("Failed to add tray icon");
    } else {
        unsafe { show_balloon(&mut nid, "Balloon", "Monitoring started") };
    }

    nid
}

unsafe fn show_balloon(nid: &mut NOTIFYICONDATAW, title: &str, message: &str) {
    nid.uFlags = NIF_INFO;
    let msg_u16: Vec<u16> = message.encode_utf16().collect();
    nid.szInfo[..msg_u16.len()].copy_from_slice(&msg_u16);
    let title_u16: Vec<u16> = title.encode_utf16().collect();
    nid.szInfoTitle[..title_u16.len()].copy_from_slice(&title_u16);
    nid.dwInfoFlags = NIIF_INFO;

    let success = unsafe { Shell_NotifyIconW(NIM_MODIFY, nid) };
    if !success.as_bool() {
        eprintln!("Failed to show balloon");
    }
}

fn main() {
    let class_name = HSTRING::from("PSBatteryHiddenWindow");
    let hwnd = unsafe { create_hidden_window(&class_name) };
    let mut nid = unsafe { add_tray_icon(hwnd) };
    show_toast("Toast", "Monitoring started");

    loop {
        let api = HidApi::new().expect("Failed to create HID API instance");
        let device_list: Vec<_> = api.device_list().collect();

        device_list.iter().for_each(|device| {
            println!(
                "VID: {:04X}, PID: {:04X}, Product: {:?}, Manufacturer: {:?}, Serial: {:?}",
                device.vendor_id(),
                device.product_id(),
                device.product_string(),
                device.manufacturer_string(),
                device.serial_number()
            );
        });

        device_list
            .iter()
            .filter(|d| d.vendor_id() == SONY_VID && SONY_PIDS.contains(&d.product_id()))
            .for_each(|device_info| {
                let device = device_info
                    .open_device(&api)
                    .expect("Failed to open device");
                let mut buf = [0u8; BUFFER_SIZE];

                if device.read_timeout(&mut buf, 200).is_err() {
                    println!(
                        "Failed to read controller PID {:04X}",
                        device_info.product_id()
                    );
                    return;
                }

                let battery = buf[BATTERY_OFFSET] & BATTERY_MASK;
                let percentage = battery * 10;
                println!(
                    "Controller PID {:04X} battery: {}%",
                    device_info.product_id(),
                    percentage
                );

                if battery <= 2 {
                    show_toast(
                        "Controller Battery Low",
                        &format!(
                            "Controller {:04X} battery at {}%",
                            device_info.product_id(),
                            percentage
                        ),
                    );
                    unsafe {
                        show_balloon(
                            &mut nid,
                            "Controller Battery Low",
                            &format!(
                                "Controller {:04X} battery at {}%",
                                device_info.product_id(),
                                percentage
                            ),
                        );
                    }
                }
            });

        thread::sleep(Duration::from_secs(POLL_INTERVAL_SECS));
    }
}
