use config::Config;

use std::path::PathBuf;

#[derive(Debug, PartialEq, Clone)]
pub struct RuntimeConfigPaths {
    /// Memos binary file path.
    pub memos_bin: PathBuf,
    /// Memos data directory path.
    pub memos_data: PathBuf,
    /// Memos database file path.
    ///
    /// File name can be one of:
    /// - memos_prod.db
    /// - memos_dev.db
    /// - memos_demo.db
    pub memos_db_file: PathBuf,
    /// Memospot backup directory path.
    ///
    /// Memospot binary file path.
    pub memospot_bin: PathBuf,
    /// Memospot configuration file path.
    pub memospot_config_file: PathBuf,
    /// Memospot current working directory path.
    pub memospot_cwd: PathBuf,
    /// Memospot data directory path.
    pub memospot_data: PathBuf,
    /// Memospot resources directory path.
    ///
    /// This field is set at later stage, at Tauri Builder.
    pub _memospot_resources: PathBuf,
}
#[derive(Debug, PartialEq, Clone)]
pub struct RuntimeConfig {
    /// Store paths used throughout the app.
    pub paths: RuntimeConfigPaths,

    /// Memos URL
    ///
    /// URL always ends with a slash.
    pub memos_url: String,

    /// Whether Memospot is managing a local Memos server.
    pub managed_server: bool,

    /// Store current YAML config. May be modified during app runtime.
    ///
    /// This is the main configuration object used throughout the app.
    pub yaml: Config,

    /// Store initial YAML to compare with current YAML and save the file
    /// if configuration changed.
    ///
    /// DO NOT modify this field after app startup.
    pub __yaml__: Config,
}
