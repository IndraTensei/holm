//! Start with Windows (HKCU Run key), opening files and links, showing files in Explorer.

use std::os::windows::process::CommandExt;

use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::ERROR_SUCCESS,
        System::Registry::{
            RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
            HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SZ,
        },
        UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL},
    },
};

use crate::util::wide;

const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const VALUE: &str = "Holm";
/// The Run value written before the rename to Holm.
const LEGACY_VALUE: &str = "TenseiIsland";

fn run_key(access: windows::Win32::System::Registry::REG_SAM_FLAGS) -> Option<HKEY> {
    let path = wide(RUN_KEY);
    let mut key = HKEY::default();
    let status = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(path.as_ptr()),
            Some(0),
            access,
            &mut key,
        )
    };
    (status == ERROR_SUCCESS).then_some(key)
}

pub fn autostart_enabled() -> bool {
    run_value_exists(VALUE)
}

fn run_value_exists(value: &str) -> bool {
    let Some(key) = run_key(KEY_QUERY_VALUE) else {
        return false;
    };
    let name = wide(value);
    let status = unsafe { RegQueryValueExW(key, PCWSTR(name.as_ptr()), None, None, None, None) };
    unsafe {
        let _ = RegCloseKey(key);
    }
    status == ERROR_SUCCESS
}

/// Before the rename the Run value was "TenseiIsland" and pointed at tensei-island.exe: if it is
/// there, start with Windows stays on under the new name and path.
pub fn migrate_legacy_autostart() {
    if !run_value_exists(LEGACY_VALUE) {
        return;
    }
    if let Some(key) = run_key(KEY_SET_VALUE) {
        let name = wide(LEGACY_VALUE);
        unsafe {
            let _ = RegDeleteValueW(key, PCWSTR(name.as_ptr()));
            let _ = RegCloseKey(key);
        }
    }
    let _ = set_autostart(true);
}

pub fn set_autostart(on: bool) -> Result<(), String> {
    let key = run_key(KEY_SET_VALUE).ok_or("Could not open the Run key")?;
    let name = wide(VALUE);
    let status = unsafe {
        if on {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let command = wide(&format!("\"{}\" --autostart", exe.display()));
            let bytes =
                std::slice::from_raw_parts(command.as_ptr() as *const u8, command.len() * 2);
            RegSetValueExW(key, PCWSTR(name.as_ptr()), Some(0), REG_SZ, Some(bytes))
        } else {
            let status = RegDeleteValueW(key, PCWSTR(name.as_ptr()));
            if status.0 == 2 {
                ERROR_SUCCESS
            } else {
                status
            }
        }
    };
    unsafe {
        let _ = RegCloseKey(key);
    }
    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(format!("Registry error {}", status.0))
    }
}

fn shell_open(target: &str) -> Result<(), String> {
    let verb = wide("open");
    let file = wide(target);
    let result = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(verb.as_ptr()),
            PCWSTR(file.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    // ShellExecute returns a value above 32 on success.
    if result.0 as isize > 32 {
        Ok(())
    } else {
        Err(format!("Could not open {target}"))
    }
}

#[tauri::command]
pub fn autostart_get() -> bool {
    autostart_enabled()
}

#[tauri::command]
pub fn autostart_set(on: bool) -> Result<bool, String> {
    set_autostart(on)?;
    Ok(autostart_enabled())
}

#[tauri::command]
pub fn open_path(path: String) -> Result<(), String> {
    shell_open(&path)
}

#[tauri::command]
pub fn reveal_path(path: String) -> Result<(), String> {
    std::process::Command::new("explorer.exe")
        .raw_arg(format!("/select,\"{}\"", path.replace('"', "")))
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Links and Windows settings pages only (https:, ms-settings:, spotify:, …), never programs.
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    let lower = url.to_ascii_lowercase();
    let allowed = [
        "https://",
        "http://",
        "ms-settings:",
        "spotify:",
        "mailto:",
        "ms-actioncenter:",
    ];
    if !allowed.iter().any(|p| lower.starts_with(p)) {
        return Err("Not an allowed link".into());
    }
    shell_open(&url)
}

#[tauri::command]
pub fn path_exists(paths: Vec<String>) -> Vec<bool> {
    paths
        .iter()
        .map(|p| std::path::Path::new(p).exists())
        .collect()
}
