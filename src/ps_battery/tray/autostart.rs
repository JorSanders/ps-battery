use crate::log_err;
use std::{ffi::OsStr, os::windows::ffi::OsStrExt, ptr};
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SZ, RRF_RT_REG_SZ, RegCloseKey,
    RegDeleteValueW, RegGetValueW, RegOpenKeyExW, RegSetValueExW,
};
use windows::core::PCWSTR;

const APP_NAME: &str = "PS Battery";
const RUN_SUBKEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

pub fn is_enabled() -> bool {
    let sub = to_wide(RUN_SUBKEY);
    let val = to_wide(APP_NAME);
    let mut buf = [0u16; 260];
    #[allow(clippy::cast_possible_truncation)]
    let mut cb = (buf.len() * 2) as u32;
    unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            PCWSTR(sub.as_ptr()),
            PCWSTR(val.as_ptr()),
            RRF_RT_REG_SZ,
            None,
            Some(buf.as_mut_ptr().cast()),
            Some(&raw mut cb),
        )
        .is_ok()
    }
}

pub fn enable() -> bool {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            log_err!("current_exe failed: {e}");
            return false;
        }
    };

    let sub = to_wide(RUN_SUBKEY);
    let name = to_wide(APP_NAME);
    let exe_str = exe.to_string_lossy();
    let exe_u16 = to_wide(&exe_str);

    let mut hkey = HKEY(ptr::null_mut());
    unsafe {
        let open = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(sub.as_ptr()),
            Some(0),
            KEY_SET_VALUE | KEY_QUERY_VALUE,
            &raw mut hkey,
        );
        if let Err(e) = open.ok() {
            log_err!("RegOpenKeyExW failed: {e}");
            return false;
        }
        let bytes = std::slice::from_raw_parts(exe_u16.as_ptr().cast::<u8>(), exe_u16.len() * 2);
        let set = RegSetValueExW(hkey, PCWSTR(name.as_ptr()), Some(0), REG_SZ, Some(bytes));
        if let Err(e) = RegCloseKey(hkey).ok() {
            log_err!("RegCloseKey failed: {e}");
        }
        set.is_ok()
    }
}

pub fn disable() -> bool {
    let sub = to_wide(RUN_SUBKEY);
    let mut hkey = HKEY(ptr::null_mut());
    unsafe {
        let open = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(sub.as_ptr()),
            Some(0),
            KEY_SET_VALUE | KEY_QUERY_VALUE,
            &raw mut hkey,
        );
        if let Err(e) = open.ok() {
            log_err!("RegOpenKeyExW failed: {e}");
            return false;
        }
        let name = to_wide(APP_NAME);
        let del = RegDeleteValueW(hkey, PCWSTR(name.as_ptr()));
        if let Err(e) = RegCloseKey(hkey).ok() {
            log_err!("RegCloseKey failed: {e}");
        }
        del.is_ok()
    }
}
