use crate::{log_err, log_info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex};
use windows::ApplicationModel::{StartupTask, StartupTaskState};
use windows::Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize};
use windows::core::HSTRING;

/// Must stay byte-identical to the TaskId in Package.appxmanifest, or the
/// lookup silently returns not-found.
const TASK_ID: &str = "PsBatteryAutostart";

static IS_ENABLED: AtomicBool = AtomicBool::new(false);
/// False until the worker has resolved the `StartupTask`, and stays false
/// when that fails, so the menu can gray out a toggle that would go nowhere.
static IS_WORKER_AVAILABLE: AtomicBool = AtomicBool::new(false);
static PENDING_REQUEST: (Mutex<Option<Request>>, Condvar) = (Mutex::new(None), Condvar::new());

#[derive(Clone, Copy)]
enum Request {
    Enable,
    Disable,
    Refresh,
}

fn send(request: Request) {
    let (lock, condvar) = &PENDING_REQUEST;
    *lock.lock().expect("autostart request poisoned") = Some(request);
    condvar.notify_one();
}

pub fn is_enabled() -> bool {
    IS_ENABLED.load(Ordering::Acquire)
}

pub fn is_available() -> bool {
    IS_WORKER_AVAILABLE.load(Ordering::Acquire)
}

pub fn enable() {
    send(Request::Enable);
}

pub fn disable() {
    send(Request::Disable);
}

/// Asks the worker to re-read the state, so a change made outside the app
/// (Task Manager, Windows Settings) shows up in the tray menu.
pub fn request_refresh() {
    send(Request::Refresh);
}

/// Spawns the only thread allowed to touch `StartupTask`. WinRT calls are
/// kept off the UI thread because `IAsyncOperation::join` waits on a raw
/// `WaitForSingleObject` that pumps no messages, which would freeze the tray
/// menu that `window_proc` is drawing.
pub fn start_worker() {
    std::thread::spawn(|| {
        if let Err(e) = unsafe { RoInitialize(RO_INIT_MULTITHREADED) } {
            log_err!("RoInitialize failed: {e}");
            return;
        }

        let Some(task) = resolve_task() else { return };
        refresh_cached_state(&task);
        IS_WORKER_AVAILABLE.store(true, Ordering::Release);

        let (lock, condvar) = &PENDING_REQUEST;
        loop {
            let request = {
                let mut guard = lock.lock().expect("autostart request poisoned");
                loop {
                    if let Some(request) = guard.take() {
                        break request;
                    }
                    guard = condvar.wait(guard).expect("autostart condvar wait failed");
                }
            };

            match request {
                Request::Enable => request_enable(&task),
                Request::Disable => {
                    if let Err(e) = task.Disable() {
                        log_err!("StartupTask Disable failed: {e}");
                    }
                }
                Request::Refresh => {}
            }
            refresh_cached_state(&task);
        }
    });
}

fn resolve_task() -> Option<StartupTask> {
    let operation = match StartupTask::GetAsync(&HSTRING::from(TASK_ID)) {
        Ok(operation) => operation,
        Err(e) => {
            log_err!("StartupTask::GetAsync failed: {e}");
            return None;
        }
    };
    match operation.join() {
        Ok(task) => Some(task),
        Err(e) => {
            log_err!("StartupTask::GetAsync did not complete: {e}");
            None
        }
    }
}

fn request_enable(task: &StartupTask) {
    let operation = match task.RequestEnableAsync() {
        Ok(operation) => operation,
        Err(e) => {
            log_err!("RequestEnableAsync failed: {e}");
            return;
        }
    };
    match operation.join() {
        Ok(state) => log_info!("Autostart enable request resolved to state {}", state.0),
        Err(e) => log_err!("RequestEnableAsync did not complete: {e}"),
    }
}

fn refresh_cached_state(task: &StartupTask) {
    match task.State() {
        Ok(state) => {
            let enabled =
                state == StartupTaskState::Enabled || state == StartupTaskState::EnabledByPolicy;
            IS_ENABLED.store(enabled, Ordering::Release);
        }
        Err(e) => log_err!("StartupTask State failed: {e}"),
    }
}
