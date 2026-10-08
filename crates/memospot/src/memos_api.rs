use crate::fl;
use crate::memos_version::MemosVersionStore;
use anyhow::{Result, anyhow};
use config::Config;
use dialog::error_dialog;
use log::{info, warn};
use tauri_plugin_http::reqwest;

/// Memos URL.
///
/// It's ensured to end with a slash.
///
/// If remote server is enabled, return the configured URL.
/// Otherwise, return the default Memos address for the spawned server,
/// using the effective port resolved at startup.
pub fn get_url(config: &Config, effective_port: u16) -> String {
    let remote = &config.memospot.remote;
    let url = remote.url.as_deref().unwrap_or_default();

    if remote.enabled != Some(true) || url.is_empty() {
        return format!("http://localhost:{}/", effective_port);
    }

    if !url.starts_with("http") {
        error_dialog!(fl!("error-invalid-server-url", url = url));
    }

    format!("{}/", url.trim_end_matches('/'))
}

/// Query Memos version via API.
///
/// Supports:
///     - v0.23.0+ (/api/v1/workspace)
///     - v0.26.0+ (/api/v1/instance)
pub async fn query_version(memos_url: &str) -> Result<String, anyhow::Error> {
    const TIMEOUT_MS: u64 = 1_000;
    const ENDPOINTS: [&str; 2] = ["api/v1/instance/profile", "api/v1/workspace/profile"];

    let mut last_error = anyhow!("failed to query server version via API");

    for endpoint in ENDPOINTS {
        let endpoint = format!("{memos_url}{endpoint}");
        let url = match reqwest::Url::parse(&endpoint) {
            Ok(url) => url,
            Err(e) => {
                return Err(anyhow!("failed to parse server URL: {}", e));
            }
        };
        let client = reqwest::Client::new();
        let request = client
            .get(url)
            .header("User-Agent", "Memospot")
            .timeout(std::time::Duration::from_millis(TIMEOUT_MS))
            .send()
            .await;

        match request {
            Ok(response) => {
                if !response.status().is_success() {
                    let code = response.status();
                    last_error = anyhow!("server responded with status code {code}");
                    continue;
                }
                let json = response
                    .json::<serde_json::Value>()
                    .await
                    .unwrap_or_default();
                let version = json
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                return Ok(version.to_string());
            }
            Err(e) => {
                last_error = e.into();
                continue;
            }
        }
    }
    Err(last_error)
}

/// Poll Memos server until the API responds.
///
/// Server version is queried and stored in the global state, available via
/// [`crate::memos_version::MemosVersionStore::get()`].
pub async fn wait_api_ready(memos_url: &str) {
    const INTERVAL_MS: u64 = 100;
    const TIMEOUT_MS: u128 = 15_000;

    let mut version = String::new();
    let mut last_error = String::new();
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(INTERVAL_MS));

    let time_start = tokio::time::Instant::now();
    loop {
        if time_start.elapsed().as_millis() > TIMEOUT_MS {
            break;
        }
        interval.tick().await;
        match query_version(memos_url).await {
            Ok(v) => {
                version = v;
                break;
            }
            Err(e) => last_error = e.to_string(),
        }
    }

    if version.is_empty() {
        warn!(
            "failed to query server version via API: {last_error}. Giving up after {TIMEOUT_MS} ms."
        );
        return;
    }
    info!(
        "API ready in <{} ms. Version: {}.",
        time_start.elapsed().as_millis(),
        version
    );
    MemosVersionStore::set(version);
}

/// Ping the Memos API to check if it is ready.
pub async fn ping_api(
    memos_url: &str,
    timeout_millis: u64,
    user_agent: &str,
) -> Result<bool, String> {
    let url = memos_url.trim_end_matches('/');
    let endpoint = format!("{url}/healthz");

    let url = reqwest::Url::parse(&endpoint).unwrap();
    let client = reqwest::Client::new();
    if let Ok(response) = client
        .get(url)
        .header("User-Agent", user_agent)
        .timeout(std::time::Duration::from_millis(if timeout_millis < 100 {
            1000
        } else {
            timeout_millis
        }))
        .send()
        .await
        && response.status().is_success()
        && let Ok(body) = response.text().await
        && body.starts_with("Service ready.")
    {
        return Ok(true);
    }

    Ok(false)
}
