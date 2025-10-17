use std::sync::{OnceLock, RwLock};

use crate::ps_battery::get_controller_info::ConnectionType;

#[derive(Clone)]
pub struct ControllerStatus {
    pub name: String,
    pub battery_percent: u8,
    pub is_charging: bool,
    pub connection_type: ConnectionType,
}
static CONTROLLERS: OnceLock<RwLock<Vec<ControllerStatus>>> = OnceLock::new();

pub fn set_controllers(status: Vec<ControllerStatus>) {
    if CONTROLLERS.get().is_none() {
        if CONTROLLERS.set(RwLock::new(status)).is_err() {
            eprintln!("Failed to initialize controller store");
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
