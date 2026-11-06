//! Settings and the small data store (notes, shelf, pinned clips).
//!
//! The frontend owns the settings schema and its defaults (src/lib/stores/settings.svelte.ts).
//! Rust keeps the JSON object, saves it, broadcasts changes and reads the handful of keys it
//! needs itself through the typed accessors below, each with its own default.

use std::{path::PathBuf, sync::Mutex};

use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::util::{self, merge, read_json, write_json};

pub struct Config {
    dir: PathBuf,
    settings: Mutex<Value>,
    data: Mutex<Value>,
}

impl Config {
    pub fn load(app: &AppHandle) -> Self {
        let dir = app
            .path()
            .app_config_dir()
            .unwrap_or_else(|_| util::home_dir().join(".tensei-island"));
        let _ = std::fs::create_dir_all(&dir);
        Self {
            settings: Mutex::new(read_json(&dir.join("settings.json"))),
            data: Mutex::new(read_json(&dir.join("data.json"))),
            dir,
        }
    }

    pub fn dir(&self) -> &PathBuf {
        &self.dir
    }

    pub fn snapshot(&self) -> Value {
        self.settings
            .lock()
            .map(|s| s.clone())
            .unwrap_or(Value::Null)
    }

    pub fn bool(&self, key: &str, default: bool) -> bool {
        self.settings
            .lock()
            .ok()
            .and_then(|s| s.get(key).and_then(Value::as_bool))
            .unwrap_or(default)
    }

    pub fn u64(&self, key: &str, default: u64) -> u64 {
        self.settings
            .lock()
            .ok()
            .and_then(|s| s.get(key).and_then(Value::as_u64))
            .unwrap_or(default)
    }

    pub fn string(&self, key: &str, default: &str) -> String {
        self.settings
            .lock()
            .ok()
            .and_then(|s| s.get(key).and_then(Value::as_str).map(str::to_string))
            .unwrap_or_else(|| default.to_string())
    }

    pub fn patch(&self, patch: &Value) -> Value {
        let mut settings = self.settings.lock().expect("settings lock");
        merge(&mut settings, patch);
        if let Err(err) = write_json(&self.dir.join("settings.json"), &settings) {
            util::log(&self.dir, &format!("settings save failed: {err}"));
        }
        settings.clone()
    }

    pub fn data_get(&self, key: &str) -> Value {
        self.data
            .lock()
            .ok()
            .and_then(|d| d.get(key).cloned())
            .unwrap_or(Value::Null)
    }

    pub fn data_set(&self, key: &str, value: Value) {
        let mut data = self.data.lock().expect("data lock");
        if let Some(map) = data.as_object_mut() {
            if value.is_null() {
                map.remove(key);
            } else {
                map.insert(key.to_string(), value);
            }
        }
        if let Err(err) = write_json(&self.dir.join("data.json"), &data) {
            util::log(&self.dir, &format!("data save failed: {err}"));
        }
    }
}

#[tauri::command]
pub fn settings_get(config: State<'_, Config>) -> Value {
    config.snapshot()
}

#[tauri::command]
pub fn settings_patch(app: AppHandle, config: State<'_, Config>, patch: Value) -> Value {
    let next = config.patch(&patch);
    let _ = app.emit("settings://changed", &next);
    crate::on_settings_changed(&app, &patch);
    next
}

#[tauri::command]
pub fn data_get(config: State<'_, Config>, key: String) -> Value {
    config.data_get(&key)
}

#[tauri::command]
pub fn data_set(config: State<'_, Config>, key: String, value: Value) {
    config.data_set(&key, value);
}
