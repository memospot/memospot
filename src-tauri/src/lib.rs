mod cmd;
mod init;
mod localize;
mod memos;
mod menu;
mod process;
mod runtime_config;
mod sqlite;
mod utils;
mod webview;
mod window;
mod zip;

use crate::runtime_config::{RuntimeConfig, RuntimeConfigPaths};
use dialog::*;
use log::{debug, info, warn};
use std::env;
use std::path::PathBuf;
use tauri::Manager;
use tauri_utils::config::WindowConfig;
use window::{WebviewWindowExt, WindowConfigExt};

#[warn(unused_extern_crates)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    localize::localize();

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
        },
        is_managed_server: true,
        memos_url: String::new(),
        user_agent: String::new(),
        yaml: yaml_config.clone(),
        __yaml__: yaml_config,
    };
    init::setup_logger(&config);

    #[cfg(debug_assertions)]
    {
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

    config.is_managed_server = config.memos_url.starts_with(&format!(
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

    config.to_global_store();

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
    let custom_user_agent = config.yaml.memospot.remote.user_agent.as_deref();
    config.user_agent = custom_user_agent
        .filter(|x| !x.is_empty() && config.yaml.memospot.remote.enabled.unwrap_or(false))
        .map(|x| x.to_string())
        .unwrap_or_else(|| {
            format!("Mozilla/5.0 (x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Memospot/{}", &app_version)
        });

    let window_config = &mut tauri_ctx.config_mut().app.windows;
    if !window_config.is_empty() {
        window_config[0] = WindowConfig {
            title: "Memospot".into(),
            url: tauri::WebviewUrl::App("/loader".into()),
            user_agent: Some(config.user_agent.clone()),
            drag_drop_enabled: false, // Stop Tauri from handling drag-and-drop events and pass them to the webview.
            visible: false, // Prevent theme flashing. The frontend code calls getCurrentWebviewWindow().show() immediately after configuring the theme.
            ..Default::default()
        }
        .restore_window_state();
    }

    if config.is_managed_server {
        let config_ = config.clone();
        tauri::async_runtime::spawn(async move {
            init::migrate_database(&config_).await;
            memos::spawn(&config_).unwrap_or_else(|e| {
                panic_dialog!("Failed to spawn Memos server:\n{}", e);
            });
        });
    }

    let config_ = config.clone();
    let Ok(tauri_app) = tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(cmd::MemosURL::manage(config_.memos_url.clone()))
        .manage(cmd::Language::manage(
            config_.yaml.memospot.window.language.unwrap_or_default(),
        ))
        .invoke_handler(tauri::generate_handler![
            cmd::get_memos_url,
            cmd::get_theme,
            cmd::get_language,
            cmd::set_language,
            cmd::ping_memos,
            cmd::get_env,
            cmd::get_config,
            cmd::get_default_config,
            cmd::set_config,
            cmd::path_exists
        ])
        .setup(move |app| {
            let handle = app.handle();
            if let Some(main_window) = app.get_webview_window("main") {
                main_window.set_menu(menu::build(handle)?).ok();
                menu::update_with_memos_version(handle);
            }

            if config_.yaml.memospot.updater.enabled.is_some_and(|e| !e) {
                warn!("Disabling updater plugin by user config.");
                handle.remove_plugin("tauri-plugin-updater");
            }
            if env::var("FLATPAK_ID").is_ok_and(|id| !id.is_empty()) {
                debug!("Running in Flatpak. Disabling updater plugin.");
                handle.remove_plugin("tauri-plugin-updater");
            }

            info!(
                "webview url: {}",
                &app.get_webview_window("main").unwrap().url().unwrap()
            );

            if config_.is_managed_server {
                return Ok(());
            }

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
                    .set_title(&format!("Memospot - {}", title_url))
                    .unwrap_or_default();
            }

            Ok(())
        })
        .build(tauri_ctx)
    else {
        panic_dialog!("Failed to build Tauri application!");
    };

    tauri_app.run(move |app_handle, run_event| {
        match run_event {
            tauri::RunEvent::MenuEvent(menu_event) => {
                menu::handle_event(app_handle, menu_event);
            }
            tauri::RunEvent::WindowEvent {
                label,
                event: window_event,
                ..
            } => {
                if label == "main" {
                    match window_event {
                        tauri::WindowEvent::Resized { .. }
                        | tauri::WindowEvent::Moved { .. } => {
                            if let Some(main_window) = app_handle.get_webview_window("main") {
                                main_window.persist_window_state();
                            }
                        }
                        _ => {}
                    }
                }
            }
            tauri::RunEvent::ExitRequested { api, .. } => {
                debug!("Exit requested.");
                api.prevent_exit();

                #[cfg(not(debug_assertions))]
                let final_config = RuntimeConfig::from_global_store();

                #[cfg(debug_assertions)]
                let mut final_config = RuntimeConfig::from_global_store();

                #[cfg(debug_assertions)]
                {
                    // Restore previous mode and port.
                    final_config.yaml.memos.mode = final_config.__yaml__.memos.mode.clone();
                    final_config.yaml.memos.port = final_config.__yaml__.memos.port;
                }

                // if config.yaml != config.__yaml__ {
                if final_config.yaml != final_config.__yaml__ {
                    info!("Configuration has changed. Saving…");
                    tauri::async_runtime::block_on(async {
                        if let Err(e) = final_config.yaml.save_to_file(&config_path).await {
                            error_dialog!(
                                "Failed to save config file:\n`{}`\n\n{}",
                                &config_path.display(),
                                e
                            );
                        }
                    })
                }

                debug!("Shutting down Memos server…");
                process::kill_children();
                {
                    let db_file = final_config.paths.memos_db_file.clone();
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
