use std::{ffi::OsStr, os::windows::ffi::OsStrExt, ptr};
use windows::Win32::Foundation::WIN32_ERROR;
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
    let mut cb = (buf.len() * 2) as u32;
    unsafe {
        let res: WIN32_ERROR = RegGetValueW(
            HKEY_CURRENT_USER,
            PCWSTR(sub.as_ptr()),
            PCWSTR(val.as_ptr()),
            RRF_RT_REG_SZ,
            None,
            Some(buf.as_mut_ptr() as _),
            Some(&mut cb),
        );
        res.is_ok()
    }
}

pub fn enable() -> bool {
    let sub = to_wide(RUN_SUBKEY);
    let mut hkey = HKEY(ptr::null_mut());
    unsafe {
        let open = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(sub.as_ptr()),
            Some(0),
            KEY_SET_VALUE | KEY_QUERY_VALUE,
            &mut hkey,
        );
        if open.is_err() {
            return false;
        }
        let exe = std::env::current_exe().unwrap_or_default();
        let exe_str = exe.to_string_lossy();
        let exe_u16 = to_wide(&exe_str);
        let bytes = std::slice::from_raw_parts(exe_u16.as_ptr() as *const u8, exe_u16.len() * 2);
        let name = to_wide(APP_NAME);
        let set = RegSetValueExW(hkey, PCWSTR(name.as_ptr()), Some(0), REG_SZ, Some(bytes));
        let res = RegCloseKey(hkey);
        if res != WIN32_ERROR(0) {
            eprintln!("RegCloseKey failed with code {:?}", res);
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
            &mut hkey,
        );
        if open.is_err() {
            return false;
        }
        let name = to_wide(APP_NAME);
        let del = RegDeleteValueW(hkey, PCWSTR(name.as_ptr()));
        let res = RegCloseKey(hkey);
        if res != WIN32_ERROR(0) {
            eprintln!("RegCloseKey failed with code {:?}", res);
        }
        del.is_ok()
    }
}
