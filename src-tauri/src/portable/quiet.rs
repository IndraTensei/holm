//! Do not disturb on macOS and Linux: not read yet, so it counts as off.
//! The Windows version is ../quiet.rs.

use tauri::AppHandle;

pub fn start(app: AppHandle) {
    let _ = app;
}

#[tauri::command]
pub fn quiet_get() -> bool {
    false
}
