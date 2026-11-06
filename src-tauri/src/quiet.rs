//! Windows' Do not disturb: while it is on, the island keeps its news to itself (no pop-ups,
//! no chimes) and shows what it held once it is off again.
//!
//! Windows 11 keeps the switch in `NOC_GLOBAL_SETTING_TOASTS_ENABLED` (0 = Do not disturb on,
//! absent or 1 = off). The thread blocks in RegNotifyChangeKeyValue until the key changes: no polling.

use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

use tauri::{AppHandle, Emitter};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::ERROR_SUCCESS,
        System::Registry::{
            RegCloseKey, RegGetValueW, RegNotifyChangeKeyValue, RegOpenKeyExW, HKEY,
            HKEY_CURRENT_USER, KEY_NOTIFY, KEY_QUERY_VALUE, REG_NOTIFY_CHANGE_LAST_SET,
            RRF_RT_REG_DWORD,
        },
    },
};

use crate::util::wide;

const KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Notifications\\Settings";
const VALUE: &str = "NOC_GLOBAL_SETTING_TOASTS_ENABLED";

static QUIET: AtomicBool = AtomicBool::new(false);

fn read(key: HKEY) -> bool {
    let name = wide(VALUE);
    let mut value = 1u32;
    let mut size = std::mem::size_of::<u32>() as u32;
    let status = unsafe {
        RegGetValueW(
            key,
            PCWSTR::null(),
            PCWSTR(name.as_ptr()),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut value as *mut u32 as *mut core::ffi::c_void),
            Some(&mut size),
        )
    };
    status == ERROR_SUCCESS && value == 0
}

pub fn start(app: AppHandle) {
    let _ = thread::Builder::new().name("quiet".into()).spawn(move || {
        let path = wide(KEY);
        let mut key = HKEY::default();
        let opened = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(path.as_ptr()),
                Some(0),
                KEY_QUERY_VALUE | KEY_NOTIFY,
                &mut key,
            )
        };
        if opened != ERROR_SUCCESS {
            return;
        }
        loop {
            let quiet = read(key);
            if quiet != QUIET.swap(quiet, Ordering::Relaxed) {
                let _ = app.emit("island://quiet", quiet);
            }
            // Blocks until a value under the key changes.
            let status = unsafe {
                RegNotifyChangeKeyValue(key, false, REG_NOTIFY_CHANGE_LAST_SET, None, false)
            };
            if status != ERROR_SUCCESS {
                break;
            }
        }
        unsafe {
            let _ = RegCloseKey(key);
        }
    });
}

/// For the frontend at start-up, before any change event.
#[tauri::command]
pub fn quiet_get() -> bool {
    QUIET.load(Ordering::Relaxed)
}
