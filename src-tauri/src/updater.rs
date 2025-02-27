use crate::{fl, memos};
use dialog::{confirm_dialog, MessageType};
use log::{error, info, warn};
use tauri_plugin_updater::UpdaterExt;

/// Initialize the updater in the background.
pub fn spawn(app: &tauri::AppHandle) {
    let app_ = app.clone();
    tauri::async_runtime::spawn(async move {
        update(app_).await.unwrap_or_else(|e| {
            error!("updater: failed with error {}", e);
        });
    });
}

async fn update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    let updater = app
        .updater_builder()
        .on_before_exit(|| {
            info!("updater: preparing to install update");
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
            warn!("updater: user declined update download");
            return Ok(());
        }

        info!("updater: downloading update…");
        let mut downloaded = 0;
        update
            .download_and_install(
                |chunk_length, content_length| {
                    downloaded += chunk_length;
                    info!("updater: downloaded {downloaded} from {content_length:?}");
                },
                || {
                    info!("updater: download finished");
                },
            )
            .await?;

        info!("updater: update installed, restarting application");
        app.restart();
    } else {
        info!("updater: no update available");
    }

    Ok(())
}
