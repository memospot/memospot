// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod webview;

use home_dir::HomeDirExt;
use log::{error, info};
use memospot::confirm_dialog;
use memospot::find_open_port;
use memospot::panic_dialog;
use native_dialog::MessageType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
use std::sync::Mutex;
use tauri::api::process::{Command, CommandEvent};
use tauri::ipc::RemoteDomainAccessScope;
use tauri::Manager;
use tauri::State;

#[cfg(target_os = "macos")]
use window_shadows::set_shadow;

struct MemosPort(Mutex<u16>);

#[tauri::command]
fn js_get_memos_port(memos_port: State<MemosPort>) -> u16 {
    *memos_port.0.lock().unwrap()
}

#[derive(Default, Serialize, Deserialize)]
struct MemosLog {
    time: String,
    latency: String,
    method: String,
    uri: String,
    status: u16,
    error: String,
}

static LOGGING_CONFIG_YAML: &str = r#"
# https://github.com/estk/log4rs#quick-start
appenders:
    file:
        encoder:
            pattern: "{d(%Y-%m-%d %H:%M:%S)(utc)} - {h({l})}: {m}{n}"
        path: "%LOGPATH%"
        kind: rolling_file
        policy:
            trigger:
                kind: size
                limit: 10 mb
            roller:
                kind: fixed_window
                pattern: memos.log.{}.gz
                count: 5
                base: 1
root:
    # debug | info | warn | error | off
    level: error
    appenders:
        - file
"#;

/// Enable logging if logging_config.yaml exists
///
/// Return true if logging is enabled
async fn setup_logger(data_path: &Path) -> bool {
    let log_config: PathBuf = data_path.join("logging_config.yaml");
    let log_path: PathBuf = data_path.join("memos.log");

    if !std::path::Path::new(&log_config).exists() {
        // logging is disabled
        return false;
    }

    if log4rs::init_file(&log_config, Default::default()).is_ok() {
        // logging is enabled and config is ok
        return true;
    }

    // Logging is enabled, but config is bad
    if let Ok(mut file) = File::create(&log_config) {
        let config_template = LOGGING_CONFIG_YAML
            .replace("    ", "  ")
            .replace("%LOGPATH%", &log_path.to_string_lossy());

        if let Err(e) = file.write_all(config_template.as_bytes()) {
            panic_dialog!(
                "Failed to write to `{}`:\n{}",
                log_config.to_string_lossy(),
                e.to_string()
            );
        }
        if let Err(e) = file.flush() {
            panic_dialog!(
                "Failed to flush `{}` to disk:\n{}",
                log_config.to_string_lossy(),
                &e
            );
        }
    } else {
        panic_dialog!(
            "Failed to truncate `{}`. Please delete it and restart the application.",
            log_config.to_string_lossy()
        );
    }

    if log4rs::init_file(&log_config, Default::default()).is_ok() {
        // logging is enabled and config was reset
        return true;
    }

    panic_dialog!(
        "Failed to setup logging!\nPlease delete `{}` and restart the application.",
        log_config.to_string_lossy()
    );
}

#[tokio::main]
async fn main() {
    let data_path = if std::env::consts::OS == "windows" {
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_owned()
    } else {
        "~/.memospot".expand_home().unwrap()
    };

    if !data_path.exists() {
        if let Err(e) = std::fs::create_dir_all(&data_path) {
            panic_dialog!(
                "Failed to create data directory `{}`:\n{}",
                data_path.to_string_lossy(),
                e.to_string()
            );
        }
    }

    let debug_memos = setup_logger(&data_path).await;
    info!("Starting Memospot");
    info!("Data path: {}", data_path.to_string_lossy());

    if !webview::is_available() {
        if confirm_dialog(
            "Error",
            "WebView2 is required for this application to work and it's not available on this system!\n\nDo you want to install it?",
            MessageType::Error,
        ) {
            let _= webview::install().await;
            if !webview::is_available() {
                panic_dialog!("WebView2 is still not available!\n\nPlease install it manually and relaunch the application.");
            }
        } else {
            exit(1);
        }
    }

    let mut memos_bin = "memos".to_owned();
    if std::env::consts::OS == "windows" {
        memos_bin.push_str(".exe");
    }
    let current_exe = std::env::current_exe().unwrap();
    let cwd = current_exe.parent().unwrap();
    let memos_server_bin = cwd.join(memos_bin).clone();
    let memos_path = std::path::Path::new(&memos_server_bin);
    if !memos_path.exists() {
        panic_dialog!(
            "Unable to find memos server at:\n{}",
            memos_server_bin.display()
        );
    }

    #[cfg(dev)]
    static MODE: &str = "demo";
    #[cfg(not(dev))]
    static MODE: &str = "prod";

    let open_port = find_open_port();
    let memos_env_vars: HashMap<String, String> = HashMap::from_iter(vec![
        ("MEMOS_MODE".to_owned(), MODE.to_owned()),
        ("MEMOS_PORT".to_owned(), open_port.to_string()),
        ("MEMOS_ADDR".to_owned(), "127.0.0.1".to_owned()),
        (
            "MEMOS_DATA".to_owned(),
            data_path.to_string_lossy().to_string(),
        ),
    ]);

    tauri::async_runtime::spawn(async move {
        let cmd = Command::new(memos_server_bin.clone().to_str().unwrap())
            .envs(memos_env_vars)
            .spawn();
        if cmd.is_err() {
            panic_dialog!("Failed to spawn memos server");
        }

        if !debug_memos {
            return;
        }

        let (mut rx, _) = cmd.unwrap();
        while let Some(event) = rx.recv().await {
            if let CommandEvent::Stdout(line) = event {
                let json: MemosLog = serde_json::from_str(&line).unwrap_or_default();
                if json.time.is_empty() {
                    continue;
                }

                if !json.error.is_empty() {
                    error!(
                        "latency={}, method={}, uri={}, {}",
                        json.latency, json.method, json.uri, json.error
                    );
                    continue;
                }

                info!(
                    "latency={}, method={}, uri={}, status={}",
                    json.latency, json.method, json.uri, json.status
                );
            }
        }
    });

    let tauri_run = tauri::Builder::default()
        .setup(move |app| {
            // Note:
            //  By using `127.0.0.1` to access server, all remote links (target="_blank") opens in the webview (must use the middle mouse button). Some remote websites doesn't work properly when accessed from a webview (sites using Amazon CloudFront).
            // Using `localhost`, makes all links open in the default browser, fixing those issues.
            let domains = ["localhost"];
            for domain in domains.iter() {
                app.ipc_scope().configure_remote_access(
                    RemoteDomainAccessScope::new(domain.to_string())
                        .add_window("main")
                        .add_plugin("shell")
                        .enable_tauri_api(),
                );
            }

            // Shadows looks bad on Windows
            #[cfg(target_os = "macos")]
            if let Some(window) = app.get_window("main") {
                let _ = set_shadow(&window, true);
            }

            Ok(())
        })
        .manage(MemosPort(Mutex::new(open_port)))
        .invoke_handler(tauri::generate_handler![js_get_memos_port])
        .run(tauri::generate_context!());

    if tauri_run.is_err() {
        panic_dialog!("Failed to run Tauri application");
    }
}
