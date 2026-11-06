//! AI coding tools on this PC: which are installed, which run right now, when each was last used
//! and what it used today. Everything is read from the files each tool already keeps (session
//! logs, its own SQLite database); nothing leaves the PC and nothing is written.
//!
//! Scans run only when asked (the Agents page or the AI tools settings are on screen). The large
//! Claude Code logs are read incrementally: each scan reads only what was appended since the last.

use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::Mutex,
    time::UNIX_EPOCH,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use windows::Win32::{
    Foundation::CloseHandle,
    System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    },
};

use crate::{quotas::parse_time, util};

/// One tool as the settings and the Agents page see it.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    id: &'static str,
    name: &'static str,
    /// Found on PATH, in its install folder, or by its data folder.
    installed: bool,
    running: bool,
    last_used: Option<i64>,
    today: Option<Today>,
    /// Has plan limits (the usage rings): Claude Code and Codex.
    plan: bool,
}

#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Today {
    /// Tokens sent and received, cache reads left out (they would dwarf the rest).
    tokens: Option<u64>,
    sessions: u32,
    /// Spend the tool itself recorded, in US dollars (opencode).
    cost: Option<f64>,
}

#[derive(Clone, Copy)]
enum Base {
    Home,
    Local,
    Roaming,
}

struct Spec {
    id: &'static str,
    name: &'static str,
    /// Program names to look for on PATH and in the process list (without .exe).
    exes: &'static [&'static str],
    /// Data folders: their presence means installed, their newest file means last used.
    data: &'static [(Base, &'static str)],
    plan: bool,
}

const TOOLS: &[Spec] = &[
    Spec {
        id: "claude",
        name: "Claude Code",
        exes: &["claude"],
        data: &[(Base::Home, ".claude/projects")],
        plan: true,
    },
    Spec {
        id: "codex",
        name: "Codex",
        exes: &["codex"],
        data: &[(Base::Home, ".codex/sessions")],
        plan: true,
    },
    Spec {
        id: "opencode",
        name: "OpenCode",
        exes: &["opencode"],
        data: &[(Base::Home, ".local/share/opencode")],
        plan: false,
    },
    Spec {
        id: "antigravity",
        name: "Antigravity",
        exes: &["agy", "antigravity"],
        data: &[(Base::Home, ".gemini/antigravity/brain")],
        plan: false,
    },
    Spec {
        id: "gemini",
        name: "Gemini CLI",
        exes: &["gemini"],
        data: &[(Base::Home, ".gemini/tmp")],
        plan: false,
    },
    Spec {
        id: "copilot",
        name: "GitHub Copilot CLI",
        exes: &["copilot"],
        data: &[
            (Base::Home, ".copilot/session-state"),
            (Base::Home, ".copilot/chats"),
        ],
        plan: false,
    },
    Spec {
        id: "cline",
        name: "Cline",
        exes: &["cline"],
        data: &[(Base::Home, ".cline/data/sessions")],
        plan: false,
    },
    Spec {
        id: "cursor",
        name: "Cursor Agent",
        exes: &["cursor-agent"],
        data: &[(Base::Home, ".cursor/chats")],
        plan: false,
    },
    Spec {
        id: "amp",
        name: "Amp",
        exes: &["amp"],
        data: &[(Base::Home, ".local/share/amp"), (Base::Home, ".amp")],
        plan: false,
    },
    Spec {
        id: "qwen",
        name: "Qwen Code",
        exes: &["qwen"],
        data: &[(Base::Home, ".qwen/tmp")],
        plan: false,
    },
    Spec {
        id: "crush",
        name: "Crush",
        exes: &["crush"],
        data: &[(Base::Local, "crush"), (Base::Home, ".local/share/crush")],
        plan: false,
    },
    Spec {
        id: "droid",
        name: "Factory Droid",
        exes: &["droid"],
        data: &[(Base::Home, ".factory/sessions")],
        plan: false,
    },
    Spec {
        id: "kiro",
        name: "Kiro CLI",
        exes: &["kiro-cli", "kiro"],
        data: &[(Base::Home, ".kiro")],
        plan: false,
    },
    Spec {
        id: "goose",
        name: "Goose",
        exes: &["goose"],
        data: &[(Base::Roaming, "Block/goose")],
        plan: false,
    },
    Spec {
        id: "aider",
        name: "Aider",
        exes: &["aider"],
        data: &[],
        plan: false,
    },
];

fn base(b: Base) -> PathBuf {
    match b {
        Base::Home => util::home_dir(),
        Base::Local => std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_default(),
        Base::Roaming => std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_default(),
    }
}

fn on_path(exes: &[&str]) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| {
        exes.iter().any(|exe| {
            ["exe", "cmd", "bat"]
                .iter()
                .any(|ext| dir.join(format!("{exe}.{ext}")).is_file())
        })
    })
}

fn processes() -> HashSet<String> {
    let mut names = HashSet::new();
    unsafe {
        let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return names;
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snap, &mut entry).is_ok() {
            loop {
                let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(0);
                let name = String::from_utf16_lossy(&entry.szExeFile[..len]).to_lowercase();
                names.insert(name.trim_end_matches(".exe").to_string());
                if Process32NextW(snap, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snap);
    }
    names
}

fn mtime(meta: &fs::Metadata) -> i64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Newest modification time under `dir`, at most `depth` levels down and `budget` entries.
fn newest(dir: &Path, depth: u32, budget: &mut u32) -> i64 {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    let mut best = 0;
    for entry in entries.flatten() {
        if *budget == 0 {
            break;
        }
        *budget -= 1;
        let Ok(meta) = entry.metadata() else { continue };
        best = best.max(mtime(&meta));
        if meta.is_dir() && depth > 0 {
            best = best.max(newest(&entry.path(), depth - 1, budget));
        }
    }
    best
}

/// Entries directly under `dir` changed since `since` (sessions started or continued today).
fn changed_since(dir: &Path, since: i64) -> u32 {
    fs::read_dir(dir)
        .map(|e| {
            e.flatten()
                .filter(|e| e.metadata().map(|m| mtime(&m) >= since).unwrap_or(false))
                .count() as u32
        })
        .unwrap_or(0)
}

// ------------------------------------------------------------------ Claude Code

/// Running totals for the day, fed by appended log lines.
#[derive(Default)]
struct ClaudeDay {
    since: i64,
    /// File → bytes already read.
    offsets: HashMap<PathBuf, u64>,
    /// Message ids already counted (a reply is written in several lines).
    seen: HashSet<String>,
    sessions: HashSet<String>,
    tokens: u64,
}

static CLAUDE: Mutex<Option<ClaudeDay>> = Mutex::new(None);

fn claude_today(since: i64) -> Today {
    let mut guard = CLAUDE.lock().unwrap_or_else(|e| e.into_inner());
    if guard.as_ref().map(|d| d.since) != Some(since) {
        *guard = Some(ClaudeDay {
            since,
            ..Default::default()
        });
    }
    let day = guard.as_mut().expect("claude day");
    let root = util::home_dir().join(".claude/projects");
    for project in fs::read_dir(&root).into_iter().flatten().flatten() {
        for file in fs::read_dir(project.path()).into_iter().flatten().flatten() {
            let path = file.path();
            if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }
            let Ok(meta) = file.metadata() else { continue };
            if mtime(&meta) < since {
                continue;
            }
            let from = day.offsets.get(&path).copied().unwrap_or(0);
            if meta.len() <= from {
                continue;
            }
            let Ok(mut f) = fs::File::open(&path) else {
                continue;
            };
            if f.seek(SeekFrom::Start(from)).is_err() {
                continue;
            }
            // Line by line, one buffer: a day's log can be tens of megabytes.
            let mut reader = BufReader::with_capacity(64 * 1024, f);
            let mut line = Vec::new();
            let mut pos = from;
            loop {
                line.clear();
                match reader.read_until(b'\n', &mut line) {
                    // Only whole lines: one still being written is read next time.
                    Ok(n) if n > 0 && line.last() == Some(&b'\n') => pos += n as u64,
                    _ => break,
                }
                // Cheap filter before parsing: only replies carry usage.
                if line.len() < 20 || !contains(&line, b"\"usage\"") {
                    continue;
                }
                // Only the fields needed; the message content is skipped, not built.
                let Ok(entry) = serde_json::from_slice::<LogLine>(&line) else {
                    continue;
                };
                let at = entry.timestamp.as_deref().and_then(parse_time).unwrap_or(0);
                let Some(msg) = entry.message else { continue };
                let (Some(id), Some(u)) = (msg.id, msg.usage) else {
                    continue;
                };
                if at < since || !day.seen.insert(id) {
                    continue;
                }
                day.tokens += u.input_tokens + u.output_tokens + u.cache_creation_input_tokens;
                if let Some(session) = entry.session_id {
                    day.sessions.insert(session);
                }
            }
            day.offsets.insert(path.clone(), pos);
        }
    }
    Today {
        tokens: Some(day.tokens),
        sessions: day.sessions.len() as u32,
        cost: None,
    }
}

#[derive(Deserialize)]
struct LogLine {
    timestamp: Option<String>,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
    message: Option<LogMessage>,
}

#[derive(Deserialize)]
struct LogMessage {
    id: Option<String>,
    usage: Option<LogUsage>,
}

#[derive(Deserialize)]
struct LogUsage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
    #[serde(default)]
    cache_creation_input_tokens: u64,
}

fn contains(hay: &[u8], needle: &[u8]) -> bool {
    hay.windows(needle.len()).any(|w| w == needle)
}

// ------------------------------------------------------------------ Codex

fn codex_today(since: i64) -> Today {
    let root = util::home_dir().join(".codex/sessions");
    let mut files = Vec::new();
    collect_files(&root, 3, since, &mut files);
    let mut today = Today::default();
    let mut tokens = 0u64;
    for path in files {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        // Totals are cumulative per session: today's share is the last total minus the last one
        // from before midnight.
        let (mut before, mut last, mut active) = (0u64, 0u64, false);
        for line in text.lines() {
            if !line.contains("\"token_count\"") {
                continue;
            }
            let Ok(o) = serde_json::from_str::<Value>(line) else {
                continue;
            };
            let total = &o["payload"]["info"]["total_token_usage"];
            let Some(all) = total.get("total_tokens").and_then(Value::as_u64) else {
                continue;
            };
            let used = all.saturating_sub(
                total
                    .get("cached_input_tokens")
                    .and_then(Value::as_u64)
                    .unwrap_or(0),
            );
            let at = o
                .get("timestamp")
                .and_then(Value::as_str)
                .and_then(parse_time)
                .unwrap_or(0);
            if at < since {
                before = used;
            } else {
                last = used;
                active = true;
            }
        }
        if active {
            today.sessions += 1;
            tokens += last.saturating_sub(before);
        }
    }
    today.tokens = Some(tokens);
    today
}

fn collect_files(dir: &Path, depth: u32, since: i64, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        if mtime(&meta) < since {
            continue;
        }
        if meta.is_dir() && depth > 0 {
            collect_files(&entry.path(), depth - 1, since, out);
        } else if meta.is_file() {
            out.push(entry.path());
        }
    }
}

// ------------------------------------------------------------------ OpenCode, Cline (SQLite)

fn opencode_today(since: i64) -> Option<Today> {
    let db = util::home_dir().join(".local/share/opencode/opencode.db");
    let rows = sqlite::query(
        &db,
        "select session_id, data from message where time_created >= ?1",
        since,
    )?;
    let mut today = Today {
        tokens: Some(0),
        cost: Some(0.0),
        ..Default::default()
    };
    let mut sessions = HashSet::new();
    for row in rows {
        let Ok(d) = serde_json::from_str::<Value>(&row[1]) else {
            continue;
        };
        if d.get("role").and_then(Value::as_str) != Some("assistant") {
            continue;
        }
        sessions.insert(row[0].clone());
        let t = &d["tokens"];
        let n = |v: &Value| v.as_u64().unwrap_or(0);
        *today.tokens.as_mut()? +=
            n(&t["input"]) + n(&t["output"]) + n(&t["reasoning"]) + n(&t["cache"]["write"]);
        *today.cost.as_mut()? += d.get("cost").and_then(Value::as_f64).unwrap_or(0.0);
    }
    today.sessions = sessions.len() as u32;
    Some(today)
}

fn cline_today(since: i64) -> Option<Today> {
    let db = util::home_dir().join(".cline/data/db/sessions.db");
    let rows = sqlite::query(&db, "select started_at from sessions where ?1 >= 0", 0)?;
    let sessions = rows
        .iter()
        .filter(|r| parse_time(&r[0]).or_else(|| r[0].parse().ok()).unwrap_or(0) >= since)
        .count() as u32;
    Some(Today {
        sessions,
        ..Default::default()
    })
}

/// Windows ships SQLite (winsqlite3.dll, Windows 10 and later): loaded on first use, so reading
/// another tool's database costs no bundled library. Read-only, one query at a time.
mod sqlite {
    use std::{
        ffi::{c_char, c_int, c_void, CStr, CString},
        path::Path,
        sync::OnceLock,
    };

    use windows::{
        core::{s, w},
        Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW},
    };

    type Db = *mut c_void;
    type Stmt = *mut c_void;

    struct Api {
        open: unsafe extern "system" fn(*const c_char, *mut Db, c_int, *const c_char) -> c_int,
        busy: unsafe extern "system" fn(Db, c_int) -> c_int,
        prepare: unsafe extern "system" fn(
            Db,
            *const c_char,
            c_int,
            *mut Stmt,
            *mut *const c_char,
        ) -> c_int,
        bind: unsafe extern "system" fn(Stmt, c_int, i64) -> c_int,
        step: unsafe extern "system" fn(Stmt) -> c_int,
        count: unsafe extern "system" fn(Stmt) -> c_int,
        text: unsafe extern "system" fn(Stmt, c_int) -> *const c_char,
        finalize: unsafe extern "system" fn(Stmt) -> c_int,
        close: unsafe extern "system" fn(Db) -> c_int,
    }

    static API: OnceLock<Option<Api>> = OnceLock::new();

    // Each transmute's target is the Api field it fills: the C signature from sqlite3.h.
    #[allow(clippy::missing_transmute_annotations)]
    fn api() -> Option<&'static Api> {
        API.get_or_init(|| unsafe {
            let lib = LoadLibraryW(w!("winsqlite3.dll")).ok()?;
            macro_rules! f {
                ($name:expr) => {
                    std::mem::transmute(GetProcAddress(lib, $name)?)
                };
            }
            Some(Api {
                open: f!(s!("sqlite3_open_v2")),
                busy: f!(s!("sqlite3_busy_timeout")),
                prepare: f!(s!("sqlite3_prepare_v2")),
                bind: f!(s!("sqlite3_bind_int64")),
                step: f!(s!("sqlite3_step")),
                count: f!(s!("sqlite3_column_count")),
                text: f!(s!("sqlite3_column_text")),
                finalize: f!(s!("sqlite3_finalize")),
                close: f!(s!("sqlite3_close")),
            })
        })
        .as_ref()
    }

    const READONLY: c_int = 0x1;
    const ROW: c_int = 100;

    /// Runs `sql` with one integer parameter and returns every row as text columns.
    pub fn query(db: &Path, sql: &str, param: i64) -> Option<Vec<Vec<String>>> {
        if !db.is_file() {
            return None;
        }
        let api = api()?;
        let path = CString::new(db.to_string_lossy().as_bytes()).ok()?;
        let sql = CString::new(sql).ok()?;
        unsafe {
            let mut handle: Db = std::ptr::null_mut();
            if (api.open)(path.as_ptr(), &mut handle, READONLY, std::ptr::null()) != 0 {
                (api.close)(handle);
                return None;
            }
            (api.busy)(handle, 1500);
            let mut stmt: Stmt = std::ptr::null_mut();
            if (api.prepare)(handle, sql.as_ptr(), -1, &mut stmt, std::ptr::null_mut()) != 0 {
                (api.close)(handle);
                return None;
            }
            (api.bind)(stmt, 1, param);
            let columns = (api.count)(stmt);
            let mut rows = Vec::new();
            while (api.step)(stmt) == ROW {
                let row = (0..columns)
                    .map(|i| {
                        let p = (api.text)(stmt, i);
                        if p.is_null() {
                            String::new()
                        } else {
                            CStr::from_ptr(p).to_string_lossy().into_owned()
                        }
                    })
                    .collect();
                rows.push(row);
            }
            (api.finalize)(stmt);
            (api.close)(handle);
            Some(rows)
        }
    }
}

// ------------------------------------------------------------------ scan

fn scan(since: i64) -> Vec<Tool> {
    let running = processes();
    let now = util::now_ms();
    TOOLS
        .iter()
        .map(|spec| {
            let dirs: Vec<PathBuf> = spec
                .data
                .iter()
                .map(|(b, rel)| base(*b).join(rel))
                .collect();
            let has_data = dirs.iter().any(|d| d.exists());
            let installed = has_data || on_path(spec.exes);
            let mut last = 0;
            if installed {
                for dir in &dirs {
                    let mut budget = 4000;
                    last = last.max(newest(dir, 3, &mut budget));
                }
            }
            let today = if !installed || last < since {
                None
            } else {
                match spec.id {
                    "claude" => Some(claude_today(since)),
                    "codex" => Some(codex_today(since)),
                    "opencode" => opencode_today(since),
                    "cline" => cline_today(since),
                    _ => {
                        let sessions: u32 = dirs.iter().map(|d| changed_since(d, since)).sum();
                        Some(Today {
                            sessions,
                            ..Default::default()
                        })
                    }
                }
            };
            Tool {
                id: spec.id,
                name: spec.name,
                installed,
                running: spec.exes.iter().any(|e| running.contains(*e))
                    || (last > 0 && now - last < 90_000),
                last_used: (last > 0).then_some(last),
                today,
                plan: spec.plan,
            }
        })
        .collect()
}

/// `since`: local midnight in Unix ms, from the frontend (it knows the time zone).
#[tauri::command]
pub async fn harness_scan(since: i64) -> Vec<Tool> {
    tauri::async_runtime::spawn_blocking(move || scan(since))
        .await
        .unwrap_or_default()
}
