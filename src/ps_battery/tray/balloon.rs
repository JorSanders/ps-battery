use windows::{
    Win32::UI::Shell::{
        NIF_INFO, NIIF_ERROR, NIIF_INFO, NIIF_WARNING, NIM_MODIFY, NOTIFYICONDATAW,
        Shell_NotifyIconW,
    },
    core::BOOL,
};

pub enum BalloonIcon {
    Info,
    Warning,
    Error,
}

pub struct ShowBalloonArgs<'a> {
    pub notify: &'a mut NOTIFYICONDATAW,
    pub title: &'a str,
    pub message: &'a str,
    pub icon: BalloonIcon,
}

pub fn show_balloon(args: &mut ShowBalloonArgs) {
    unsafe {
        let title_utf16: Vec<u16> = args.title.encode_utf16().chain(Some(0)).collect();
        let msg_utf16: Vec<u16> = args.message.encode_utf16().chain(Some(0)).collect();

        args.notify.uFlags |= NIF_INFO;

        let msg_len = msg_utf16.len().min(args.notify.szInfo.len());
        args.notify.szInfo[..msg_len].copy_from_slice(&msg_utf16[..msg_len]);

        let title_len = title_utf16.len().min(args.notify.szInfoTitle.len());
        args.notify.szInfoTitle[..title_len].copy_from_slice(&title_utf16[..title_len]);

        args.notify.dwInfoFlags = match args.icon {
            BalloonIcon::Info => NIIF_INFO,
            BalloonIcon::Warning => NIIF_WARNING,
            BalloonIcon::Error => NIIF_ERROR,
        };

        let res = Shell_NotifyIconW(NIM_MODIFY, args.notify);
        if res == BOOL(0) {
            eprintln!("Shell_NotifyIconW failed");
        }
    }
}
