use std::sync::{OnceLock, RwLock};

#[derive(Clone)]
pub struct ControllerStatus {
    pub name: String,
    pub battery_percent: u8,
    pub is_charging: bool,
    pub is_fully_charged: bool,
    pub is_bluetooth: bool,
    pub path: String,
    pub last_read_failed: bool,
}
static CONTROLLERS: OnceLock<RwLock<Vec<ControllerStatus>>> = OnceLock::new();

pub fn set_controllers(status: Vec<ControllerStatus>) {
    if CONTROLLERS.get().is_none() {
        if CONTROLLERS.set(RwLock::new(status)).is_err() {
            eprintln!(" !! Failed to initialize controller store");
        }
        return;
    }

    let mut lock = CONTROLLERS
        .get()
        .expect("controller store missing")
        .write()
        .expect("controller store poisoned");
    *lock = status;
}

pub fn get_controllers() -> Vec<ControllerStatus> {
    CONTROLLERS
        .get_or_init(|| RwLock::new(Vec::new()))
        .read()
        .expect("controller store poisoned")
        .clone()
}
