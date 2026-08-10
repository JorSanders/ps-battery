use crate::log_err;
use std::{ffi::OsStr, os::windows::ffi::OsStrExt, ptr};
use windows::Win32::Foundation::ERROR_FILE_NOT_FOUND;
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE, REG_SZ, RRF_RT_REG_SZ, RegCloseKey, RegDeleteValueW,
    RegGetValueW, RegOpenKeyExW, RegSetValueExW,
};
use windows::core::PCWSTR;

const APP_NAME: &str = "PS Battery";
const RUN_SUBKEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

fn to_wide(text: &str) -> Vec<u16> {
    OsStr::new(text).encode_wide().chain(Some(0)).collect()
}

pub fn is_enabled() -> bool {
    let subkey = to_wide(RUN_SUBKEY);
    let name = to_wide(APP_NAME);
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            PCWSTR(name.as_ptr()),
            RRF_RT_REG_SZ,
            None,
            None,
            None,
        )
    };
    // A value that does not exist means autostart is off, so only other
    // errors are unexpected.
    if let Err(e) = status.ok()
        && status != ERROR_FILE_NOT_FOUND
    {
        log_err!("RegGetValueW for the autostart entry failed: {e}");
    }
    status.is_ok()
}

pub fn enable() -> bool {
    let executable_path = match std::env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            log_err!("current_exe failed: {e}");
            return false;
        }
    };

    let subkey = to_wide(RUN_SUBKEY);
    let name = to_wide(APP_NAME);
    // Quoted so a path with spaces cannot be misread as a shorter program
    // name with arguments.
    let quoted_executable_path = format!("\"{}\"", executable_path.to_string_lossy());
    let executable_path_utf16 = to_wide(&quoted_executable_path);

    let mut hkey = HKEY(ptr::null_mut());
    let open = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            Some(0),
            KEY_SET_VALUE,
            &raw mut hkey,
        )
    };
    if let Err(e) = open.ok() {
        log_err!("RegOpenKeyExW failed: {e}");
        return false;
    }
    let bytes: Vec<u8> = executable_path_utf16
        .iter()
        .flat_map(|wide_char| wide_char.to_le_bytes())
        .collect();
    let set =
        unsafe { RegSetValueExW(hkey, PCWSTR(name.as_ptr()), Some(0), REG_SZ, Some(&bytes)) }.ok();
    if let Err(e) = &set {
        log_err!("RegSetValueExW failed: {e}");
    }
    if let Err(e) = unsafe { RegCloseKey(hkey) }.ok() {
        log_err!("RegCloseKey failed: {e}");
    }
    set.is_ok()
}

pub fn disable() -> bool {
    let subkey = to_wide(RUN_SUBKEY);
    let mut hkey = HKEY(ptr::null_mut());
    let open = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            Some(0),
            KEY_SET_VALUE,
            &raw mut hkey,
        )
    };
    if let Err(e) = open.ok() {
        log_err!("RegOpenKeyExW failed: {e}");
        return false;
    }
    let name = to_wide(APP_NAME);
    let delete = unsafe { RegDeleteValueW(hkey, PCWSTR(name.as_ptr())) };
    // A value that is already gone means autostart is already off.
    let already_removed = delete == ERROR_FILE_NOT_FOUND;
    if let Err(e) = delete.ok()
        && !already_removed
    {
        log_err!("RegDeleteValueW failed: {e}");
    }
    if let Err(e) = unsafe { RegCloseKey(hkey) }.ok() {
        log_err!("RegCloseKey failed: {e}");
    }
    delete.is_ok() || already_removed
}
