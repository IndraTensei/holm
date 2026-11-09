//! Wi-Fi and Bluetooth toggles on macOS and Linux: not there yet. `allowed: false` with no
//! radios is what Windows reports when it denies access, so the page hides the toggles.
//! The Windows version is ../radios.rs.

use serde::Serialize;

#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Radios {
    allowed: bool,
    wifi: Option<bool>,
    bluetooth: Option<bool>,
    ssid: Option<String>,
    signal: Option<u32>,
}

#[tauri::command]
pub async fn radios_get() -> Radios {
    Radios::default()
}

#[tauri::command]
pub async fn radio_set(kind: String, on: bool) -> Result<Radios, String> {
    let _ = (kind, on);
    Err("Wi-Fi and Bluetooth toggles aren't available on this system yet.".into())
}
