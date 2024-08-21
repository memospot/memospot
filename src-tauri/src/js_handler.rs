//! This module contains the Tauri handlers for the JS API.
//!
//! Handlers and state management must be registered
//! in `tauri::Builder`, at `main.rs`.
//!
//! The TypeScript/JavaScript API is defined in `src-ui/src/tauri.ts`.

use tauri::{command, State};
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
pub async fn get_env(name: &str) -> Result<String, String> {
    Ok(std::env::var(String::from(name)).unwrap_or(String::from("")))
}
