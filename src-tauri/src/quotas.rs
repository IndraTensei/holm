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
    out
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
        let label = match minutes {
            Some(300) => "5-hour".to_string(),
            Some(10_080) => "Weekly".to_string(),
            Some(m) if m > 0 => format!("{m}-minute"),
            _ if field.starts_with("primary") => "Primary".to_string(),
            _ => "Secondary".to_string(),
        };
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

/// Last good readings and when the next check is due, kept in data.json so a restart shows the
/// numbers at once and keeps to the refresh interval and any backoff, even after a failed check.
const CACHE_KEY: &str = "quotaCache";

fn wants(config: &Config, provider: &str) -> bool {
    match provider {
        "claude" => config.bool("quotaClaude", true),
        "codex" => config.bool("quotaCodex", true),
        _ => false,
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
                |app: &AppHandle| app.state::<Config>().u64("quotaMinutes", 5).clamp(2, 60) * 60;

            let cached = app.state::<Config>().data_get(CACHE_KEY);
            let saved: Vec<Provider> = cached
                .get("providers")
                .and_then(|p| serde_json::from_value(p.clone()).ok())
                .unwrap_or_default();
            let find = |name: &str| saved.iter().find(|p| p.provider == name && p.ok).cloned();
            let (mut last_claude, mut last_codex) = (find("claude"), find("codex"));
            let mut backoff = cached
                .get("backoff")
                .and_then(Value::as_u64)
                .unwrap_or(1)
                .clamp(1, 12);
            // Older caches only kept the time of the last good reading.
            let next_at = cached
                .get("nextAt")
                .and_then(Value::as_i64)
                .unwrap_or_else(|| {
                    cached.get("at").and_then(Value::as_i64).unwrap_or(0)
                        + interval(&app) as i64 * 1000
                });

            {
                let config = app.state::<Config>();
                // Readings older than one interval are shown dimmed with their age until the next check.
                let old = util::now_ms() - interval(&app) as i64 * 1000;
                let shown: Vec<Provider> = [&last_claude, &last_codex]
                    .into_iter()
                    .flatten()
                    .filter(|p| wants(&config, &p.provider))
                    .map(|p| Provider {
                        error: if p.fetched_at < old {
                            "Waiting for the next check".into()
                        } else {
                            String::new()
                        },
                        ..p.clone()
                    })
                    .collect();
                if !shown.is_empty() {
                    publish(&app, &shown);
                }
            }
            let wait = (next_at - util::now_ms()).max(0) as u64 / 1000;
            if wait > 0 {
                let _ = rx.recv_timeout(Duration::from_secs(wait.min(3600)));
                while rx.try_recv().is_ok() {}
            }

            loop {
                let config = app.state::<Config>();
                let mut providers = Vec::new();
                // Only tools on this PC: a missing sign-in folder means the tool is not installed.
                if config.bool("quotaClaude", true) && claude_home().exists() {
                    providers.push(keep_last(read_claude(agent), &mut last_claude));
                }
                if config.bool("quotaCodex", true) && codex_home().join("auth.json").exists() {
                    providers.push(keep_last(read_codex(agent), &mut last_codex));
                }
                if !providers.is_empty() {
                    publish(&app, &providers);
                }
                // 429: double the wait each time (up to an hour), back to normal after a success,
                // and never sooner than the service asked for.
                let limited = providers.iter().any(|p| p.rate_limited);
                backoff = if limited { (backoff * 2).min(12) } else { 1 };
                let asked = providers
                    .iter()
                    .filter_map(|p| p.retry_after)
                    .max()
                    .unwrap_or(0);
                let wait = (interval(&app) * backoff).max(asked).min(3600);
                let good: Vec<&Provider> =
                    [&last_claude, &last_codex].into_iter().flatten().collect();
                config.data_set(
                    CACHE_KEY,
                    serde_json::json!({
                        "providers": good,
                        "nextAt": util::now_ms() + wait as i64 * 1000,
                        "backoff": backoff,
                    }),
                );
                let _ = rx.recv_timeout(Duration::from_secs(wait));
                // Collapse a burst of refresh requests into one round.
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
