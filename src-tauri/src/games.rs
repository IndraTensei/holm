//! Game mode: notices when a game comes to the front so the island can play a short intro
//! ("Opening Valorant"), then step aside until the player alt-tabs out, and say how long they
//! played when the game closes.
//!
//! No extra thread and no process scan: the island's once-a-second check hands us the foreground
//! window, and an exe path is resolved only when that window changes. A game is a known exe, an
//! exe the user added, or a big window from a store's games folder (Steam, Epic, Riot, …), named
//! after its install folder. Handles are opened with PROCESS_QUERY_LIMITED_INFORMATION only, the
//! same right Task Manager uses, and closed straight away.

use std::collections::{hash_map::Entry, HashMap};

use serde::Serialize;
use windows::{
    core::PWSTR,
    Win32::{
        Foundation::{CloseHandle, HWND, RECT},
        System::Threading::{
            GetExitCodeProcess, OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
            PROCESS_QUERY_LIMITED_INFORMATION,
        },
        UI::WindowsAndMessaging::{
            GetWindowRect, GetWindowTextW, GetWindowThreadProcessId, IsWindow,
        },
    },
};

use crate::{installed, png, util};

/// The game's logo as a PNG data URL. First the icon its launcher registered with Windows for it
/// (Settings → Apps: Riot's and Steam's are the official 256 px logos, and found by name even
/// when anti-cheat hides the game's exe), then the exe's own icon. Takes a few tens of ms, so
/// it runs on its own thread after the intro has started.
pub fn logo(game: &Game) -> Option<String> {
    installed::logo_file(&game.name, &game.path)
        .and_then(|file| icon(&file))
        .or_else(|| (!game.path.is_empty()).then(|| icon(&game.path)).flatten())
}

/// The icon of a file (exe or .ico) through the shell, the same image Explorer shows, up to
/// 128 px. None when the shell has no icon for it.
pub fn icon(path: &str) -> Option<String> {
    use base64::Engine;
    use windows::{
        core::HSTRING,
        Win32::{
            Foundation::SIZE,
            Graphics::Gdi::{
                CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, BITMAP,
                BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HGDIOBJ,
            },
            System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED},
            UI::Shell::{
                IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF_BIGGERSIZEOK,
                SIIGBF_ICONONLY,
            },
        },
    };
    const SIDE: i32 = 128;
    unsafe {
        let com = CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok();
        let pixels = (|| {
            let factory: IShellItemImageFactory =
                SHCreateItemFromParsingName(&HSTRING::from(path), None).ok()?;
            let bitmap = factory
                .GetImage(
                    SIZE { cx: SIDE, cy: SIDE },
                    SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK,
                )
                .ok()?;
            let object = HGDIOBJ(bitmap.0);
            let mut info = BITMAP::default();
            let read = GetObjectW(
                object,
                std::mem::size_of::<BITMAP>() as i32,
                Some(&mut info as *mut BITMAP as *mut core::ffi::c_void),
            );
            let (width, height) = (info.bmWidth, info.bmHeight.abs());
            let mut header = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -height, // top-down rows
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut bgra = vec![0u8; (width.max(0) * height * 4) as usize];
            let dc = CreateCompatibleDC(None);
            let lines = if read > 0 && width > 0 && height > 0 && width <= 512 && height <= 512 {
                GetDIBits(
                    dc,
                    windows::Win32::Graphics::Gdi::HBITMAP(bitmap.0),
                    0,
                    height as u32,
                    Some(bgra.as_mut_ptr() as *mut core::ffi::c_void),
                    &mut header,
                    DIB_RGB_COLORS,
                )
            } else {
                0
            };
            let _ = DeleteDC(dc);
            let _ = DeleteObject(object);
            (lines > 0).then_some((width as u32, height as u32, bgra))
        })();
        if com {
            CoUninitialize();
        }
        let (width, height, mut px) = pixels?;
        // BGRA → RGBA. The shell hands back premultiplied alpha; PNG wants it straight.
        let premultiplied = px.chunks_exact(4).all(|p| p[0] <= p[3] && p[1] <= p[3] && p[2] <= p[3]);
        for p in px.chunks_exact_mut(4) {
            p.swap(0, 2);
            let alpha = p[3] as u32;
            if premultiplied && alpha > 0 && alpha < 255 {
                for c in &mut p[..3] {
                    *c = ((*c as u32 * 255 + alpha / 2) / alpha).min(255) as u8;
                }
            }
        }
        // An icon with no visible pixel at all isn't worth showing.
        if px.chunks_exact(4).all(|p| p[3] == 0) {
            return None;
        }
        let png = png::encode_rgba(width, height, &px);
        Some(format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(png)
        ))
    }
}

/// Well-known games by exe name (lower case) → the name to show.
const KNOWN: &[(&str, &str)] = &[
    ("gta5.exe", "Grand Theft Auto V"),
    ("gta5_enhanced.exe", "Grand Theft Auto V"),
    ("gtaiv.exe", "Grand Theft Auto IV"),
    ("rdr2.exe", "Red Dead Redemption 2"),
    ("valorant-win64-shipping.exe", "Valorant"),
    ("league of legends.exe", "League of Legends"),
    ("cs2.exe", "Counter-Strike 2"),
    ("dota2.exe", "Dota 2"),
    ("fortniteclient-win64-shipping.exe", "Fortnite"),
    ("r5apex.exe", "Apex Legends"),
    ("r5apex_dx12.exe", "Apex Legends"),
    ("overwatch.exe", "Overwatch 2"),
    ("cod.exe", "Call of Duty"),
    ("tslgame.exe", "PUBG: Battlegrounds"),
    ("rainbowsix.exe", "Rainbow Six Siege"),
    ("rainbowsix_vulkan.exe", "Rainbow Six Siege"),
    ("rocketleague.exe", "Rocket League"),
    ("destiny2.exe", "Destiny 2"),
    ("marvel-win64-shipping.exe", "Marvel Rivals"),
    ("robloxplayerbeta.exe", "Roblox"),
    ("minecraft.windows.exe", "Minecraft"),
    ("eldenring.exe", "Elden Ring"),
    ("cyberpunk2077.exe", "Cyberpunk 2077"),
    ("witcher3.exe", "The Witcher 3"),
    ("bg3.exe", "Baldur's Gate 3"),
    ("bg3_dx11.exe", "Baldur's Gate 3"),
    ("helldivers2.exe", "Helldivers 2"),
    ("forzahorizon5.exe", "Forza Horizon 5"),
    ("genshinimpact.exe", "Genshin Impact"),
    ("starrail.exe", "Honkai: Star Rail"),
    ("fc25.exe", "EA Sports FC 25"),
    ("fc26.exe", "EA Sports FC 26"),
];

/// Store libraries: the folder after the marker is the game's name.
const LIBRARIES: &[&str] = &[
    "\\steamapps\\common\\",
    "\\epic games\\",
    "\\riot games\\",
    "\\rockstar games\\",
    "\\ea games\\",
    "\\ubisoft game launcher\\games\\",
    "\\xboxgames\\",
    "\\gog galaxy\\games\\",
    "\\battle.net\\",
];

/// Launchers and helpers that live in those folders but are not games.
const NOT_GAMES: &[&str] = &[
    "launcher",
    "riot client",
    "social club",
    "epic online services",
    "steamworks shared",
    "directx",
    "crash",
    "setup",
    "unins",
    "redist",
    "anticheat",
    "battleye",
    "updater",
];

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub name: String,
    pub exe: String,
    /// Full path of the exe, for its icon; empty when only the window title was readable.
    #[serde(skip)]
    pub path: String,
}

/// The game's icon, sent after `game://launch` once it has been read (`game://icon`).
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameIcon {
    pub exe: String,
    /// A PNG data URL.
    pub icon: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Ended {
    pub name: String,
    pub exe: String,
    pub duration_ms: i64,
}

pub enum Change {
    Launched(Game),
    Ended(Ended),
}

struct Session {
    game: Game,
    started: i64,
    /// The game's window, to notice the exit when the process itself cannot be opened.
    window: isize,
}

#[derive(Default)]
pub struct Tracker {
    last_foreground: isize,
    /// The game owning the current foreground window, if any.
    current: Option<Game>,
    /// Games seen this run, by process id, until their process exits.
    sessions: HashMap<u32, Session>,
}

impl Tracker {
    /// Called about once a second. Returns the game in front (if any) and what changed.
    /// `custom` is read only when the foreground window changes.
    pub fn tick(
        &mut self,
        foreground: HWND,
        island: HWND,
        monitor: RECT,
        custom: impl FnOnce() -> Vec<(String, String)>,
    ) -> (Option<Game>, Vec<Change>) {
        let mut changes = Vec::new();

        // Sessions whose process has exited.
        let now = util::now_ms();
        let ended: Vec<u32> = self
            .sessions
            .keys()
            .copied()
            .filter(|pid| !alive(*pid, self.sessions[pid].window))
            .collect();
        for pid in ended {
            if let Some(s) = self.sessions.remove(&pid) {
                changes.push(Change::Ended(Ended {
                    name: s.game.name,
                    exe: s.game.exe,
                    duration_ms: now - s.started,
                }));
            }
        }

        let handle = foreground.0 as isize;
        // The island itself in front (summoned over a game): keep the last answer.
        if handle == island.0 as isize {
            return (self.current.clone(), changes);
        }
        if handle != self.last_foreground {
            let mut pid = 0u32;
            unsafe { GetWindowThreadProcessId(foreground, Some(&mut pid)) };
            let decided = if pid == 0 {
                Some(None)
            } else if let Some(s) = self.sessions.get(&pid) {
                Some(Some(s.game.clone()))
            } else {
                match exe_path(pid) {
                    Some(path) => Some(identify(&path, &custom(), big(foreground, monitor))),
                    // Anti-cheat can refuse even a limited query: fall back to the window title.
                    None => Some(by_title(foreground)),
                }
            };
            match decided {
                // A store game whose window is still a small splash: look again next second.
                Some(None) if pid != 0 && maybe_game_later(pid) => {}
                Some(found) => {
                    self.last_foreground = handle;
                    if let Some(game) = &found {
                        if let Entry::Vacant(slot) = self.sessions.entry(pid) {
                            slot.insert(Session {
                                game: game.clone(),
                                started: now,
                                window: handle,
                            });
                            changes.push(Change::Launched(game.clone()));
                        }
                    }
                    self.current = found;
                }
                None => {
                    self.last_foreground = handle;
                    self.current = None;
                }
            }
        }
        (self.current.clone(), changes)
    }
}

/// The front window's program name without ".exe" ("vivaldi", "T3 Code"), or "" if unknown.
pub fn app_name(hwnd: HWND) -> String {
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    exe_path(pid)
        .and_then(|p| {
            std::path::Path::new(&p)
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
        })
        .unwrap_or_default()
}

/// A process's program name without ".exe", lower case ("spotify", "youtube music").
pub fn process_name(pid: u32) -> Option<String> {
    let path = exe_path(pid)?;
    let stem = std::path::Path::new(&path)
        .file_stem()?
        .to_string_lossy()
        .to_lowercase();
    Some(stem)
}

fn exe_path(pid: u32) -> Option<String> {
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buffer = [0u16; 1024];
        let mut len = buffer.len() as u32;
        let ok = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut len,
        )
        .is_ok();
        let _ = CloseHandle(process);
        ok.then(|| String::from_utf16_lossy(&buffer[..len as usize]))
    }
}

fn alive(pid: u32, window: isize) -> bool {
    const STILL_ACTIVE: u32 = 259;
    unsafe {
        let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return IsWindow(Some(HWND(window as *mut core::ffi::c_void))).as_bool();
        };
        let mut code = 0u32;
        let ok = GetExitCodeProcess(process, &mut code).is_ok();
        let _ = CloseHandle(process);
        ok && code == STILL_ACTIVE
    }
}

/// A known game whose process cannot be queried, recognised by its window title.
fn by_title(hwnd: HWND) -> Option<Game> {
    let mut buffer = [0u16; 256];
    let len = unsafe { GetWindowTextW(hwnd, &mut buffer) }.max(0) as usize;
    let title = String::from_utf16_lossy(&buffer[..len])
        .trim()
        .to_lowercase();
    if title.is_empty() {
        return None;
    }
    KNOWN
        .iter()
        .find(|(_, name)| name.to_lowercase() == title)
        .map(|(exe, name)| Game {
            name: (*name).to_string(),
            exe: (*exe).to_string(),
            path: String::new(),
        })
}

/// The window covers most of its monitor: a game, not a launcher or a dialog.
fn big(hwnd: HWND, monitor: RECT) -> bool {
    let mut r = RECT::default();
    if unsafe { GetWindowRect(hwnd, &mut r) }.is_err() {
        return false;
    }
    let area = |r: &RECT| ((r.right - r.left).max(0) as i64) * ((r.bottom - r.top).max(0) as i64);
    let screen = area(&monitor);
    screen > 0 && area(&r) * 10 >= screen * 6
}

/// A small window from a store library could still grow into the game (splash screens).
fn maybe_game_later(pid: u32) -> bool {
    exe_path(pid)
        .map(|p| library_name(&p.to_lowercase(), &p).is_some())
        .unwrap_or(false)
}

fn library_name(lower: &str, original: &str) -> Option<String> {
    for marker in LIBRARIES {
        if let Some(at) = lower.find(marker) {
            let rest = &original[at + marker.len()..];
            let folder = rest.split('\\').next().unwrap_or("").trim();
            let exe = lower.rsplit('\\').next().unwrap_or("");
            let folder_lower = folder.to_lowercase();
            if folder.is_empty()
                || NOT_GAMES
                    .iter()
                    .any(|n| folder_lower.contains(n) || exe.contains(n))
            {
                return None;
            }
            return Some(folder.to_string());
        }
    }
    None
}

/// Known exe → custom list → store library folder (only for big windows).
pub fn identify(path: &str, custom: &[(String, String)], big: bool) -> Option<Game> {
    let lower = path.to_lowercase();
    let exe = lower.rsplit(['\\', '/']).next().unwrap_or("").to_string();
    for (custom_exe, name) in custom {
        let mut want = custom_exe.trim().to_lowercase();
        if !want.ends_with(".exe") {
            want.push_str(".exe");
        }
        if want == exe {
            let name = if name.trim().is_empty() {
                exe.trim_end_matches(".exe").to_string()
            } else {
                name.trim().to_string()
            };
            return Some(Game { name, exe, path: path.to_string() });
        }
    }
    if let Some((_, name)) = KNOWN.iter().find(|(known, _)| *known == exe) {
        return Some(Game {
            name: (*name).to_string(),
            exe,
            path: path.to_string(),
        });
    }
    if big {
        if let Some(name) = library_name(&lower, path) {
            return Some(Game { name, exe, path: path.to_string() });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::identify;

    #[test]
    fn finds_games() {
        let none: Vec<(String, String)> = Vec::new();
        let gta = identify(r"D:\Games\Grand Theft Auto V\GTA5.exe", &none, false).unwrap();
        assert_eq!(gta.name, "Grand Theft Auto V");
        let steam = identify(
            r"C:\Program Files (x86)\Steam\steamapps\common\Hades II\Ship\Hades2.exe",
            &none,
            true,
        )
        .unwrap();
        assert_eq!(steam.name, "Hades II");
        // A store game only counts once its window is big.
        assert!(identify(
            r"C:\Program Files (x86)\Steam\steamapps\common\Hades II\Ship\Hades2.exe",
            &none,
            false
        )
        .is_none());
        // Launchers in a library folder are not games.
        assert!(identify(
            r"C:\Riot Games\Riot Client\RiotClientServices.exe",
            &none,
            true
        )
        .is_none());
        assert!(identify(r"C:\Windows\explorer.exe", &none, true).is_none());
        let custom = vec![("MyGame".to_string(), "My game".to_string())];
        assert_eq!(
            identify(r"E:\x\mygame.exe", &custom, false).unwrap().name,
            "My game"
        );
    }
}
