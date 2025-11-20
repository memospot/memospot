use crate::{fl, memos, RuntimeConfig};
use chrono::DateTime;
use dialog::{confirm_dialog, MessageType};
use log::{debug, error, info, warn};
use std::{
    env,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri_plugin_updater::UpdaterExt;

/// Check whether the updater is enabled.
///
/// This is true if the updater is not explicitly disabled by the
/// user and the application is not running in a Flatpak sandbox.
pub fn is_enabled(config: &RuntimeConfig) -> bool {
    let is_flatpak = env::var("FLATPAK_ID").is_ok_and(|v| !v.is_empty());
    let is_enabled = config
        .yaml
        .memospot
        .updater
        .enabled
        .is_some_and(|enabled| enabled && !is_flatpak);
    debug!("enabled: {is_enabled}");
    is_enabled
}

/// Check if the updater should be run.
///
/// True if the last check time is more than the configured check interval ago.
pub fn should_run(config: &RuntimeConfig) -> bool {
    let check_interval = config
        .yaml
        .memospot
        .updater
        .check_interval
        .clone()
        .unwrap_or_default()
        .parse::<humantime::Duration>()
        .unwrap_or_default();

    let last_check =
        Duration::from_secs(config.yaml.memospot.updater.last_check.unwrap_or_default());

    let unix_time_now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let timestamp =
        DateTime::from_timestamp(last_check.as_secs() as i64, 0).unwrap_or_default();
    let datefmt = timestamp.format("%Y-%m-%d %H:%M:%S");

    debug!("last check: {datefmt} UTC");
    debug!("check interval: {check_interval}");

    let should_run = check_interval.as_secs() > 0
        && last_check.as_secs() + check_interval.as_secs() < unix_time_now;

    debug!("should run: {should_run}");
    should_run
}

/// Initialize the updater in the background.
pub fn spawn(app: &tauri::AppHandle) {
    let app_ = app.clone();
    tauri::async_runtime::spawn(async move {
        update(app_).await.unwrap_or_else(|e| {
            error!("failed with error {e}");
        });
    });
}

/// Check for updates and prompt the user to install them.
async fn update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    let updater = app
        .updater_builder()
        .on_before_exit(|| {
            info!("preparing to install update");
            memos::shutdown();
        })
        .build()?
        .check()
        .await?;

    if let Some(update) = updater {
        let user_confirmed = confirm_dialog(
            fl!("dialog-update-title").as_str(),
            fl!("dialog-update-message", version = update.clone().version).as_str(),
            MessageType::Info,
        );
        if !user_confirmed {
            warn!("user declined update download");
            return Ok(());
        }

        info!("downloading update…");
        let mut downloaded = 0;
        update
            .download_and_install(
                |chunk_length, content_length| {
                    downloaded += chunk_length;
                    info!("downloaded {downloaded} from {content_length:?}");
                },
                || {
                    info!("download finished");
                },
            )
            .await?;

        info!("update installed, restarting application");
        app.restart();
    } else {
        info!("no update available");
    }

    Ok(())
}
