use crate::log_err;
use windows::Win32::UI::Shell::{
    QUNS_APP, QUNS_BUSY, QUNS_PRESENTATION_MODE, QUNS_QUIET_TIME, QUNS_RUNNING_D3D_FULL_SCREEN,
    SHQueryUserNotificationState,
};

/// True in the states where Windows suppresses notification balloons: busy,
/// presenting, fullscreen Direct3D, quiet time and fullscreen store apps. The
/// alert falls back to a sound so it still reaches the user.
pub fn is_balloon_suppressed() -> bool {
    let state = match unsafe { SHQueryUserNotificationState() } {
        Ok(state) => state,
        Err(e) => {
            log_err!("SHQueryUserNotificationState failed: {e}");
            return false;
        }
    };
    matches!(
        state,
        QUNS_BUSY
            | QUNS_RUNNING_D3D_FULL_SCREEN
            | QUNS_PRESENTATION_MODE
            | QUNS_QUIET_TIME
            | QUNS_APP
    )
}
