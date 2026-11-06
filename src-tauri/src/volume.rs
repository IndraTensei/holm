//! System volume: a callback from Core Audio drives the volume live activity (the island's
//! answer to the Windows volume flyout), and the Music page can set the level.

use std::{
    sync::{
        mpsc::{self, RecvTimeoutError, Sender},
        Mutex, OnceLock,
    },
    thread,
    time::Duration,
};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use windows::Win32::{
    Media::Audio::{
        eConsole, eRender,
        Endpoints::{
            IAudioEndpointVolume, IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl,
        },
        IMMDeviceEnumerator, MMDeviceEnumerator, AUDIO_VOLUME_NOTIFICATION_DATA,
    },
    System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_ALL},
};
use windows_core::implement;

use crate::util::ensure_mta;

#[derive(Serialize, Clone, Copy, Default)]
pub struct Volume {
    level: f32,
    muted: bool,
}

enum Msg {
    Set(f32),
    Mute(bool),
    Get(Sender<Volume>),
}

static SENDER: OnceLock<Mutex<Sender<Msg>>> = OnceLock::new();

#[implement(IAudioEndpointVolumeCallback)]
struct Callback {
    app: AppHandle,
}

impl IAudioEndpointVolumeCallback_Impl for Callback_Impl {
    fn OnNotify(&self, data: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> windows_core::Result<()> {
        if let Some(data) = unsafe { data.as_ref() } {
            let volume = Volume {
                level: data.fMasterVolume,
                muted: data.bMuted.as_bool(),
            };
            let _ = self.app.emit("volume://changed", volume);
        }
        Ok(())
    }
}

fn default_device_id(enumerator: &IMMDeviceEnumerator) -> Option<String> {
    unsafe {
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole).ok()?;
        let id = device.GetId().ok()?;
        let text = id.to_string().ok();
        CoTaskMemFree(Some(id.0 as *const core::ffi::c_void));
        text
    }
}

pub fn start(app: AppHandle) {
    let (tx, rx) = mpsc::channel::<Msg>();
    let _ = SENDER.set(Mutex::new(tx));
    thread::Builder::new()
        .name("volume".into())
        .spawn(move || unsafe {
            ensure_mta();
            let Ok(enumerator) =
                CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL)
            else {
                return;
            };
            loop {
                let device_id = default_device_id(&enumerator);
                let endpoint = enumerator
                    .GetDefaultAudioEndpoint(eRender, eConsole)
                    .and_then(|device| device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None));
                let Ok(endpoint) = endpoint else {
                    // No output device right now (unplugged headset): look again shortly.
                    match rx.recv_timeout(Duration::from_secs(3)) {
                        Ok(Msg::Get(reply)) => {
                            let _ = reply.send(Volume::default());
                        }
                        Err(RecvTimeoutError::Disconnected) => return,
                        _ => {}
                    }
                    continue;
                };
                let callback: IAudioEndpointVolumeCallback = Callback { app: app.clone() }.into();
                let _ = endpoint.RegisterControlChangeNotify(&callback);
                loop {
                    match rx.recv_timeout(Duration::from_secs(3)) {
                        Ok(Msg::Set(level)) => {
                            let _ = endpoint.SetMasterVolumeLevelScalar(
                                level.clamp(0.0, 1.0),
                                std::ptr::null(),
                            );
                        }
                        Ok(Msg::Mute(muted)) => {
                            let _ = endpoint.SetMute(muted, std::ptr::null());
                        }
                        Ok(Msg::Get(reply)) => {
                            let level = endpoint.GetMasterVolumeLevelScalar().unwrap_or(0.0);
                            let muted = endpoint.GetMute().map(|m| m.as_bool()).unwrap_or(false);
                            let _ = reply.send(Volume { level, muted });
                        }
                        Err(RecvTimeoutError::Timeout) => {
                            // The default output changed (headphones, HDMI): follow it.
                            if default_device_id(&enumerator) != device_id {
                                break;
                            }
                        }
                        Err(RecvTimeoutError::Disconnected) => return,
                    }
                }
                let _ = endpoint.UnregisterControlChangeNotify(&callback);
            }
        })
        .expect("volume thread");
}

fn send(msg: Msg) {
    if let Some(tx) = SENDER.get() {
        if let Ok(tx) = tx.lock() {
            let _ = tx.send(msg);
        }
    }
}

#[tauri::command]
pub async fn volume_get() -> Volume {
    let (tx, rx) = mpsc::channel();
    send(Msg::Get(tx));
    tauri::async_runtime::spawn_blocking(move || {
        rx.recv_timeout(Duration::from_secs(2)).unwrap_or_default()
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
pub fn volume_set(level: f32) {
    send(Msg::Set(level));
}

#[tauri::command]
pub fn volume_mute(muted: bool) {
    send(Msg::Mute(muted));
}
