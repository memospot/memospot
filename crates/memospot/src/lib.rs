mod cmd;
mod event;
mod i18n;
mod init;
mod memos;
mod memos_log;
mod memos_version;
mod menu;
mod route;
mod runtime_config;
mod sqlite;
#[cfg(test)]
mod tests;
mod updater;
mod utils;
mod webview;
mod window;
mod window_ext;
mod zip;

use crate::event::handle_run_events;
use crate::route::Route;
#[cfg(debug_assertions)]
use crate::runtime_config::apply_debug_overrides;
use crate::runtime_config::{
    ActiveServer, AppState, ConfigStore, RuntimeContext, RuntimePaths,
};
use crate::window::Window;
use dialog::*;
use i18n::*;
use log::{debug, info, warn};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::webview::PageLoadEvent;
use tauri::{Listener, Manager, async_runtime};
use tauri_utils::config::WindowConfig;
use window_ext::WindowConfigExt;

#[warn(unused_extern_crates)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    localize();

    init::ensure_webview();

    let memospot_data = init::data_path("memospot");
    let config_path = memospot_data.join("memospot.yaml");
    let mut current_config = init::config(&config_path);
    let initial_config = current_config.clone();

    let locale = current_config
        .memospot
        .window
        .locale
        .clone()
        .unwrap_or_default();
    reload(locale.as_str());
    init::setup_logger(&current_config, &memospot_data);

    // Effective Memos settings for the running server, including debug-only
    // overrides that never enter the current configuration.
    let mut effective_memos = current_config.memos.clone();
    #[cfg(debug_assertions)]
    apply_debug_overrides(&mut effective_memos);

    // Cleanup orphaned Memos processes using the effective startup port.
    memos::find_and_kill_orphaned(&effective_memos, &memospot_data);

    let effective_port = init::memos_port(&effective_memos);
    effective_memos.port = Some(effective_port);
    #[cfg(not(debug_assertions))]
    {
        // Persist the resolved port in the current configuration, as before.
        // In debug builds the port override is runtime-only and must not
        // reach the configuration file.
        current_config.memos.port = Some(effective_port);
    }

    let memos_data = init::memos_data(&effective_memos, &memospot_data);
    let memos_db_file = init::database(&effective_memos, &memos_data);
    let memos_url = memos::get_url(&current_config, effective_port);
    let is_managed_server =
        memos_url.starts_with(&format!("http://localhost:{}", effective_port));

    info!("Memos data directory: {}", memos_data.to_string_lossy());
    info!("Memos URL: {}", memos_url);

    info!("Starting Memospot.");
    info!("Memospot data path: {}", memospot_data.to_string_lossy());

    let memospot_bin = env::current_exe().unwrap();
    let memospot_cwd = memospot_bin.parent().unwrap().to_path_buf();

    init::set_env_vars(&current_config);

    {
        let url = memos_url.clone();
        async_runtime::spawn(async move {
            memos::wait_api_ready(&url).await;
        });
    }

    let mut tauri_ctx = tauri::generate_context!();

    let app_version = tauri_ctx.package_info().version.to_string();
    let user_agent = current_config.memospot.remote.user_agent.as_deref()
        .filter(|v| !v.is_empty() && current_config.memospot.remote.enabled.unwrap_or_default())
        .map(|v| v.to_string())
        .unwrap_or_else(|| {
            format!("Mozilla/5.0 (x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Memospot/{}", &app_version)
        });
    warn!("WebView user agent: {}", &user_agent);

    let should_run_updater =
        updater::is_enabled(&current_config) && updater::should_run(&current_config);
    if should_run_updater {
        let unix_time_now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        current_config.memospot.updater.last_check = Some(unix_time_now);
    }

    let memos_bin = init::find_memos(&effective_memos, &memospot_data, &memospot_cwd);

    let runtime = RuntimeContext {
        paths: RuntimePaths {
            memos_bin,
            memos_data,
            memos_db_file,
            memospot_bin,
            memospot_config_file: config_path.clone(),
            memospot_cwd,
            memospot_data,
        },
        active_server: ActiveServer {
            url: memos_url,
            user_agent,
            managed: is_managed_server,
        },
        memos: effective_memos,
    };

    let main_title = if runtime.active_server.managed {
        #[cfg(debug_assertions)]
        let title = "Memospot - DEBUG";
        #[cfg(not(debug_assertions))]
        let title = "Memospot";
        title.to_string()
    } else {
        let url = runtime
            .active_server
            .url
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
            url: tauri::WebviewUrl::App(Route::Loader.into()),
            user_agent: Some(runtime.active_server.user_agent.clone()),
            // Stop Tauri from handling drag-and-drop events and pass them to the webview.
            drag_drop_enabled: false,
            incognito: cfg!(debug_assertions),
            // Prevent theme flashing on release builds.
            // The frontend code calls getCurrentWebviewWindow().show() after configuring the theme.
            visible: cfg!(debug_assertions),
            // Doesn't work as it relies on injecting a polyfill, and we are redirecting to the server.
            // The zoom hotkeys are handled via menu accelerators instead.
            zoom_hotkeys_enabled: false,
            ..Default::default()
        }
        .restore_window_state(&current_config);
    }

    let app_state = AppState {
        runtime,
        config: ConfigStore::new(current_config, initial_config, config_path),
    };

    if app_state.runtime.active_server.managed {
        let runtime = app_state.runtime.clone();
        let current = app_state.config.snapshot().current;
        async_runtime::spawn(async move {
            init::migrate_database(&current, &runtime.paths).await;
            memos::spawn(&runtime, &current).expect_dialog(fl!("panic-failed-to-spawn-memos"));
        });
    }

    // Allowing plain `http` for remote URLs is not desirable, but only macOS restricts it.
    // This is used to keep the behavior consistent across platforms.
    #[cfg(target_os = "macos")]
    {
        let invalid_url_error = fl!(
            "error-invalid-server-url",
            url = app_state.runtime.active_server.url.clone()
        );
        let parsed_url = url::Url::parse(&app_state.runtime.active_server.url)
            .expect_dialog(&invalid_url_error);
        let domain = parsed_url
            .host()
            .expect_dialog(invalid_url_error)
            .to_string();

        debug!("macOS exception domain: {domain}");
        tauri_ctx.config_mut().bundle.macos.exception_domain = Some(domain);
    }

    let config_store = app_state.config.clone();
    let Ok(tauri_app) = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            app.get_webview_window(Window::Main.into())
                .map(|w| w.set_focus().ok());
        }))
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            cmd::get_memos_url,
            cmd::get_theme,
            cmd::get_reduce_animation_status,
            cmd::get_locale_preference,
            cmd::get_effective_locale,
            cmd::set_locale,
            cmd::ping_memos,
            cmd::get_env,
            cmd::get_config,
            cmd::get_default_config,
            cmd::set_config,
            cmd::path_exists,
            cmd::zoom_in,
            cmd::zoom_out,
            cmd::reset_zoom,
            cmd::toggle_menu_bar,
            cmd::open_settings
        ])
        .on_page_load(move |webview, payload| {
            // Register shortcuts via JS injected on each page load.
            //
            // This ensures shortcuts like Ctrl+H, Ctrl+, and Numpad zoom still work
            // even when the native menu bar and its accelerators are hidden.
            if matches!(payload.event(), PageLoadEvent::Finished) {
                static POLYFILL: &str =
                    include_str!(concat!(env!("OUT_DIR"), "/shortcut_polyfill.js"));
                webview.eval(POLYFILL).ok();

                let inject_reduce_animation_polyfill = config_store
                    .snapshot()
                    .current
                    .memospot
                    .window
                    .reduce_animation
                    .unwrap_or(false);

                if inject_reduce_animation_polyfill {
                    static REDUCE_MOTION_POLYFILL: &str =
                        include_str!("polyfills/reduce_animation.js");
                      webview.eval(REDUCE_MOTION_POLYFILL).ok();
                    if cfg!(debug_assertions) {
                        webview.eval("console.warn('Memospot reduce_motion polyfill loaded successfully.');").ok();
                    }
                }
            }
        })
        .setup(move |app| {
            let app_handle = app.handle();

            // Remove the updater plugin to use custom logic.
            app_handle.remove_plugin("tauri-plugin-updater");

            let app_handle_ = app_handle.clone();
            app.listen(event::SHORTCUT_EVENT, move |shortcut_event| {
                event::handle_shortcut_event(&app_handle_, shortcut_event.payload());
            });

            // The menu must be set at the application level to also work in macOS.
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
        panic_dialog!("failed to build Tauri application");
    };

    tauri_app.run(handle_run_events);
}
