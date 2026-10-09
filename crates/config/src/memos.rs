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
    /// Use demo mode with pre-seeded data. This is intended for development and testing purposes,
    /// and should not be used in production, as data is purged on each run.
    pub demo: Option<bool>,
    /// DEPRECATED: `MEMOS_MODE` is now retired starting from v0.26.0.
    /// Database is always in `prod` mode unless `MEMOS_DEMO=true` is set.
    ///
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
impl Memos {
    /// Normalize the mode/demo compatibility invariant.
    ///
    /// Unknown modes default to `prod`, and `demo` always mirrors whether
    /// the mode is `demo`, so persisted values match the normalized form
    /// used at startup and process setup.
    pub fn normalize(&mut self) {
        let mode = match self.mode.as_deref() {
            Some("prod") => "prod",
            Some("dev") => "dev",
            Some("demo") => "demo",
            _ => "prod",
        };

        self.mode = Some(mode.to_string());
        self.demo = Some(mode == "demo");
    }
}
impl Default for Memos {
    fn default() -> Self {
        Self {
            binary_path: None,
            working_dir: None,
            data: None,
            demo: Some(false),
            mode: Some("prod".to_string()),
            addr: Some("127.0.0.1".to_string()),
            port: Some(5230),
            env: EnvironmentVariables::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_sets_demo_for_demo_mode() {
        // GIVEN demo mode with a stale demo flag.
        let mut memos = Memos {
            mode: Some("demo".to_string()),
            demo: Some(false),
            ..Default::default()
        };

        // WHEN normalized THEN the flag mirrors the mode.
        memos.normalize();

        // THEN demo is enabled.
        assert_eq!(memos.demo, Some(true));
    }

    #[test]
    fn normalize_disables_demo_for_non_demo_modes() {
        // GIVEN prod mode with a stale demo flag.
        let mut memos = Memos {
            mode: Some("prod".to_string()),
            demo: Some(true),
            ..Default::default()
        };

        // WHEN normalized THEN the flag mirrors the mode.
        memos.normalize();

        // THEN demo is disabled.
        assert_eq!(memos.demo, Some(false));
    }

    #[test]
    fn normalize_defaults_unknown_mode_to_prod() {
        // GIVEN an unknown mode with demo enabled.
        let mut memos = Memos {
            mode: Some("staging".to_string()),
            demo: Some(true),
            ..Default::default()
        };

        // WHEN normalized THEN the mode falls back to prod.
        memos.normalize();

        // THEN the mode is prod and demo is disabled.
        assert_eq!(memos.mode, Some("prod".to_string()));
        assert_eq!(memos.demo, Some(false));
    }
}
