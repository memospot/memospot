//! This module contains the Tauri command handlers for the JS API.
//!
//! Command handlers and state management must be registered
//! in `tauri::Builder`, at `lib.rs`.
//!
//! The TypeScript/JavaScript API is defined in `src-ui/src/lib/tauri.ts`.

use config::Config;
use dialog::error_dialog;
use json_patch::Patch;
use log::{debug, error, info};
use serde_json::json;
use tauri::{command, State};
use tauri_plugin_http::reqwest;
use tokio::sync::Mutex;

use crate::runtime_config::RuntimeConfig;

pub struct MemosURL(pub Mutex<String>);
impl MemosURL {
    pub fn manage(url: String) -> Self {
        Self(Mutex::new(url))
    }
}

#[command]
pub async fn get_memos_url(memos_url: State<'_, MemosURL>) -> Result<String, String> {
    Ok(memos_url.0.lock().await.clone())
}

#[command]
pub async fn get_theme() -> Result<String, String> {
    let config = RuntimeConfig::from_global_store();
    Ok(config.yaml.memospot.window.theme.unwrap_or_default())
}

#[command]
pub async fn get_reduce_animation_status() -> Result<bool, String> {
    let config = RuntimeConfig::from_global_store();
    Ok(config
        .yaml
        .memospot
        .window
        .reduce_animation
        .unwrap_or_default())
}

pub struct Locale(pub Mutex<String>);
impl Locale {
    pub fn manage(locale: String) -> Self {
        Self(Mutex::new(locale))
    }
}
#[command]
pub async fn get_locale(locale: State<'_, Locale>) -> Result<String, String> {
    Ok(locale.0.lock().await.clone())
}

#[command]
pub async fn set_locale(new: String, locale: State<'_, Locale>) -> Result<bool, String> {
    debug!("Setting locale to {}", new);
    *locale.0.lock().await = new.clone();

    let mut config = RuntimeConfig::from_global_store();
    config.yaml.memospot.window.locale = Some(new.clone());
    RuntimeConfig::to_global_store(&config);

    info!("Configuration updated by user. Saving…");

    let config_path = config.paths.memospot_config_file.clone();
    if let Err(e) = config.yaml.save_to_file(&config_path).await {
        error_dialog!(
            "Failed to save configuration file:\n`{}`\n\n{}",
            &config_path.display(),
            e
        );
    }

    let current_locale = locale.0.lock().await.clone();
    debug!("Current locale is now {}", current_locale);

    Ok(true)
}

#[command]
pub async fn ping_memos(memos_url: &str, timeout_millis: u64) -> Result<bool, String> {
    let config = RuntimeConfig::from_global_store();
    let url = memos_url.trim_end_matches('/').to_string() + "/";
    let endpoint = format!("{}healthz", url);

    let url = reqwest::Url::parse(&endpoint).unwrap();
    let client = reqwest::Client::new();
    if let Ok(response) = client
        .get(url)
        .header("User-Agent", &config.user_agent)
        .timeout(std::time::Duration::from_millis(if timeout_millis < 100 {
            1000
        } else {
            timeout_millis
        }))
        .send()
        .await
    {
        if response.status().is_success() {
            if let Ok(body) = response.text().await {
                if body.starts_with("Service ready.") {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

#[command]
pub async fn get_env(name: &str) -> Result<String, String> {
    Ok(std::env::var(String::from(name)).unwrap_or(String::from("")))
}

/// Get the app config from the global store.
#[command]
pub async fn get_config() -> Result<String, String> {
    let config = RuntimeConfig::from_global_store();
    let serialized = match serde_json::to_string(&config.yaml) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to serialize config: {}", e);
            String::from("{}")
        }
    };
    Ok(serialized)
}

/// Get the default app config.
#[command]
pub async fn get_default_config() -> Result<String, String> {
    let serialized = match serde_json::to_string(&RuntimeConfig::default().yaml) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to serialize config: {}", e);
            String::from("{}")
        }
    };
    Ok(serialized)
}

/// Apply a configuration patch.
#[command]
pub async fn set_config(patch: String) -> Result<bool, String> {
    debug!("Applying configuration patch: {:?}", patch);

    let mut runtime_config = RuntimeConfig::from_global_store();

    let mut deserialized_config = serde_json::from_str(
        serde_json::to_string(&runtime_config.yaml)
            .unwrap_or_else(|e| {
                error!("Failed to serialize config: {}", e);
                String::from("{}")
            })
            .as_str(),
    )
    .unwrap_or_else(|e| {
        error!("Failed to parse configuration: {}", e);
        json!({})
    });

    let deserialized_patch: Patch = match serde_json::from_str(patch.as_str()) {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to parse configuration patch: {}", e);
            return Ok(false);
        }
    };

    if deserialized_patch.is_empty() {
        error!("Received empty configuration patch. No changes applied.");
        return Ok(false);
    }

    json_patch::patch(&mut deserialized_config, &deserialized_patch).unwrap_or_else(|e| {
        error!("Failed to apply configuration patch: {}", e);
    });

    let new_config: Config = match serde_json::from_value(deserialized_config) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to deserialize configuration: {}", e);
            return Ok(false);
        }
    };

    runtime_config.yaml = new_config.clone();
    RuntimeConfig::to_global_store(&runtime_config);

    info!("Configuration updated by user. Saving…");

    let config_path = runtime_config.paths.memospot_config_file.clone();
    if let Err(e) = runtime_config.yaml.save_to_file(&config_path).await {
        error_dialog!(
            "Failed to save configuration file:\n`{}`\n\n{}",
            &config_path.display(),
            e
        );
    }

    Ok(true)
}

#[command]
pub async fn path_exists(path: String) -> Result<bool, String> {
    Ok(std::path::Path::new(&path).exists())
}
