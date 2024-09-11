// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod init;
mod js_handler;
mod memos;
mod runtime_config;
mod sqlite;
mod webview;
mod zip;

use crate::runtime_config::{RuntimeConfig, RuntimeConfigPaths};
use config::Config;
use memospot::*;

use log::{debug, info};
use std::{ops::IndexMut, path::PathBuf};
use tauri::Manager;

#[warn(unused_extern_crates)]
fn main() {
    init::ensure_webview();

    let memospot_data = init::data_path("memospot");
    let config_path = memospot_data.join("memospot.yaml");
    let yaml_config = init::config(&config_path);

    let mut rtcfg = RuntimeConfig {
        paths: RuntimeConfigPaths {
            memos_bin: PathBuf::new(),
            memos_data: PathBuf::new(),
            memos_db_file: PathBuf::new(),
            _memospot_backups: PathBuf::new(),
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

    rtcfg.yaml.memos.port = Some(init::memos_port(&rtcfg));
    rtcfg.paths.memos_data = init::memos_data(&rtcfg);
    rtcfg.paths.memos_db_file = init::database(&rtcfg);
    rtcfg.memos_url = init::memos_url(&rtcfg);
    info!(
        "Memos data directory: {}",
        rtcfg.paths.memos_data.to_string_lossy()
    );
    info!("Memos URL: {}", rtcfg.memos_url);

    rtcfg.managed_server = rtcfg.memos_url.starts_with(&format!(
        "http://localhost:{}",
        rtcfg.yaml.memos.port.unwrap_or_default()
    ));

    init::setup_logger(&rtcfg);

    info!("Starting Memospot.");
    info!(
        "Memospot data path: {}",
        rtcfg.paths.memospot_data.to_string_lossy()
    );

    rtcfg.paths._memospot_backups = init::backup_directory(&rtcfg);
    rtcfg.paths.memospot_bin = std::env::current_exe().unwrap();
    rtcfg.paths.memospot_cwd = rtcfg.paths.memospot_bin.parent().unwrap().to_path_buf();
    rtcfg.paths.memos_bin = init::find_memos(&rtcfg);

    let mut tauri_ctx = tauri::generate_context!();
    let app_version = tauri_ctx.package_info().version.to_string();
    tauri_ctx.config_mut().tauri.updater.active =
        rtcfg.yaml.memospot.updater.enabled.unwrap_or_default();

    if !tauri_ctx.config_mut().tauri.windows.is_empty() {
        let base_user_agent = "Mozilla/5.0 (x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
        let window_config = tauri_ctx.config_mut().tauri.windows.index_mut(0);
        window_config.center = rtcfg.yaml.memospot.window.center.unwrap_or_default();
        window_config.fullscreen = rtcfg.yaml.memospot.window.fullscreen.unwrap_or_default();
        window_config.maximized = rtcfg.yaml.memospot.window.maximized.unwrap_or_default();
        window_config.resizable = rtcfg.yaml.memospot.window.resizable.unwrap_or_default();
        window_config.width = rtcfg.yaml.memospot.window.width.unwrap_or_default() as f64;
        window_config.height = rtcfg.yaml.memospot.window.height.unwrap_or_default() as f64;
        window_config.x = Some(rtcfg.yaml.memospot.window.x.unwrap_or_default() as f64);
        window_config.y = Some(rtcfg.yaml.memospot.window.y.unwrap_or_default() as f64);
        window_config.user_agent =
            Some(format!("{} Memospot/{}", base_user_agent, &app_version));
        window_config.title = format!("Memospot {}", &app_version);
    }

    let mut rtcfg_setup = rtcfg.clone();
    let Ok(tauri_app) = tauri::Builder::default()
        .manage(js_handler::MemosURL::manage(rtcfg.memos_url.clone()))
        .invoke_handler(tauri::generate_handler![
            js_handler::get_memos_url,
            js_handler::get_env
        ])
        .setup(move |app| {
            if !rtcfg_setup.managed_server {
                info!(
                    "Using custom Memos address: {}. Memos server will not be started.",
                    rtcfg_setup.memos_url
                );
                let title_url = rtcfg_setup
                    .memos_url
                    .trim_start_matches("http://")
                    .trim_start_matches("https://")
                    .trim_end_matches("/");
                if let Some(main_window) = app.get_window("main") {
                    main_window
                        .set_title(&format!("Memospot {} - {}", app_version, title_url))
                        .unwrap_or_default();
                }
                return Ok(());
            }

            // Add Tauri resource directory as `_memospot_resources`.
            rtcfg_setup.paths._memospot_resources = app.path_resolver().resource_dir().unwrap();

            tauri::async_runtime::spawn(async move {
                init::migrate_database(&rtcfg_setup).await;

                if let Err(err) = memos::spawn(&rtcfg_setup) {
                    panic_dialog!("Failed to spawn Memos server:\n{}", err);
                };
            });
            Ok(())
        })
        .build(tauri_ctx)
    else {
        panic_dialog!("Failed to build Tauri application!");
    };

    tauri_app.run(move |app_handle, event| {
        match event {
            tauri::RunEvent::WindowEvent { label, event, .. } => {
                if label != "main" {
                    return;
                }
                if let tauri::WindowEvent::Resized { .. } = event {
                    let main_window = app_handle.get_window("main").unwrap();
                    rtcfg.yaml.memospot.window.maximized =
                        Some(main_window.is_maximized().unwrap_or_default());
                    rtcfg.yaml.memospot.window.width =
                        Some(main_window.inner_size().unwrap_or_default().width);
                    rtcfg.yaml.memospot.window.height =
                        Some(main_window.outer_size().unwrap_or_default().height);
                    rtcfg.yaml.memospot.window.x =
                        Some(main_window.outer_position().unwrap_or_default().x);
                    rtcfg.yaml.memospot.window.y =
                        Some(main_window.outer_position().unwrap_or_default().y);
                }
            }
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
                // Save the config file, if it has changed.
                if rtcfg.yaml != rtcfg.__yaml__ {
                    info!("Configuration has changed. Saving…");
                    if let Err(e) = Config::save_file(&config_path, &rtcfg.yaml) {
                        error_dialog!(
                            "Failed to save config file:\n`{}`\n\n{}",
                            config_path.to_string_lossy(),
                            e.to_string()
                        );
                    }
                }
                // Handle Memos shutdown.
                tauri::api::process::kill_children();
                tauri::async_runtime::block_on(async {
                    let wal = rtcfg.paths.memos_db_file.with_extension("db-wal");
                    let mut retries = 10;
                    while wal.exists() && retries > 0 {
                        if retries < 10 {
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        }
                        debug!("Checkpointing database WAL…");
                        sqlite::checkpoint(&rtcfg).await;
                        retries -= 1;
                    }
                });

                info!("Memospot closed.");
                app_handle.exit(0);
            }
            _ => {}
        }
    });
}
