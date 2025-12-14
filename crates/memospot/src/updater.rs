use crate::{fl, memos, RuntimeConfig};
use chrono::DateTime;
use log::{debug, error, info, warn};
use std::{
    env,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{async_runtime, AppHandle, Runtime};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_updater::UpdaterExt;

const LATEST_RELEASE_URL: &str = "https://github.com/memospot/memospot/releases/latest";

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
    let check_interval_config = config
        .yaml
        .memospot
        .updater
        .check_interval
        .clone()
        .unwrap_or_default();
    let check_interval = check_interval_config
        .parse::<humantime::Duration>()
        .unwrap_or_default()
        .as_secs();

    let last_check_config = config.yaml.memospot.updater.last_check.unwrap_or_default();
    let last_check = Duration::from_secs(last_check_config).as_secs();

    let unix_time_now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let timestamp = DateTime::from_timestamp(last_check as i64, 0).unwrap_or_default();
    let datefmt = timestamp.format("%Y-%m-%d %H:%M:%S");

    debug!("last check: {datefmt} UTC");
    debug!("check interval: {check_interval}");

    let should_run = check_interval > 0 && last_check + check_interval < unix_time_now;

    debug!("should run: {should_run}");
    should_run
}

/// Initialize the updater in the background.
pub fn spawn<R: Runtime>(app: &AppHandle<R>) {
    let app_ = app.clone();
    async_runtime::spawn(async move {
        if let Err(e) = update(app_).await {
            error!("failed with error {e}");
        };
    });
}

pub fn manual_check<R: Runtime>(app: AppHandle<R>) {
    async_runtime::spawn(async move {
        match update(app.clone()).await {
            Err(e) => error!("failed with error {e}"),
            Ok(update_available) => {
                if !update_available {
                    app.dialog()
                        .message(fl!("dialog-update-no-update"))
                        .kind(MessageDialogKind::Info)
                        .title(if cfg!(target_os = "linux") {
                            "".into()
                        } else {
                            fl!("dialog-generic-info")
                        })
                        .blocking_show();
                }
            }
        }
    });
}

/// Check for updates and prompt the user to install them.
async fn update<R: Runtime>(app: AppHandle<R>) -> tauri_plugin_updater::Result<bool> {
    debug!("auto-updater is starting");
    let updater = app
        .updater_builder()
        .on_before_exit(|| {
            info!("preparing to install update");
            async_runtime::block_on(async move {
                memos::shutdown().await;
            });
        })
        .build()?
        .check()
        .await?;

    let Some(update) = updater else {
        info!("no update available");
        return Ok(false);
    };

    let new_version = update.version.to_owned();
    let user_confirmed = app
        .dialog()
        .message(fl!("dialog-update-message", version = new_version.clone()).as_str())
        .title(if cfg!(target_os = "linux") {
            "".into()
        } else {
            fl!("dialog-update-title")
        })
        .buttons(MessageDialogButtons::OkCancel)
        .blocking_show();
    if !user_confirmed {
        warn!("user declined update download");
        return Ok(true);
    }

    info!("downloading update…");
    let mut downloaded = 0;
    let update_install = update
        .download_and_install(
            |chunk_length, content_length| {
                downloaded += chunk_length;
                info!("downloaded {downloaded} from {content_length:?}");
            },
            || {
                info!("download finished");
            },
        )
        .await;
    if let Err(e) = update_install {
        error!("failed to auto update to version {new_version}: {e}");
        let user_confirmed = app
            .dialog()
            .message(
                fl!(
                    "dialog-update-manually-prompt",
                    version = new_version,
                    error = e.to_string()
                )
                .as_str(),
            )
            .title(if cfg!(target_os = "linux") {
                "".into()
            } else {
                fl!("dialog-update-failed-title")
            })
            .buttons(MessageDialogButtons::OkCancel)
            .blocking_show();
        if user_confirmed {
            app.opener().open_url(LATEST_RELEASE_URL, None::<&str>).ok();
            return Ok(true);
        }
    }

    info!("update installed, restarting application");
    app.restart();
}
