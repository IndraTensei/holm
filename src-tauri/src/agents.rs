//! Live activity for coding agents (Claude Code first) through Claude Code's HTTP hooks.
//!
//! A tiny HTTP server on 127.0.0.1 receives every hook event, keeps a small picture of each
//! session (what it is doing, files edited, the last answer) and answers permission requests
//! with the choice made in the island. Nothing leaves the PC; the token in the hook headers
//! keeps other local programs from posting.

use std::{
    collections::{hash_map::RandomState, HashMap, VecDeque},
    hash::{BuildHasher, Hasher},
    io::Read,
    path::PathBuf,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use serde::Serialize;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::{
    config::Config,
    util::{self, file_name, now_ms, truncate},
};

pub const DEFAULT_PORT: u16 = 47821;
const MARK: &str = "src=tensei-island";
const EVENTS: [&str; 12] = [
    "SessionStart",
    "UserPromptSubmit",
    "PreToolUse",
    "PermissionRequest",
    "PostToolUse",
    "PostToolUseFailure",
    "Notification",
    "Stop",
    "StopFailure",
    "SubagentStart",
    "SubagentStop",
    "SessionEnd",
];

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    verb: String,
    target: String,
    tool: String,
    kind: &'static str,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    at: i64,
    kind: &'static str,
    text: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgentSession {
    id: String,
    agent: &'static str,
    project: String,
    cwd: String,
    /// idle | working | waiting | done | error | ended
    status: &'static str,
    activity: Option<Activity>,
    prompt: String,
    message: String,
    turn_started_at: i64,
    updated_at: i64,
    tools: u32,
    edited: Vec<String>,
    subagents: u32,
    log: VecDeque<LogEntry>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Approval {
    id: String,
    session_id: String,
    project: String,
    tool: String,
    title: String,
    detail: String,
    created_at: i64,
    expires_at: i64,
}

struct Pending {
    info: Approval,
    tx: mpsc::Sender<bool>,
}

#[derive(Default)]
pub struct Agents {
    sessions: Mutex<HashMap<String, AgentSession>>,
    pending: Mutex<HashMap<String, Pending>>,
}

#[derive(Serialize, Clone)]
struct Snapshot {
    sessions: Vec<AgentSession>,
    approvals: Vec<Approval>,
}

impl Agents {
    fn snapshot(&self) -> Snapshot {
        let mut sessions: Vec<_> = self
            .sessions
            .lock()
            .expect("sessions")
            .values()
            .cloned()
            .collect();
        sessions.sort_by_key(|s| std::cmp::Reverse(s.updated_at));
        let mut approvals: Vec<_> = self
            .pending
            .lock()
            .expect("pending")
            .values()
            .map(|p| p.info.clone())
            .collect();
        approvals.sort_by_key(|a| a.created_at);
        Snapshot {
            sessions,
            approvals,
        }
    }
}

pub fn random_token() -> String {
    let mut out = String::new();
    for salt in 0..2u64 {
        let mut hasher = RandomState::new().build_hasher();
        hasher.write_u64(now_ms() as u64 ^ salt.rotate_left(17));
        hasher.write_u32(std::process::id());
        out.push_str(&format!("{:016x}", hasher.finish()));
    }
    out
}

pub fn token(config: &Config) -> String {
    let existing = config.string("hookToken", "");
    if existing.len() >= 16 {
        return existing;
    }
    let token = random_token();
    config.patch(&json!({ "hookToken": token }));
    token
}

pub fn start(app: AppHandle) {
    let agents = Arc::new(Agents::default());
    app.manage(agents.clone());
    let config = app.state::<Config>();
    let port = config.u64("hookPort", DEFAULT_PORT as u64) as u16;
    let secret = token(&config);
    let log_dir = config.dir().clone();
    let app2 = app.clone();
    thread::Builder::new()
        .name("agents-http".into())
        .spawn(move || {
            let server = match tiny_http::Server::http(("127.0.0.1", port)) {
                Ok(server) => server,
                Err(err) => {
                    util::log(
                        &log_dir,
                        &format!("agent hook server could not listen on {port}: {err}"),
                    );
                    let _ = app.emit("agents://error", format!("Port {port} is busy"));
                    return;
                }
            };
            for request in server.incoming_requests() {
                let app = app.clone();
                let agents = agents.clone();
                let secret = secret.clone();
                // Permission requests wait for a decision; everything else answers at once.
                thread::spawn(move || handle(app, agents, &secret, request));
            }
        })
        .expect("agents thread");

    // Housekeeping: drop sessions that went quiet hours ago.
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(60));
        if let Some(agents) = app2.try_state::<Arc<Agents>>() {
            let now = now_ms();
            let mut changed = false;
            agents.sessions.lock().expect("sessions").retain(|_, s| {
                let keep = match s.status {
                    "ended" => now - s.updated_at < 5 * 60_000,
                    _ => now - s.updated_at < 3 * 3_600_000,
                };
                changed |= !keep;
                keep
            });
            if changed {
                let _ = app2.emit("agents://state", agents.snapshot());
            }
        }
    });
}

fn respond(request: tiny_http::Request, status: u16, body: String) {
    let header = tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
        .expect("header");
    let response = tiny_http::Response::from_string(body)
        .with_status_code(status)
        .with_header(header);
    let _ = request.respond(response);
}

fn handle(app: AppHandle, agents: Arc<Agents>, secret: &str, mut request: tiny_http::Request) {
    let url = request.url().to_string();
    if url.starts_with("/health") {
        return respond(request, 200, "{\"ok\":true}".into());
    }
    let authorized = request
        .headers()
        .iter()
        .any(|h| h.field.equiv("X-Island-Token") && h.value.as_str() == secret);
    if !url.starts_with("/hook") || !authorized {
        return respond(request, 403, String::new());
    }
    let mut body = String::new();
    if request
        .as_reader()
        .take(4 * 1024 * 1024)
        .read_to_string(&mut body)
        .is_err()
    {
        return respond(request, 400, String::new());
    }
    let Ok(event) = serde_json::from_str::<Value>(&body) else {
        return respond(request, 400, String::new());
    };
    let name = event
        .get("hook_event_name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    if name == "PermissionRequest" {
        let timeout = app
            .state::<Config>()
            .u64("approvalTimeout", 60)
            .clamp(10, 590);
        let answer = wait_for_decision(&app, &agents, &event, timeout);
        let body = match answer {
            Some(true) => json!({
                "hookSpecificOutput": { "hookEventName": "PermissionRequest", "decision": { "behavior": "allow" } }
            })
            .to_string(),
            Some(false) => json!({
                "hookSpecificOutput": {
                    "hookEventName": "PermissionRequest",
                    "decision": { "behavior": "deny", "message": "Denied from Tensei Island." }
                }
            })
            .to_string(),
            // No answer in time: an empty reply lets Claude Code ask in the terminal as usual.
            None => String::new(),
        };
        return respond(request, 200, body);
    }

    apply(&app, &agents, &name, &event);
    respond(request, 200, String::new());
}

fn text(event: &Value, key: &str) -> String {
    event
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

fn project_of(cwd: &str) -> String {
    let name = file_name(cwd.trim_end_matches(['\\', '/']));
    if name.is_empty() {
        "Claude Code".into()
    } else {
        name
    }
}

fn session_entry<'a>(
    sessions: &'a mut HashMap<String, AgentSession>,
    event: &Value,
) -> &'a mut AgentSession {
    let id = text(event, "session_id");
    let cwd = text(event, "cwd");
    let session = sessions.entry(id.clone()).or_insert_with(|| AgentSession {
        id,
        agent: "claude",
        project: project_of(&cwd),
        cwd: cwd.clone(),
        status: "idle",
        activity: None,
        prompt: String::new(),
        message: String::new(),
        turn_started_at: now_ms(),
        updated_at: now_ms(),
        tools: 0,
        edited: Vec::new(),
        subagents: 0,
        log: VecDeque::new(),
    });
    if !cwd.is_empty() && cwd != session.cwd {
        session.project = project_of(&cwd);
        session.cwd = cwd;
    }
    session.updated_at = now_ms();
    session
}

fn push_log(session: &mut AgentSession, kind: &'static str, text: String) {
    session.log.push_front(LogEntry {
        at: now_ms(),
        kind,
        text,
    });
    session.log.truncate(24);
}

/// A short, human description of a tool call: "Editing main.rs", "Running npm test".
pub fn describe(tool: &str, input: &Value) -> Activity {
    let s = |key: &str| {
        input
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    };
    let (verb, target, kind) = match tool {
        "Bash" | "PowerShell" => (
            "Running",
            truncate(&s("command").replace('\n', " "), 80),
            "run",
        ),
        "Read" => ("Reading", file_name(&s("file_path")), "read"),
        "Edit" | "MultiEdit" => ("Editing", file_name(&s("file_path")), "edit"),
        "Write" => ("Writing", file_name(&s("file_path")), "edit"),
        "NotebookEdit" => ("Editing", file_name(&s("notebook_path")), "edit"),
        "Grep" => ("Searching", truncate(&s("pattern"), 60), "search"),
        "Glob" => ("Finding", truncate(&s("pattern"), 60), "search"),
        "LS" => ("Listing", file_name(&s("path")), "read"),
        "WebFetch" => (
            "Fetching",
            s("url").split('/').nth(2).unwrap_or("a page").to_string(),
            "web",
        ),
        "WebSearch" => ("Searching the web", truncate(&s("query"), 60), "web"),
        "Task" | "Agent" => ("Delegating", truncate(&s("description"), 60), "agent"),
        "TodoWrite" | "TaskCreate" | "TaskUpdate" => ("Planning", String::from("tasks"), "plan"),
        "ExitPlanMode" => ("Proposing", String::from("a plan"), "plan"),
        other if other.starts_with("mcp__") => {
            let mut parts = other.trim_start_matches("mcp__").splitn(2, "__");
            let server = parts.next().unwrap_or("").to_string();
            let name = parts.next().unwrap_or("").replace('_', " ");
            return Activity {
                verb: "Using".into(),
                target: format!("{server} · {name}"),
                tool: other.into(),
                kind: "tool",
            };
        }
        other => ("Using", other.to_string(), "tool"),
    };
    Activity {
        verb: verb.into(),
        target,
        tool: tool.into(),
        kind,
    }
}

fn apply(app: &AppHandle, agents: &Agents, name: &str, event: &Value) {
    let mut done: Option<Value> = None;
    let mut attention: Option<Value> = None;
    {
        let mut sessions = agents.sessions.lock().expect("sessions");
        let session = session_entry(&mut sessions, event);
        match name {
            "SessionStart" => {
                session.status = "idle";
                push_log(
                    session,
                    "start",
                    format!("Session {}", text(event, "source")),
                );
            }
            "UserPromptSubmit" => {
                session.status = "working";
                session.prompt = truncate(&text(event, "prompt"), 240);
                session.turn_started_at = now_ms();
                session.tools = 0;
                session.edited.clear();
                session.activity = Some(Activity {
                    verb: "Thinking".into(),
                    target: String::new(),
                    tool: String::new(),
                    kind: "think",
                });
                push_log(session, "prompt", session.prompt.clone());
            }
            "PreToolUse" => {
                let tool = text(event, "tool_name");
                let activity = describe(&tool, event.get("tool_input").unwrap_or(&Value::Null));
                session.status = "working";
                push_log(
                    session,
                    activity.kind,
                    format!("{} {}", activity.verb, activity.target)
                        .trim()
                        .to_string(),
                );
                session.activity = Some(activity);
            }
            "PostToolUse" | "PostToolUseFailure" => {
                session.tools += 1;
                let tool = text(event, "tool_name");
                let input = event.get("tool_input").unwrap_or(&Value::Null);
                if matches!(
                    tool.as_str(),
                    "Edit" | "MultiEdit" | "Write" | "NotebookEdit"
                ) {
                    let path = input
                        .get("file_path")
                        .or_else(|| input.get("notebook_path"))
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let file = file_name(path);
                    if !file.is_empty() && !session.edited.contains(&file) {
                        session.edited.push(file);
                    }
                }
                if name == "PostToolUseFailure" {
                    let error = truncate(&text(event, "error"), 160);
                    push_log(
                        session,
                        "error",
                        if error.is_empty() {
                            format!("{tool} failed")
                        } else {
                            error
                        },
                    );
                }
                if session.status != "waiting" {
                    session.status = "working";
                }
                session.activity = Some(Activity {
                    verb: "Thinking".into(),
                    target: String::new(),
                    tool: String::new(),
                    kind: "think",
                });
            }
            "Notification" => {
                let kind = text(event, "notification_type");
                let message = truncate(&text(event, "message"), 200);
                let ours_pending = agents
                    .pending
                    .lock()
                    .map(|p| p.values().any(|x| x.info.session_id == session.id))
                    .unwrap_or(false);
                if kind == "idle_prompt" {
                    session.status = "done";
                } else if !ours_pending {
                    session.status = "waiting";
                    attention = Some(
                        json!({ "sessionId": session.id, "project": session.project, "message": message }),
                    );
                }
                push_log(session, "note", message);
            }
            "Stop" => {
                session.status = "done";
                session.activity = None;
                session.message = truncate(&text(event, "last_assistant_message"), 900);
                done = Some(json!({
                    "sessionId": session.id,
                    "project": session.project,
                    "message": session.message,
                    "durationMs": now_ms() - session.turn_started_at,
                    "tools": session.tools,
                    "edited": session.edited,
                }));
                push_log(session, "done", "Finished".into());
            }
            "StopFailure" => {
                session.status = "error";
                session.activity = None;
                push_log(session, "error", truncate(&text(event, "error"), 160));
            }
            "SubagentStart" => {
                session.subagents += 1;
                push_log(
                    session,
                    "agent",
                    format!("Subagent {}", text(event, "agent_type")),
                );
            }
            "SubagentStop" => {
                session.subagents = session.subagents.saturating_sub(1);
            }
            "SessionEnd" => {
                session.status = "ended";
                session.activity = None;
            }
            _ => {}
        }
    }
    let _ = app.emit("agents://state", agents.snapshot());
    if let Some(done) = done {
        let _ = app.emit("agents://done", done);
    }
    if let Some(attention) = attention {
        let _ = app.emit("agents://attention", attention);
    }
}

fn wait_for_decision(
    app: &AppHandle,
    agents: &Arc<Agents>,
    event: &Value,
    timeout_s: u64,
) -> Option<bool> {
    let tool = text(event, "tool_name");
    let input = event.get("tool_input").cloned().unwrap_or(Value::Null);
    let activity = describe(&tool, &input);
    let detail = match tool.as_str() {
        "Bash" | "PowerShell" => truncate(
            input.get("command").and_then(Value::as_str).unwrap_or(""),
            400,
        ),
        _ => input
            .get("file_path")
            .or_else(|| input.get("url"))
            .or_else(|| input.get("pattern"))
            .and_then(Value::as_str)
            .map(|v| truncate(v, 300))
            .unwrap_or_else(|| truncate(&input.to_string(), 300)),
    };
    let (tx, rx) = mpsc::channel();
    let id = random_token()[..12].to_string();
    let project;
    {
        let mut sessions = agents.sessions.lock().expect("sessions");
        let session = session_entry(&mut sessions, event);
        session.status = "waiting";
        push_log(
            session,
            "ask",
            format!(
                "Asks to {} {}",
                activity.verb.to_lowercase(),
                activity.target
            ),
        );
        project = session.project.clone();
    }
    let info = Approval {
        id: id.clone(),
        session_id: text(event, "session_id"),
        project,
        tool: tool.clone(),
        title: format!("{} {}", activity.verb, activity.target)
            .trim()
            .to_string(),
        detail,
        created_at: now_ms(),
        expires_at: now_ms() + timeout_s as i64 * 1000,
    };
    agents.pending.lock().expect("pending").insert(
        id.clone(),
        Pending {
            info: info.clone(),
            tx,
        },
    );
    let _ = app.emit("agents://approval", &info);
    let _ = app.emit("agents://state", agents.snapshot());

    let answer = rx.recv_timeout(Duration::from_secs(timeout_s)).ok();
    agents.pending.lock().expect("pending").remove(&id);
    {
        let mut sessions = agents.sessions.lock().expect("sessions");
        let session = session_entry(&mut sessions, event);
        session.status = "working";
        let note = match answer {
            Some(true) => "Allowed from the island",
            Some(false) => "Denied from the island",
            None => "Left to the terminal",
        };
        push_log(session, "ask", note.into());
    }
    let _ = app.emit("agents://state", agents.snapshot());
    answer
}

#[tauri::command]
pub fn agents_state(agents: State<'_, Arc<Agents>>) -> Value {
    serde_json::to_value(agents.snapshot()).unwrap_or(Value::Null)
}

#[tauri::command]
pub fn agents_decide(agents: State<'_, Arc<Agents>>, id: String, allow: bool) -> bool {
    agents
        .pending
        .lock()
        .expect("pending")
        .get(&id)
        .map(|p| p.tx.send(allow).is_ok())
        .unwrap_or(false)
}

#[tauri::command]
pub fn agents_clear(app: AppHandle, agents: State<'_, Arc<Agents>>, id: Option<String>) {
    {
        let mut sessions = agents.sessions.lock().expect("sessions");
        match id {
            Some(id) => {
                sessions.remove(&id);
            }
            None => sessions.retain(|_, s| matches!(s.status, "working" | "waiting")),
        }
    }
    let _ = app.emit("agents://state", agents.snapshot());
}

// ---------------------------------------------------------------- hook installation

fn claude_settings_path() -> PathBuf {
    std::env::var_os("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| util::home_dir().join(".claude"))
        .join("settings.json")
}

fn is_ours(handler: &Value) -> bool {
    ["url", "command"].iter().any(|k| {
        handler
            .get(*k)
            .and_then(Value::as_str)
            .is_some_and(|v| v.contains(MARK))
    })
}

fn strip_ours(hooks: &mut serde_json::Map<String, Value>) {
    for event in EVENTS {
        if let Some(groups) = hooks.get_mut(event).and_then(Value::as_array_mut) {
            for group in groups.iter_mut() {
                if let Some(list) = group.get_mut("hooks").and_then(Value::as_array_mut) {
                    list.retain(|h| !is_ours(h));
                }
            }
            groups.retain(|g| {
                g.get("hooks")
                    .and_then(Value::as_array)
                    .is_some_and(|l| !l.is_empty())
            });
        }
        if hooks
            .get(event)
            .and_then(Value::as_array)
            .is_some_and(|g| g.is_empty())
        {
            hooks.remove(event);
        }
    }
}

fn read_claude_settings(path: &PathBuf) -> Result<Value, String> {
    match std::fs::read_to_string(path) {
        Ok(text) if text.trim().is_empty() => Ok(json!({})),
        Ok(text) => serde_json::from_str::<Value>(text.trim_start_matches('\u{feff}'))
            .ok()
            .filter(Value::is_object)
            .ok_or_else(|| {
                format!(
                    "{} is not valid JSON, so it was left unchanged.",
                    path.display()
                )
            }),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(json!({})),
        Err(err) => Err(err.to_string()),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HookStatus {
    installed: bool,
    events: usize,
    path: String,
    claude_found: bool,
    port: u16,
    mode: String,
}

#[tauri::command]
pub fn hooks_status(config: State<'_, Config>) -> HookStatus {
    let path = claude_settings_path();
    let settings = read_claude_settings(&path).unwrap_or_else(|_| json!({}));
    let mut events = 0;
    let mut mode = String::new();
    if let Some(hooks) = settings.get("hooks").and_then(Value::as_object) {
        for event in EVENTS {
            let ours: Vec<&Value> = hooks
                .get(event)
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|g| g.get("hooks").and_then(Value::as_array))
                .flatten()
                .filter(|h| is_ours(h))
                .collect();
            if let Some(first) = ours.first() {
                events += 1;
                mode = first
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
            }
        }
    }
    HookStatus {
        installed: events == EVENTS.len(),
        events,
        path: path.display().to_string(),
        claude_found: path.parent().is_some_and(|p| p.exists()),
        port: config.u64("hookPort", DEFAULT_PORT as u64) as u16,
        mode,
    }
}

/// Adds the island's hooks to Claude Code's user settings, keeping everything else as it was.
/// `mode` is "http" (Claude Code's HTTP hooks) or "curl" (a command hook, for older versions).
#[tauri::command]
pub fn hooks_install(config: State<'_, Config>, mode: String) -> Result<HookStatus, String> {
    let path = claude_settings_path();
    let mut settings = read_claude_settings(&path)?;
    if path.exists() {
        let backup = path.with_extension("json.tensei-backup");
        if !backup.exists() {
            let _ = std::fs::copy(&path, &backup);
        }
    }
    let port = config.u64("hookPort", DEFAULT_PORT as u64);
    let secret = token(&config);
    let approval = config.u64("approvalTimeout", 60).clamp(10, 590);
    let url = format!("http://127.0.0.1:{port}/hook?{MARK}");
    let root = settings
        .as_object_mut()
        .ok_or("settings.json is not an object")?;
    let hooks = root.entry("hooks").or_insert_with(|| json!({}));
    if !hooks.is_object() {
        *hooks = json!({});
    }
    let hooks = hooks.as_object_mut().expect("hooks object");
    strip_ours(hooks);
    for event in EVENTS {
        let timeout = if event == "PermissionRequest" {
            approval + 10
        } else {
            5
        };
        let handler = if mode == "curl" {
            json!({
                "type": "command",
                "command": format!(
                    "curl -s -m {timeout} -X POST -H \"Content-Type: application/json\" -H \"X-Island-Token: {secret}\" --data-binary @- \"{url}\""
                ),
                "timeout": timeout,
            })
        } else {
            json!({
                "type": "http",
                "url": url,
                "timeout": timeout,
                "headers": { "X-Island-Token": secret },
            })
        };
        let groups = hooks.entry(event).or_insert_with(|| json!([]));
        if !groups.is_array() {
            *groups = json!([]);
        }
        groups
            .as_array_mut()
            .expect("array")
            .push(json!({ "hooks": [handler] }));
    }
    util::write_json(&path, &settings).map_err(|e| e.to_string())?;
    Ok(hooks_status(config))
}

#[tauri::command]
pub fn hooks_uninstall(config: State<'_, Config>) -> Result<HookStatus, String> {
    let path = claude_settings_path();
    if !path.exists() {
        return Ok(hooks_status(config));
    }
    let mut settings = read_claude_settings(&path)?;
    if let Some(root) = settings.as_object_mut() {
        if let Some(hooks) = root.get_mut("hooks").and_then(Value::as_object_mut) {
            strip_ours(hooks);
            if hooks.is_empty() {
                root.remove("hooks");
            }
        }
    }
    util::write_json(&path, &settings).map_err(|e| e.to_string())?;
    Ok(hooks_status(config))
}
