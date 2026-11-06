//! Small helpers shared by every module: JSON files, timestamps, COM threads, Win32 strings.

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::Value;

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

/// Reads a JSON object from disk; anything missing or unreadable becomes an empty object.
pub fn read_json(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(text.trim_start_matches('\u{feff}')).ok())
        .filter(Value::is_object)
        .unwrap_or_else(|| Value::Object(Default::default()))
}

/// Writes atomically (temp file + rename) so a crash never leaves half a settings file.
pub fn write_json(path: &Path, value: &Value) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    {
        let mut file = fs::File::create(&tmp)?;
        file.write_all(
            serde_json::to_string_pretty(value)
                .unwrap_or_default()
                .as_bytes(),
        )?;
        file.sync_all()?;
    }
    fs::rename(&tmp, path)
}

pub fn home_dir() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Shallow merge: keys in `patch` replace keys in `base`. `null` removes a key.
pub fn merge(base: &mut Value, patch: &Value) {
    if let (Some(base), Some(patch)) = (base.as_object_mut(), patch.as_object()) {
        for (key, value) in patch {
            if value.is_null() {
                base.remove(key);
            } else {
                base.insert(key.clone(), value.clone());
            }
        }
    }
}

pub fn truncate(text: &str, max: usize) -> String {
    let text = text.trim();
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

pub fn file_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}

/// Joins the calling thread to the multithreaded COM apartment. Safe to call more than once.
pub fn ensure_mta() {
    unsafe {
        let _ = windows::Win32::System::Com::CoInitializeEx(
            None,
            windows::Win32::System::Com::COINIT_MULTITHREADED,
        );
    }
}

pub fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// A raw window handle that can cross threads (the handle itself is just a number).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Hwnd(pub isize);

impl Hwnd {
    pub fn get(self) -> windows::Win32::Foundation::HWND {
        windows::Win32::Foundation::HWND(self.0 as *mut core::ffi::c_void)
    }
    pub fn from(hwnd: windows::Win32::Foundation::HWND) -> Self {
        Hwnd(hwnd.0 as isize)
    }
}

/// Appends a line to the app log, kept small by truncating when it grows past 512 KB.
pub fn log(dir: &Path, line: &str) {
    let path = dir.join("island.log");
    if fs::metadata(&path)
        .map(|m| m.len() > 512 * 1024)
        .unwrap_or(false)
    {
        let _ = fs::remove_file(&path);
    }
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(file, "{} {}", now_ms(), line);
    }
}
