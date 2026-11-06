//! Now playing, through Windows media controls (GSMTC): Spotify, browsers, Media Player, Pear
//! and anything else that shows in the volume flyout's media card.
//!
//! Event driven: one thread subscribes to the session manager and each session, coalesces the
//! events and emits `media://state` only when something changed. Album art is read once per
//! track. Control requests from the UI are queued to the same thread, which owns the WinRT objects.

use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Receiver, RecvTimeoutError, Sender},
        Mutex, OnceLock,
    },
    thread,
    time::Duration,
};

use base64::Engine;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use windows::{
    Foundation::TypedEventHandler,
    Media::{
        Control::{
            GlobalSystemMediaTransportControlsSession as Session,
            GlobalSystemMediaTransportControlsSessionManager as Manager,
            GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
        },
        MediaPlaybackAutoRepeatMode,
    },
    Storage::Streams::{DataReader, IRandomAccessStreamReference},
};

use crate::util::{ensure_mta, now_ms};

enum Msg {
    Refresh,
    Control {
        session: Option<String>,
        action: String,
        value: Option<f64>,
    },
}

static SENDER: OnceLock<Mutex<Sender<Msg>>> = OnceLock::new();

#[derive(Serialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct SessionState {
    id: String,
    app: String,
    title: String,
    artist: String,
    album: String,
    art: Option<String>,
    status: &'static str,
    position: f64,
    duration: f64,
    updated_at: i64,
    can_play_pause: bool,
    can_next: bool,
    can_prev: bool,
    can_seek: bool,
    shuffle: Option<bool>,
    repeat: Option<&'static str>,
}

#[derive(Serialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct MediaState {
    sessions: Vec<SessionState>,
    current: Option<String>,
}

struct Subscription {
    session: Session,
    tokens: [i64; 3],
}

impl Drop for Subscription {
    fn drop(&mut self) {
        let _ = self.session.RemoveMediaPropertiesChanged(self.tokens[0]);
        let _ = self.session.RemovePlaybackInfoChanged(self.tokens[1]);
        let _ = self.session.RemoveTimelinePropertiesChanged(self.tokens[2]);
    }
}

pub fn start(app: AppHandle) {
    let (tx, rx) = mpsc::channel();
    let _ = SENDER.set(Mutex::new(tx.clone()));
    thread::Builder::new()
        .name("media".into())
        .spawn(move || {
            ensure_mta();
            // The media service can be slow to answer right after sign-in: retry for a while.
            for attempt in 0..30 {
                match run(&app, tx.clone(), &rx) {
                    Ok(()) => return,
                    Err(_) => thread::sleep(Duration::from_secs(2 + attempt)),
                }
            }
        })
        .expect("media thread");
}

fn send(msg: Msg) {
    if let Some(tx) = SENDER.get() {
        if let Ok(tx) = tx.lock() {
            let _ = tx.send(msg);
        }
    }
}

fn run(app: &AppHandle, tx: Sender<Msg>, rx: &Receiver<Msg>) -> windows_core::Result<()> {
    let manager = Manager::RequestAsync()?.join()?;
    let on_change = {
        let tx = tx.clone();
        move || {
            let _ = tx.send(Msg::Refresh);
        }
    };
    {
        let notify = on_change.clone();
        manager.SessionsChanged(&TypedEventHandler::new(move |_, _| {
            notify();
            Ok(())
        }))?;
    }
    {
        let notify = on_change.clone();
        manager.CurrentSessionChanged(&TypedEventHandler::new(move |_, _| {
            notify();
            Ok(())
        }))?;
    }

    let mut subscriptions: HashMap<String, Subscription> = HashMap::new();
    let mut art: HashMap<String, (String, Option<String>)> = HashMap::new();
    let mut last = MediaState {
        current: Some(String::from("\u{0}")),
        ..Default::default()
    };

    let _ = tx.send(Msg::Refresh);
    loop {
        let mut controls = Vec::new();
        match rx.recv_timeout(Duration::from_secs(20)) {
            Ok(Msg::Refresh) => {}
            Ok(control @ Msg::Control { .. }) => controls.push(control),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return Ok(()),
        }
        // Coalesce bursts: players fire several events for one change.
        thread::sleep(Duration::from_millis(70));
        while let Ok(msg) = rx.try_recv() {
            if let Msg::Control { .. } = msg {
                controls.push(msg);
            }
        }
        for control in controls {
            if let Msg::Control {
                session,
                action,
                value,
            } = control
            {
                let _ = perform(&manager, session.as_deref(), &action, value);
            }
        }

        let state = read_state(&manager, &tx, &mut subscriptions, &mut art).unwrap_or_default();
        if state != last {
            let _ = app.emit("media://state", &state);
            last = state;
        }
    }
}

fn read_state(
    manager: &Manager,
    tx: &Sender<Msg>,
    subscriptions: &mut HashMap<String, Subscription>,
    art: &mut HashMap<String, (String, Option<String>)>,
) -> windows_core::Result<MediaState> {
    let sessions = manager.GetSessions()?;
    let mut seen = Vec::new();
    let mut out = Vec::new();
    for session in sessions {
        let id = session
            .SourceAppUserModelId()
            .map(|s| s.to_string())
            .unwrap_or_default();
        if id.is_empty() {
            continue;
        }
        seen.push(id.clone());
        if !subscriptions.contains_key(&id) {
            if let Ok(sub) = subscribe(&session, tx) {
                subscriptions.insert(id.clone(), sub);
            }
        }
        if let Ok(state) = read_session(&session, &id, art) {
            out.push(state);
        }
    }
    subscriptions.retain(|id, _| seen.contains(id));
    art.retain(|id, _| seen.contains(id));
    let current = manager
        .GetCurrentSession()
        .ok()
        .and_then(|s| s.SourceAppUserModelId().ok())
        .map(|s| s.to_string());
    Ok(MediaState {
        sessions: out,
        current,
    })
}

fn subscribe(session: &Session, tx: &Sender<Msg>) -> windows_core::Result<Subscription> {
    let a = tx.clone();
    let b = tx.clone();
    let c = tx.clone();
    let t0 = session.MediaPropertiesChanged(&TypedEventHandler::new(move |_, _| {
        let _ = a.send(Msg::Refresh);
        Ok(())
    }))?;
    let t1 = session.PlaybackInfoChanged(&TypedEventHandler::new(move |_, _| {
        let _ = b.send(Msg::Refresh);
        Ok(())
    }))?;
    let t2 = session.TimelinePropertiesChanged(&TypedEventHandler::new(move |_, _| {
        let _ = c.send(Msg::Refresh);
        Ok(())
    }))?;
    Ok(Subscription {
        session: session.clone(),
        tokens: [t0, t1, t2],
    })
}

const TICKS_PER_SECOND: f64 = 10_000_000.0;
/// 100-ns intervals between 1601-01-01 and 1970-01-01.
const EPOCH_DIFF: i64 = 116_444_736_000_000_000;

fn read_session(
    session: &Session,
    id: &str,
    art: &mut HashMap<String, (String, Option<String>)>,
) -> windows_core::Result<SessionState> {
    let props = session.TryGetMediaPropertiesAsync()?.join()?;
    let title = props.Title().map(|s| s.to_string()).unwrap_or_default();
    let mut artist = props.Artist().map(|s| s.to_string()).unwrap_or_default();
    if artist.is_empty() {
        artist = props
            .AlbumArtist()
            .map(|s| s.to_string())
            .unwrap_or_default();
    }
    let album = props
        .AlbumTitle()
        .map(|s| s.to_string())
        .unwrap_or_default();

    let key = format!("{title}\u{1}{artist}\u{1}{album}");
    let cover = match art.get(id) {
        Some((cached_key, Some(url))) if *cached_key == key => Some(url.clone()),
        _ => {
            let url = props
                .Thumbnail()
                .ok()
                .and_then(|thumb| read_thumbnail(&thumb).ok());
            art.insert(id.to_string(), (key, url.clone()));
            url
        }
    };

    let playback = session.GetPlaybackInfo()?;
    let status = match playback.PlaybackStatus()? {
        Status::Playing => "playing",
        Status::Paused => "paused",
        Status::Stopped => "stopped",
        Status::Changing => "changing",
        Status::Opened => "opened",
        _ => "closed",
    };
    let controls = playback.Controls()?;
    let shuffle = playback.IsShuffleActive().ok().and_then(|v| v.Value().ok());
    let repeat = playback
        .AutoRepeatMode()
        .ok()
        .and_then(|v| v.Value().ok())
        .map(|mode| match mode {
            MediaPlaybackAutoRepeatMode::Track => "track",
            MediaPlaybackAutoRepeatMode::List => "list",
            _ => "none",
        });

    let timeline = session.GetTimelineProperties()?;
    let start = timeline.StartTime()?.Duration as f64;
    let end = timeline.EndTime()?.Duration as f64;
    let position = timeline.Position()?.Duration as f64;
    let updated = timeline.LastUpdatedTime()?.UniversalTime;
    let updated_at = if updated > EPOCH_DIFF {
        (updated - EPOCH_DIFF) / 10_000
    } else {
        now_ms()
    };

    Ok(SessionState {
        id: id.to_string(),
        app: friendly_name(id),
        title,
        artist,
        album,
        art: cover,
        status,
        position: ((position - start) / TICKS_PER_SECOND).max(0.0),
        duration: ((end - start) / TICKS_PER_SECOND).max(0.0),
        updated_at,
        can_play_pause: controls.IsPlayPauseToggleEnabled().unwrap_or(false),
        can_next: controls.IsNextEnabled().unwrap_or(false),
        can_prev: controls.IsPreviousEnabled().unwrap_or(false),
        can_seek: controls.IsPlaybackPositionEnabled().unwrap_or(false),
        shuffle,
        repeat,
    })
}

fn read_thumbnail(thumb: &IRandomAccessStreamReference) -> windows_core::Result<String> {
    let stream = thumb.OpenReadAsync()?.join()?;
    let size = stream.Size()?;
    if size == 0 || size > 4 * 1024 * 1024 {
        return Err(windows_core::Error::empty());
    }
    let content_type = stream
        .ContentType()
        .map(|s| s.to_string())
        .unwrap_or_default();
    let reader = DataReader::CreateDataReader(&stream.GetInputStreamAt(0)?)?;
    reader.LoadAsync(size as u32)?.join()?;
    let mut bytes = vec![0u8; size as usize];
    reader.ReadBytes(&mut bytes)?;
    let mime = if content_type.starts_with("image/") {
        content_type
    } else {
        sniff(&bytes).to_string()
    };
    Ok(format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&bytes)
    ))
}

fn sniff(bytes: &[u8]) -> &'static str {
    match bytes {
        [0x89, b'P', b'N', b'G', ..] => "image/png",
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'E', b'B', b'P', ..] => "image/webp",
        _ => "image/jpeg",
    }
}

fn friendly_name(id: &str) -> String {
    let lower = id.to_ascii_lowercase();
    let known = [
        ("spotify", "Spotify"),
        ("youtube-music", "Pear"),
        ("pear", "Pear"),
        ("zunemusic", "Media Player"),
        ("zunevideo", "Films & TV"),
        ("msedge", "Edge"),
        ("chrome", "Chrome"),
        ("firefox", "Firefox"),
        ("brave", "Brave"),
        ("opera", "Opera"),
        ("vivaldi", "Vivaldi"),
        ("applemusic", "Apple Music"),
        ("apple.music", "Apple Music"),
        ("tidal", "TIDAL"),
        ("deezer", "Deezer"),
        ("amazon", "Amazon Music"),
        ("foobar", "foobar2000"),
        ("musicbee", "MusicBee"),
        ("vlc", "VLC"),
        ("discord", "Discord"),
    ];
    for (needle, name) in known {
        if lower.contains(needle) {
            return name.to_string();
        }
    }
    let base = id.rsplit(['\\', '/', '!']).next().unwrap_or(id);
    let base = base.trim_end_matches(".exe").trim_end_matches(".EXE");
    let base = base.rsplit('.').next().unwrap_or(base);
    let mut chars = base.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => "Player".into(),
    }
}

fn find(manager: &Manager, id: Option<&str>) -> Option<Session> {
    let sessions = manager.GetSessions().ok()?;
    if let Some(id) = id {
        for session in sessions {
            if session
                .SourceAppUserModelId()
                .map(|s| s.to_string())
                .unwrap_or_default()
                == id
            {
                return Some(session);
            }
        }
        None
    } else {
        manager.GetCurrentSession().ok()
    }
}

fn perform(
    manager: &Manager,
    id: Option<&str>,
    action: &str,
    value: Option<f64>,
) -> windows_core::Result<()> {
    let Some(session) = find(manager, id) else {
        return Ok(());
    };
    match action {
        "toggle" => {
            session.TryTogglePlayPauseAsync()?.join()?;
        }
        "play" => {
            session.TryPlayAsync()?.join()?;
        }
        "pause" => {
            session.TryPauseAsync()?.join()?;
        }
        "next" => {
            session.TrySkipNextAsync()?.join()?;
        }
        "prev" => {
            session.TrySkipPreviousAsync()?.join()?;
        }
        "seek" => {
            if let Some(seconds) = value {
                let start = session.GetTimelineProperties()?.StartTime()?.Duration;
                session
                    .TryChangePlaybackPositionAsync(
                        start + (seconds.max(0.0) * TICKS_PER_SECOND) as i64,
                    )?
                    .join()?;
            }
        }
        "shuffle" => {
            let active = session
                .GetPlaybackInfo()?
                .IsShuffleActive()
                .ok()
                .and_then(|v| v.Value().ok())
                .unwrap_or(false);
            session.TryChangeShuffleActiveAsync(!active)?.join()?;
        }
        "repeat" => {
            let mode = session
                .GetPlaybackInfo()?
                .AutoRepeatMode()
                .ok()
                .and_then(|v| v.Value().ok())
                .unwrap_or(MediaPlaybackAutoRepeatMode::None);
            let next = match mode {
                MediaPlaybackAutoRepeatMode::None => MediaPlaybackAutoRepeatMode::List,
                MediaPlaybackAutoRepeatMode::List => MediaPlaybackAutoRepeatMode::Track,
                _ => MediaPlaybackAutoRepeatMode::None,
            };
            session.TryChangeAutoRepeatModeAsync(next)?.join()?;
        }
        _ => {}
    }
    Ok(())
}

#[tauri::command]
pub fn media_control(session: Option<String>, action: String, value: Option<f64>) {
    send(Msg::Control {
        session,
        action,
        value,
    });
}

#[tauri::command]
pub fn media_refresh() {
    send(Msg::Refresh);
}
