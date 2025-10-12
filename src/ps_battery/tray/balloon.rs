use windows::Win32::UI::Shell::{NIF_TIP, NIM_MODIFY, NOTIFYICONDATAW, Shell_NotifyIconW};

pub struct ShowBalloonArgs<'a> {
    pub notify: &'a mut NOTIFYICONDATAW,
    pub title: &'a str,
    pub message: &'a str,
}

pub unsafe fn show_balloon(args: &mut ShowBalloonArgs) {
    let title_utf16: Vec<u16> = args.title.encode_utf16().chain(Some(0)).collect();
    let msg_utf16: Vec<u16> = args.message.encode_utf16().chain(Some(0)).collect();
    args.notify.uFlags = NIF_TIP;
    args.notify.szInfo[..msg_utf16.len()].copy_from_slice(&msg_utf16);
    args.notify.szInfoTitle[..title_utf16.len()].copy_from_slice(&title_utf16);
    unsafe {
        let _ = Shell_NotifyIconW(NIM_MODIFY, args.notify);
    }
}
