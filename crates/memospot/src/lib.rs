mod cmd;
mod events;
mod i18n;
mod init;
mod memos;
mod memos_log;
mod memos_version;
mod menu;
mod runtime_config;
mod sqlite;
mod updater;
mod utils;
mod webview;
mod window;
mod zip;

use crate::events::handle_run_events;
use crate::runtime_config::{RuntimeConfig, RuntimeConfigPaths};
use dialog::*;
use i18n::*;
use log::{debug, info, warn};
use std::env;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{async_runtime, Manager};
use tauri_utils::config::WindowConfig;
use window::WindowConfigExt;

#[warn(unused_extern_crates)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    localize();

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
            memospot_config_file: config_path,
            memospot_cwd: PathBuf::new(),
            memospot_data,
        },
        is_managed_server: true,
        memos_url: String::new(),
        user_agent: String::new(),
        yaml: yaml_config.clone(),
        __yaml__: yaml_config,
    };

    let locale = &config
        .yaml
        .memospot
        .window
        .locale
        .clone()
        .unwrap_or_default();

    reload(locale.as_str());
    init::setup_logger(&config);

    #[cfg(debug_assertions)]
    {
        // Use Memos in demo mode during development,
        // as it's already seeded with some data.
        config.yaml.memos.mode = Some("demo".to_string());
        // Use an upper port to use a dedicated WebView cache for development.
        config.yaml.memos.port = Some(config.yaml.memos.port.unwrap_or_default() + 1);
    }

    // Cleanup orphaned Memos processes.
    memos::find_and_kill_orphaned(&config);

    config.yaml.memos.port = Some(init::memos_port(&config));
    config.paths.memos_data = init::memos_data(&config);
    config.paths.memos_db_file = init::database(&config);
    config.memos_url = memos::get_url(&config);

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

    #[cfg(target_os = "linux")]
    init::hw_acceleration();
    init::set_env_vars(&config);

    {
        let url = config.memos_url.clone();
        async_runtime::spawn(async move {
            memos::wait_api_ready(&url).await;
        });
    }

    let mut tauri_ctx = tauri::generate_context!();

    let app_version = tauri_ctx.package_info().version.to_string();
    config.user_agent = config.yaml.memospot.remote.user_agent.as_deref()
        .filter(|x| !x.is_empty() && config.yaml.memospot.remote.enabled.unwrap_or_default())
        .map(|x| x.to_string())
        .unwrap_or_else(|| {
            format!("Mozilla/5.0 (x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Memospot/{}", &app_version)
        });
    warn!("WebView user agent: {}", &config.user_agent);

    let should_run_updater = updater::is_enabled(&config) && updater::should_run(&config);
    if should_run_updater {
        let unix_time_now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        config.yaml.memospot.updater.last_check = Some(unix_time_now);
    }

    // Store `config` and make it immutable after this point.
    config.to_global_store();
    let config = config;

    let main_title = if config.is_managed_server {
        "Memospot".to_string()
    } else {
        let url = config
            .memos_url
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .trim_end_matches("/");
        info!("running in client mode for `{url}`. Memos server will not be started");
        format!("Memospot - {url}")
    };

    let window_config = &mut tauri_ctx.config_mut().app.windows;
    if !window_config.is_empty() {
        window_config[0] = WindowConfig {
            title: main_title,
            url: tauri::WebviewUrl::App("/loader".into()),
            user_agent: Some(config.user_agent.clone()),
            // Stop Tauri from handling drag-and-drop events and pass them to the webview.
            drag_drop_enabled: false,
            incognito: cfg!(debug_assertions),
            // Prevent theme flashing on release builds. The frontend code calls getCurrentWebviewWindow().show() immediately after configuring the theme.
            visible: cfg!(debug_assertions),
            // Doesn't work as it relies on injecting a polyfill, and we are redirecting to the server.
            // TODO: register global hotkeys to change the webview zoom level.
            zoom_hotkeys_enabled: true,
            ..Default::default()
        }
        .restore_window_state();
    }

    if config.is_managed_server {
        let config_ = config.clone();
        async_runtime::spawn(async move {
            init::migrate_database(&config_).await;
            memos::spawn(&config_).expect_dialog(fl!("panic-failed-to-spawn-memos"));
        });
    }

    // Allowing plain `http` for remote URLs is not desirable, but only macOS restricts it.
    // This is used to keep the behavior consistent across platforms.
    #[cfg(target_os = "macos")]
    {
        let invalid_url_error = fl!("error-invalid-server-url", url = config.memos_url.clone());
        let parsed_url = url::Url::parse(&config.memos_url).expect_dialog(&invalid_url_error);
        let domain = parsed_url
            .host()
            .expect_dialog(invalid_url_error)
            .to_string();

        debug!("macOS exception domain: {domain}");
        tauri_ctx.config_mut().bundle.macos.exception_domain = Some(domain);
    }

    let config_ = config;
    #[allow(unused_qualifications)]
    let Ok(tauri_app) = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            app.get_webview_window("main").map(|w| w.set_focus().ok());
        }))
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(cmd::MemosURL::manage(config_.memos_url.clone()))
        .manage(cmd::Locale::manage(
            config_.yaml.memospot.window.locale.unwrap_or_default(),
        ))
        .invoke_handler(tauri::generate_handler![
            cmd::get_memos_url,
            cmd::get_theme,
            cmd::get_reduce_animation_status,
            cmd::get_locale,
            cmd::set_locale,
            cmd::ping_memos,
            cmd::get_env,
            cmd::get_config,
            cmd::get_default_config,
            cmd::set_config,
            cmd::path_exists
        ])
        .setup(move |app| {
            let app_handle = app.handle();

            // Remove the updater plugin to use custom logic.
            app_handle.remove_plugin("tauri-plugin-updater");

            // Menu must be set at the application level to also work in macOS.
            app.set_menu(menu::build(app_handle)?)?;
            menu::update_memos_version_entry(app_handle);

            if should_run_updater {
                debug!("starting updater");
                updater::spawn(app_handle);
            }

            Ok(())
        })
        .build(tauri_ctx)
    else {
        panic_dialog!("Failed to build Tauri application!");
    };

    tauri_app.run(handle_run_events);
}
