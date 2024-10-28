//! This module contains the Tauri command handlers for the JS API.
//!
//! Command handlers and state management must be registered
//! in `tauri::Builder`, at `lib.rs`.
//!
//! The TypeScript/JavaScript API is defined in `src-ui/src/tauri.ts`.

use tauri::{command, State};
use tauri_plugin_http::reqwest;
use tokio::sync::Mutex;

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
pub async fn ping_memos(memos_url: State<'_, MemosURL>) -> Result<String, String> {
    let url = memos_url.0.lock().await.clone();
    let endpoint = format!("{}healthz", url);
    let url = reqwest::Url::parse(&endpoint).unwrap();
    let client = reqwest::Client::new();
    if let Ok(response) = client
        .get(url)
        .header("User-Agent", "Memospot")
        .timeout(std::time::Duration::from_secs(1))
        .send()
        .await
    {
        if response.status().is_success() {
            return Ok(String::from("true"));
        }
    }
    Ok(String::from("false"))
}

#[command]
pub async fn get_env(name: &str) -> Result<String, String> {
    Ok(std::env::var(String::from(name)).unwrap_or(String::from("")))
}
