use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static LOG_FILE: OnceLock<Mutex<File>> = OnceLock::new();
static LOG_PATH: OnceLock<String> = OnceLock::new();

pub fn init() {
    let Ok(appdata) = std::env::var("APPDATA") else {
        crate::log_err!("APPDATA is not set, file logging disabled");
        return;
    };
    let dir = format!("{appdata}\\ps-battery");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        crate::log_err!("Failed to create log directory '{dir}': {e}");
        return;
    }
    let path = format!("{dir}\\ps-battery.log");
    let old_path = format!("{dir}\\ps-battery.old.log");
    let _ = std::fs::rename(&path, &old_path);
    match OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
    {
        Ok(file) => {
            let _ = LOG_PATH.set(path);
            let _ = LOG_FILE.set(Mutex::new(file));
        }
        Err(e) => crate::log_err!("Failed to open log file '{path}': {e}"),
    }
}

pub fn get_log_path() -> Option<&'static str> {
    LOG_PATH.get().map(String::as_str)
}

#[derive(Clone, Copy)]
pub enum LogLevel {
    Info,
    Error,
}

impl LogLevel {
    fn label(self) -> &'static str {
        match self {
            LogLevel::Info => "INFO",
            LogLevel::Error => "ERR ",
        }
    }
}

pub fn write_log(level: LogLevel, msg: &str) {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let label = level.label();

    #[cfg(debug_assertions)]
    {
        if matches!(level, LogLevel::Error) {
            eprintln!("[{secs}] [{label}] {msg}");
        } else {
            println!("[{secs}] [{label}] {msg}");
        }
    }

    if let Some(file) = LOG_FILE.get()
        && let Ok(mut f) = file.lock()
    {
        let _ = writeln!(f, "[{secs}] [{label}] {msg}");
    }
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::ps_battery::logger::write_log(
            $crate::ps_battery::logger::LogLevel::Info,
            &format!($($arg)*),
        )
    };
}

#[macro_export]
macro_rules! log_err {
    ($($arg:tt)*) => {
        $crate::ps_battery::logger::write_log(
            $crate::ps_battery::logger::LogLevel::Error,
            &format!($($arg)*),
        )
    };
}
