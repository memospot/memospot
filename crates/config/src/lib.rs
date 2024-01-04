//! Memospot configuration management.

mod tests;

use figment::providers::Env;
use figment::providers::Format;
use figment::providers::Serialized;
use figment::providers::Yaml;
use figment::Figment;
use figment::Profile;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::io;
use std::io::ErrorKind;
use std::path::Path;
use std::thread;
use std::time::Duration;

pub type Error = io::Error;

// https://github.com/estk/log4rs/blob/main/docs/Configuration.md
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct LogRotation {
    pub enabled: bool,
    pub max_size: String,
    pub amount: u16,
    pub path_mask: String,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Log {
    pub enabled: bool,
    pub file: String,
    pub level: String,
    pub pattern: String,
    pub rotation: LogRotation,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Memos {
    pub mode: String,
    pub addr: String,
    pub port: u16,
    pub data: String,
    pub driver: String,
    pub dsn: String,
    pub telemetry: bool,
    pub log: Log,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Memospot {
    pub log: Log,
}

/// Memospot configuration.
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Config {
    pub memos: Memos,
    pub memospot: Memospot,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            memos: Memos {
                mode: "prod".to_owned(),
                addr: "127.0.0.1".to_owned(),
                port: 0,
                data: "".to_owned(),
                driver: "sqlite".to_owned(),
                dsn: "".to_owned(),
                telemetry: true,
                log: Log {
                    enabled: false,
                    file: "memos.log".to_owned(),
                    level: "info".to_owned(),
                    pattern: "{d(%Y-%m-%d %H:%M:%S)} - {h({l})}: {m}{n}".to_owned(),
                    rotation: LogRotation {
                        enabled: true,
                        max_size: "10 mb".to_owned(),
                        amount: 5,
                        path_mask: "$ENV{MEMOSPOT_DATA}/memos.log.{}.gz".to_owned(),
                    },
                },
            },
            memospot: Memospot {
                log: Log {
                    enabled: false,
                    file: "memospot.log".to_owned(),
                    level: "info".to_owned(),
                    pattern: "{d(%Y-%m-%d %H:%M:%S)} - {h({l})}: {m}{n}".to_owned(),
                    rotation: LogRotation {
                        enabled: true,
                        max_size: "10 mb".to_owned(),
                        amount: 5,
                        path_mask: "$ENV{MEMOSPOT_DATA}/memospot.log.{}.gz".to_owned(),
                    },
                },
            },
        }
    }
}

impl Config {
    const CONFIG_HEADER: &'static str = r#"
# Memospot configuration file.
#
# Some fields are automatically set by the application.
#! User comments will be lost when the file is rewritten.

"#;

    pub fn to_string(&self) -> io::Result<String> {
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
    pub fn parse_file(cfg_path: &Path) -> io::Result<Config> {
        Figment::new()
            .merge(Yaml::file(cfg_path.to_str().unwrap_or_default()))
            .extract::<Config>()
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }

    /// Save configuration to supplied file.
    pub fn save_file(cfg_path: &Path, config: &Config) -> io::Result<()> {
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
    pub fn reset_file(cfg_path: &Path) -> io::Result<()> {
        let default_config = Config::default();
        Config::save_file(cfg_path, &default_config)
    }
}
