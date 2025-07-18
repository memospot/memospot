use crate::memos::Memos;
use crate::memospot::Memospot;
use crate::migration::MigrationExt;

use anyhow::{bail, Error, Result};
use figment::providers::{Env, Format, Json, Serialized, Yaml};
use figment::{Figment, Profile};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::io::AsyncWriteExt;
use ts_rs::TS;
use uuid::Uuid;

#[derive(TS, Default, Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Config {
    pub memos: Memos,
    pub memospot: Memospot,
}

#[cfg(debug_assertions)]
pub const CONFIG_PROFILE: &str = "debug";
#[cfg(not(debug_assertions))]
pub const CONFIG_PROFILE: &str = "release";

impl Config {
    const CONFIG_HEADER: &'static str = r#"#
#! User comments on this file will be lost whenever
#    the configuration is updated by Memospot !
#
# For an explained configuration file, see:
# https://memospot.github.io/configuration
#
"#;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn to_string(&self) -> Result<String, Error> {
        Ok(serde_yaml::to_string(&self)?)
    }

    /// Initialize configuration from a file.
    pub fn init(cfg_path: &Path) -> Result<Config, Error> {
        let default_config = Config::default();

        let figment = Figment::new()
            .merge(Yaml::file(cfg_path))
            .merge(Env::prefixed("MEMOSPOT_"))
            .migrate()
            .select(Profile::from_env_or("MEMOSPOT_PROFILE", CONFIG_PROFILE))
            .join(Serialized::defaults(default_config));

        Ok(figment.extract::<Config>()?)
    }

    /// Load configuration from a JSON string.
    pub fn from_json(json: &str) -> Result<Config, Error> {
        let default_config = Config::default();

        let figment = Figment::new()
            .merge(Json::string(json))
            .merge(Env::prefixed("MEMOSPOT_"))
            .select(Profile::from_env_or("MEMOSPOT_PROFILE", CONFIG_PROFILE))
            .join(Serialized::defaults(default_config));

        Ok(figment.extract::<Config>()?)
    }

    /// Parse configuration file.
    pub fn parse_file(cfg_path: &Path) -> Result<Config, Error> {
        let parsed_config = Figment::new()
            .merge(Yaml::file(cfg_path))
            .extract::<Config>()?;
        Ok(parsed_config)
    }

    /// Save current configuration to a file.
    ///
    /// This function uses an intermediate rename operation and delayed attempts to achieve atomicity.
    pub async fn save_to_file(&self, file_path: &Path) -> Result<()> {
        if file_path.is_dir() {
            bail!("provided configuration file is a directory");
        }
        let Ok(yaml) = serde_yaml::to_string(&self) else {
            bail!("failed to serialize configuration");
        };

        let file_contents = Self::CONFIG_HEADER.to_string() + &yaml;
        let time_start = tokio::time::Instant::now();

        let mut last_error: Option<Error> = None;
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
        loop {
            interval.tick().await;
            if time_start.elapsed() > tokio::time::Duration::from_secs(5) {
                bail!("unable to write configuration. Timed out after 5 seconds. Last error: {last_error:?}");
            }

            let uuid = Uuid::new_v4();
            let tmp_file = file_path.with_file_name(format!("{uuid}.tmp"));

            let mut file = match tokio::fs::File::create(&tmp_file).await {
                Ok(f) => f,
                Err(e) => {
                    last_error = Some(e.into());
                    continue;
                }
            };

            if let Err(e) = file.write_all(file_contents.as_bytes()).await {
                last_error = Some(e.into());
                tokio::fs::remove_file(&tmp_file).await.ok();
                continue;
            }

            if let Err(e) = file.flush().await {
                last_error = Some(e.into());
                tokio::fs::remove_file(&tmp_file).await.ok();
                continue;
            }

            if let Err(e) = tokio::fs::rename(&tmp_file, file_path).await {
                last_error = Some(e.into());
                tokio::fs::remove_file(&tmp_file).await.ok();
                continue;
            }

            break;
        }
        Ok(())
    }

    /// Reset configuration file to defaults and save it.
    pub async fn reset_file(cfg_path: &Path) -> Result<()> {
        let default_config = Self::default();
        default_config.save_to_file(cfg_path).await
    }

    /// Reset configuration file to defaults and save it.
    ///
    /// Blocking version of `reset_file`.
    pub fn reset_file_blocking(cfg_path: &Path) -> Result<()> {
        let default_config = Self::default();
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async { default_config.save_to_file(cfg_path).await })
    }
}
