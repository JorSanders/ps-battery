use windows::Win32::Media::Audio::*;
use windows::core::*;

pub enum AlertSound {
    Notify,
    Exclamation,
    Critical,
}

pub fn play_sound(alert: AlertSound) {
    let file_path = match alert {
        AlertSound::Notify => r"C:\Windows\Media\Windows Notify System Generic.wav",
        AlertSound::Exclamation => r"C:\Windows\Media\Windows Exclamation.wav",
        AlertSound::Critical => r"C:\Windows\Media\Windows Critical Stop.wav",
    };

    let path: Vec<u16> = file_path.encode_utf16().chain(Some(0)).collect();
    let result: BOOL;
    unsafe {
        result = PlaySoundW(PCWSTR(path.as_ptr()), None, SND_FILENAME | SND_ASYNC);
    }
    if !result.as_bool() {
        eprintln!("Failed to play sound: {}", file_path);
    }
}
