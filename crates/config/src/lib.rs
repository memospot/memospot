//! Memospot configuration management.

mod tests;

pub mod default;
mod log;
mod memos;
mod memospot;

use crate::memos::Memos;
use crate::memospot::Memospot;

use figment::providers::{Env, Format, Serialized, Yaml};
use figment::{Figment, Profile};
use serde::{Deserialize, Serialize};

use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use std::time::Duration;
use std::{fs, io, thread};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Config {
    pub memos: Memos,
    pub memospot: Memospot,
}

impl Config {
    const CONFIG_HEADER: &'static str = r#"#
#! User comments on this file will be lost whenever the configuration is updated by Memospot.
#
# To specify custom environment variables for Memos, use the `env` key, like so:
# memos:
#     env:
#         NEW_ENV_VAR: "my value" # always quote custom env values.
#
# You may specify a custom data directory for Memos, like a synced folder or a network share:
# memos:
#     data: "/path/to/data"
"#;

    pub fn to_string(&self) -> Result<String> {
        serde_yaml::to_string(&self).map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }

    pub fn init(cfg_path: &Path) -> io::Result<Config> {
        #[cfg(debug_assertions)]
        const DEFAULT_PROFILE: &str = "debug";
        #[cfg(not(debug_assertions))]
        const DEFAULT_PROFILE: &str = "release";

        let default_config = Config::default();

        // This the base figment, without our `Config` defaults.
        let figment = Figment::new()
            .merge(Yaml::file(cfg_path.to_str().unwrap_or_default()))
            .merge(Env::prefixed("MEMOSPOT_"))
            .select(Profile::from_env_or("MEMOSPOT_PROFILE", DEFAULT_PROFILE))
            .join(Serialized::defaults(default_config));

        figment
            .extract::<Config>()
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }

    /// Parse configuration file.
    pub fn parse_file(cfg_path: &Path) -> Result<Config> {
        Figment::new()
            .merge(Yaml::file(cfg_path.to_str().unwrap_or_default()))
            .extract::<Config>()
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }

    /// Save configuration to supplied file.
    pub fn save_file(cfg_path: &Path, config: &Config) -> Result<()> {
        if cfg_path.is_dir() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "provided path is a directory",
            ));
        }

        let Ok(yaml) = serde_yaml::to_string(&config) else {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "failed to serialize configuration",
            ));
        };

        let file_contents = Self::CONFIG_HEADER.to_string() + &yaml;

        let mut last_error = Error::new(ErrorKind::Other, "unable to write configuration");
        for retry in 0..10 {
            if retry > 0 {
                thread::sleep(Duration::from_millis(100 * retry));
            }

            if let Err(e) = fs::write(cfg_path, file_contents.clone()) {
                last_error = e;
                continue;
            };

            return Ok(());
        }
        Err(last_error)
    }

    /// Reset configuration file to defaults.
    pub fn reset_file(cfg_path: &Path) -> Result<()> {
        let default_config = Config::default();
        Config::save_file(cfg_path, &default_config)
    }
}
