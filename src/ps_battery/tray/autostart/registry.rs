use crate::log_err;
use std::{ffi::OsStr, os::windows::ffi::OsStrExt, ptr};
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SZ, RRF_RT_REG_SZ, RegCloseKey,
    RegDeleteValueW, RegGetValueW, RegOpenKeyExW, RegSetValueExW,
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
    unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            PCWSTR(name.as_ptr()),
            RRF_RT_REG_SZ,
            None,
            None,
            None,
        )
        .is_ok()
    }
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
    let executable_path_str = executable_path.to_string_lossy();
    let executable_path_utf16 = to_wide(&executable_path_str);

    let mut hkey = HKEY(ptr::null_mut());
    unsafe {
        let open = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            Some(0),
            KEY_SET_VALUE | KEY_QUERY_VALUE,
            &raw mut hkey,
        );
        if let Err(e) = open.ok() {
            log_err!("RegOpenKeyExW failed: {e}");
            return false;
        }
        let bytes = std::slice::from_raw_parts(
            executable_path_utf16.as_ptr().cast::<u8>(),
            executable_path_utf16.len() * 2,
        );
        let set = RegSetValueExW(hkey, PCWSTR(name.as_ptr()), Some(0), REG_SZ, Some(bytes));
        if let Err(e) = RegCloseKey(hkey).ok() {
            log_err!("RegCloseKey failed: {e}");
        }
        set.is_ok()
    }
}

pub fn disable() -> bool {
    let subkey = to_wide(RUN_SUBKEY);
    let mut hkey = HKEY(ptr::null_mut());
    unsafe {
        let open = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            Some(0),
            KEY_SET_VALUE | KEY_QUERY_VALUE,
            &raw mut hkey,
        );
        if let Err(e) = open.ok() {
            log_err!("RegOpenKeyExW failed: {e}");
            return false;
        }
        let name = to_wide(APP_NAME);
        let delete = RegDeleteValueW(hkey, PCWSTR(name.as_ptr()));
        if let Err(e) = RegCloseKey(hkey).ok() {
            log_err!("RegCloseKey failed: {e}");
        }
        delete.is_ok()
    }
}
