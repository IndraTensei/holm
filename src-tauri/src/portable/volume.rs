//! System volume on macOS and Linux: not there yet, so no volume pop-up and no slider.
//! The Windows version is ../volume.rs.

use serde::Serialize;
use tauri::AppHandle;

#[derive(Serialize, Clone, Copy, Default)]
pub struct Volume {
    level: f32,
    muted: bool,
}

pub fn start(app: AppHandle) {
    let _ = app;
}

#[tauri::command]
pub async fn volume_get() -> Volume {
    Volume::default()
}

#[tauri::command]
pub fn volume_set(level: f32) {
    let _ = level;
}

#[tauri::command]
pub fn volume_mute(muted: bool) {
    let _ = muted;
}
