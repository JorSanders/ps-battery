use windows::{
    Win32::Foundation::*, Win32::UI::Shell::*, Win32::UI::WindowsAndMessaging::*, core::*,
};

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

pub unsafe fn create_hidden_window(class_name: &HSTRING) -> HWND {
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

    unsafe {
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
    }
}

pub unsafe fn add_tray_icon(hwnd: HWND) -> NOTIFYICONDATAW {
    let mut nid = NOTIFYICONDATAW::default();
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_ICON | NIF_TIP | NIF_MESSAGE;
    nid.uCallbackMessage = WM_USER + 1;

    unsafe {
        nid.hIcon = LoadIconW(None, IDI_APPLICATION).expect("Failed to load icon");
    }

    let tip = "PS Battery";
    let tip_u16: Vec<u16> = tip.encode_utf16().collect();
    nid.szTip[..tip_u16.len()].copy_from_slice(&tip_u16);

    unsafe {
        if !Shell_NotifyIconW(NIM_ADD, &mut nid).as_bool() {
            eprintln!("Failed to add tray icon");
        }
    }

    nid
}

pub unsafe fn show_balloon(nid: &mut NOTIFYICONDATAW, title: &str, message: &str) {
    nid.uFlags = NIF_INFO;

    let msg_u16: Vec<u16> = message.encode_utf16().collect();
    nid.szInfo[..msg_u16.len()].copy_from_slice(&msg_u16);

    let title_u16: Vec<u16> = title.encode_utf16().collect();
    nid.szInfoTitle[..title_u16.len()].copy_from_slice(&title_u16);

    nid.dwInfoFlags = NIIF_INFO;

    unsafe {
        if !Shell_NotifyIconW(NIM_MODIFY, nid).as_bool() {
            eprintln!("Failed to show balloon");
        }
    }
}
