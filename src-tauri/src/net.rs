//! Outbound HTTP for the frontend's public data (lyrics, weather), so it does not depend on
//! those services sending CORS headers. Only the hosts listed here can be reached.

use std::{sync::OnceLock, time::Duration};

use serde::Serialize;
use serde_json::Value;

const ALLOWED: [&str; 3] = [
    "lrclib.net",
    "api.open-meteo.com",
    "geocoding-api.open-meteo.com",
];

pub fn agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        let tls = ureq::tls::TlsConfig::builder()
            .provider(ureq::tls::TlsProvider::NativeTls)
            .root_certs(ureq::tls::RootCerts::PlatformVerifier)
            .build();
        ureq::Agent::config_builder()
            .tls_config(tls)
            .timeout_global(Some(Duration::from_secs(15)))
            .http_status_as_error(false)
            .user_agent("TenseiIsland/0.1 (https://github.com/IndraTensei)")
            .build()
            .into()
    })
}

#[derive(Serialize)]
pub struct JsonReply {
    status: u16,
    body: Value,
}

fn host_of(url: &str) -> Option<&str> {
    let rest = url.strip_prefix("https://")?;
    rest.split(['/', '?', '#']).next()
}

#[tauri::command]
pub async fn fetch_json(url: String) -> Result<JsonReply, String> {
    let host = host_of(&url).ok_or("Only https links are allowed")?;
    if !ALLOWED.contains(&host) {
        return Err(format!("{host} is not on the allowed list"));
    }
    tauri::async_runtime::spawn_blocking(move || {
        let mut response = agent()
            .get(&url)
            .header("Accept", "application/json")
            .call()
            .map_err(|e| e.to_string())?;
        let status = response.status().as_u16();
        let body = response
            .body_mut()
            .read_json::<Value>()
            .unwrap_or(Value::Null);
        Ok(JsonReply { status, body })
    })
    .await
    .map_err(|e| e.to_string())?
}
