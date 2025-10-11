use std::sync::{OnceLock, RwLock};

#[derive(Clone)]
pub struct ControllerStatus {
    pub name: String,
    pub battery: u8,
    pub charging: bool,
}

pub static CONTROLLERS: OnceLock<RwLock<Vec<ControllerStatus>>> = OnceLock::new();

pub fn set_controllers(status: Vec<ControllerStatus>) {
    if let Some(lock) = CONTROLLERS.get() {
        *lock.write().unwrap() = status;
    } else {
        let _ = CONTROLLERS.set(RwLock::new(status));
    }
}

pub fn get_controllers() -> Vec<ControllerStatus> {
    CONTROLLERS
        .get_or_init(|| RwLock::new(Vec::new()))
        .read()
        .unwrap()
        .clone()
}
