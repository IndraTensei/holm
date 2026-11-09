//! Start at login and opening files and links on macOS and Linux. Same commands as the Windows
//! version in ../shell.rs. Start at login is a LaunchAgent on macOS and an XDG autostart entry on
//! Linux, both in the user's own folders, both started with `--autostart`.

use std::{fs, path::PathBuf, process::Command};

use crate::util::home_dir;

#[cfg(target_os = "macos")]
fn autostart_file() -> PathBuf {
    home_dir().join("Library/LaunchAgents/in.tenseitech.holm.plist")
}

#[cfg(not(target_os = "macos"))]
fn autostart_file() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".config"))
        .join("autostart/holm.desktop")
}

#[cfg(target_os = "macos")]
fn autostart_contents(exe: &str) -> String {
    let exe = exe
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>in.tenseitech.holm</string>
  <key>ProgramArguments</key><array><string>{exe}</string><string>--autostart</string></array>
  <key>RunAtLoad</key><true/>
</dict>
</plist>
"#
    )
}

#[cfg(not(target_os = "macos"))]
fn autostart_contents(exe: &str) -> String {
    // Desktop Entry Exec quoting: the path in double quotes, with ", `, $ and \ escaped.
    let quoted = exe
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('`', "\\`")
        .replace('$', "\\$");
    format!(
        "[Desktop Entry]\nType=Application\nName=Holm\nComment=A lean Dynamic Island\n\
         Exec=\"{quoted}\" --autostart\nX-GNOME-Autostart-enabled=true\nNoDisplay=true\n"
    )
}

pub fn autostart_enabled() -> bool {
    autostart_file().is_file()
}

/// Nothing to migrate: the old name only ever shipped on Windows.
pub fn migrate_legacy_autostart() {}

/// An entry that names an exe that no longer exists (a moved AppImage) is pointed at this one.
pub fn repair_autostart() {
    let Ok(text) = fs::read_to_string(autostart_file()) else {
        return;
    };
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    if !text.contains(&*exe.to_string_lossy()) {
        let _ = set_autostart(true);
    }
}

pub fn set_autostart(on: bool) -> Result<(), String> {
    let file = autostart_file();
    if !on {
        return match fs::remove_file(&file) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.to_string()),
            _ => Ok(()),
        };
    }
    // An AppImage runs from a temporary mount; $APPIMAGE is the file the user actually has.
    let exe = std::env::var("APPIMAGE")
        .ok()
        .map(PathBuf::from)
        .or_else(|| std::env::current_exe().ok())
        .ok_or("Couldn't find Holm's own program file.")?;
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    fs::write(&file, autostart_contents(&exe.to_string_lossy())).map_err(|e| e.to_string())
}

/// The system's own opener: `open` on macOS, `xdg-open` elsewhere.
fn shell_open(target: &str) -> Result<(), String> {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    Command::new(opener)
        .arg(target)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
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

/// macOS selects the file in Finder; Linux file managers differ, so the folder opens instead.
#[tauri::command]
pub fn reveal_path(path: String) -> Result<(), String> {
    if cfg!(target_os = "macos") {
        return Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string());
    }
    let folder = std::path::Path::new(&path)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or(path);
    shell_open(&folder)
}

/// Links only (https:, spotify:, mailto:), never programs.
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    let lower = url.to_ascii_lowercase();
    let allowed = ["https://", "http://", "spotify:", "mailto:"];
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
