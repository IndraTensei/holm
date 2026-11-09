//! The island window on macOS and Linux: top centre of a monitor, kept on top, click-through
//! outside the island. Same commands and events as the Windows version in ../island.rs, built
//! on Tauri's own window calls instead of Win32.
//!
//! Not there yet on these systems: stepping aside for full-screen apps and games, and knowing
//! which maximized app is in front (Make way and Learn). Those read as "nothing in front".
//! Wayland doesn't let an app place its own window or read the pointer outside it, so there the
//! island sits wherever the compositor puts it and opens with the shortcut or a click.

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, State, WebviewWindow};

use crate::config::Config;

pub const WIN_W: f64 = 760.0;
pub const WIN_H: f64 = 560.0;

#[derive(Clone, Copy, Default, PartialEq)]
struct Geometry {
    x: i32,
    y: i32,
    scale: f64,
    monitor_top: i32,
}

pub struct Island {
    window: WebviewWindow,
    /// Island rectangle inside the window, CSS px: x, y, width, height.
    rect: Mutex<[f64; 4]>,
    geometry: Mutex<Geometry>,
    user_hidden: AtomicBool,
    summoned: AtomicBool,
    shown: AtomicBool,
}

#[derive(Serialize, Clone)]
pub struct MonitorInfo {
    index: usize,
    name: String,
    primary: bool,
    width: i32,
    height: i32,
    scale: f64,
}

impl Island {
    fn new(window: WebviewWindow) -> Self {
        Self {
            window,
            rect: Mutex::new([WIN_W / 2.0 - 100.0, 0.0, 200.0, 34.0]),
            geometry: Mutex::new(Geometry::default()),
            user_hidden: AtomicBool::new(false),
            summoned: AtomicBool::new(false),
            shown: AtomicBool::new(true),
        }
    }

    /// Puts the window at the top centre of the chosen monitor. Returns true when it moved.
    fn place(&self, monitor_index: usize) -> bool {
        let list = self.window.available_monitors().unwrap_or_default();
        let target = list
            .get(monitor_index)
            .cloned()
            .or_else(|| self.window.primary_monitor().ok().flatten())
            .or_else(|| list.first().cloned());
        let Some(target) = target else {
            return false;
        };
        let scale = target.scale_factor();
        let width = (WIN_W * scale).round() as i32;
        let height = (WIN_H * scale).round() as i32;
        let origin = target.position();
        let size = target.size();
        let next = Geometry {
            x: origin.x + (size.width as i32 - width) / 2,
            y: origin.y,
            scale,
            monitor_top: origin.y,
        };
        let mut current = self.geometry.lock().expect("geometry");
        if *current == next {
            return false;
        }
        *current = next;
        drop(current);
        let _ = self
            .window
            .set_size(PhysicalSize::new(width as u32, height as u32));
        let _ = self
            .window
            .set_position(PhysicalPosition::new(next.x, next.y));
        true
    }

    fn visible(&self) -> bool {
        self.summoned.load(Ordering::Relaxed) || !self.user_hidden.load(Ordering::Relaxed)
    }

    fn refresh(&self, app: &AppHandle) {
        let want = self.visible();
        if want != self.shown.swap(want, Ordering::Relaxed) {
            let _ = if want {
                self.window.show()
            } else {
                self.window.hide()
            };
            let _ = app.emit("island://shown", want);
        }
    }
}

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let window = app
        .get_webview_window("island")
        .ok_or_else(|| tauri::Error::WindowNotFound)?;
    let island = Arc::new(Island::new(window.clone()));
    let monitor = app.state::<Config>().u64("monitor", 0) as usize;
    island.place(monitor);
    let _ = window.set_ignore_cursor_events(true);
    // Over every desktop and full-screen space, not just the one it started on.
    let _ = window.set_visible_on_all_workspaces(true);
    let _ = window.show();
    let _ = window.set_always_on_top(true);
    app.manage(island.clone());
    spawn_poll(app.clone(), island);
    Ok(())
}

/// Compares the pointer with the island's rectangle, like the Windows poll: 25 times a second,
/// 60 near the island, and only changes reach the page.
fn spawn_poll(app: AppHandle, island: Arc<Island>) {
    thread::Builder::new()
        .name("island-poll".into())
        .spawn(move || {
            let mut inside = false;
            let mut near = false;
            let mut near_sent = false;
            let mut elapsed: u64 = 0;
            loop {
                let step = if near || inside { 16 } else { 40 };
                thread::sleep(Duration::from_millis(step));
                elapsed += step;

                let geometry = *island.geometry.lock().expect("geometry");
                let rect = *island.rect.lock().expect("rect");
                if let (Ok(point), true) = (app.cursor_position(), geometry.scale > 0.0) {
                    let s = geometry.scale;
                    let margin = 3.0;
                    let left = geometry.x as f64 + (rect[0] - margin) * s;
                    let top = if rect[1] <= 16.0 {
                        geometry.y as f64
                    } else {
                        geometry.y as f64 + (rect[1] - margin) * s
                    };
                    let right = geometry.x as f64 + (rect[0] + rect[2] + margin) * s;
                    let bottom = geometry.y as f64 + (rect[1] + rect[3] + margin) * s;
                    let now_inside = island.visible()
                        && point.x >= left
                        && point.x <= right
                        && point.y >= top
                        && point.y <= bottom;
                    near = point.x >= left - 160.0 * s
                        && point.x <= right + 160.0 * s
                        && point.y <= bottom + 120.0 * s
                        && point.y >= geometry.monitor_top as f64 - 4.0;
                    if near != near_sent {
                        near_sent = near;
                        let _ = app.emit("island://near", near);
                    }
                    if now_inside != inside {
                        inside = now_inside;
                        let _ = island.window.set_ignore_cursor_events(!inside);
                        let _ = app.emit("island://hover", inside);
                    }
                }
                if elapsed % 2000 < step {
                    let monitor = app.state::<Config>().u64("monitor", 0) as usize;
                    if island.place(monitor) {
                        let _ = app.emit("island://placed", ());
                    }
                }
                island.refresh(&app);
            }
        })
        .expect("island poll thread");
}

pub fn set_user_hidden(app: &AppHandle, hidden: bool) {
    if let Some(island) = app.try_state::<Arc<Island>>() {
        island.user_hidden.store(hidden, Ordering::Relaxed);
        island.refresh(app);
        let _ = app.emit("island://hidden", hidden);
    }
}

pub fn user_hidden(app: &AppHandle) -> bool {
    app.try_state::<Arc<Island>>()
        .map(|island| island.user_hidden.load(Ordering::Relaxed))
        .unwrap_or(false)
}

pub fn reposition(app: &AppHandle) {
    if let Some(island) = app.try_state::<Arc<Island>>() {
        let monitor = app.state::<Config>().u64("monitor", 0) as usize;
        island.place(monitor);
    }
}

/// The pointer in the island window's CSS pixels.
#[tauri::command]
pub fn island_cursor(app: AppHandle, island: State<'_, Arc<Island>>) -> Option<(f64, f64)> {
    let geometry = *island.geometry.lock().ok()?;
    let point = app.cursor_position().ok()?;
    (geometry.scale > 0.0).then(|| {
        (
            (point.x - geometry.x as f64) / geometry.scale,
            (point.y - geometry.y as f64) / geometry.scale,
        )
    })
}

#[tauri::command]
pub fn island_set_rect(island: State<'_, Arc<Island>>, x: f64, y: f64, width: f64, height: f64) {
    if x.is_finite() && y.is_finite() && width.is_finite() && height.is_finite() {
        *island.rect.lock().expect("rect") = [x, y, width.max(1.0), height.max(1.0)];
    }
}

/// The island never takes focus on these systems, so there is nothing to give back.
#[tauri::command]
pub fn island_release_focus() {}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IslandState {
    maximized: bool,
    app: String,
    hidden: bool,
    shown: bool,
}

#[tauri::command]
pub fn island_state(island: State<'_, Arc<Island>>) -> IslandState {
    IslandState {
        maximized: false,
        app: String::new(),
        hidden: island.user_hidden.load(Ordering::Relaxed),
        shown: island.shown.load(Ordering::Relaxed),
    }
}

#[tauri::command]
pub fn island_set_hidden(app: AppHandle, hidden: bool) {
    crate::set_hidden(&app, hidden);
}

#[tauri::command]
pub fn island_summon(app: AppHandle, on: bool) {
    let Some(island) = app.try_state::<Arc<Island>>() else {
        return;
    };
    if on && island.user_hidden.load(Ordering::Relaxed) {
        crate::set_hidden(&app, false);
    }
    island.summoned.store(on, Ordering::Relaxed);
    island.refresh(&app);
}

#[tauri::command]
pub fn monitors_list(app: AppHandle) -> Vec<MonitorInfo> {
    let primary = app
        .primary_monitor()
        .ok()
        .flatten()
        .map(|m| *m.position());
    app.available_monitors()
        .unwrap_or_default()
        .into_iter()
        .enumerate()
        .map(|(index, m)| MonitorInfo {
            index,
            name: m
                .name()
                .cloned()
                .unwrap_or_else(|| format!("Display {}", index + 1)),
            primary: Some(*m.position()) == primary,
            width: m.size().width as i32,
            height: m.size().height as i32,
            scale: m.scale_factor(),
        })
        .collect()
}

#[tauri::command]
pub async fn open_settings(app: AppHandle) -> Result<(), String> {
    crate::open_settings_window(&app).map_err(|e| e.to_string())
}
