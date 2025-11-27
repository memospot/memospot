//! Main window menu bar.
//!
//! Events fired here are handled by the [`crate::events::handle_menu_event`] function.
use crate::fl;
use crate::memos_version::MemosVersionStore;
use crate::runtime_config::RuntimeConfig;
use log::{debug, error};
use std::convert::AsRef;
use strum_macros::AsRefStr;
use strum_macros::FromRepr;
#[cfg(target_os = "macos")]
use tauri::menu::AboutMetadata;
use tauri::{
    async_runtime,
    menu::MenuId,
    menu::{Menu, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder},
    AppHandle, Manager, Runtime,
};
use tokio::time::{self, Duration, Instant};

#[derive(AsRefStr, FromRepr, Clone, Copy)]
pub enum MainMenu {
    #[strum(serialize = "appmenu")]
    App,
    #[strum(serialize = "appmenu-browse-data-directory")]
    AppBrowseDataDirectory,
    #[strum(serialize = "appmenu-settings")]
    AppSettings,
    #[strum(serialize = "appmenu-open-in-browser")]
    AppOpenInBrowser,
    #[strum(serialize = "appmenu-quit")]
    AppQuit,
    #[strum(serialize = "appmenu-check-for-updates")]
    AppUpdate,
    #[strum(serialize = "viewmenu")]
    View,
    #[strum(serialize = "viewmenu-new-window")]
    ViewNewWindow,
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
    #[strum(serialize = "helpmenu-documentation")]
    HelpMemospotDocumentation,
    #[strum(serialize = "helpmenu-release-notes")]
    HelpMemospotReleaseNotes,
    #[strum(serialize = "helpmenu-report-issue")]
    HelpMemospotReportIssue,
    #[strum(serialize = "helpmenu-memospot-version")]
    HelpMemospotVersion,
    #[strum(serialize = "helpmenu-documentation")]
    HelpMemosDocumentation,
    #[strum(serialize = "helpmenu-release-notes")]
    HelpMemosReleaseNotes,
    #[strum(serialize = "helpmenu-memos-version")]
    HelpMemosVersion,
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
pub fn build_empty<R: Runtime>(handle: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    Menu::with_items(handle, &[])
}

/// Build the main menu.
pub fn build<R: Runtime>(handle: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let config = RuntimeConfig::from_global_store();
    if config.yaml.memospot.window.hide_menu_bar == Some(true) {
        return build_empty(handle);
    }

    let settings =
        MenuItemBuilder::with_id(MainMenu::AppSettings.id(), MainMenu::AppSettings.text())
            .accelerator("CmdOrCtrl+,")
            .build(handle)?;

    let browse_data_directory = MenuItemBuilder::with_id(
        MainMenu::AppBrowseDataDirectory.id(),
        MainMenu::AppBrowseDataDirectory.text(),
    )
    .accelerator("CmdOrCtrl+D")
    .build(handle)?;

    let check_for_updates =
        MenuItemBuilder::with_id(MainMenu::AppUpdate.id(), MainMenu::AppUpdate.text())
            .build(handle)?;

    let open_in_browser = MenuItemBuilder::with_id(
        MainMenu::AppOpenInBrowser.id(),
        MainMenu::AppOpenInBrowser.text(),
    )
    .accelerator("CmdOrCtrl+B")
    .build(handle)?;

    #[cfg(target_os = "macos")]
    let app_name = handle.config().product_name.clone().unwrap_or_default();

    let quit = MenuItemBuilder::with_id(MainMenu::AppQuit.id(), MainMenu::AppQuit.text())
        .accelerator("CmdOrCtrl+Q")
        .build(handle)?;

    #[cfg(target_os = "macos")]
    let mac_menu = &SubmenuBuilder::new(handle, app_name)
        .about(Some(AboutMetadata::default()))
        .separator()
        .item(&settings)
        .item(&browse_data_directory)
        .item(&check_for_updates)
        .item(&open_in_browser)
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
            &open_in_browser,
            &PredefinedMenuItem::separator(handle)?,
            &quit,
        ])
        .build()?;

    let view_menu = &SubmenuBuilder::new(handle, MainMenu::View.text())
        .items(&[
            &MenuItemBuilder::with_id(
                MainMenu::ViewNewWindow.id(),
                MainMenu::ViewNewWindow.text(),
            )
            .accelerator("CmdOrCtrl+N")
            .build(handle)?,
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
                format!("Memos v{}", MemosVersionStore::get()),
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

/// Update menu after Memos version is known.
///
/// Display current Memos version in the help menu.
///
/// * This function must never interrupt the program flow.
pub fn update_memos_version_entry<R: Runtime>(handle: &AppHandle<R>) {
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
                    "unable to set Memos version in Help menu. Timed out after {TIMEOUT_MS}ms."
                );
                break;
            }
            if !MemosVersionStore::get().is_empty() {
                break;
            }
        }

        let Some(main_window) = handle_.get_webview_window("main") else {
            error!("unable to set Memos version in Help menu. Main window not found.");
            return;
        };
        let Some(menu) = main_window.menu() else {
            return;
        };

        // Find and update the Memos version entry in the Help menu.
        let version_text = format!("Memos v{}", MemosVersionStore::get());
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
