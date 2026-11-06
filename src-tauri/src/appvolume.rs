//! Each player's own volume through the Windows mixer (one audio session per app): Spotify,
//! browsers, VLC, Apple Music and anything else that plays sound, with no setup. The Music page
//! sets the playing app's volume instead of the whole PC's.
//!
//! A media session (GSMTC) names its app by an app id ("Spotify.exe", "MSEdge",
//! "SpotifyAB.SpotifyMusic_…!Spotify"); audio sessions name a process. They are matched by the
//! program name. No match (an app that plays through another process) falls back to the PC volume.

use serde::Serialize;
use windows::Win32::{
    Media::Audio::{
        eConsole, eRender, IAudioSessionControl2, IAudioSessionManager2, IMMDeviceEnumerator,
        ISimpleAudioVolume, MMDeviceEnumerator,
    },
    System::Com::{CoCreateInstance, CLSCTX_ALL},
};
use windows_core::Interface;

use crate::util::ensure_mta;

#[derive(Serialize, Clone, Copy)]
pub struct AppVolume {
    level: f32,
    muted: bool,
}

/// "SpotifyAB.SpotifyMusic_zpdnekdrzrea0!Spotify" → "spotify", "C:\…\vlc.exe" → "vlc".
fn app_key(id: &str) -> String {
    let lower = id.to_ascii_lowercase();
    if lower.contains("youtube-music") || lower.contains("pear") {
        return "youtube music".into();
    }
    let tail = lower.rsplit(['!', '\\', '/']).next().unwrap_or(&lower);
    let tail = tail.trim_end_matches(".exe");
    // Firefox registers a hash of its install folder as its app id.
    if tail.len() == 16 && tail.chars().all(|c| c.is_ascii_hexdigit()) {
        return "firefox".into();
    }
    tail.rsplit('.').next().unwrap_or(tail).to_string()
}

fn matches(process: &str, key: &str) -> bool {
    process == key
        || (key.len() >= 3 && process.contains(key))
        || (process.len() >= 3 && key.contains(process))
}

/// The mixer entries of the app behind a media session.
fn volumes(session: &str) -> Vec<ISimpleAudioVolume> {
    let key = app_key(session);
    let mut found = Vec::new();
    unsafe {
        ensure_mta();
        let Ok(enumerator) =
            CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL)
        else {
            return found;
        };
        let Ok(device) = enumerator.GetDefaultAudioEndpoint(eRender, eConsole) else {
            return found;
        };
        let Ok(manager) = device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None) else {
            return found;
        };
        let Ok(list) = manager.GetSessionEnumerator() else {
            return found;
        };
        for i in 0..list.GetCount().unwrap_or(0) {
            let Ok(control) = list.GetSession(i) else {
                continue;
            };
            let Ok(control2) = control.cast::<IAudioSessionControl2>() else {
                continue;
            };
            let pid = control2.GetProcessId().unwrap_or(0);
            if pid == 0 {
                continue;
            }
            let Some(name) = crate::games::process_name(pid) else {
                continue;
            };
            if matches(&name, &key) {
                if let Ok(volume) = control.cast::<ISimpleAudioVolume>() {
                    found.push(volume);
                }
            }
        }
    }
    found
}

fn get(session: &str) -> Option<AppVolume> {
    let first = volumes(session).into_iter().next()?;
    unsafe {
        Some(AppVolume {
            level: first.GetMasterVolume().ok()?,
            muted: first.GetMute().map(|m| m.as_bool()).unwrap_or(false),
        })
    }
}

/// The app's own volume, or None when its sound cannot be found in the mixer.
#[tauri::command]
pub async fn app_volume_get(session: String) -> Option<AppVolume> {
    tauri::async_runtime::spawn_blocking(move || get(&session))
        .await
        .ok()
        .flatten()
}

#[tauri::command]
pub async fn app_volume_set(session: String, level: f32) -> bool {
    tauri::async_runtime::spawn_blocking(move || {
        let list = volumes(&session);
        for v in &list {
            unsafe {
                let _ = v.SetMasterVolume(level.clamp(0.0, 1.0), std::ptr::null());
                let _ = v.SetMute(false, std::ptr::null());
            }
        }
        !list.is_empty()
    })
    .await
    .unwrap_or(false)
}

#[tauri::command]
pub async fn app_volume_mute(session: String, muted: bool) -> bool {
    tauri::async_runtime::spawn_blocking(move || {
        let list = volumes(&session);
        for v in &list {
            unsafe {
                let _ = v.SetMute(muted, std::ptr::null());
            }
        }
        !list.is_empty()
    })
    .await
    .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{app_key, matches};

    #[test]
    fn keys() {
        assert_eq!(app_key("Spotify.exe"), "spotify");
        assert_eq!(
            app_key("SpotifyAB.SpotifyMusic_zpdnekdrzrea0!Spotify"),
            "spotify"
        );
        assert_eq!(app_key("C:\\Program Files\\VideoLAN\\VLC\\vlc.exe"), "vlc");
        assert_eq!(app_key("MSEdge"), "msedge");
        assert_eq!(app_key("308046B0AF4A39CB"), "firefox");
        assert!(matches(
            "youtube music",
            &app_key("com.github.th-ch.youtube-music")
        ));
        assert!(matches("chrome", &app_key("Chrome")));
    }
}
