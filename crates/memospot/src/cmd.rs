//! This module contains the Tauri command handlers for the JS API.
//!
//! Command handlers and state management must be registered
//! in `tauri::Builder`, at `lib.rs`.
//!
//! The TypeScript/JavaScript API is defined in `src-ui/src/lib/tauri.ts`.

use crate::runtime_config::{AppState, ConfigUpdateResult};
use crate::{i18n, memos, menu};
use config::Config;
use i18n_embed::LanguageLoader;
use json_patch::Patch;
use log::{debug, error};
use tauri::{AppHandle, Runtime, State, command};

fn apply_locale<R: Runtime>(app: &AppHandle<R>, state: &AppState) {
    let current_locale = state
        .config
        .snapshot()
        .current
        .memospot
        .window
        .locale
        .clone()
        .unwrap_or_default();
    debug!("current locale set to {current_locale}");

    if current_locale.is_empty() || current_locale == "system" {
        i18n::localize();
    } else {
        i18n::reload(current_locale.as_str());
    }

    match menu::build(app) {
        Ok(menu_bar) => {
            if let Err(error) = app.set_menu(menu_bar) {
                error!("failed to update menu locale: {error}");
            } else {
                menu::update_memos_version_entry(app);
            }
        }
        Err(error) => error!("failed to rebuild menu after locale change: {error}"),
    }
}

#[command]
pub async fn get_memos_url(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.runtime.active_server.url.clone())
}

#[command]
pub async fn get_theme(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.snapshot();
    Ok(config
        .current
        .memospot
        .window
        .theme
        .clone()
        .unwrap_or_default())
}

#[command]
pub async fn get_reduce_animation_status(state: State<'_, AppState>) -> Result<bool, String> {
    let config = state.config.snapshot();
    Ok(config
        .current
        .memospot
        .window
        .reduce_animation
        .unwrap_or_default())
}

#[command]
pub async fn get_locale_preference(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.snapshot();
    Ok(config
        .current
        .memospot
        .window
        .locale
        .clone()
        .unwrap_or_default())
}

#[command]
pub async fn get_effective_locale() -> Result<String, String> {
    Ok(i18n::LOCALE_LOADER.current_language().to_string())
}

/// Set the application locale.
///
/// The preference is persisted through the managed configuration store and,
/// only after a successful synchronized update, applied live by reloading the
/// localization and rebuilding the menu.
#[command]
pub async fn set_locale<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    new: String,
) -> Result<ConfigUpdateResult, String> {
    debug!("setting locale to {new}");

    let update = state
        .config
        .update_and_persist(|config| {
            config.memospot.window.locale = Some(new.clone());
        })
        .await
        .map_err(|e| e.to_string())?;

    if update.locale_changed {
        apply_locale(&app, &state);
    }

    Ok(update.result)
}

#[command]
pub async fn ping_memos(
    state: State<'_, AppState>,
    memos_url: &str,
    timeout_millis: u64,
) -> Result<bool, String> {
    let user_agent = state.runtime.active_server.user_agent.clone();
    memos::ping_api(memos_url, timeout_millis, &user_agent).await
}

#[command]
pub async fn get_env(name: &str) -> Result<String, String> {
    Ok(std::env::var(String::from(name)).unwrap_or(String::from("")))
}

/// Get the current app config.
#[command]
pub async fn get_config(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.snapshot();
    let serialized = match serde_json::to_string(&*config.current) {
        Ok(s) => s,
        Err(e) => {
            error!("failed to serialize config: {e}");
            String::from("{}")
        }
    };
    Ok(serialized)
}

/// Get the default app config.
#[command]
pub async fn get_default_config() -> Result<String, String> {
    let serialized = match serde_json::to_string(&Config::default()) {
        Ok(s) => s,
        Err(e) => {
            error!("failed to serialize config: {e}");
            String::from("{}")
        }
    };
    Ok(serialized)
}

/// Apply a configuration patch.
///
/// The patch is validated and persisted through the managed configuration
/// store. Invalid patches and persistence failures are returned as errors.
#[command]
pub async fn set_config<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    patch: String,
) -> Result<ConfigUpdateResult, String> {
    debug!("applying configuration patch: {patch:?}");

    let deserialized_patch: Patch = match serde_json::from_str(patch.as_str()) {
        Ok(p) => p,
        Err(e) => {
            error!("failed to deserialize configuration patch: {e}");
            return Err(format!("failed to deserialize configuration patch: {e}"));
        }
    };

    if deserialized_patch.is_empty() {
        error!("received empty configuration patch. No changes applied.");
        return Err("received empty configuration patch. No changes applied.".to_string());
    }

    let update = state
        .config
        .apply_patch_and_persist(&deserialized_patch)
        .await
        .map_err(|e| e.to_string())?;
    debug!(
        "configuration updated. Restart required: {}",
        update.result.restart_required
    );

    if update.locale_changed {
        apply_locale(&app, &state);
    }

    Ok(update.result)
}

/// Check if a path exists.
///
/// Tauri [implements](https://v2.tauri.app/plugin/file-system/#exists)
/// something similar, but it's walled by the permission system.
#[command]
pub async fn path_exists(path: String) -> Result<bool, String> {
    Ok(std::path::Path::new(&path).exists())
}

#[command]
pub fn zoom_in<R: Runtime>(app: AppHandle<R>) {
    use std::sync::atomic::Ordering;
    let current = crate::event::ZOOM_LEVEL.load(Ordering::Relaxed) as f64 / 100.0;
    crate::event::apply_zoom(&app, current + crate::event::ZOOM_STEP);
}

#[command]
pub fn zoom_out<R: Runtime>(app: AppHandle<R>) {
    use std::sync::atomic::Ordering;
    let current = crate::event::ZOOM_LEVEL.load(Ordering::Relaxed) as f64 / 100.0;
    crate::event::apply_zoom(&app, current - crate::event::ZOOM_STEP);
}

#[command]
pub fn reset_zoom<R: Runtime>(app: AppHandle<R>) {
    crate::event::apply_zoom(&app, 1.0);
}

#[command]
pub fn toggle_menu_bar<R: Runtime>(app: AppHandle<R>) {
    use tauri::Manager;
    if let Some(main_window) = app.get_webview_window(crate::window::Window::Main.into()) {
        match main_window.is_menu_visible() {
            Ok(true) => main_window.hide_menu().ok(),
            Ok(false) => main_window.show_menu().ok(),
            Err(_) => None,
        };
    }
}

#[command]
pub fn open_settings<R: Runtime>(app: AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        let empty_menu = crate::menu::build_empty(&app)
            .unwrap_or_else(|_| tauri::menu::Menu::with_items(&app, &[]).unwrap());
        let new_window = tauri::WebviewWindowBuilder::new(
            &app,
            crate::window::Window::Settings.to_string(),
            tauri::WebviewUrl::App(crate::route::Route::Settings.into()),
        )
        .title(crate::menu::MainMenu::AppSettings.text().replace("&", ""))
        .center()
        .min_inner_size(800.0, 600.0)
        .inner_size(1160.0, 720.0)
        .auto_resize()
        .disable_drag_drop_handler()
        .zoom_hotkeys_enabled(true)
        .visible(cfg!(debug_assertions))
        .focused(true)
        .menu(empty_menu);

        #[cfg(not(target_os = "macos"))]
        new_window.build().ok();
        #[cfg(target_os = "macos")]
        new_window
            .title_bar_style(tauri::TitleBarStyle::Visible)
            .build()
            .ok();
    });
}
