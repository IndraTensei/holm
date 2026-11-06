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
        atomic::{AtomicBool, AtomicIsize, Ordering},
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

use crate::{config::Config, util::Hwnd};

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
    geometry: Mutex<Geometry>,
    fullscreen_hidden: AtomicBool,
    user_hidden: AtomicBool,
    maximized: AtomicBool,
    previous_foreground: AtomicIsize,
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
            geometry: Mutex::new(Geometry::default()),
            fullscreen_hidden: AtomicBool::new(false),
            user_hidden: AtomicBool::new(false),
            maximized: AtomicBool::new(false),
            previous_foreground: AtomicIsize::new(0),
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
        !self.fullscreen_hidden.load(Ordering::Relaxed) && !self.user_hidden.load(Ordering::Relaxed)
    }

    fn apply_visibility(&self) {
        self.show(self.visible());
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
    // later full-screen hide/show cycles use ShowWindow without activation.
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
            let mut elapsed: u64 = 0;
            let mut near = false;
            loop {
                let step = if near || inside { 16 } else { 40 };
                thread::sleep(Duration::from_millis(step));
                elapsed += step;

                let geometry = *island.geometry.lock().expect("geometry");
                let rect = *island.rect.lock().expect("rect");
                let mut point = POINT::default();
                let have_point = unsafe { GetCursorPos(&mut point).is_ok() };

                if have_point && geometry.scale > 0.0 {
                    let s = geometry.scale;
                    let margin = 3.0;
                    let left = geometry.x as f64 + (rect[0] - margin) * s;
                    let top = geometry.y as f64 + (rect[1] - margin).max(0.0) * s;
                    let right = geometry.x as f64 + (rect[0] + rect[2] + margin) * s;
                    let bottom = geometry.y as f64 + (rect[1] + rect[3] + margin) * s;
                    let (px, py) = (point.x as f64, point.y as f64);
                    let now_inside =
                        island.visible() && px >= left && px <= right && py >= top && py <= bottom;
                    near = px >= left - 160.0 * s
                        && px <= right + 160.0 * s
                        && py <= bottom + 120.0 * s
                        && py >= geometry.monitor.top as f64 - 4.0;
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
                        let _ = island.window.set_ignore_cursor_events(!inside);
                        let _ = app.emit("island://hover", inside);
                    }
                }

                if elapsed % 1000 < step {
                    let config = app.state::<Config>();
                    if config.bool("hideInFullscreen", true) {
                        let full = island.fullscreen_app_running(&geometry);
                        if full != island.fullscreen_hidden.swap(full, Ordering::Relaxed) {
                            island.apply_visibility();
                            let _ = app.emit("island://fullscreen", full);
                        }
                    } else if island.fullscreen_hidden.swap(false, Ordering::Relaxed) {
                        island.apply_visibility();
                    }
                    let maximized = island.foreground_maximized(&geometry);
                    if maximized != island.maximized.swap(maximized, Ordering::Relaxed) {
                        let _ = app.emit("island://maximized", maximized);
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
            }
        })
        .expect("island poll thread");
}

pub fn set_user_hidden(app: &AppHandle, hidden: bool) {
    if let Some(island) = app.try_state::<Arc<Island>>() {
        island.user_hidden.store(hidden, Ordering::Relaxed);
        island.apply_visibility();
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

#[tauri::command]
pub fn island_set_rect(island: State<'_, Arc<Island>>, x: f64, y: f64, width: f64, height: f64) {
    if x.is_finite() && y.is_finite() && width.is_finite() && height.is_finite() {
        *island.rect.lock().expect("rect") = [x, y, width.max(1.0), height.max(1.0)];
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

#[tauri::command]
pub fn island_set_hidden(app: AppHandle, hidden: bool) {
    set_user_hidden(&app, hidden);
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

#[tauri::command]
pub fn open_settings(app: AppHandle) -> Result<(), String> {
    crate::open_settings_window(&app).map_err(|e| e.to_string())
}
