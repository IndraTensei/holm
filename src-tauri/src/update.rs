//! Updates and uninstall, from Settings → About.
//!
//! Updates: the latest GitHub release is asked for (one small JSON request, nothing about the user
//! is sent). An installed copy downloads that release's setup exe to the temp folder, checks it,
//! and runs it in passive mode (`/P`: a progress window, no questions) with `/R` so Holm starts
//! again when it's done; Holm quits first so its files can be replaced. A portable copy can't
//! replace itself, so it gets the download page instead.
//!
//! Uninstall: an installed copy runs its own uninstall.exe (which asks for confirmation and offers
//! to delete settings too) and quits. A portable copy has nothing to run; the page says how to
//! remove it.

use std::{
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};

use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};

use crate::{net::agent, sha256};

const LATEST_API: &str = "https://api.github.com/repos/IndraTensei/holm/releases/latest";
const DOWNLOAD_PREFIX: &str = "https://github.com/IndraTensei/holm/releases/download/";
pub const RELEASES_PAGE: &str = "https://github.com/IndraTensei/holm/releases/latest";
/// Far above the ~2 MB installer; a guard against a wrong or hostile response.
const MAX_INSTALLER: u64 = 64 * 1024 * 1024;
/// The shared agent gives up on any request after 15 s, which a slow connection can need for the
/// installer alone; the download gets this long instead.
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(600);

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    current: String,
    latest: String,
    available: bool,
    /// The release page, for "What's new".
    url: String,
    /// This copy was installed (so it can update itself), not run as the portable exe.
    installed: bool,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Progress {
    received: u64,
    total: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UninstallInfo {
    /// An uninstaller sits next to this exe (an installed copy).
    installed: bool,
    /// Where settings and data live, for the portable copy's instructions.
    data_dir: String,
}

/// "v0.1.10" → [0, 1, 10]; anything after a '-' or '+' (pre-release, build) is ignored.
fn parse_version(text: &str) -> Vec<u64> {
    text.trim()
        .trim_start_matches(['v', 'V'])
        .split(['-', '+'])
        .next()
        .unwrap_or("")
        .split('.')
        .map(|part| part.parse().unwrap_or(0))
        .collect()
}

fn newer(latest: &str, current: &str) -> bool {
    let (mut a, mut b) = (parse_version(latest), parse_version(current));
    let len = a.len().max(b.len());
    a.resize(len, 0);
    b.resize(len, 0);
    a > b
}

fn uninstaller() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let path = exe.parent()?.join("uninstall.exe");
    path.is_file().then_some(path)
}

struct Installer {
    url: String,
    size: u64,
    /// The SHA-256 GitHub publishes for the file ("sha256:<hex>"), when it gives one.
    sha256: Option<String>,
}

struct Release {
    tag: String,
    url: String,
    /// The x64 setup exe, when the release has one.
    installer: Option<Installer>,
}

fn latest_release() -> Result<Release, String> {
    let mut response = agent()
        .get(LATEST_API)
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|_| "Couldn't reach GitHub. Check your connection and try again.".to_string())?;
    let status = response.status().as_u16();
    if status != 200 {
        return Err(format!(
            "GitHub answered {status}. Try again in a few minutes."
        ));
    }
    let body: Value = response
        .body_mut()
        .read_json()
        .map_err(|_| "GitHub's answer couldn't be read. Try again later.".to_string())?;
    let tag = body["tag_name"].as_str().unwrap_or_default().to_string();
    if tag.is_empty() {
        return Err("GitHub didn't say which version is the latest. Try again later.".into());
    }
    let installer = body["assets"].as_array().and_then(|assets| {
        assets.iter().find_map(|asset| {
            let name = asset["name"].as_str()?;
            let url = asset["browser_download_url"].as_str()?;
            (name.ends_with("_x64-setup.exe") && url.starts_with(DOWNLOAD_PREFIX)).then(|| Installer {
                url: url.to_string(),
                size: asset["size"].as_u64().unwrap_or(0),
                sha256: asset["digest"]
                    .as_str()
                    .and_then(|d| d.strip_prefix("sha256:"))
                    .map(str::to_ascii_lowercase),
            })
        })
    });
    Ok(Release {
        tag,
        url: body["html_url"]
            .as_str()
            .unwrap_or(RELEASES_PAGE)
            .to_string(),
        installer,
    })
}

#[tauri::command]
pub async fn update_check(app: AppHandle) -> Result<UpdateInfo, String> {
    let current = app.package_info().version.to_string();
    tauri::async_runtime::spawn_blocking(move || {
        let release = latest_release()?;
        Ok(UpdateInfo {
            available: newer(&release.tag, &current),
            latest: release.tag.trim_start_matches('v').to_string(),
            current,
            url: release.url,
            installed: uninstaller().is_some(),
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

fn failed<E>(_: E) -> String {
    "The download stopped. Check your connection and try again.".to_string()
}

/// Downloads the latest installer to `dir`, reporting progress, and checks it is exactly the file
/// the release lists: same size, an exe, and the same SHA-256 when GitHub publishes one.
fn download(app: &AppHandle, installer: &Installer, dir: &Path) -> Result<PathBuf, String> {
    let (url, size) = (installer.url.as_str(), installer.size);
    let mut response = agent()
        .get(url)
        .config()
        .timeout_global(Some(DOWNLOAD_TIMEOUT))
        .build()
        .call()
        .map_err(failed)?;
    if response.status().as_u16() != 200 {
        return Err(format!(
            "The download failed ({}). Try again in a few minutes.",
            response.status().as_u16()
        ));
    }
    let name = url.rsplit('/').next().unwrap_or("Holm-setup.exe");
    let path = dir.join(name);
    let mut reader = response
        .body_mut()
        .with_config()
        .limit(MAX_INSTALLER)
        .reader();
    let mut bytes = Vec::with_capacity(size as usize);
    let mut chunk = [0u8; 64 * 1024];
    let mut last_sent = 0u64;
    loop {
        let read = reader.read(&mut chunk).map_err(failed)?;
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..read]);
        let received = bytes.len() as u64;
        if received - last_sent >= 128 * 1024 || received == size {
            last_sent = received;
            let _ = app.emit("update://progress", Progress { received, total: size });
        }
    }
    if (size > 0 && bytes.len() as u64 != size) || !bytes.starts_with(b"MZ") {
        return Err("The downloaded installer isn't complete. Try again.".into());
    }
    if let Some(expected) = &installer.sha256 {
        if sha256::hex(&bytes) != *expected {
            return Err(
                "The downloaded installer doesn't match the release, so it wasn't run. Try again."
                    .into(),
            );
        }
    }
    std::fs::write(&path, &bytes).map_err(|e| format!("Couldn't save the installer: {e}"))?;
    Ok(path)
}

/// Downloads and runs the latest installer, then quits so it can replace Holm's files.
#[tauri::command]
pub async fn update_install(app: AppHandle) -> Result<(), String> {
    if uninstaller().is_none() {
        return Err("This is the portable copy: download the new version from the releases page.".into());
    }
    let current = app.package_info().version.to_string();
    let handle = app.clone();
    let installer = tauri::async_runtime::spawn_blocking(move || {
        let release = latest_release()?;
        if !newer(&release.tag, &current) {
            return Err("You already have the latest version.".to_string());
        }
        let installer = release
            .installer
            .ok_or("This release has no installer yet. Try again in a few minutes.")?;
        download(&handle, &installer, &std::env::temp_dir())
    })
    .await
    .map_err(|e| e.to_string())??;
    std::process::Command::new(&installer)
        .args(["/P", "/R"])
        .spawn()
        .map_err(|e| format!("Couldn't start the installer: {e}"))?;
    app.exit(0);
    Ok(())
}

/// The installer from the last update can't delete itself, and it starts Holm again (`/R`) while
/// it is still running, so its file is removed from a background thread a little later, trying a
/// few times until the installer has let go of it.
pub fn clean_downloads() {
    let _ = std::thread::Builder::new()
        .name("clean-downloads".into())
        .spawn(|| {
            for wait in [5, 30, 120] {
                std::thread::sleep(Duration::from_secs(wait));
                let Ok(entries) = std::fs::read_dir(std::env::temp_dir()) else {
                    return;
                };
                let mut left = 0;
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if name.starts_with("Holm_")
                        && name.ends_with("_x64-setup.exe")
                        && std::fs::remove_file(entry.path()).is_err()
                    {
                        left += 1;
                    }
                }
                if left == 0 {
                    return;
                }
            }
        });
}

#[tauri::command]
pub fn uninstall_info(app: AppHandle) -> UninstallInfo {
    UninstallInfo {
        installed: uninstaller().is_some(),
        data_dir: app
            .path()
            .app_config_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default(),
    }
}

/// Runs Holm's uninstaller (it asks to confirm, and whether to delete settings) and quits.
#[tauri::command]
pub fn uninstall(app: AppHandle) -> Result<(), String> {
    let path = uninstaller().ok_or("This is the portable copy: there's nothing to uninstall.")?;
    std::process::Command::new(path)
        .spawn()
        .map_err(|e| format!("Couldn't start the uninstaller: {e}"))?;
    app.exit(0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::newer;

    #[test]
    fn compares_versions_by_number() {
        assert!(newer("v0.1.5", "0.1.4"));
        assert!(newer("v0.1.10", "0.1.9"));
        assert!(newer("v0.2.0", "0.1.99"));
        assert!(newer("v1.0", "0.9.9"));
        assert!(!newer("v0.1.4", "0.1.4"));
        assert!(!newer("v0.1.3", "0.1.4"));
        assert!(!newer("v0.1.4-beta.1", "0.1.4"));
    }
}
