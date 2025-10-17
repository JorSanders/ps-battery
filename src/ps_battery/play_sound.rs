use windows::Win32::Media::Audio::*;
use windows::core::PCWSTR;

const SOUND_FILE_NOTIFY: &str = r"C:\Windows\Media\Windows Notify System Generic.wav";
const SOUND_FILE_EXCLAMATION: &str = r"C:\Windows\Media\Windows Exclamation.wav";
const SOUND_FILE_CRITICAL: &str = r"C:\Windows\Media\Windows Critical Stop.wav";

pub enum AlertSound {
    Notify,
    Exclamation,
    Critical,
}

pub struct PlaySoundArgs {
    pub alert: AlertSound,
}

pub fn play_sound(args: &PlaySoundArgs) {
    let file_path = match args.alert {
        AlertSound::Notify => SOUND_FILE_NOTIFY,
        AlertSound::Exclamation => SOUND_FILE_EXCLAMATION,
        AlertSound::Critical => SOUND_FILE_CRITICAL,
    };

    let wide: Vec<u16> = file_path.encode_utf16().chain(Some(0)).collect();
    let result = unsafe { PlaySoundW(PCWSTR(wide.as_ptr()), None, SND_FILENAME | SND_ASYNC) };

    if result.as_bool() {
        println!(" -> Sound played. Path: '{}'", file_path)
    } else {
        eprintln!(" !! Failed to play sound '{}'", file_path);
    }
}
