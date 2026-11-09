//! Clipboard history on macOS and Linux: not there yet, so the list stays empty.
//! The Windows version is ../clipboard.rs.

use serde::Serialize;
use tauri::AppHandle;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    id: i64,
    text: String,
    at: i64,
}

pub fn start(app: AppHandle) {
    let _ = app;
}

#[tauri::command]
pub fn clipboard_list() -> Vec<Clip> {
    Vec::new()
}

#[tauri::command]
pub fn clipboard_copy(text: String) {
    let _ = text;
}

#[tauri::command]
pub fn clipboard_clear() {}
