use std::collections::HashMap;

use crate::log::Log;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(TS, Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Backups {
    /// Enable backups.
    pub enabled: Option<bool>,
    /// Directory where Memospot will store backups.
    pub path: Option<String>,
}
impl Default for Backups {
    fn default() -> Self {
        Self {
            enabled: Some(true),
            path: None,
        }
    }
}
#[derive(TS, Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Migrations {
    /// Enable database migrations.
    /// Can be disabled to use Memospot with an uncertified Memos version.
    pub enabled: Option<bool>,
}
impl Default for Migrations {
    fn default() -> Self {
        Self {
            enabled: Some(true),
        }
    }
}

#[derive(TS, Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct RemoteServer {
    /// Enable remote server. This will disable spawning a local Memos server.
    pub enabled: Option<bool>,
    pub url: Option<String>,
    pub user_agent: Option<String>,
}
impl Default for RemoteServer {
    fn default() -> Self {
        Self {
            enabled: Some(false),
            url: None,
            user_agent: None,
        }
    }
}

#[derive(TS, Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct EnvironmentVariables {
    pub enabled: Option<bool>,
    pub vars: Option<HashMap<String, String>>,
}
impl Default for EnvironmentVariables {
    fn default() -> Self {
        Self {
            enabled: Some(false),
            vars: None,
        }
    }
}

#[derive(TS, Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Updater {
    pub enabled: Option<bool>,
}
impl Default for Updater {
    fn default() -> Self {
        Self {
            enabled: Some(true),
        }
    }
}

#[derive(TS, Debug, PartialEq, Clone, Deserialize, Serialize)]
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
    /// Hide main menu bar.
    pub hide_menu_bar: Option<bool>,
    /// Theme.
    pub theme: Option<String>,
    /// Language.
    pub language: Option<String>,
}
impl Default for Window {
    fn default() -> Self {
        Self {
            center: Some(true),
            fullscreen: Some(false),
            resizable: Some(true),
            maximized: Some(false),
            width: Some(1280),
            height: Some(720),
            x: Some(0),
            y: Some(0),
            hide_menu_bar: Some(false),
            theme: None,
            language: None,
        }
    }
}

#[derive(TS, Default, Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Memospot {
    /// Backups settings.
    pub backups: Backups,
    /// Custom system environment variables to pass to Memospot.
    pub env: EnvironmentVariables,
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
