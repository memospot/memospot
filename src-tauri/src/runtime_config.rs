use config::Config;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::{Arc, Mutex};

type RuntimeConfigStore = Arc<Mutex<RuntimeConfig>>;

#[derive(TS, Debug, PartialEq, Clone, Serialize, Deserialize)]
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
}

#[derive(TS, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[ts(export)]
enum ExportTSBindings {
    Config(Config),
}

#[derive(TS, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Store paths used throughout the app.
    pub paths: RuntimeConfigPaths,

    /// Memos URL
    ///
    /// URL always ends with a slash.
    pub memos_url: String,

    /// User-Agent header sent to Memos server.
    pub user_agent: String,

    /// Whether Memospot is managing a local Memos server.
    pub is_managed_server: bool,

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
impl Default for RuntimeConfig {
    fn default() -> Self {
        Self::new()
    }
}
impl RuntimeConfig {
    pub fn new() -> Self {
        Self {
            paths: RuntimeConfigPaths {
                memos_bin: PathBuf::new(),
                memos_data: PathBuf::new(),
                memos_db_file: PathBuf::new(),
                memospot_bin: PathBuf::new(),
                memospot_config_file: PathBuf::new(),
                memospot_cwd: PathBuf::new(),
                memospot_data: PathBuf::new(),
            },
            is_managed_server: true,
            memos_url: String::new(),
            user_agent: String::new(),
            yaml: Config::default(),
            __yaml__: Config::default(),
        }
    }

    fn global_store() -> &'static RuntimeConfigStore {
        static GLOBAL_STORE: LazyLock<RuntimeConfigStore> = LazyLock::new(Default::default);
        &GLOBAL_STORE
    }

    /// Retrieve the configuration previously stored in the global store.
    pub fn from_global_store() -> RuntimeConfig {
        Self::global_store().lock().unwrap().clone()
    }

    /// Store current configuration at the global store.
    ///
    /// It can be retrieved with [`from_global_store()`].
    pub fn to_global_store(&self) {
        let mut store = Self::global_store().lock().unwrap();
        *store = self.clone();
    }
}
