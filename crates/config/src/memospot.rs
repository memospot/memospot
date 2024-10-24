use std::collections::HashMap;

use crate::log::Log;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Backups {
    /// Enable backups.
    pub enabled: Option<bool>,
    /// Directory where Memospot will store backups.
    pub path: Option<String>,
}
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Migrations {
    /// Enable database migrations.
    /// Can be disabled to use Memospot with an uncertified Memos version.
    pub enabled: Option<bool>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct RemoteServer {
    /// Enable remote server. This will disable spawning a local Memos server.
    pub enabled: Option<bool>,
    pub url: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Updater {
    pub enabled: Option<bool>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Window {
    /// Whether the window should be centered upon creation.
    pub center: Option<bool>,
    /// Whether the window should be fullscreen upon creation.
    pub fullscreen: Option<bool>,
    /// Whether the window should be resizable.
    pub resizable: Option<bool>,
    /// (Managed) Whether the window should be maximized upon creation.
    pub maximized: Option<bool>,
    /// (Managed) The window's initial width.
    pub width: Option<u32>,
    /// (Managed) The window's initial height.
    pub height: Option<u32>,
    /// (Managed) The window's initial x position.
    pub x: Option<i32>,
    /// (Managed) The window's initial y position.
    pub y: Option<i32>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Memospot {
    /// Backups settings.
    pub backups: Backups,
    /// Custom system environment variables to pass to Memospot.
    pub env: Option<HashMap<String, String>>,
    /// Database migrations settings.
    pub migrations: Migrations,
    // Log settings.
    pub log: Log,
    /// Remote server settings.
    pub remote: RemoteServer,
    /// Updater settings.
    pub updater: Updater,
    /// Window settings.
    pub window: Window,
}
