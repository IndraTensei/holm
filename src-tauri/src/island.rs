//! The island window: placement at the top of a monitor, click-through outside the island's
//! shape, hover detection, hiding during full-screen apps and keeping the window on top.
//!
//! The window is a fixed transparent canvas (WIN_W × WIN_H logical px). The frontend reports
//! where the island currently is inside it (`island_set_rect`); a light polling thread compares
//! the cursor with that rectangle and turns click-through off only while the cursor is over the
//! island. Polling runs at 60 Hz near the island and 25 Hz elsewhere: cheaper and more reliable
//! than a low-level mouse hook, which would sit in every input event of the system.

use std::{
    sync::{
        atomic::{AtomicBool, AtomicI64, AtomicIsize, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, State, WebviewWindow};
use windows::Win32::{
    Foundation::{HWND, LPARAM, POINT, RECT},
    Graphics::Gdi::{
        EnumDisplayMonitors, GetMonitorInfoW, MonitorFromWindow, HDC, HMONITOR, MONITORINFO,
        MONITORINFOEXW, MONITOR_DEFAULTTONEAREST,
    },
    UI::{
        HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI},
        Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON},
        Shell::{
            SHQueryUserNotificationState, QUNS_PRESENTATION_MODE, QUNS_RUNNING_D3D_FULL_SCREEN,
        },
        WindowsAndMessaging::{
            GetClassNameW, GetCursorPos, GetForegroundWindow, GetWindowRect, IsIconic, IsWindow,
            IsZoomed, SetForegroundWindow, SetWindowPos, ShowWindow, HWND_TOPMOST, SWP_NOACTIVATE,
            SWP_NOMOVE, SWP_NOSIZE, SW_HIDE, SW_SHOWNOACTIVATE,
        },
    },
};
use windows_core::BOOL;

use crate::{
    config::Config,
    games,
    near::{Near, Zone},
    util,
    util::Hwnd,
};

/// How long the island stays up over a game for its intro before stepping aside.
const INTRO_MS: i64 = 4600;

/// For this long after start, focus that lands on the island without being asked for (no hover,
/// no shortcut) goes back to the window that had it: starting at sign-in must not steal focus.
const STARTUP_GUARD_MS: u64 = 5000;

pub const WIN_W: f64 = 760.0;
pub const WIN_H: f64 = 560.0;

#[derive(Clone, Copy, Default)]
struct Geometry {
    x: i32,
    y: i32,
    scale: f64,
    monitor: RECT,
    hmonitor: isize,
}

pub struct Island {
    window: WebviewWindow,
    hwnd: Hwnd,
    /// Island rectangle inside the window, CSS px: x, y, width, height.
    rect: Mutex<[f64; 4]>,
    /// The last rectangle that wasn't the top-edge strip of a line: Make way's near zone is
    /// measured from this, so it doesn't shrink when the island makes way (see near.rs).
    full_rect: Mutex<[f64; 4]>,
    geometry: Mutex<Geometry>,
    fullscreen_hidden: AtomicBool,
    user_hidden: AtomicBool,
    maximized: AtomicBool,
    /// Program of the maximized window in front ("" when none), for per-app behaviour.
    front_app: Mutex<String>,
    previous_foreground: AtomicIsize,
    /// A game is in front and smart hide is on.
    game_hidden: AtomicBool,
    /// Brought back with the shortcut: shown over full screen and games until closed.
    summoned: AtomicBool,
    /// Unix ms until which a game intro keeps the island up.
    intro_until: AtomicI64,
    /// What the window last was, so ShowWindow runs only on a change.
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

struct Monitor {
    handle: HMONITOR,
    rect: RECT,
    primary: bool,
    name: String,
}

fn monitors() -> Vec<Monitor> {
    unsafe extern "system" fn collect(
        handle: HMONITOR,
        _: HDC,
        _: *mut RECT,
        data: LPARAM,
    ) -> BOOL {
        let list = &mut *(data.0 as *mut Vec<Monitor>);
        let mut info = MONITORINFOEXW::default();
        info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
        if GetMonitorInfoW(handle, &mut info as *mut MONITORINFOEXW as *mut MONITORINFO).as_bool() {
            let name_len = info
                .szDevice
                .iter()
                .position(|c| *c == 0)
                .unwrap_or(info.szDevice.len());
            list.push(Monitor {
                handle,
                rect: info.monitorInfo.rcMonitor,
                primary: info.monitorInfo.dwFlags & 1 != 0,
                name: String::from_utf16_lossy(&info.szDevice[..name_len]),
            });
        }
        BOOL(1)
    }
    let mut list: Vec<Monitor> = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(collect),
            LPARAM(&mut list as *mut _ as isize),
        );
    }
    // Primary first, then left to right: a stable order for the settings list.
    list.sort_by_key(|m| (!m.primary, m.rect.left, m.rect.top));
    list
}

fn monitor_scale(handle: HMONITOR) -> f64 {
    let (mut dpi_x, mut dpi_y) = (96u32, 96u32);
    unsafe {
        let _ = GetDpiForMonitor(handle, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);
    }
    (dpi_x.max(48) as f64) / 96.0
}

impl Island {
    pub fn new(window: WebviewWindow) -> tauri::Result<Self> {
        let hwnd = Hwnd::from(window.hwnd()?);
        Ok(Self {
            window,
            hwnd,
            rect: Mutex::new([WIN_W / 2.0 - 100.0, 0.0, 200.0, 34.0]),
            full_rect: Mutex::new([WIN_W / 2.0 - 100.0, 0.0, 200.0, 34.0]),
            geometry: Mutex::new(Geometry::default()),
            fullscreen_hidden: AtomicBool::new(false),
            user_hidden: AtomicBool::new(false),
            maximized: AtomicBool::new(false),
            front_app: Mutex::new(String::new()),
            previous_foreground: AtomicIsize::new(0),
            game_hidden: AtomicBool::new(false),
            summoned: AtomicBool::new(false),
            intro_until: AtomicI64::new(0),
            shown: AtomicBool::new(true),
        })
    }

    /// Puts the window at the top centre of the chosen monitor, sized for that monitor's scale.
    /// Returns true when the placement changed.
    fn place(&self, monitor_index: usize) -> bool {
        let list = monitors();
        let Some(target) = list.get(monitor_index).or_else(|| list.first()) else {
            return false;
        };
        let scale = monitor_scale(target.handle);
        let width = (WIN_W * scale).round() as i32;
        let height = (WIN_H * scale).round() as i32;
        let x = target.rect.left + ((target.rect.right - target.rect.left) - width) / 2;
        let y = target.rect.top;
        let next = Geometry {
            x,
            y,
            scale,
            monitor: target.rect,
            hmonitor: target.handle.0 as isize,
        };
        let mut current = self.geometry.lock().expect("geometry");
        let changed = current.x != next.x
            || current.y != next.y
            || (current.scale - next.scale).abs() > f64::EPSILON
            || current.hmonitor != next.hmonitor;
        if changed {
            *current = next;
            drop(current);
            let _ = self
                .window
                .set_size(PhysicalSize::new(width as u32, height as u32));
            let _ = self.window.set_position(PhysicalPosition::new(x, y));
        }
        changed
    }

    fn show(&self, visible: bool) {
        unsafe {
            let _ = ShowWindow(
                self.hwnd.get(),
                if visible { SW_SHOWNOACTIVATE } else { SW_HIDE },
            );
            if visible {
                self.keep_on_top();
            }
        }
    }

    fn keep_on_top(&self) {
        unsafe {
            let _ = SetWindowPos(
                self.hwnd.get(),
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
    }

    fn visible(&self) -> bool {
        if self.summoned.load(Ordering::Relaxed) {
            return true;
        }
        if self.user_hidden.load(Ordering::Relaxed) {
            return false;
        }
        let stepping_aside = self.fullscreen_hidden.load(Ordering::Relaxed)
            || self.game_hidden.load(Ordering::Relaxed);
        !stepping_aside || util::now_ms() < self.intro_until.load(Ordering::Relaxed)
    }

    /// Shows or hides the window when what it should be differs from what it is, and tells the
    /// frontend (which stops its loops while hidden).
    fn refresh(&self, app: &AppHandle) {
        let want = self.visible();
        if want != self.shown.swap(want, Ordering::Relaxed) {
            self.show(want);
            let _ = app.emit("island://shown", want);
        }
    }

    /// A window filling our monitor (borderless full screen, a game, a presentation).
    fn fullscreen_app_running(&self, geometry: &Geometry) -> bool {
        unsafe {
            if let Ok(state) = SHQueryUserNotificationState() {
                if state == QUNS_RUNNING_D3D_FULL_SCREEN || state == QUNS_PRESENTATION_MODE {
                    return true;
                }
            }
            let foreground = GetForegroundWindow();
            if foreground.0.is_null() || foreground == self.hwnd.get() || shell_window(foreground) {
                return false;
            }
            if MonitorFromWindow(foreground, MONITOR_DEFAULTTONEAREST).0 as isize
                != geometry.hmonitor
            {
                return false;
            }
            let mut rect = RECT::default();
            if GetWindowRect(foreground, &mut rect).is_err() {
                return false;
            }
            let m = geometry.monitor;
            rect.left <= m.left
                && rect.top <= m.top
                && rect.right >= m.right
                && rect.bottom >= m.bottom
        }
    }

    fn foreground_maximized(&self, geometry: &Geometry) -> bool {
        unsafe {
            let foreground = GetForegroundWindow();
            if foreground.0.is_null() || foreground == self.hwnd.get() || shell_window(foreground) {
                return false;
            }
            IsZoomed(foreground).as_bool()
                && !IsIconic(foreground).as_bool()
                && MonitorFromWindow(foreground, MONITOR_DEFAULTTONEAREST).0 as isize
                    == geometry.hmonitor
        }
    }
}

fn shell_window(hwnd: HWND) -> bool {
    let mut buffer = [0u16; 64];
    let len = unsafe { GetClassNameW(hwnd, &mut buffer) } as usize;
    let class = String::from_utf16_lossy(&buffer[..len.min(buffer.len())]);
    matches!(
        class.as_str(),
        "Progman"
            | "WorkerW"
            | "Shell_TrayWnd"
            | "Shell_SecondaryTrayWnd"
            | "XamlExplorerHostIslandWindow"
            | "Windows.UI.Core.CoreWindow"
    )
}

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let window = app
        .get_webview_window("island")
        .ok_or_else(|| tauri::Error::WindowNotFound)?;
    let island = Arc::new(Island::new(window.clone())?);
    let monitor = app.state::<Config>().u64("monitor", 0) as usize;
    island.place(monitor);
    let _ = window.set_ignore_cursor_events(true);
    // The first reveal goes through Tauri so the WebView2 controller is marked visible too;
    // later full-screen hide/show cycles use ShowWindow without activation. That first show (and
    // WebView2 finishing its load) can take focus: the poll gives it back (see STARTUP_GUARD_MS).
    let before = unsafe { GetForegroundWindow() };
    if before != island.hwnd.get() {
        island
            .previous_foreground
            .store(before.0 as isize, Ordering::Relaxed);
    }
    let _ = window.show();
    island.keep_on_top();
    app.manage(island.clone());
    spawn_poll(app.clone(), island);
    Ok(())
}

fn spawn_poll(app: AppHandle, island: Arc<Island>) {
    thread::Builder::new()
        .name("island-poll".into())
        .spawn(move || {
            let mut inside = false;
            // What the frontend was last told; lags `inside` while a drag passes over.
            let mut hover_sent = false;
            let mut elapsed: u64 = 0;
            // "Make way" drops the island to a line over a maximized app while the pointer is
            // close (see stores/fit); near.rs decides, without flicker. Sent on change only.
            let mut near = Near::default();
            let mut tracker = games::Tracker::default();
            let mut last_game: Option<games::Game> = None;
            let mut last_foreground: isize = 0;
            loop {
                let step = if near.on() || inside { 16 } else { 40 };
                thread::sleep(Duration::from_millis(step));
                elapsed += step;

                let geometry = *island.geometry.lock().expect("geometry");
                let rect = *island.rect.lock().expect("rect");
                let full = *island.full_rect.lock().expect("full rect");
                let mut point = POINT::default();
                let have_point = unsafe { GetCursorPos(&mut point).is_ok() };

                if have_point && geometry.scale > 0.0 {
                    let s = geometry.scale;
                    let margin = 3.0;
                    let left = geometry.x as f64 + (rect[0] - margin) * s;
                    // The island hangs from the top of the screen: the gap above it counts too, so
                    // a pointer pushed to the edge (how the line opens over a maximized app) stays in.
                    let top = if rect[1] <= 16.0 {
                        geometry.y as f64
                    } else {
                        geometry.y as f64 + (rect[1] - margin) * s
                    };
                    let right = geometry.x as f64 + (rect[0] + rect[2] + margin) * s;
                    let bottom = geometry.y as f64 + (rect[1] + rect[3] + margin) * s;
                    let (px, py) = (point.x as f64, point.y as f64);
                    let now_inside =
                        island.visible() && px >= left && px <= right && py >= top && py <= bottom;
                    let zone = Zone {
                        left: geometry.x as f64 + (full[0] - margin) * s,
                        right: geometry.x as f64 + (full[0] + full[2] + margin) * s,
                        bottom: geometry.y as f64 + (full[1] + full[3] + margin) * s,
                        top: geometry.monitor.top as f64,
                        scale: s,
                    };
                    if let Some(on) = near.update(px, py, zone, elapsed) {
                        let _ = app.emit("island://near", on);
                    }
                    if now_inside != inside {
                        inside = now_inside;
                        if inside {
                            let foreground = unsafe { GetForegroundWindow() };
                            if foreground != island.hwnd.get() {
                                island
                                    .previous_foreground
                                    .store(foreground.0 as isize, Ordering::Relaxed);
                            }
                        }
                        // Click-through follows the cursor at once, so a dragged file can land.
                        let _ = island.window.set_ignore_cursor_events(!inside);
                    }
                    // Dragging a window or a tab across the top of the screen should not open
                    // the island: arriving with the button down counts only once it is released.
                    // Once open, pressing inside (a click) never closes it.
                    let button = unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) } < 0;
                    let want = if hover_sent {
                        inside
                    } else {
                        inside && !button
                    };
                    if want != hover_sent {
                        hover_sent = want;
                        let _ = app.emit("island://hover", want);
                    }
                }

                // Games: checked the moment the front window changes (GetForegroundWindow is
                // nearly free), and once a second to notice games that exit. Before the full
                // screen check, so an intro starts before full screen could hide the island.
                let second = elapsed % 1000 < step;
                let foreground = unsafe { GetForegroundWindow() };
                if second || foreground.0 as isize != last_foreground {
                    last_foreground = foreground.0 as isize;
                    let config = app.state::<Config>();
                    let (game, changes) =
                        tracker.tick(foreground, island.hwnd.get(), geometry.monitor, || {
                            custom_games(&config)
                        });
                    for change in changes {
                        match change {
                            games::Change::Launched(g) => {
                                if config.bool("gameIntro", true)
                                    && !island.user_hidden.load(Ordering::Relaxed)
                                {
                                    island
                                        .intro_until
                                        .store(util::now_ms() + INTRO_MS, Ordering::Relaxed);
                                    let _ = app.emit("game://launch", &g);
                                    // The logo follows on its own thread, so the intro isn't held.
                                    let (app, game) = (app.clone(), g.clone());
                                    let _ = thread::Builder::new().name("game-icon".into()).spawn(
                                        move || {
                                            if let Some(icon) = games::logo(&game) {
                                                let _ = app.emit(
                                                    "game://icon",
                                                    games::GameIcon { exe: game.exe, icon },
                                                );
                                            }
                                        },
                                    );
                                }
                            }
                            games::Change::Ended(e) => {
                                let _ = app.emit("game://ended", &e);
                            }
                        }
                    }
                    let hide = config.bool("smartHide", true) && game.is_some();
                    island.game_hidden.store(hide, Ordering::Relaxed);
                    if game != last_game {
                        let _ = app.emit("game://active", &game);
                        last_game = game;
                    }
                }

                if second {
                    let config = app.state::<Config>();
                    if config.bool("hideInFullscreen", true) {
                        let full = island.fullscreen_app_running(&geometry);
                        if full != island.fullscreen_hidden.swap(full, Ordering::Relaxed) {
                            let _ = app.emit("island://fullscreen", full);
                        }
                    } else {
                        island.fullscreen_hidden.store(false, Ordering::Relaxed);
                    }
                    let maximized = island.foreground_maximized(&geometry);
                    let name = if maximized {
                        games::app_name(foreground)
                    } else {
                        String::new()
                    };
                    let was = island.maximized.swap(maximized, Ordering::Relaxed);
                    let mut front = island.front_app.lock().expect("front lock");
                    if maximized != was || name != *front {
                        *front = name.clone();
                        let _ = app.emit(
                            "island://maximized",
                            Front {
                                maximized,
                                app: name,
                            },
                        );
                    }
                }
                if elapsed % 2000 < step {
                    let monitor = app.state::<Config>().u64("monitor", 0) as usize;
                    if island.place(monitor) {
                        let _ = app.emit("island://placed", ());
                    }
                    if island.visible() {
                        island.keep_on_top();
                    }
                }
                if elapsed < STARTUP_GUARD_MS
                    && foreground == island.hwnd.get()
                    && !inside
                    && !island.summoned.load(Ordering::Relaxed)
                {
                    let previous = HWND(island.previous_foreground.load(Ordering::Relaxed)
                        as *mut core::ffi::c_void);
                    if !previous.0.is_null() && unsafe { IsWindow(Some(previous)) }.as_bool() {
                        let _ = unsafe { SetForegroundWindow(previous) };
                    }
                }

                // Cheap (a few atomics) and catches the end of a game intro on time.
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

/// The pointer in the island window's CSS pixels. The page cannot always know: a flick to the
/// screen edge can stop before the window stops being click-through, so no move reaches it.
#[tauri::command]
pub fn island_cursor(island: State<'_, Arc<Island>>) -> Option<(f64, f64)> {
    let geometry = *island.geometry.lock().ok()?;
    let mut point = POINT::default();
    unsafe { GetCursorPos(&mut point).ok()? };
    (geometry.scale > 0.0).then(|| {
        (
            (point.x - geometry.x) as f64 / geometry.scale,
            (point.y - geometry.y) as f64 / geometry.scale,
        )
    })
}

#[tauri::command]
pub fn island_set_rect(island: State<'_, Arc<Island>>, x: f64, y: f64, width: f64, height: f64) {
    if x.is_finite() && y.is_finite() && width.is_finite() && height.is_finite() {
        let rect = [x, y, width.max(1.0), height.max(1.0)];
        *island.rect.lock().expect("rect") = rect;
        // The line's hover strip is 1 px tall; anything taller is the island itself.
        if height > 2.0 {
            *island.full_rect.lock().expect("full rect") = rect;
        }
    }
}

/// Gives keyboard focus back to the app that had it before the island was used.
#[tauri::command]
pub fn island_release_focus(island: State<'_, Arc<Island>>) {
    unsafe {
        if GetForegroundWindow() != island.hwnd.get() {
            return;
        }
        let previous =
            HWND(island.previous_foreground.load(Ordering::Relaxed) as *mut core::ffi::c_void);
        if !previous.0.is_null() && IsWindow(Some(previous)).as_bool() {
            let _ = SetForegroundWindow(previous);
        }
    }
}

/// A maximized window in front of the island's monitor, and its program.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Front {
    maximized: bool,
    app: String,
}

/// What the window state is right now. Events only report changes, and the frontend can start
/// after the first ones went out (slow first load), so it asks once at start.
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
        maximized: island.maximized.load(Ordering::Relaxed),
        app: island
            .front_app
            .lock()
            .map(|a| a.clone())
            .unwrap_or_default(),
        hidden: island.user_hidden.load(Ordering::Relaxed),
        shown: island.shown.load(Ordering::Relaxed),
    }
}

#[tauri::command]
pub fn island_set_hidden(app: AppHandle, hidden: bool) {
    crate::set_hidden(&app, hidden);
}

/// The open shortcut: bring the island back over anything (a game, full screen, hidden) until
/// it closes again (`on = false`).
#[tauri::command]
pub fn island_summon(app: AppHandle, on: bool) {
    let Some(island) = app.try_state::<Arc<Island>>() else {
        return;
    };
    if on {
        let foreground = unsafe { GetForegroundWindow() };
        if foreground != island.hwnd.get() {
            island
                .previous_foreground
                .store(foreground.0 as isize, Ordering::Relaxed);
        }
        if island.user_hidden.load(Ordering::Relaxed) {
            crate::set_hidden(&app, false);
        }
    }
    island.summoned.store(on, Ordering::Relaxed);
    island.refresh(&app);
}

/// Games the user added in settings: `customGames: [{ exe, name }]`.
fn custom_games(config: &Config) -> Vec<(String, String)> {
    config
        .snapshot()
        .get("customGames")
        .and_then(serde_json::Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(|g| {
                    let exe = g.get("exe")?.as_str()?.to_string();
                    let name = g.get("name").and_then(|n| n.as_str()).unwrap_or("");
                    Some((exe, name.to_string()))
                })
                .collect()
        })
        .unwrap_or_default()
}

#[tauri::command]
pub fn monitors_list() -> Vec<MonitorInfo> {
    monitors()
        .into_iter()
        .enumerate()
        .map(|(index, m)| MonitorInfo {
            index,
            name: m.name,
            primary: m.primary,
            width: m.rect.right - m.rect.left,
            height: m.rect.bottom - m.rect.top,
            scale: monitor_scale(m.handle),
        })
        .collect()
}

/// Async on purpose: creating a WebView2 window from a synchronous command (run on the main
/// thread) can stall the new webview's setup on Windows, leaving a blank white window.
#[tauri::command]
pub async fn open_settings(app: AppHandle, section: Option<String>) -> Result<(), String> {
    crate::open_settings_at(&app, section.as_deref()).map_err(|e| e.to_string())
}
