//! Plan usage rings: Claude (5-hour and weekly limits) and Codex, read with the sign-ins that
//! Claude Code and the Codex CLI already keep on this PC. Tokens are only held in memory for the
//! request and never written anywhere. Approach adapted from BrimDeck (Apache-2.0).

use std::{
    path::PathBuf,
    sync::{
        mpsc::{self, Sender},
        Mutex, OnceLock,
    },
    thread,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};

use crate::{config::Config, util};

static TRIGGER: OnceLock<Mutex<Sender<()>>> = OnceLock::new();
/// What was last sent to the windows, for a window that opens later (settings).
static LAST: Mutex<Option<Value>> = Mutex::new(None);

fn publish<T: Serialize>(app: &AppHandle, providers: &T) {
    if let Ok(value) = serde_json::to_value(providers) {
        *LAST.lock().unwrap_or_else(|e| e.into_inner()) = Some(value);
    }
    let _ = app.emit("quotas://update", providers);
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct QuotaItem {
    label: String,
    percent: f64,
    resets_at: Option<i64>,
    window_minutes: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    provider: String,
    ok: bool,
    plan: String,
    error: String,
    items: Vec<QuotaItem>,
    fetched_at: i64,
    /// The service answered 429: wait longer before asking again.
    #[serde(skip)]
    rate_limited: bool,
    /// Seconds the service asked us to wait (Retry-After), if it said.
    #[serde(skip)]
    retry_after: Option<u64>,
}

/// Keeps the last good numbers on screen when a refresh fails (offline, rate limited), with the
/// error alongside so the Agents page can say why they are not fresh.
fn keep_last(fresh: Provider, last: &mut Option<Provider>) -> Provider {
    if fresh.ok {
        *last = Some(fresh.clone());
        return fresh;
    }
    match last {
        Some(good) => Provider {
            error: fresh.error,
            rate_limited: fresh.rate_limited,
            retry_after: fresh.retry_after,
            ..good.clone()
        },
        None => fresh,
    }
}

/// Days since 1970-01-01 for a civil date (Howard Hinnant's algorithm).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// RFC 3339 timestamp ("2026-11-06T12:30:00.123+05:30") to Unix milliseconds.
pub fn parse_time(text: &str) -> Option<i64> {
    let b = text.as_bytes();
    if b.len() < 19 {
        return None;
    }
    let num = |s: &str| s.parse::<i64>().ok();
    let (y, mo, d) = (num(&text[0..4])?, num(&text[5..7])?, num(&text[8..10])?);
    let (h, mi, s) = (
        num(&text[11..13])?,
        num(&text[14..16])?,
        num(&text[17..19])?,
    );
    let mut rest = &text[19..];
    let mut millis = 0i64;
    if let Some(frac) = rest.strip_prefix('.') {
        let digits: String = frac.chars().take_while(|c| c.is_ascii_digit()).collect();
        let padded = format!("{:0<3}", &digits[..digits.len().min(3)]);
        millis = padded.parse().unwrap_or(0);
        rest = &frac[digits.len()..];
    }
    let offset_minutes = match rest.chars().next() {
        Some('+') | Some('-') if rest.len() >= 6 => {
            let sign = if rest.starts_with('-') { -1 } else { 1 };
            sign * (num(&rest[1..3])? * 60 + num(&rest[4..6])?)
        }
        _ => 0,
    };
    let seconds = days_from_civil(y, mo, d) * 86_400 + h * 3600 + mi * 60 + s - offset_minutes * 60;
    Some(seconds * 1000 + millis)
}

fn time_value(value: Option<&Value>) -> Option<i64> {
    match value? {
        Value::String(text) => parse_time(text),
        Value::Number(n) => n.as_f64().map(|v| {
            if v < 1e12 {
                (v * 1000.0) as i64
            } else {
                v as i64
            }
        }),
        _ => None,
    }
}

fn claude_home() -> PathBuf {
    std::env::var_os("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| util::home_dir().join(".claude"))
}

fn codex_home() -> PathBuf {
    std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| util::home_dir().join(".codex"))
}

fn read_claude(agent: &ureq::Agent) -> Provider {
    let mut out = Provider {
        provider: "claude".into(),
        fetched_at: util::now_ms(),
        ..Default::default()
    };
    let auth = util::read_json(&claude_home().join(".credentials.json"));
    let oauth = auth.get("claudeAiOauth").cloned().unwrap_or(Value::Null);
    let token = oauth
        .get("accessToken")
        .and_then(Value::as_str)
        .unwrap_or("");
    out.plan = oauth
        .get("subscriptionType")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    if token.is_empty() {
        out.error = "Sign in to Claude Code on this PC".into();
        return out;
    }
    let response = agent
        .get("https://api.anthropic.com/api/oauth/usage")
        .header("Authorization", &format!("Bearer {token}"))
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("Accept", "application/json")
        .call();
    let mut response = match response {
        Ok(r) => r,
        Err(err) => {
            out.error = format!("Offline ({err})");
            return out;
        }
    };
    match response.status().as_u16() {
        200 => {}
        401 | 403 => {
            out.error = "Sign-in expired: run Claude Code once".into();
            return out;
        }
        429 => {
            out.error = "Usage service is busy, trying again later".into();
            out.rate_limited = true;
            out.retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.trim().parse().ok());
            return out;
        }
        code => {
            out.error = format!("HTTP {code}");
            return out;
        }
    }
    let Ok(root) = response.body_mut().read_json::<Value>() else {
        out.error = "Unexpected reply".into();
        return out;
    };
    claude_items(&root, &mut out);
    out
}

/// Fills `out` from a usage reply: the service's, or the copy Claude Code keeps.
fn claude_items(root: &Value, out: &mut Provider) {
    let mut add = |label: &str, percent: Option<f64>, reset: Option<i64>, window: Option<u32>| {
        if let Some(p) = percent {
            if p >= 0.0 && !out.items.iter().any(|i| i.label == label) {
                out.items.push(QuotaItem {
                    label: label.into(),
                    percent: p.min(100.0),
                    resets_at: reset,
                    window_minutes: window,
                });
            }
        }
    };
    for (key, label, window) in [
        ("five_hour", "5-hour", 300),
        ("seven_day", "Weekly", 10_080),
    ] {
        let w = root.get(key);
        add(
            label,
            w.and_then(|w| w.get("utilization")).and_then(Value::as_f64),
            time_value(w.and_then(|w| w.get("resets_at"))),
            Some(window),
        );
    }
    if let Some(limits) = root.get("limits").and_then(Value::as_array) {
        for limit in limits {
            let kind = limit.get("kind").and_then(Value::as_str).unwrap_or("");
            let percent = limit.get("percent").and_then(Value::as_f64);
            let reset = time_value(limit.get("resets_at"));
            let scope = limit.get("scope");
            let model = scope
                .and_then(|s| s.get("model"))
                .and_then(|m| m.get("display_name"))
                .and_then(Value::as_str)
                .unwrap_or("");
            match kind {
                "session" => add("5-hour", percent, reset, Some(300)),
                "weekly_all" => add("Weekly", percent, reset, Some(10_080)),
                _ if !model.is_empty() => {
                    add(&format!("{model} weekly"), percent, reset, Some(10_080))
                }
                _ => {}
            }
        }
    }
    for (key, name) in [
        ("seven_day_opus", "Opus weekly"),
        ("seven_day_sonnet", "Sonnet weekly"),
    ] {
        let w = root.get(key);
        add(
            name,
            w.and_then(|w| w.get("utilization")).and_then(Value::as_f64),
            time_value(w.and_then(|w| w.get("resets_at"))),
            Some(10_080),
        );
    }
    out.ok = !out.items.is_empty();
    if !out.ok {
        out.error = "No limits reported".into();
    }
}

fn claude_plan() -> String {
    util::read_json(&claude_home().join(".credentials.json"))
        .get("claudeAiOauth")
        .and_then(|o| o.get("subscriptionType"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

/// Claude Code keeps its last usage reply in its own config (`cachedUsageUtilization`) and
/// refreshes it while it runs. Reading that costs nothing and asks no service, so the island reads
/// it every check and needs the (rate limited, shared with Claude Code) service only when it is old.
fn claude_local() -> Option<Provider> {
    let path = std::env::var_os("CLAUDE_CONFIG_DIR")
        .map(|d| PathBuf::from(d).join(".claude.json"))
        .unwrap_or_else(|| util::home_dir().join(".claude.json"));
    let config = util::read_json(&path);
    let cached = config.get("cachedUsageUtilization")?;
    let mut out = Provider {
        provider: "claude".into(),
        fetched_at: cached.get("fetchedAtMs").and_then(Value::as_i64)?,
        plan: claude_plan(),
        ..Default::default()
    };
    claude_items(cached.get("utilization")?, &mut out);
    out.ok.then_some(out)
}

/// Codex writes its current limits into each session log as it works: the newest one is a free,
/// local reading while Codex is in use.
fn codex_local() -> Option<Provider> {
    let file = newest_file(&codex_home().join("sessions"), 3)?;
    let text = std::fs::read_to_string(file).ok()?;
    let (o, limits) = text
        .lines()
        .rev()
        .filter(|l| l.contains("\"rate_limits\""))
        .find_map(|l| {
            let o: Value = serde_json::from_str(l).ok()?;
            let limits = o.get("payload")?.get("rate_limits")?.clone();
            limits.is_object().then_some((o, limits))
        })?;
    let mut out = Provider {
        provider: "codex".into(),
        fetched_at: o
            .get("timestamp")
            .and_then(Value::as_str)
            .and_then(parse_time)?,
        plan: limits
            .get("plan_type")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        ..Default::default()
    };
    for key in ["primary", "secondary"] {
        let w = &limits[key];
        let Some(used) = w.get("used_percent").and_then(Value::as_f64) else {
            continue;
        };
        let minutes = w
            .get("window_minutes")
            .and_then(Value::as_u64)
            .map(|m| m as u32);
        let reset = w.get("resets_at").and_then(Value::as_i64).map(|t| {
            if t < 100_000_000_000 {
                t * 1000
            } else {
                t
            }
        });
        out.items.push(QuotaItem {
            label: window_label(minutes, key == "primary"),
            percent: used.clamp(0.0, 100.0),
            resets_at: reset,
            window_minutes: minutes,
        });
    }
    out.ok = !out.items.is_empty();
    out.ok.then_some(out)
}

fn newest_file(dir: &std::path::Path, depth: u32) -> Option<PathBuf> {
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        let found = if meta.is_dir() {
            if depth == 0 {
                continue;
            }
            newest_file(&entry.path(), depth - 1)
                .and_then(|p| std::fs::metadata(&p).ok()?.modified().ok().map(|t| (t, p)))
        } else {
            meta.modified().ok().map(|t| (t, entry.path()))
        };
        if let Some((t, p)) = found {
            if best.as_ref().map_or(true, |(b, _)| t > *b) {
                best = Some((t, p));
            }
        }
    }
    best.map(|(_, p)| p)
}

/// 300 → "5-hour", 10080 → "Weekly", 43200 → "Monthly".
fn window_label(minutes: Option<u32>, primary: bool) -> String {
    match minutes {
        Some(300) => "5-hour".into(),
        Some(1440) => "Daily".into(),
        Some(10_080) => "Weekly".into(),
        Some(43_200) | Some(44_640) => "Monthly".into(),
        Some(m) if m > 0 && m % 1440 == 0 => format!("{}-day", m / 1440),
        Some(m) if m > 0 && m % 60 == 0 => format!("{}-hour", m / 60),
        Some(m) if m > 0 => format!("{m}-minute"),
        _ if primary => "Primary".into(),
        _ => "Secondary".into(),
    }
}

fn read_codex(agent: &ureq::Agent) -> Provider {
    let mut out = Provider {
        provider: "codex".into(),
        fetched_at: util::now_ms(),
        ..Default::default()
    };
    let auth = util::read_json(&codex_home().join("auth.json"));
    let tokens = auth.get("tokens").cloned().unwrap_or(Value::Null);
    let token = tokens
        .get("access_token")
        .and_then(Value::as_str)
        .unwrap_or("");
    if token.is_empty() {
        out.error = "Sign in to Codex with ChatGPT".into();
        return out;
    }
    let mut request = agent
        .get("https://chatgpt.com/backend-api/wham/usage")
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/json");
    if let Some(account) = tokens.get("account_id").and_then(Value::as_str) {
        request = request.header("ChatGPT-Account-Id", account);
    }
    let mut response = match request.call() {
        Ok(r) => r,
        Err(err) => {
            out.error = format!("Offline ({err})");
            return out;
        }
    };
    if response.status().as_u16() != 200 {
        out.error = match response.status().as_u16() {
            401 | 403 => "Sign-in expired: run Codex once".into(),
            code => format!("HTTP {code}"),
        };
        return out;
    }
    let Ok(root) = response.body_mut().read_json::<Value>() else {
        out.error = "Unexpected reply".into();
        return out;
    };
    out.plan = root
        .get("plan_type")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let limits = root.get("rate_limit").cloned().unwrap_or(Value::Null);
    let now = util::now_ms();
    for field in ["primary_window", "secondary_window"] {
        let Some(window) = limits.get(field) else {
            continue;
        };
        let Some(used) = window.get("used_percent").and_then(Value::as_f64) else {
            continue;
        };
        let minutes = window
            .get("limit_window_seconds")
            .and_then(Value::as_f64)
            .map(|s| (s / 60.0) as u32);
        let reset = time_value(window.get("reset_at")).or_else(|| {
            window
                .get("reset_after_seconds")
                .and_then(Value::as_f64)
                .map(|s| now + (s * 1000.0) as i64)
        });
        let label = window_label(minutes, field.starts_with("primary"));
        out.items.push(QuotaItem {
            label,
            percent: used.clamp(0.0, 100.0),
            resets_at: reset,
            window_minutes: minutes,
        });
    }
    out.ok = !out.items.is_empty();
    out
}

/// Last good readings and when each service may be asked next, kept in data.json so a restart
/// shows the numbers at once and keeps to the backoff.
const CACHE_KEY: &str = "quotaCache";
/// A reading this recent needs no service call.
const FRESH_MS: i64 = 15 * 60_000;
/// Never ask a usage service more often than this.
const API_EVERY_MS: i64 = 5 * 60_000;

/// One provider's readings and how soon its usage service may be asked again.
#[derive(Default)]
struct Source {
    last: Option<Provider>,
    next_api: i64,
    backoff: u64,
}

impl Source {
    fn refresh(
        &mut self,
        local: Option<Provider>,
        api: impl FnOnce() -> Provider,
        force: bool,
    ) -> Option<Provider> {
        let now = util::now_ms();
        // The tool's own copy wins whenever it is newer than what we have.
        if let Some(p) = local {
            if self
                .last
                .as_ref()
                .map_or(true, |l| p.fetched_at > l.fetched_at)
            {
                self.last = Some(p);
            }
        }
        let fresh = self
            .last
            .as_ref()
            .is_some_and(|l| now - l.fetched_at < FRESH_MS);
        if fresh && !force {
            return self.last.clone();
        }
        // "Refresh now" still keeps to a backoff the service asked for.
        if now >= self.next_api || (force && self.backoff <= 1) {
            let reply = api();
            if reply.rate_limited {
                self.backoff = (self.backoff.max(1) * 2).min(12);
                let wait = (API_EVERY_MS / 1000 * self.backoff as i64)
                    .max(reply.retry_after.unwrap_or(0) as i64)
                    .min(3600);
                self.next_api = now + wait * 1000;
            } else {
                self.backoff = 1;
                self.next_api = now + API_EVERY_MS;
            }
            return Some(keep_last(reply, &mut self.last));
        }
        // Between calls: the last reading, marked once it is getting old.
        self.last.clone().map(|p| {
            if now - p.fetched_at > FRESH_MS && p.error.is_empty() {
                Provider {
                    error: "Waiting for the next check".into(),
                    ..p
                }
            } else {
                p
            }
        })
    }
}

pub fn start(app: AppHandle) {
    let (tx, rx) = mpsc::channel::<()>();
    let _ = TRIGGER.set(Mutex::new(tx));
    thread::Builder::new()
        .name("quotas".into())
        .spawn(move || {
            // Give the island's frontend time to start listening.
            thread::sleep(Duration::from_secs(4));
            let agent = crate::net::agent();
            let interval =
                |app: &AppHandle| app.state::<Config>().u64("quotaMinutes", 1).clamp(1, 60) * 60;

            let cached = app.state::<Config>().data_get(CACHE_KEY);
            let saved: Vec<Provider> = cached
                .get("providers")
                .and_then(|p| serde_json::from_value(p.clone()).ok())
                .unwrap_or_default();
            let source = |name: &str| Source {
                last: saved
                    .iter()
                    .find(|p| p.provider == name && p.ok)
                    .cloned()
                    .map(|mut p| {
                        // Readings saved by older versions carry labels like "43200-minute".
                        if p.provider == "codex" {
                            for (i, item) in p.items.iter_mut().enumerate() {
                                item.label = window_label(item.window_minutes, i == 0);
                            }
                        }
                        p
                    }),
                next_api: cached["nextApi"][name]
                    .as_i64()
                    .or_else(|| cached.get("nextAt").and_then(Value::as_i64))
                    .unwrap_or(0),
                backoff: cached["backoff"][name].as_u64().unwrap_or(1).clamp(1, 12),
            };
            let (mut claude, mut codex) = (source("claude"), source("codex"));
            let (mut sent, mut stored) = (String::new(), String::new());
            let mut force = false;

            loop {
                let config = app.state::<Config>();
                let mut providers = Vec::new();
                // Only tools on this PC: a missing sign-in folder means the tool is not installed.
                if config.bool("quotaClaude", true) && claude_home().exists() {
                    providers.extend(claude.refresh(claude_local(), || read_claude(agent), force));
                }
                if config.bool("quotaCodex", true) && codex_home().join("auth.json").exists() {
                    providers.extend(codex.refresh(codex_local(), || read_codex(agent), force));
                }
                // Windows hear only about changes.
                let json = serde_json::to_string(&providers).unwrap_or_default();
                if json != sent {
                    publish(&app, &providers);
                    sent = json;
                }
                let good: Vec<&Provider> =
                    [&claude.last, &codex.last].into_iter().flatten().collect();
                let cache = serde_json::json!({
                    "providers": good,
                    "nextApi": { "claude": claude.next_api, "codex": codex.next_api },
                    "backoff": { "claude": claude.backoff, "codex": codex.backoff },
                });
                let text = cache.to_string();
                if text != stored {
                    config.data_set(CACHE_KEY, cache);
                    stored = text;
                }
                // A refresh request ("Refresh now", a usage toggle) wakes this early and asks now.
                force = rx.recv_timeout(Duration::from_secs(interval(&app))).is_ok();
                while rx.try_recv().is_ok() {}
            }
        })
        .expect("quota thread");
}

#[tauri::command]
pub fn quotas_get() -> Value {
    LAST.lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .unwrap_or(Value::Array(Vec::new()))
}

#[tauri::command]
pub fn quotas_refresh() {
    if let Some(tx) = TRIGGER.get() {
        if let Ok(tx) = tx.lock() {
            let _ = tx.send(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_time;

    #[test]
    fn parses_rfc3339() {
        assert_eq!(parse_time("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(
            parse_time("2026-11-06T00:00:00.500Z"),
            Some(1_793_923_200_500)
        );
        assert_eq!(
            parse_time("2026-11-06T05:30:00+05:30"),
            Some(1_793_923_200_000)
        );
    }
}
