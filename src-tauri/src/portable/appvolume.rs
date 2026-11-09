//! Per-app volume on macOS and Linux: not there yet, so the music page shows no app slider.
//! The Windows version is ../appvolume.rs.

use serde::Serialize;

#[derive(Serialize, Clone, Copy)]
pub struct AppVolume {
    level: f32,
    muted: bool,
}

#[tauri::command]
pub async fn app_volume_get(session: String) -> Option<AppVolume> {
    let _ = session;
    None
}

#[tauri::command]
pub async fn app_volume_set(session: String, level: f32) -> bool {
    let _ = (session, level);
    false
}

#[tauri::command]
pub async fn app_volume_mute(session: String, muted: bool) -> bool {
    let _ = (session, muted);
    false
}
