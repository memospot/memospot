//! Webview windows.
//!
//! Single home for "how a window is built": the [`Window`] label enum plus
//! the builders that open settings and main-clone windows. Menu, command,
//! and shortcut dispatch all call in here instead of carrying their own
//! builder copies.

use std::convert::AsRef;
use std::fmt;
use strum::{AsRefStr, EnumString, IntoStaticStr};

#[derive(AsRefStr, EnumString, IntoStaticStr)]
pub enum Window {
    #[strum(serialize = "main")]
    Main,
    #[strum(serialize = "settings")]
    Settings,
}

impl Window {
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl fmt::Display for Window {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

use crate::menu::MainMenu;
use crate::route::Route;
use log::{debug, error};
use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};
use uuid::Uuid;

/// Open the settings window, focusing it if already open.
///
/// Fire-and-forget: build failures are logged at this seam. If focusing an
/// existing window fails, falls back to opening fresh so the action always
/// yields a visible settings window.
pub fn open_settings_window<R: Runtime>(app: AppHandle<R>) {
    if let Some(existing) = app.get_webview_window(Window::Settings.into()) {
        if existing.set_focus().is_ok() {
            return;
        }
        debug!("failed to focus existing settings window; opening fresh");
    }
    tauri::async_runtime::spawn(async move {
        let empty_menu = match crate::menu::build_empty(&app) {
            Ok(menu) => menu,
            Err(e) => {
                error!("failed to build empty menu for settings window: {e}");
                match tauri::menu::Menu::with_items(&app, &[]) {
                    Ok(menu) => menu,
                    Err(e) => {
                        error!("failed to build fallback empty menu: {e}");
                        return;
                    }
                }
            }
        };
        let new_window = WebviewWindowBuilder::new(
            &app,
            Window::Settings.to_string(),
            WebviewUrl::App(Route::Settings.into()),
        )
        .title(MainMenu::AppSettings.text().replace("&", ""))
        .center()
        .min_inner_size(800.0, 600.0)
        .inner_size(1160.0, 720.0)
        .auto_resize()
        .disable_drag_drop_handler()
        .zoom_hotkeys_enabled(true)
        .visible(cfg!(debug_assertions))
        .focused(true)
        .menu(empty_menu);

        build_window(new_window, "settings");
    });
}

/// Open a fresh clone of the main window.
///
/// Clones carry UUID labels by design, so every call opens new.
/// Fire-and-forget: build failures are logged at this seam.
pub fn open_main_clone_window<R: Runtime>(app: AppHandle<R>, title: String) {
    tauri::async_runtime::spawn(async move {
        let empty_menu = match crate::menu::build_empty(&app) {
            Ok(menu) => menu,
            Err(e) => {
                error!("failed to build empty menu for new window: {e}");
                match tauri::menu::Menu::with_items(&app, &[]) {
                    Ok(menu) => menu,
                    Err(e) => {
                        error!("failed to build fallback empty menu: {e}");
                        return;
                    }
                }
            }
        };
        let builder = WebviewWindowBuilder::new(
            &app,
            Uuid::new_v4(),
            WebviewUrl::App(Route::Loader.into()),
        )
        .title(title)
        .auto_resize()
        .disable_drag_drop_handler()
        .visible(cfg!(debug_assertions))
        .focused(true)
        .menu(empty_menu);

        build_window(builder, "main clone");
    });
}

/// Shared spawn tail: build with the macOS title-bar treatment, logging failures.
fn build_window<R: Runtime>(builder: WebviewWindowBuilder<R, AppHandle<R>>, what: &str) {
    #[cfg(not(target_os = "macos"))]
    if let Err(e) = builder.build() {
        error!("failed to open {what} window: {e}");
    }
    #[cfg(target_os = "macos")]
    if let Err(e) = builder
        .title_bar_style(tauri::TitleBarStyle::Visible)
        .build()
    {
        error!("failed to open {what} window: {e}");
    }
}
