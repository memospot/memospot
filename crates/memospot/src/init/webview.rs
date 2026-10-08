/// WebView setup.
use crate::fl;
use crate::webview;
use dialog::*;
use log::warn;
use std::process::exit;

/// Ensure that WebView is available.
pub fn ensure_webview() {
    if webview::is_available() {
        return;
    }

    let user_confirmed = confirm_dialog(
        fl("prompt-install-webview-title").as_str(),
        fl("prompt-install-webview-message").as_str(),
        MessageType::Error,
    );
    if !user_confirmed {
        warn!("webview setup: user declined to setup.");
        exit(1);
    }

    tauri::async_runtime::block_on(async move {
        if let Err(e) = webview::install().await {
            error_dialog!(fl!(
                "error-failed-to-install-webview",
                error = e.to_string()
            ));

            if let Err(e) = webview::open_install_website() {
                warn!("webview setup: unable to open website: {e}");
            }
            exit(1)
        }
    });

    if !webview::is_available() {
        panic_dialog!(fl!("error-failed-to-install-webview", error = ""));
    }
}
