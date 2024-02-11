// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod init;
mod js_handler;
mod memos;
mod runtime_config;
mod sqlite;
mod webview;

use config::Config;
use memospot::*;

use log::{debug, info};
use std::env;
use std::path::PathBuf;

use crate::runtime_config::{RuntimeConfig, RuntimeConfigPaths};

fn main() {
    init::ensure_webview();

    let memospot_data = init::data_path("memospot");
    let config_path = memospot_data.join("memospot.yaml");
    let yaml_config = init::config(&config_path);

    let mut rconfig = RuntimeConfig {
        paths: RuntimeConfigPaths {
            memos_bin: PathBuf::new(),
            memos_data: PathBuf::new(),
            memos_db_file: PathBuf::new(),
            memospot_bin: PathBuf::new(),
            memospot_config_file: config_path.clone(),
            memospot_cwd: PathBuf::new(),
            memospot_data: memospot_data.clone(),
        },
        yaml: yaml_config.clone(),
        __yaml__: yaml_config.clone(),
    };

    rconfig.yaml.memos.port = init::memos_port(&rconfig);
    rconfig.paths.memos_data = init::memos_data(&rconfig);
    rconfig.paths.memos_db_file = init::database(&rconfig);

    // TODO: fix logger.
    let _debug_memos = init::setup_logger(&rconfig);

    info!("Starting Memospot");
    info!(
        "Data path: {}",
        rconfig.paths.memospot_data.to_string_lossy()
    );

    rconfig.paths.memospot_bin = std::env::current_exe().unwrap();
    rconfig.paths.memospot_cwd = rconfig.paths.memospot_bin.parent().unwrap().to_path_buf();
    rconfig.paths.memos_bin = init::find_memos(&rconfig);

    info!(
        "Memos server found at: {}",
        rconfig.paths.memos_bin.to_string_lossy()
    );
    info!(
        "Memos data directory: {}",
        rconfig.paths.memos_data.to_string_lossy()
    );

    tauri::async_runtime::block_on(async {
        if let Err(e) = sqlite::migrate(&mut rconfig).await {
            panic_dialog!("Failed to run database migrations:\n{}", e.to_string());
        }
        info!("Database migrations completed successfully.");
    });

    // Save the config file, if it has changed.
    if rconfig.yaml != rconfig.__yaml__ {
        if let Err(e) = Config::save_file(&config_path, &rconfig.yaml) {
            panic_dialog!(
                "Failed to save config file:\n`{}`\n\n{}",
                config_path.to_string_lossy(),
                e.to_string()
            );
        }
    }

    if let Err(err) = memos::spawn(&rconfig) {
        panic_dialog!("Failed to spawn Memos server:\n{}", err);
    };

    let Ok(tauri_app) = tauri::Builder::default()
        .manage(js_handler::MemosPort::manage(rconfig.yaml.memos.port))
        .invoke_handler(tauri::generate_handler![js_handler::get_memos_port])
        .build(tauri::generate_context!())
    else {
        panic_dialog!("Failed to build Tauri application!");
    };

    tauri_app.run(move |app_handle, event| {
        if let tauri::RunEvent::ExitRequested { api, .. } = event {
            api.prevent_exit();

            // Handle Memos shutdown
            tauri::api::process::kill_children();
            tauri::async_runtime::block_on(async {
                sqlite::checkpoint(&rconfig).await;
            });

            debug!("Memospot closed");
            app_handle.exit(0);
        }
    });
}
