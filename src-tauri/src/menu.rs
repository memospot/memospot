use crate::memos;
use crate::runtime_config::RuntimeConfig;
use dialog::{confirm_dialog, info_dialog, MessageType};

use crate::fl;
use log::{debug, error, warn};
use std::convert::AsRef;
use strum_macros::AsRefStr;
use strum_macros::FromRepr;
#[cfg(target_os = "macos")]
use tauri::menu::AboutMetadata;
use tauri::{
    async_runtime,
    menu::{Menu, MenuEvent, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder},
    AppHandle, Manager, Runtime,
};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_updater::UpdaterExt;
use tokio::time::{self, Duration, Instant};
use url::Url;

#[derive(AsRefStr, FromRepr, Clone, Copy)]
enum MainMenu {
    #[strum(serialize = "appmenu")]
    App,
    #[strum(serialize = "appmenu-settings")]
    AppSettings,
    #[strum(serialize = "appmenu-browse-data-directory")]
    AppBrowseDataDirectory,
    #[strum(serialize = "appmenu-check-for-updates")]
    AppUpdate,
    #[strum(serialize = "appmenu-quit")]
    AppQuit,
    #[strum(serialize = "viewmenu")]
    View,
    #[strum(serialize = "viewmenu-developer-tools")]
    ViewDevTools,
    #[strum(serialize = "viewmenu-hide-menu-bar")]
    ViewHideMenuBar,
    #[strum(serialize = "viewmenu-refresh")]
    ViewRefresh,
    #[strum(serialize = "viewmenu-reload-view")]
    ViewReload,
    #[strum(serialize = "windowmenu")]
    Window,
    #[strum(serialize = "helpmenu")]
    Help,
    #[strum(serialize = "helpmenu-memospot-version")]
    HelpMemospotVersion,
    #[strum(serialize = "helpmenu-documentation")]
    HelpMemospotDocumentation,
    #[strum(serialize = "helpmenu-release-notes")]
    HelpMemospotReleaseNotes,
    #[strum(serialize = "helpmenu-report-issue")]
    HelpMemospotReportIssue,
    #[strum(serialize = "helpmenu-memos-version")]
    HelpMemosVersion,
    #[strum(serialize = "helpmenu-documentation")]
    HelpMemosDocumentation,
    #[strum(serialize = "helpmenu-release-notes")]
    HelpMemosReleaseNotes,
}
impl MainMenu {
    pub fn index(&self) -> usize {
        *self as usize
    }
}

/// Update menu after Memos version is known.
///
/// Display current Memos version in the help menu.
pub fn update_with_memos_version<R: Runtime>(handle: &AppHandle<R>) {
    const INTERVAL_MS: u64 = 100;
    const TIMEOUT_MS: u128 = 15000;

    let handle_ = handle.clone();
    async_runtime::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(INTERVAL_MS));
        let time_start = Instant::now();

        loop {
            interval.tick().await;
            if time_start.elapsed().as_millis() > TIMEOUT_MS {
                debug!(
                    "Unable to set Memos version in Help menu. Timed out ({}ms).",
                    TIMEOUT_MS
                );
                break;
            }
            if !memos::get_version().is_empty() {
                break;
            }
        }

        let Some(main_window) = handle_.get_webview_window("main") else {
            error!("Unable to set Memos version in menu. Main window not found.");
            return;
        };

        // Find and update the Memos version in the Help menu.
        if let Some(menu) = main_window.menu() {
            let version_text = format!("Memos v{}", memos::get_version());

            menu.items()
                .iter()
                .flat_map(|item| item.iter())
                .filter_map(|menu| menu.as_submenu())
                .find_map(|submenu| {
                    submenu
                        .get(&MainMenu::HelpMemosVersion.index().to_string())
                        .and_then(|entry| entry.as_menuitem().cloned())
                })
                .map(|menuitem| menuitem.set_text(version_text));
        }
    });
}

pub fn build_empty<R: Runtime>(handle: &AppHandle<R>) -> tauri::Result<tauri::menu::Menu<R>> {
    Menu::with_items(handle, &[])
}

pub fn build<R: Runtime>(handle: &AppHandle<R>) -> tauri::Result<tauri::menu::Menu<R>> {
    let config = RuntimeConfig::from_global_store();
    if config.yaml.memospot.window.hide_menu_bar == Some(true) {
        return build_empty(handle);
    }

    let check_for_updates = MenuItemBuilder::with_id(
        MainMenu::AppUpdate.index(),
        fl(MainMenu::AppUpdate.as_ref()),
    )
    .build(handle)?;

    let settings = MenuItemBuilder::new(fl(MainMenu::AppSettings.as_ref()))
        .id(MainMenu::AppSettings.index())
        .accelerator("CmdOrCtrl+,")
        .build(handle)?;

    let browse_data_directory =
        MenuItemBuilder::new(fl(MainMenu::AppBrowseDataDirectory.as_ref()))
            .id(MainMenu::AppBrowseDataDirectory.index())
            .accelerator("CmdOrCtrl+D")
            .build(handle)?;

    #[cfg(target_os = "macos")]
    let app_name = handle.config().product_name.clone().unwrap_or_default();

    #[cfg(target_os = "macos")]
    let mac_menu = &SubmenuBuilder::new(handle, app_name)
        .about(Some(AboutMetadata::default()))
        .separator()
        .item(&settings)
        .item(&browse_data_directory)
        .item(&check_for_updates)
        .separator()
        .services()
        .separator()
        .hide()
        .hide_others()
        .show_all()
        .separator()
        .quit()
        .build()?;

    #[cfg(not(target_os = "macos"))]
    let app_menu = &SubmenuBuilder::new(handle, fl(MainMenu::App.as_ref()))
        .items(&[
            &settings,
            &browse_data_directory,
            &check_for_updates,
            &PredefinedMenuItem::separator(handle)?,
            &MenuItemBuilder::with_id(
                MainMenu::AppQuit.index(),
                fl(MainMenu::AppQuit.as_ref()),
            )
            .accelerator("CmdOrCtrl+W")
            .build(handle)?,
        ])
        .build()?;

    let view_menu = &SubmenuBuilder::new(handle, fl(MainMenu::View.as_ref()))
        .items(&[
            #[cfg(target_os = "macos")]
            &PredefinedMenuItem::fullscreen(handle, None)?,
            #[cfg(any(debug_assertions, feature = "devtools"))]
            &MenuItemBuilder::with_id(
                MainMenu::ViewDevTools.index(),
                fl(MainMenu::ViewDevTools.as_ref()),
            )
            .accelerator("CmdOrCtrl+Shift+C")
            .build(handle)?,
            #[cfg(not(target_os = "macos"))]
            &MenuItemBuilder::with_id(
                MainMenu::ViewHideMenuBar.index(),
                fl(MainMenu::ViewHideMenuBar.as_ref()),
            )
            .accelerator("CmdOrCtrl+H")
            .build(handle)?,
            &MenuItemBuilder::with_id(
                MainMenu::ViewRefresh.index(),
                fl(MainMenu::ViewRefresh.as_ref()),
            )
            .accelerator("F5")
            .build(handle)?,
            &MenuItemBuilder::with_id(
                MainMenu::ViewReload.index(),
                fl(MainMenu::ViewReload.as_ref()),
            )
            .accelerator("CmdOrCtrl+R")
            .build(handle)?,
        ])
        .build()?;

    #[cfg(target_os = "macos")]
    let window_menu = &SubmenuBuilder::new(handle, fl(MainMenu::Window.as_ref()))
        .items(&[
            &PredefinedMenuItem::minimize(handle, None)?,
            &PredefinedMenuItem::maximize(handle, None)?,
            &PredefinedMenuItem::separator(handle)?,
            &PredefinedMenuItem::close_window(handle, None)?,
        ])
        .build()?;

    let help_menu = &SubmenuBuilder::new(handle, fl(MainMenu::Help.as_ref()))
        .item(
            &MenuItemBuilder::with_id(
                MainMenu::HelpMemospotVersion.index(),
                format!("Memospot v{}", handle.package_info().version),
            )
            .enabled(false)
            .build(handle)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id(
                MainMenu::HelpMemospotDocumentation.index(),
                fl(MainMenu::HelpMemospotDocumentation.as_ref()),
            )
            .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id(
                MainMenu::HelpMemospotReleaseNotes.index(),
                fl(MainMenu::HelpMemospotReleaseNotes.as_ref()),
            )
            .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id(
                MainMenu::HelpMemospotReportIssue.index(),
                fl(MainMenu::HelpMemospotReportIssue.as_ref()),
            )
            .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id(
                MainMenu::HelpMemosVersion.index(),
                format!("Memos v{}", memos::get_version()),
            )
            .enabled(false)
            .build(handle)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id(
                MainMenu::HelpMemosDocumentation.index(),
                fl(MainMenu::HelpMemosDocumentation.as_ref()),
            )
            .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id(
                MainMenu::HelpMemosReleaseNotes.index(),
                fl(MainMenu::HelpMemosReleaseNotes.as_ref()),
            )
            .build(handle)?,
        )
        .build()?;

    #[cfg(target_os = "macos")]
    let menu = Menu::with_items(handle, &[mac_menu, view_menu, window_menu, help_menu])?;

    #[cfg(not(target_os = "macos"))]
    let menu = Menu::with_items(handle, &[app_menu, view_menu, help_menu])?;

    Ok(menu)
}

pub fn handle_event<R: Runtime>(handle: &AppHandle<R>, event: MenuEvent) {
    let mut webview = handle.get_webview_window("main").unwrap();
    let open_link = |url| {
        handle.opener().open_url(url, None::<&str>).ok();
    };

    #[cfg(debug_assertions)]
    debug!("menu event: {:?}", event);

    let Ok(event_id) = event.id().0.parse::<usize>() else {
        return;
    };

    match MainMenu::from_repr(event_id).unwrap() {
        MainMenu::AppQuit => {
            handle.exit(0);
        }
        MainMenu::AppBrowseDataDirectory => {
            let config = RuntimeConfig::from_global_store();
            handle
                .opener()
                .open_url(
                    config.paths.memospot_data.to_string_lossy().to_string(),
                    None::<&str>,
                )
                .ok();
        }
        MainMenu::AppSettings => {
            let handle_ = handle.clone();
            tauri::async_runtime::spawn(async move {
                let window_builder = tauri::WebviewWindowBuilder::new(
                    &handle_,
                    "settings",
                    tauri::WebviewUrl::App("/settings".into()),
                )
                .title(fl(MainMenu::AppSettings.as_ref()).replace("&", ""))
                .center()
                .min_inner_size(800.0, 600.0)
                .inner_size(1160.0, 720.0)
                .disable_drag_drop_handler()
                .visible(cfg!(debug_assertions))
                .focused(true)
                .menu(build_empty(&handle_).unwrap());

                #[cfg(not(target_os = "macos"))]
                window_builder.build().ok();
                #[cfg(target_os = "macos")]
                window_builder
                    .title_bar_style(tauri::TitleBarStyle::Visible)
                    .build()
                    .ok();
            });
        }
        MainMenu::AppUpdate => {
            let handle_ = handle.clone();
            tauri::async_runtime::spawn(async move {
                if let Some(update) = handle_.updater().unwrap().check().await.unwrap() {
                    let user_confirmed = confirm_dialog(
                        fl!("dialog-update-title").as_str(),
                        fl!("dialog-update-message", version = update.version).as_str(),
                        MessageType::Info,
                    );
                    if user_confirmed {
                        handle_
                            .opener()
                            .open_url(update.download_url.as_str(), None::<&str>)
                            .ok();
                    } else {
                        warn!("User declined update download.");
                    }
                } else {
                    info_dialog(fl!("dialog-update-no-update").as_str());
                }
            });
        }
        #[cfg(any(debug_assertions, feature = "devtools"))]
        MainMenu::ViewDevTools => {
            webview.open_devtools();
        }
        MainMenu::ViewHideMenuBar => {
            if let Some(main_window) = handle.get_webview_window("main") {
                main_window.remove_menu().ok();
            }
        }
        MainMenu::ViewRefresh => {
            let url = webview.url().unwrap().join("./").unwrap();
            webview.navigate(url).ok();
        }
        MainMenu::ViewReload => {
            handle.set_menu(build(handle).unwrap()).ok();
            let url = Url::parse(if cfg!(debug_assertions) {
                "http://localhost:1420" // Same as build.devUrl in Tauri.toml.
            } else {
                "tauri://localhost"
            })
            .unwrap();
            webview.navigate(url).ok();
        }
        MainMenu::HelpMemospotDocumentation => {
            open_link("https://memospot.github.io/");
        }
        MainMenu::HelpMemospotReleaseNotes => {
            let url = format!(
                "https://github.com/memospot/memospot/releases/tag/v{}",
                handle.package_info().version
            );
            open_link(url.as_str());
        }
        MainMenu::HelpMemospotReportIssue => {
            open_link("https://github.com/memospot/memospot/issues/new");
        }
        MainMenu::HelpMemosDocumentation => {
            open_link("https://usememos.com/docs");
        }
        MainMenu::HelpMemosReleaseNotes => {
            let url = format!(
                "https://www.usememos.com/changelog/{}",
                memos::get_version().replace(".", "-")
            );
            open_link(url.as_str());
        }
        _ => {
            error!("unhandled menu event: {}", event.id().0)
        }
    }
}
