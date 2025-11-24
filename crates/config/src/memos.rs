//! Memos configuration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

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

/// Memos configuration.
#[derive(TS, Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Memos {
    /// Memos binary path.
    pub binary_path: Option<String>,
    /// Memos current working directory.
    pub working_dir: Option<String>,
    /// Directory where Memos will store its database and assets.
    pub data: Option<String>,
    /// Server mode. Each mode uses a different database file.
    ///
    /// Can be one of:
    /// - prod
    /// - dev
    /// - demo
    pub mode: Option<String>,
    /// Server address.
    ///
    /// This should be "127.0.0.1" whenever running under Memospot.
    ///
    /// Binding to all addresses "0.0.0.0" will trigger a firewall warning on Windows.
    pub addr: Option<String>,
    /// Last port used by Memos.
    ///
    /// Memospot will try to reuse this port on subsequent runs, and will find a new
    /// free port if the previous one is already in use or if this value is set to 0.
    pub port: Option<u16>,

    /// Custom environment variables to pass to Memos.
    pub env: EnvironmentVariables,
    // Memos server log settings.
    // pub log: Log,
}
impl Default for Memos {
    fn default() -> Self {
        Self {
            binary_path: None,
            working_dir: None,
            data: None,
            mode: Some("prod".to_string()),
            addr: Some("127.0.0.1".to_string()),
            port: Some(5230),
            env: EnvironmentVariables::default(),
        }
    }
}
