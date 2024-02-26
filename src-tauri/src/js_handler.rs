//! This module contains the Tauri handlers for the JS API.
//!
//! Handlers and state management must be registered
//! in `tauri::Builder`, at `main.rs`.

use tauri::{command, State};
use tokio::sync::Mutex;

pub struct MemosPort(pub Mutex<u16>);
impl MemosPort {
    pub fn manage(port: u16) -> Self {
        Self(Mutex::new(port))
    }
}

#[command]
pub async fn get_memos_port(memos_port: State<'_, MemosPort>) -> Result<u16, String> {
    Ok(*memos_port.0.lock().await)
}
