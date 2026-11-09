//! Now playing on macOS and Linux: not there yet (Linux: MPRIS over D-Bus; macOS: no public
//! API). With no sessions the island simply shows no music. The Windows version is ../media.rs.

use tauri::AppHandle;

pub fn start(app: AppHandle) {
    let _ = app;
}

#[tauri::command]
pub fn media_control(session: Option<String>, action: String, value: Option<f64>) {
    let _ = (session, action, value);
}

#[tauri::command]
pub fn media_refresh() {}
