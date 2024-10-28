mod cmd;
mod init;
mod memos;
pub mod process;
mod runtime_config;
mod sqlite;
mod utils;
mod webview;
mod window;
mod zip;

use crate::runtime_config::{RuntimeConfig, RuntimeConfigPaths};
use config::Config;
use dialog::*;
use log::{debug, info, warn};
use std::env;
use std::path::PathBuf;
use tauri::path::BaseDirectory;
use tauri::Manager;
use tauri_utils::config::WindowConfig;
use window::{WebviewWindowExt, WindowConfigExt};

#[warn(unused_extern_crates)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init::ensure_webview();

    let memospot_data = init::data_path("memospot");
    let config_path = memospot_data.join("memospot.yaml");
    let yaml_config = init::config(&config_path);

    let mut config = RuntimeConfig {
        paths: RuntimeConfigPaths {
            memos_bin: PathBuf::new(),
            memos_data: PathBuf::new(),
            memos_db_file: PathBuf::new(),
            memospot_bin: PathBuf::new(),
            memospot_config_file: config_path.clone(),
            memospot_cwd: PathBuf::new(),
            memospot_data: memospot_data.clone(),
            _memospot_resources: PathBuf::new(),
        },
        managed_server: true,
        memos_url: String::new(),
        yaml: yaml_config.clone(),
        __yaml__: yaml_config,
    };
    init::setup_logger(&config);
    if cfg!(debug_assertions) {
        // Use Memos in demo mode during development,
        // as it's already seeded with some data.
        config.yaml.memos.mode = Some("demo".to_string());
        // Use an upper port to use a dedicated WebView cache for development.
        config.yaml.memos.port = Some(config.yaml.memos.port.unwrap_or_default() + 1);
    }

    config.yaml.memos.port = Some(init::memos_port(&config));
    config.paths.memos_data = init::memos_data(&config);
    config.paths.memos_db_file = init::database(&config);
    config.memos_url = init::memos_url(&config);

    info!(
        "Memos data directory: {}",
        config.paths.memos_data.to_string_lossy()
    );
    info!("Memos URL: {}", config.memos_url);

    config.managed_server = config.memos_url.starts_with(&format!(
        "http://localhost:{}",
        config.yaml.memos.port.unwrap_or_default()
    ));

    info!("Starting Memospot.");
    info!(
        "Memospot data path: {}",
        config.paths.memospot_data.to_string_lossy()
    );

    config.paths.memospot_bin = env::current_exe().unwrap();
    config.paths.memospot_cwd = config.paths.memospot_bin.parent().unwrap().to_path_buf();
    config.paths.memos_bin = init::find_memos(&config);

    #[cfg(target_os = "linux")]
    init::hw_acceleration();
    init::set_env_vars(&config);

    {
        let url = config.memos_url.clone();
        tauri::async_runtime::spawn(async move {
            memos::wait_api_ready(&url, 100, 15000).await;
        });
    }

    let mut tauri_ctx = tauri::generate_context!();
    let app_version = tauri_ctx.package_info().version.to_string();

    let window_config = &mut tauri_ctx.config_mut().app.windows;
    if !window_config.is_empty() {
        let custom_user_agent = config.yaml.memospot.remote.user_agent.as_deref();
        let user_agent = if config.yaml.memospot.remote.enabled.unwrap_or_default()
            && custom_user_agent.is_some()
        {
            custom_user_agent.unwrap_or_default().to_string()
        } else {
            format!("Mozilla/5.0 (x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Memospot/{}", &app_version)
        };

        window_config[0] = WindowConfig {
            title: format!("Memospot {}", &app_version),
            user_agent: Some(user_agent),
            drag_drop_enabled: false, // Stop Tauri from handling drag-and-drop events and pass them to the webview.
            ..Default::default()
        }
        .restore_attribs_from(&config);
    }

    let mut config_ = config.clone();
    let Ok(tauri_app) = tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_http::init())
        .manage(cmd::MemosURL::manage(config_.memos_url.clone()))
        .invoke_handler(tauri::generate_handler![
            cmd::get_memos_url,
            cmd::ping_memos,
            cmd::get_env
        ])
        .setup(move |app| {
            // Disable updater if the user has disabled it via config file or if running in Flatpak.
            if !config_.yaml.memospot.updater.enabled.unwrap_or_default()
                || env::var("FLATPAK_ID").is_ok()
            {
                app.handle().remove_plugin("tauri-plugin-updater");
            }

            if !config_.managed_server {
                info!(
                    "Using custom Memos address: {}. Memos server will not be started.",
                    config_.memos_url
                );
                let title_url = config_
                    .memos_url
                    .trim_start_matches("http://")
                    .trim_start_matches("https://")
                    .trim_end_matches("/");
                if let Some(main_window) = app.get_webview_window("main") {
                    main_window
                        .set_title(&format!("Memospot {} - {}", app_version, title_url))
                        .unwrap_or_default();
                }
                return Ok(());
            }

            // Add Tauri resource directory as `_memospot_resources`.
            config_.paths._memospot_resources =
                app.path().resolve(".", BaseDirectory::Resource).unwrap();
            debug!(
                "Tauri resource directory: {}",
                config_.paths._memospot_resources.to_string_lossy()
            );

            tauri::async_runtime::spawn(async move {
                init::migrate_database(&config_).await;
                memos::spawn(&config_).unwrap_or_else(|e| {
                    panic_dialog!("Failed to spawn Memos server:\n{}", e);
                });
            });
            Ok(())
        })
        .build(tauri_ctx)
    else {
        panic_dialog!("Failed to build Tauri application!");
    };

    tauri_app.run(move |app_handle, run_event| {
        match run_event {
            tauri::RunEvent::WindowEvent {
                label,
                event: window_event,
                ..
            } => {
                if label != "main" {
                    return;
                }
                match window_event {
                    tauri::WindowEvent::Resized { .. } | tauri::WindowEvent::Moved { .. } => {
                        if let Some(main_window) = app_handle.get_webview_window("main") {
                            main_window.store_attribs_to(&mut config);
                        }
                    }
                    _ => {}
                }
            }
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();

                #[cfg(debug_assertions)]
                {
                    // Restore previous mode and port.
                    config.yaml.memos.mode = config.__yaml__.memos.mode.clone();
                    config.yaml.memos.port = config.__yaml__.memos.port;
                }

                // Save the config file, if it has changed.
                if config.yaml != config.__yaml__ {
                    info!("Configuration has changed. Saving…");
                    if let Err(e) = Config::save_file(&config_path, &config.yaml) {
                        error_dialog!(
                            "Failed to save config file:\n`{}`\n\n{}",
                            config_path.to_string_lossy(),
                            e.to_string()
                        );
                    }
                }

                // Handle Memos shutdown.
                process::kill_children();
                {
                    let db_file = config.paths.memos_db_file.clone();
                    tauri::async_runtime::block_on(async move {
                        sqlite::wait_checkpoint(&db_file, 100, 15000).await;
                    });
                }
                info!("Memospot closed.");
                app_handle.cleanup_before_exit();
                std::process::exit(0);
            }
            _ => {}
        }
    });
}
