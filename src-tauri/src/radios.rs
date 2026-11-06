//! Wi-Fi and Bluetooth quick toggles (Windows.Devices.Radios) and the current Wi-Fi network
//! (WLAN API). Windows 11 24H2 only reveals the network name to apps allowed to use location;
//! without it the island shows "Connected".

use serde::Serialize;
use windows::{
    Devices::Radios::{Radio, RadioAccessStatus, RadioKind, RadioState},
    Win32::{
        Foundation::HANDLE,
        NetworkManagement::WiFi::{
            wlan_interface_state_connected, wlan_intf_opcode_current_connection, WlanCloseHandle,
            WlanEnumInterfaces, WlanFreeMemory, WlanOpenHandle, WlanQueryInterface,
            WLAN_CONNECTION_ATTRIBUTES, WLAN_INTERFACE_INFO_LIST,
        },
    },
};

use crate::util::ensure_mta;

#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Radios {
    allowed: bool,
    wifi: Option<bool>,
    bluetooth: Option<bool>,
    ssid: Option<String>,
    signal: Option<u32>,
}

fn read_radios() -> windows_core::Result<(bool, Option<bool>, Option<bool>)> {
    let access = Radio::RequestAccessAsync()?.join()?;
    let allowed = access == RadioAccessStatus::Allowed;
    let mut wifi = None;
    let mut bluetooth = None;
    for radio in Radio::GetRadiosAsync()?.join()? {
        let on = radio.State()? == RadioState::On;
        match radio.Kind()? {
            RadioKind::WiFi => wifi = Some(wifi.unwrap_or(false) || on),
            RadioKind::Bluetooth => bluetooth = Some(bluetooth.unwrap_or(false) || on),
            _ => {}
        }
    }
    Ok((allowed, wifi, bluetooth))
}

fn current_wifi() -> (Option<String>, Option<u32>) {
    unsafe {
        let mut version = 0u32;
        let mut handle = HANDLE::default();
        if WlanOpenHandle(2, None, &mut version, &mut handle) != 0 {
            return (None, None);
        }
        let mut result = (None, None);
        let mut list: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();
        if WlanEnumInterfaces(handle, None, &mut list) == 0 && !list.is_null() {
            let count = (*list).dwNumberOfItems as usize;
            let items = std::slice::from_raw_parts((*list).InterfaceInfo.as_ptr(), count);
            for info in items {
                if info.isState != wlan_interface_state_connected {
                    continue;
                }
                let mut size = 0u32;
                let mut data: *mut core::ffi::c_void = std::ptr::null_mut();
                let status = WlanQueryInterface(
                    handle,
                    &info.InterfaceGuid,
                    wlan_intf_opcode_current_connection,
                    None,
                    &mut size,
                    &mut data,
                    None,
                );
                if status == 0 && !data.is_null() {
                    let attributes = &*(data as *const WLAN_CONNECTION_ATTRIBUTES);
                    let ssid = attributes.wlanAssociationAttributes.dot11Ssid;
                    let len = (ssid.uSSIDLength as usize).min(32);
                    let name = String::from_utf8_lossy(&ssid.ucSSID[..len]).into_owned();
                    result = (
                        (!name.is_empty()).then_some(name),
                        Some(attributes.wlanAssociationAttributes.wlanSignalQuality),
                    );
                    WlanFreeMemory(data);
                } else {
                    // Connected, but the name is withheld (location permission).
                    result = (Some(String::new()), None);
                }
                break;
            }
            WlanFreeMemory(list as *const core::ffi::c_void);
        }
        let _ = WlanCloseHandle(handle, None);
        result
    }
}

fn snapshot() -> Radios {
    ensure_mta();
    let (allowed, wifi, bluetooth) = read_radios().unwrap_or((false, None, None));
    let (ssid, signal) = current_wifi();
    Radios {
        allowed,
        wifi,
        bluetooth,
        ssid,
        signal,
    }
}

#[tauri::command]
pub async fn radios_get() -> Radios {
    tauri::async_runtime::spawn_blocking(snapshot)
        .await
        .unwrap_or_default()
}

#[tauri::command]
pub async fn radio_set(kind: String, on: bool) -> Result<Radios, String> {
    tauri::async_runtime::spawn_blocking(move || {
        ensure_mta();
        let target = if kind == "bluetooth" {
            RadioKind::Bluetooth
        } else {
            RadioKind::WiFi
        };
        let radios = Radio::GetRadiosAsync()
            .and_then(|op| op.join())
            .map_err(|e| e.message())?;
        let mut changed = false;
        for radio in radios {
            if radio.Kind().ok() == Some(target) {
                let state = if on { RadioState::On } else { RadioState::Off };
                let status = radio
                    .SetStateAsync(state)
                    .and_then(|op| op.join())
                    .map_err(|e| e.message())?;
                if status != RadioAccessStatus::Allowed {
                    return Err("Windows did not allow the change".to_string());
                }
                changed = true;
            }
        }
        if !changed {
            return Err("No such radio on this PC".to_string());
        }
        Ok(snapshot())
    })
    .await
    .map_err(|e| e.to_string())?
}
