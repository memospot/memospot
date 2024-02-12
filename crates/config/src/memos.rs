//! Memos configuration

// use crate::log::Log;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Memos configuration.
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Memos {
    /// Memos server binary path.
    pub binary_path: Option<String>,
    /// Memos server current working directory.
    pub working_dir: Option<String>,
    /// Directory where Memos will store its database and assets.
    pub data: Option<String>,
    /// Server mode. Each mode uses a different database file.
    ///
    /// Can be one of:
    /// - prod
    /// - dev
    /// - demo
    pub mode: String,
    /// Server address.
    ///
    /// This should be "127.0.0.1" whenever running under Memospot.
    ///
    /// Binding to all addressess "0.0.0.0" will trigger a firewall warning on Windows.
    pub addr: String,
    /// Last port used by Memos.
    ///
    /// Memospot will try to reuse this port on subsequent runs, and will find a new free port if the previous one is already in use or if this value is set to 0.
    pub port: u16,
    /// Memos server telemetry.
    pub metric: bool,
    /// Custom environment variables to pass to Memos.
    pub env: Option<HashMap<String, String>>,
    // Memos server log settings.
    // pub log: Log,
}
