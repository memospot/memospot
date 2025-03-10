use crate::fl;
use crate::memos;
use crate::runtime_config::RuntimeConfig;
use dialog::{confirm_dialog, info_dialog, MessageType};
use log::{debug, error, warn};
use std::convert::AsRef;
use strum_macros::AsRefStr;
use strum_macros::FromRepr;
#[cfg(target_os = "macos")]
use tauri::menu::AboutMetadata;
use tauri::menu::MenuId;
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
    /// Get the MenuId for the item.
    pub fn id(&self) -> MenuId {
        let id = *self as u8;
        MenuId::new(id.to_string())
    }

    /// Get the localized text for the item.
    pub fn text(self) -> String {
        fl(self.as_ref())
    }
}

/// Build an empty menu.
pub fn build_empty<R: Runtime>(handle: &AppHandle<R>) -> tauri::Result<tauri::menu::Menu<R>> {
    Menu::with_items(handle, &[])
}

/// Build the main menu.
pub fn build<R: Runtime>(handle: &AppHandle<R>) -> tauri::Result<tauri::menu::Menu<R>> {
    let config = RuntimeConfig::from_global_store();
    if config.yaml.memospot.window.hide_menu_bar == Some(true) {
        return build_empty(handle);
    }

    let settings = MenuItemBuilder::new(MainMenu::AppSettings.text())
        .id(MainMenu::AppSettings.id())
        .accelerator("CmdOrCtrl+,")
        .build(handle)?;

    let browse_data_directory = MenuItemBuilder::new(MainMenu::AppBrowseDataDirectory.text())
        .id(MainMenu::AppBrowseDataDirectory.id())
        .accelerator("CmdOrCtrl+D")
        .build(handle)?;

    let check_for_updates =
        MenuItemBuilder::with_id(MainMenu::AppUpdate.id(), MainMenu::AppUpdate.text())
            .build(handle)?;

    let quit = MenuItemBuilder::with_id(MainMenu::AppQuit.id(), MainMenu::AppQuit.text())
        .accelerator("CmdOrCtrl+W")
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
        .item(&quit)
        .build()?;

    #[cfg(not(target_os = "macos"))]
    let app_menu = &SubmenuBuilder::new(handle, MainMenu::App.text())
        .items(&[
            &settings,
            &browse_data_directory,
            &check_for_updates,
            &PredefinedMenuItem::separator(handle)?,
            &quit,
        ])
        .build()?;

    let view_menu = &SubmenuBuilder::new(handle, MainMenu::View.text())
        .items(&[
            #[cfg(target_os = "macos")]
            &PredefinedMenuItem::fullscreen(handle, None)?,
            #[cfg(any(debug_assertions, feature = "devtools"))]
            &MenuItemBuilder::with_id(
                MainMenu::ViewDevTools.id(),
                MainMenu::ViewDevTools.text(),
            )
            .accelerator("CmdOrCtrl+Shift+C")
            .build(handle)?,
            #[cfg(not(target_os = "macos"))]
            &MenuItemBuilder::with_id(
                MainMenu::ViewHideMenuBar.id(),
                MainMenu::ViewHideMenuBar.text(),
            )
            .accelerator("CmdOrCtrl+H")
            .build(handle)?,
            &MenuItemBuilder::with_id(MainMenu::ViewRefresh.id(), MainMenu::ViewRefresh.text())
                .accelerator("F5")
                .build(handle)?,
            &MenuItemBuilder::with_id(MainMenu::ViewReload.id(), MainMenu::ViewReload.text())
                .accelerator("CmdOrCtrl+R")
                .build(handle)?,
        ])
        .build()?;

    #[cfg(target_os = "macos")]
    let window_menu = &SubmenuBuilder::new(handle, MainMenu::Window.text())
        .items(&[
            &PredefinedMenuItem::minimize(handle, None)?,
            &PredefinedMenuItem::maximize(handle, None)?,
            &PredefinedMenuItem::separator(handle)?,
            &PredefinedMenuItem::close_window(handle, None)?,
        ])
        .build()?;

    let help_menu = &SubmenuBuilder::new(handle, MainMenu::Help.text())
        .items(&[
            &MenuItemBuilder::with_id(
                MainMenu::HelpMemospotVersion.id(),
                format!("Memospot v{}", handle.package_info().version),
            )
            .enabled(false)
            .build(handle)?,
            &MenuItemBuilder::with_id(
                MainMenu::HelpMemospotDocumentation.id(),
                MainMenu::HelpMemospotDocumentation.text(),
            )
            .build(handle)?,
            &MenuItemBuilder::with_id(
                MainMenu::HelpMemospotReleaseNotes.id(),
                MainMenu::HelpMemospotReleaseNotes.text(),
            )
            .build(handle)?,
            &MenuItemBuilder::with_id(
                MainMenu::HelpMemospotReportIssue.id(),
                MainMenu::HelpMemospotReportIssue.text(),
            )
            .build(handle)?,
            &PredefinedMenuItem::separator(handle)?,
            &MenuItemBuilder::with_id(
                MainMenu::HelpMemosVersion.id(),
                format!("Memos v{}", memos::VersionStore::get()),
            )
            .enabled(false)
            .build(handle)?,
            &MenuItemBuilder::with_id(
                MainMenu::HelpMemosDocumentation.id(),
                MainMenu::HelpMemosDocumentation.text(),
            )
            .build(handle)?,
            &MenuItemBuilder::with_id(
                MainMenu::HelpMemosReleaseNotes.id(),
                MainMenu::HelpMemosReleaseNotes.text(),
            )
            .build(handle)?,
        ])
        .build()?;

    #[cfg(target_os = "macos")]
    let menu = Menu::with_items(handle, &[mac_menu, view_menu, window_menu, help_menu])?;

    #[cfg(not(target_os = "macos"))]
    let menu = Menu::with_items(handle, &[app_menu, view_menu, help_menu])?;

    Ok(menu)
}

pub fn handle_event<R: Runtime>(handle: &AppHandle<R>, event: MenuEvent) {
    #[cfg(debug_assertions)]
    debug!("menu event: {:?}", event);

    let Ok(event_id) = event.id().0.parse::<usize>() else {
        return;
    };

    let webview = handle
        .get_webview_window("main")
        .expect("menu: failed to get webview window");
    let open_link = |url| {
        handle.opener().open_url(url, None::<&str>).ok();
    };

    let Some(action) = MainMenu::from_repr(event_id) else {
        error!("menu: received bad event id");
        return;
    };

    match action {
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
                .title(MainMenu::AppSettings.text().replace("&", ""))
                .center()
                .min_inner_size(800.0, 600.0)
                .inner_size(1160.0, 720.0)
                .disable_drag_drop_handler()
                .visible(cfg!(debug_assertions))
                .focused(true)
                .menu(build_empty(&handle_).expect("failed to build menu"));

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
                let Ok(updater) = handle_.updater() else {
                    return;
                };
                let Ok(check) = updater.check().await else {
                    return;
                };
                if let Some(update) = check {
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
                    info_dialog(fl!("dialog-update-no-update"));
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
            let url = webview
                .url()
                .expect("failed to get url")
                .join("./")
                .expect("failed to join url");
            webview.navigate(url).ok();
        }
        MainMenu::ViewReload => {
            handle
                .set_menu(build(handle).expect("failed to build menu"))
                .ok();
            let url = Url::parse(if cfg!(debug_assertions) {
                "http://localhost:1420" // Same as build.devUrl in Tauri.toml.
            } else {
                "tauri://localhost"
            })
            .expect("failed to parse url");
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
                memos::VersionStore::get().replace(".", "-")
            );
            open_link(url.as_str());
        }
        _ => {
            error!("menu: unhandled event: {}", event.id().0)
        }
    }
}

/// Update menu after Memos version is known.
///
/// Display current Memos version in the help menu.
///
/// * This function must never interrupt the program flow.
pub fn update_with_memos_version<R: Runtime>(handle: &AppHandle<R>) {
    const INTERVAL_MS: u64 = 100;
    const TIMEOUT_MS: u128 = 15_000;

    let handle_ = handle.clone();
    async_runtime::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(INTERVAL_MS));
        let time_start = Instant::now();

        loop {
            interval.tick().await;
            if time_start.elapsed().as_millis() > TIMEOUT_MS {
                debug!(
                    "menu: unable to set Memos version in Help menu. Timed out after {}ms.",
                    TIMEOUT_MS
                );
                break;
            }
            if !memos::VersionStore::get().is_empty() {
                break;
            }
        }

        let Some(main_window) = handle_.get_webview_window("main") else {
            error!("menu: unable to set Memos version in Help menu. Main window not found.");
            return;
        };
        let Some(menu) = main_window.menu() else {
            return;
        };

        // Find and update the Memos version entry in the Help menu.
        let version_text = format!("Memos v{}", memos::VersionStore::get());
        menu.items()
            .iter()
            .flat_map(|item| item.iter())
            .filter_map(|menu| menu.as_submenu())
            .find_map(|submenu| {
                submenu
                    .get(&MainMenu::HelpMemosVersion.id())
                    .and_then(|entry| entry.as_menuitem().cloned())
            })
            .map(|menuitem| menuitem.set_text(version_text));
    });
}
