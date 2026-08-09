use crate::log_err;
use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MessageBoxW};
use windows::core::PCWSTR;

pub fn show_error_message_box(message: &str) {
    let text_utf16: Vec<u16> = message.encode_utf16().chain(Some(0)).collect();
    let caption_utf16: Vec<u16> = "PS Battery".encode_utf16().chain(Some(0)).collect();
    let message_box_result = unsafe {
        MessageBoxW(
            None,
            PCWSTR(text_utf16.as_ptr()),
            PCWSTR(caption_utf16.as_ptr()),
            MB_ICONERROR,
        )
    };
    if message_box_result.0 == 0 {
        log_err!("MessageBoxW failed to show the error '{message}'");
    }
}
