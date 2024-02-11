//! This module contains the Tauri handlers for the JS API.
//!
//! Handlers and state management must be registered
//! in `tauri::Builder`, at `main.rs`.

use std::sync::Mutex;
use tauri::{command, State};

pub struct MemosPort(pub Mutex<u16>);
impl MemosPort {
    pub fn manage(port: u16) -> Self {
        Self(Mutex::new(port))
    }
}

#[command]
pub fn get_memos_port(memos_port: State<MemosPort>) -> u16 {
    *memos_port.0.lock().unwrap()
}
