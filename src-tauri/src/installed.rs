//! The programs Windows lists as installed (Settings → Apps), read from the Uninstall keys. Game
//! launchers fill these in for every game they install: Riot ("VALORANT", with the game's own
//! .ico), Steam ("Steam App 1422450": Deadlock, its icon and install folder), EA, Ubisoft,
//! Battle.net, GOG and ordinary installers. So it is the one place to find a game's logo without
//! asking the game's process, which anti-cheat may hide, and without any web service.
//!
//! Read on demand when a game starts: a few hundred keys, a few milliseconds, nothing to keep.

use windows::{
    core::{PCWSTR, PWSTR},
    Win32::{
        Foundation::{ERROR_MORE_DATA, ERROR_SUCCESS},
        System::Registry::{
            RegCloseKey, RegEnumKeyExW, RegGetValueW, RegOpenKeyExW, HKEY, HKEY_CURRENT_USER,
            HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY, REG_SAM_FLAGS,
            RRF_RT_REG_SZ,
        },
    },
};

use crate::util::wide;

const UNINSTALL: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall";

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Program {
    pub name: String,
    /// DisplayIcon as written: `path`, `"path"` or `path,index`.
    pub icon: String,
    pub location: String,
}

/// Everything in the per-user and machine-wide (64- and 32-bit) Uninstall keys.
pub fn list() -> Vec<Program> {
    let mut out = Vec::new();
    for (root, view) in [
        (HKEY_CURRENT_USER, REG_SAM_FLAGS(0)),
        (HKEY_LOCAL_MACHINE, KEY_WOW64_64KEY),
        (HKEY_LOCAL_MACHINE, KEY_WOW64_32KEY),
    ] {
        read_root(root, view, &mut out);
    }
    out
}

fn read_root(root: HKEY, view: REG_SAM_FLAGS, out: &mut Vec<Program>) {
    let path = wide(UNINSTALL);
    let mut key = HKEY::default();
    let opened = unsafe { RegOpenKeyExW(root, PCWSTR(path.as_ptr()), Some(0), KEY_READ | view, &mut key) };
    if opened != ERROR_SUCCESS {
        return;
    }
    let mut buffer = [0u16; 256];
    for index in 0.. {
        let mut len = buffer.len() as u32;
        let status = unsafe {
            RegEnumKeyExW(key, index, Some(PWSTR(buffer.as_mut_ptr())), &mut len, None, None, None, None)
        };
        if status == ERROR_MORE_DATA {
            continue; // a sub key name over 255 characters: not a program entry
        }
        if status != ERROR_SUCCESS {
            break;
        }
        let mut sub = buffer[..len as usize].to_vec();
        sub.push(0);
        let name = value(key, &sub, "DisplayName");
        if name.is_empty() {
            continue;
        }
        out.push(Program {
            name,
            icon: value(key, &sub, "DisplayIcon"),
            location: value(key, &sub, "InstallLocation"),
        });
    }
    unsafe {
        let _ = RegCloseKey(key);
    }
}

/// A string value of a sub key ("" when missing). REG_EXPAND_SZ comes back expanded.
fn value(key: HKEY, sub: &[u16], name: &str) -> String {
    let name = wide(name);
    let mut size = 0u32;
    let read = |data: Option<*mut core::ffi::c_void>, size: &mut u32| unsafe {
        RegGetValueW(
            key,
            PCWSTR(sub.as_ptr()),
            PCWSTR(name.as_ptr()),
            RRF_RT_REG_SZ,
            None,
            data,
            Some(size),
        )
    };
    if read(None, &mut size) != ERROR_SUCCESS || !(2..=4096).contains(&size) {
        return String::new();
    }
    let mut data = vec![0u16; size as usize / 2];
    if read(Some(data.as_mut_ptr() as *mut core::ffi::c_void), &mut size) != ERROR_SUCCESS {
        return String::new();
    }
    let len = data.iter().position(|&c| c == 0).unwrap_or(data.len());
    String::from_utf16_lossy(&data[..len]).trim().to_string()
}

/// The icon file a game's logo can be read from: the matching program's DisplayIcon when it is
/// an .ico or .exe that exists (never an uninstaller's).
pub fn logo_file(name: &str, exe_path: &str) -> Option<String> {
    let programs = list();
    let program = find(&programs, name, exe_path)?;
    let file = icon_file(&program.icon)?;
    std::path::Path::new(&file).is_file().then_some(file)
}

/// The program a game belongs to: by install folder when the exe path is known (the deepest
/// folder holding the exe wins), else by name compared on letters and digits only.
pub fn find<'a>(programs: &'a [Program], name: &str, exe_path: &str) -> Option<&'a Program> {
    let usable = |p: &&Program| icon_file(&p.icon).is_some();
    let exe = folder_form(exe_path);
    if !exe.is_empty() {
        let by_folder = programs
            .iter()
            .filter(usable)
            .filter_map(|p| {
                let folder = folder_form(&p.location);
                (specific(&folder) && exe.starts_with(&format!("{folder}\\"))).then_some((folder.len(), p))
            })
            .max_by_key(|(len, _)| *len)
            .map(|(_, p)| p);
        if by_folder.is_some() {
            return by_folder;
        }
    }
    let want = letters(name);
    if want.len() < 2 {
        return None;
    }
    programs.iter().filter(usable).find(|p| letters(&p.name) == want)
}

/// DisplayIcon's file: `C:\a\game.exe,0`, `"C:\a\game.exe",-101` or `C:/a/game.ico`. Riot writes
/// forward slashes, which the shell won't open, so the path comes back with backslashes.
pub fn icon_file(display_icon: &str) -> Option<String> {
    let raw = display_icon.trim();
    let path = if let Some(rest) = raw.strip_prefix('"') {
        rest.split('"').next().unwrap_or("")
    } else {
        match raw.rsplit_once(',') {
            Some((path, index)) if index.trim().parse::<i32>().is_ok() => path,
            _ => raw,
        }
    }
    .trim();
    let lower = path.to_lowercase();
    let file = lower.rsplit(['\\', '/']).next().unwrap_or("");
    let kind_ok = lower.ends_with(".ico") || lower.ends_with(".exe");
    (kind_ok && !file.contains("unins")).then(|| path.replace('/', "\\"))
}

/// Lower case, backslashes, no trailing slash: for comparing folders.
fn folder_form(path: &str) -> String {
    path.trim()
        .trim_matches('"')
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase()
}

/// A folder narrow enough to say which program a file belongs to: not a drive, not a shared root.
fn specific(folder: &str) -> bool {
    const SHARED: &[&str] = &[
        "program files",
        "program files (x86)",
        "programdata",
        "windows",
        "appdata",
        "local",
        "roaming",
        "programs",
        "games",
        "steamapps",
        "common",
        "steamlibrary",
        "riot games",
        "epic games",
    ];
    let mut parts = folder.split('\\');
    let _drive = parts.next();
    match parts.next_back() {
        Some(last) if !last.is_empty() => !SHARED.contains(&last),
        _ => false,
    }
}

fn letters(name: &str) -> String {
    name.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn program(name: &str, icon: &str, location: &str) -> Program {
        Program { name: name.into(), icon: icon.into(), location: location.into() }
    }

    fn sample() -> Vec<Program> {
        vec![
            program("Riot Client", "C:/Riot Games/Riot Client/Resources/icon.ico", "C:/Riot Games/Riot Client"),
            program(
                "VALORANT",
                "C:/ProgramData/Riot Games/Metadata/valorant.live/valorant.live.ico",
                "C:/Riot Games/VALORANT/live",
            ),
            program(
                "Deadlock",
                r"C:\Program Files (x86)\Steam\steam\games\d841.ico",
                r"C:\Program Files (x86)\Steam\steamapps\common\Deadlock",
            ),
            program("Steam", r"C:\Program Files (x86)\Steam\uninstall.exe", r"C:\Program Files (x86)\Steam"),
            program("Some Tool", r"C:\Program Files\Tool\tool.exe,0", r"C:\Program Files"),
        ]
    }

    #[test]
    fn reads_display_icon_forms() {
        assert_eq!(icon_file(r"C:\a\game.exe,0").as_deref(), Some(r"C:\a\game.exe"));
        assert_eq!(icon_file(r#""C:\a b\game.exe",-101"#).as_deref(), Some(r"C:\a b\game.exe"));
        assert_eq!(icon_file("C:/x/valorant.live.ico").as_deref(), Some(r"C:\x\valorant.live.ico"));
        assert_eq!(icon_file(r"C:\Steam\uninstall.exe"), None);
        assert_eq!(icon_file(r"C:\Game\unins000.exe,0"), None);
        assert_eq!(icon_file(r"C:\a\shell32.dll,4"), None);
        assert_eq!(icon_file(""), None);
    }

    #[test]
    fn finds_by_install_folder() {
        let all = sample();
        let path = r"C:\Riot Games\VALORANT\live\ShooterGame\Binaries\Win64\VALORANT-Win64-Shipping.exe";
        assert_eq!(find(&all, "anything", path).unwrap().name, "VALORANT");
        let deadlock = r"C:\Program Files (x86)\Steam\steamapps\common\Deadlock\game\bin\win64\deadlock.exe";
        assert_eq!(find(&all, "deadlock", deadlock).unwrap().name, "Deadlock");
    }

    #[test]
    fn finds_by_name_when_the_exe_is_hidden() {
        let all = sample();
        assert_eq!(find(&all, "Valorant", "").unwrap().name, "VALORANT");
        assert!(find(&all, "Valorant 2", "").is_none());
        assert!(find(&all, "V", "").is_none());
    }

    #[test]
    fn ignores_shared_folders_and_uninstallers() {
        let all = sample();
        // "Some Tool" says its folder is Program Files: that must not claim every program there.
        assert!(find(&all, "x", r"C:\Program Files\Other\other.exe").is_none());
        // Steam's own entry has only its uninstaller as an icon.
        assert!(find(&all, "Steam", "").is_none());
    }
}
