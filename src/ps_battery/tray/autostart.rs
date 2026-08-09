mod registry;
mod startup_task;

use crate::{log_err, log_info};
use std::sync::OnceLock;
use windows::Win32::Foundation::APPMODEL_ERROR_NO_PACKAGE;
use windows::Win32::Storage::Packaging::Appx::GetCurrentPackageFullName;

static IS_PACKAGED: OnceLock<bool> = OnceLock::new();

/// MSIX (Store) builds must register startup through `StartupTask` so the
/// toggle appears in Windows Settings, while the plain .exe download has no
/// package identity and keeps using the Run registry key.
fn is_packaged() -> bool {
    *IS_PACKAGED.get_or_init(|| {
        let mut length = 0u32;
        let result = unsafe { GetCurrentPackageFullName(&raw mut length, None) };
        result != APPMODEL_ERROR_NO_PACKAGE
    })
}

pub fn init() {
    if is_packaged() {
        log_info!("Autostart backend: StartupTask (packaged)");
        startup_task::start_worker();
    } else {
        log_info!("Autostart backend: registry (unpackaged)");
        // The entry goes stale when the exe moves; rewriting it on every
        // start keeps it pointing at whichever copy actually runs.
        if registry::is_enabled() && !registry::enable() {
            log_err!("Refreshing the autostart entry at startup failed");
        }
    }
}

/// Called when the tray menu opens, so a change made outside the app is
/// picked up. No-op unpackaged, where the registry is read on every call.
pub fn request_refresh() {
    if is_packaged() {
        startup_task::request_refresh();
    }
}

pub fn is_enabled() -> bool {
    if is_packaged() {
        startup_task::is_enabled()
    } else {
        registry::is_enabled()
    }
}

pub fn is_available() -> bool {
    if is_packaged() {
        startup_task::is_available()
    } else {
        true
    }
}

/// Packaged builds apply this on a worker thread, so the returned value only
/// reports whether the request was dispatched; the menu picks up the real
/// state on its next refresh.
pub fn enable() -> bool {
    if is_packaged() {
        startup_task::enable();
        true
    } else {
        registry::enable()
    }
}

pub fn disable() -> bool {
    if is_packaged() {
        startup_task::disable();
        true
    } else {
        registry::disable()
    }
}
