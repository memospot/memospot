use config::Config;

use std::path::PathBuf;

pub struct RuntimeConfigPaths {
    /// Memos binary file path.
    pub memos_bin: PathBuf,
    /// Memos data directory path.
    pub memos_data: PathBuf,
    /// Memos database file path.
    pub memos_db_file: PathBuf,
    /// Memospot binary file path.
    pub memospot_bin: PathBuf,
    /// Memospot configuration file path.
    pub memospot_config_file: PathBuf,
    /// Memospot current working directory path.
    pub memospot_cwd: PathBuf,
    /// Memospot data directory path.
    pub memospot_data: PathBuf,
}

pub struct RuntimeConfig {
    /// Store paths used throughout the app.
    pub paths: RuntimeConfigPaths,
    /// Store current YAML config. May be modified during app runtime.
    ///
    /// This is the main configuration object used throughout the app.
    pub yaml: Config,
    /// Store initial YAML to compare with current YAML and save if changed.
    ///
    /// DO NOT modify this field after app startup.
    pub __yaml__: Config,
}
