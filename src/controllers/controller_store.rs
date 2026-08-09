use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct ControllerStatus {
    pub name: String,
    pub battery_percent: u8,
    pub is_charging: bool,
    pub is_fully_charged: bool,
    pub is_bluetooth: bool,
    pub path: String,
    pub last_read_failed: bool,
    pub unexpected_battery_data: bool,
}
static CONTROLLERS: RwLock<Vec<ControllerStatus>> = RwLock::new(Vec::new());

/// Bumped every time `set_controllers` runs, so callers can cheaply detect
/// whether a poll has completed since they last read the store.
static GENERATION: AtomicU64 = AtomicU64::new(0);

pub fn set_controllers(status: Vec<ControllerStatus>) {
    *CONTROLLERS.write().expect("controller store poisoned") = status;
    GENERATION.fetch_add(1, Ordering::Release);
}

pub fn get_controllers() -> Vec<ControllerStatus> {
    CONTROLLERS
        .read()
        .expect("controller store poisoned")
        .clone()
}

pub fn get_generation() -> u64 {
    GENERATION.load(Ordering::Acquire)
}
