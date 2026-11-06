//! Clipboard history (text) for the Shelf page. Event driven through a message-only window
//! registered with AddClipboardFormatListener. History stays in memory only, and copies that
//! password managers mark as private are skipped.

use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicIsize, Ordering},
        mpsc::{self, Receiver, Sender},
        Mutex, OnceLock,
    },
    thread,
    time::Duration,
};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{GlobalFree, HANDLE, HGLOBAL, HWND, LPARAM, LRESULT, WPARAM},
        System::{
            DataExchange::{
                AddClipboardFormatListener, CloseClipboard, EmptyClipboard, GetClipboardData,
                IsClipboardFormatAvailable, OpenClipboard, RegisterClipboardFormatW,
                SetClipboardData,
            },
            LibraryLoader::GetModuleHandleW,
            Memory::{GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE},
            Ole::CF_UNICODETEXT,
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostMessageW,
            RegisterClassW, TranslateMessage, HWND_MESSAGE, MSG, WINDOW_EX_STYLE, WINDOW_STYLE,
            WM_APP, WM_CLIPBOARDUPDATE, WNDCLASSW,
        },
    },
};

use crate::{
    config::Config,
    util::{now_ms, wide},
};

const WM_SET_TEXT: u32 = WM_APP + 1;
const MAX_TEXT: usize = 64 * 1024;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    id: i64,
    text: String,
    at: i64,
}

struct Shared {
    app: AppHandle,
    history: Mutex<VecDeque<Clip>>,
    outgoing: Mutex<Receiver<String>>,
    /// Text we put on the clipboard ourselves; not recorded again.
    own: Mutex<Option<String>>,
}

static SHARED: OnceLock<Shared> = OnceLock::new();
static OUTGOING: OnceLock<Mutex<Sender<String>>> = OnceLock::new();
static WINDOW: AtomicIsize = AtomicIsize::new(0);

pub fn start(app: AppHandle) {
    let (tx, rx) = mpsc::channel();
    let _ = OUTGOING.set(Mutex::new(tx));
    let _ = SHARED.set(Shared {
        app,
        history: Mutex::new(VecDeque::new()),
        outgoing: Mutex::new(rx),
        own: Mutex::new(None),
    });
    thread::Builder::new()
        .name("clipboard".into())
        .spawn(|| unsafe { run() })
        .expect("clipboard thread");
}

unsafe fn run() {
    let class = wide("HolmClipboard");
    let instance = GetModuleHandleW(None).unwrap_or_default();
    let wc = WNDCLASSW {
        lpfnWndProc: Some(wndproc),
        hInstance: instance.into(),
        lpszClassName: PCWSTR(class.as_ptr()),
        ..Default::default()
    };
    RegisterClassW(&wc);
    let Ok(hwnd) = CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        PCWSTR(class.as_ptr()),
        PCWSTR::null(),
        WINDOW_STYLE::default(),
        0,
        0,
        0,
        0,
        Some(HWND_MESSAGE),
        None,
        Some(instance.into()),
        None,
    ) else {
        return;
    };
    WINDOW.store(hwnd.0 as isize, Ordering::Relaxed);
    let _ = AddClipboardFormatListener(hwnd);
    let mut msg = MSG::default();
    while GetMessageW(&mut msg, None, 0, 0).as_bool() {
        let _ = TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CLIPBOARDUPDATE => {
            on_update(hwnd);
            LRESULT(0)
        }
        WM_SET_TEXT => {
            if let Some(shared) = SHARED.get() {
                let pending: Vec<String> = shared
                    .outgoing
                    .lock()
                    .map(|rx| rx.try_iter().collect())
                    .unwrap_or_default();
                if let Some(text) = pending.into_iter().last() {
                    if set_text(hwnd, &text) {
                        *shared.own.lock().expect("own") = Some(text);
                    }
                }
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn format_id(name: &str) -> u32 {
    let name = wide(name);
    unsafe { RegisterClipboardFormatW(PCWSTR(name.as_ptr())) }
}

unsafe fn open(hwnd: HWND) -> bool {
    // Another program may hold the clipboard for a moment.
    for _ in 0..8 {
        if OpenClipboard(Some(hwnd)).is_ok() {
            return true;
        }
        thread::sleep(Duration::from_millis(15));
    }
    false
}

unsafe fn on_update(hwnd: HWND) {
    let Some(shared) = SHARED.get() else { return };
    if !shared.app.state::<Config>().bool("clipboardHistory", true) {
        return;
    }
    // Content marked private by password managers and similar tools.
    for private in [
        "ExcludeClipboardContentFromMonitorProcessing",
        "Clipboard Viewer Ignore",
    ] {
        if IsClipboardFormatAvailable(format_id(private)).is_ok() {
            return;
        }
    }
    if IsClipboardFormatAvailable(CF_UNICODETEXT.0 as u32).is_err() || !open(hwnd) {
        return;
    }
    let mut skip = false;
    let history_flag = format_id("CanIncludeInClipboardHistory");
    if let Ok(handle) = GetClipboardData(history_flag) {
        let ptr = GlobalLock(HGLOBAL(handle.0)) as *const u32;
        if !ptr.is_null() {
            skip = *ptr == 0;
            let _ = GlobalUnlock(HGLOBAL(handle.0));
        }
    }
    let text = if skip { None } else { read_text() };
    let _ = CloseClipboard();
    let Some(text) = text else { return };
    if text.trim().is_empty() || text.len() > MAX_TEXT {
        return;
    }
    if shared.own.lock().expect("own").take().as_deref() == Some(text.as_str()) {
        return;
    }
    let max = shared
        .app
        .state::<Config>()
        .u64("clipboardMax", 30)
        .clamp(5, 200) as usize;
    {
        let mut history = shared.history.lock().expect("history");
        history.retain(|c| c.text != text);
        history.push_front(Clip {
            id: now_ms(),
            text,
            at: now_ms(),
        });
        history.truncate(max);
    }
    emit(shared);
}

unsafe fn read_text() -> Option<String> {
    let handle = GetClipboardData(CF_UNICODETEXT.0 as u32).ok()?;
    let global = HGLOBAL(handle.0);
    let ptr = GlobalLock(global) as *const u16;
    if ptr.is_null() {
        return None;
    }
    let max_units = GlobalSize(global) / 2;
    let mut len = 0usize;
    while len < max_units && *ptr.add(len) != 0 {
        len += 1;
    }
    let text = String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len));
    let _ = GlobalUnlock(global);
    Some(text)
}

unsafe fn set_text(hwnd: HWND, text: &str) -> bool {
    if !open(hwnd) {
        return false;
    }
    let mut ok = false;
    if EmptyClipboard().is_ok() {
        let units = wide(text);
        if let Ok(global) = GlobalAlloc(GMEM_MOVEABLE, units.len() * 2) {
            let ptr = GlobalLock(global) as *mut u16;
            if !ptr.is_null() {
                std::ptr::copy_nonoverlapping(units.as_ptr(), ptr, units.len());
                let _ = GlobalUnlock(global);
                ok = SetClipboardData(CF_UNICODETEXT.0 as u32, Some(HANDLE(global.0))).is_ok();
            }
            if !ok {
                let _ = GlobalFree(Some(global));
            }
        }
    }
    let _ = CloseClipboard();
    ok
}

fn emit(shared: &Shared) {
    let list: Vec<Clip> = shared
        .history
        .lock()
        .map(|h| h.iter().cloned().collect())
        .unwrap_or_default();
    let _ = shared.app.emit("clipboard://changed", &list);
}

#[tauri::command]
pub fn clipboard_list() -> Vec<Clip> {
    SHARED
        .get()
        .map(|s| {
            s.history
                .lock()
                .map(|h| h.iter().cloned().collect())
                .unwrap_or_default()
        })
        .unwrap_or_default()
}

#[tauri::command]
pub fn clipboard_copy(text: String) {
    if let Some(tx) = OUTGOING.get() {
        if let Ok(tx) = tx.lock() {
            let _ = tx.send(text);
        }
    }
    let hwnd = HWND(WINDOW.load(Ordering::Relaxed) as *mut core::ffi::c_void);
    if !hwnd.0.is_null() {
        unsafe {
            let _ = PostMessageW(Some(hwnd), WM_SET_TEXT, WPARAM(0), LPARAM(0));
        }
    }
}

#[tauri::command]
pub fn clipboard_clear() {
    if let Some(shared) = SHARED.get() {
        shared.history.lock().expect("history").clear();
        emit(shared);
    }
}
