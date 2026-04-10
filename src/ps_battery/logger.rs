use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static LOG_FILE: OnceLock<Mutex<File>> = OnceLock::new();
static LOG_PATH: OnceLock<String> = OnceLock::new();

pub fn init() {
    let Ok(appdata) = std::env::var("APPDATA") else { return };
    let dir = format!("{appdata}\\ps-battery");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!(" !! Failed to create log directory '{dir}': {e}");
        return;
    }
    let path = format!("{dir}\\ps-battery.log");
    let old_path = format!("{dir}\\ps-battery.old.log");
    let _ = std::fs::rename(&path, &old_path);
    match OpenOptions::new().create(true).write(true).truncate(true).open(&path) {
        Ok(file) => {
            let _ = LOG_PATH.set(path);
            let _ = LOG_FILE.set(Mutex::new(file));
        }
        Err(e) => eprintln!(" !! Failed to open log file '{path}': {e}"),
    }
}

pub fn get_log_path() -> Option<&'static str> {
    LOG_PATH.get().map(String::as_str)
}

pub fn write_log(level: &str, msg: &str) {
    #[cfg(debug_assertions)]
    {
        if level == "ERR " {
            eprintln!(" !! {msg}");
        } else {
            println!(" -> {msg}");
        }
    }

    if let Some(file) = LOG_FILE.get() {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if let Ok(mut f) = file.lock() {
            let _ = writeln!(f, "[{secs}] [{level}] {msg}");
        }
    }
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::ps_battery::logger::write_log("INFO", &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_err {
    ($($arg:tt)*) => {
        $crate::ps_battery::logger::write_log("ERR ", &format!($($arg)*))
    };
}
