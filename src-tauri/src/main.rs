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

use log::info;
use std::path::PathBuf;

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
        __yaml__: yaml_config.clone(),
    };

    rtcfg.yaml.memos.port = Some(init::memos_port(&rtcfg));
    rtcfg.paths.memos_data = init::memos_data(&rtcfg);
    rtcfg.paths.memos_db_file = init::database(&rtcfg);
    rtcfg.memos_url = init::memos_url(&rtcfg);
    info!(
        "Memos data directory: {}",
        rtcfg.paths.memos_data.to_string_lossy()
    );

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

    // Save the config file if it has changed.
    if rtcfg.yaml != rtcfg.__yaml__ {
        if let Err(e) = Config::save_file(&config_path, &rtcfg.yaml) {
            panic_dialog!(
                "Failed to save config file:\n`{}`\n\n{}",
                config_path.to_string_lossy(),
                e.to_string()
            );
        }
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
        .build(tauri::generate_context!())
    else {
        panic_dialog!("Failed to build Tauri application!");
    };

    tauri_app.run(move |app_handle, event| {
        if let tauri::RunEvent::ExitRequested { api, .. } = event {
            api.prevent_exit();

            // Handle Memos shutdown.
            tauri::api::process::kill_children();
            tauri::async_runtime::block_on(async {
                let wal = rtcfg.paths.memos_db_file.with_extension("db-wal");
                let mut retry = 10;
                while wal.exists() && retry > 0 {
                    if retry < 10 {
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    }
                    sqlite::checkpoint(&rtcfg).await;
                    retry -= 1;
                }
            });

            info!("Memospot closed.");
            app_handle.exit(0);
        }
    });
}
