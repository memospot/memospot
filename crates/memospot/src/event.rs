//! Tauri event handler.
use std::str::FromStr;
use std::sync::LazyLock;

use crate::memos;
use crate::memos_version::MemosVersionStore;
use crate::menu;
use crate::menu::MainMenu;
use crate::menu::build_empty;
use crate::route::Route;
use crate::runtime_config::RuntimeConfig;
use crate::updater;
use crate::window::Window;
use crate::window_ext::WebviewWindowExt;
use anyhow::{Error, Result, bail};
use dialog::error_dialog;
use log::info;
use log::{debug, error};
#[cfg(not(debug_assertions))]
use tauri::Url;
use tauri::WebviewUrl;
use tauri::WebviewWindow;
use tauri::WebviewWindowBuilder;
use tauri::WindowEvent;
use tauri::{AppHandle, Manager, RunEvent, Runtime, async_runtime};
use tauri_plugin_opener::OpenerExt;
use uuid::Uuid;

pub static PREVENT_EXIT: LazyLock<std::sync::Mutex<bool>> =
    LazyLock::new(|| std::sync::Mutex::new(true));

/// Handles Tauri events.
pub fn handle_run_events(app: &AppHandle, run_event: RunEvent) {
    match run_event {
        RunEvent::Exit => handle_exit_event(app),
        RunEvent::ExitRequested { .. } => handle_exit_requested_event(app, run_event),
        RunEvent::MenuEvent { .. } => handle_menu_event(app, run_event)
            .unwrap_or_else(|e| error!("failed to handle menu event: {e}")),
        RunEvent::WindowEvent { .. } => handle_window_event(app, run_event),
        _ => {}
    }
}

/// Handles the [`RunEvent::Exit`] event.
///
/// Not the recommended way to run events on exit, but it's the only
/// thing that works when closing the app via the dock on macOS.
///
/// See `handle_exit_requested_event` for details.
fn handle_exit_event<R: Runtime>(app: &AppHandle<R>) {
    debug!("RunEvent::Exit triggered");
    on_exit_cleanup(app)
}

/// Handles the `RunEvent::ExitRequested` event.
///
/// On macOS, this only triggers when closing the window with the X button.
///
/// To work around this, the built-in `quit()` menu action was replaced with
/// a custom `CmdOrCtrl+Q` binding that calls `app.exit(0)`, ensuring both
/// the shortcut and menu option trigger this event.
///
/// Closing via the dock still skips this event; in that case, we rely on
/// [`RunEvent::Exit`], which behaves correctly.
fn handle_exit_requested_event<R: Runtime>(app: &AppHandle<R>, run_event: RunEvent) {
    let RunEvent::ExitRequested { api, .. } = run_event else {
        return;
    };

    // Keep the event loop running even if all windows are closed to run cleanup code.
    api.prevent_exit();
    debug!("RunEvent::ExitRequested triggered");
    on_exit_cleanup(app)
}

fn on_exit_cleanup<R: Runtime>(app: &AppHandle<R>) {
    debug!("running before exit cleanup code…");

    #[cfg(not(debug_assertions))]
    let final_config = RuntimeConfig::from_global_store();
    #[cfg(debug_assertions)]
    let mut final_config = RuntimeConfig::from_global_store();

    #[cfg(debug_assertions)]
    {
        // Restore previous mode and port.
        final_config.yaml.memos.mode = final_config.__yaml__.memos.mode.clone();
        final_config.yaml.memos.port = final_config.__yaml__.memos.port;
    }

    if final_config.yaml != final_config.__yaml__ {
        info!("configuration has changed. Saving…");
        async_runtime::block_on(async {
            let config_path = final_config.paths.memospot_config_file;
            if let Err(e) = final_config.yaml.save_to_file(&config_path).await {
                error_dialog!(
                    "Failed to save config file:\n`{}`\n\n{}",
                    config_path.display(),
                    e
                );
            }
        })
    }

    async_runtime::block_on(async move {
        memos::shutdown().await;
        *PREVENT_EXIT.lock().unwrap() = false;
    });

    app.get_webview_window(Window::Main.into())
        .map(|w| w.hide().ok());
    loop {
        if !*PREVENT_EXIT.lock().unwrap() {
            debug!("finished pre-exit cleanup");
            break;
        }
    }
    app.get_webview_window(Window::Main.into())
        .map(|w| w.close().ok());
    info!("Memospot closed.");

    app.cleanup_before_exit();
    std::process::exit(0);
}

/// Handles main menu events defined in [`menu::build`].
fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, run_event: RunEvent) -> Result<(), Error> {
    let RunEvent::MenuEvent(menu_event, ..) = run_event else {
        bail!("expected MenuEvent");
    };
    let event_id = menu_event.id().0.parse::<usize>()?;
    let Some(action) = MainMenu::from_repr(event_id) else {
        bail!("unrecognized menu action for event id #{event_id}");
    };
    let Some(main_window) = app.get_webview_window(Window::Main.into()) else {
        bail!("main window not found");
    };
    let empty_menu = build_empty(app)?;

    let open_link = |url| {
        app.opener().open_url(url, None::<&str>).ok();
    };

    match action {
        MainMenu::AppSettings => {
            let handle_ = app.clone();
            async_runtime::spawn(async move {
                let new_window = WebviewWindowBuilder::new(
                    &handle_,
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

                #[cfg(not(target_os = "macos"))]
                new_window.build().ok();
                #[cfg(target_os = "macos")]
                new_window
                    .title_bar_style(tauri::TitleBarStyle::Visible)
                    .build()
                    .ok();
            });
        }
        MainMenu::AppBrowseDataDirectory => {
            let config = RuntimeConfig::from_global_store();
            app.opener().open_url(
                config.paths.memospot_data.to_string_lossy().to_string(),
                None::<&str>,
            )?;
        }
        MainMenu::AppOpenInBrowser => {
            let config = RuntimeConfig::from_global_store();
            app.opener().open_url(config.memos_url, None::<&str>)?;
        }
        MainMenu::AppUpdate => {
            let app_ = app.clone();
            updater::manual_check(app_);
        }
        MainMenu::AppQuit => {
            app.exit(0);
        }

        MainMenu::ViewNewWindow => {
            let main_title = main_window.title().unwrap_or_default();
            let handle_ = app.clone();
            async_runtime::spawn(async move {
                let uuid = Uuid::new_v4();
                let builder = WebviewWindowBuilder::new(
                    &handle_,
                    uuid,
                    WebviewUrl::App(Route::Loader.into()),
                )
                .title(main_title)
                .auto_resize()
                .disable_drag_drop_handler()
                .visible(cfg!(debug_assertions))
                .focused(true)
                .menu(empty_menu);
                #[cfg(not(target_os = "macos"))]
                builder.build().ok();
                #[cfg(target_os = "macos")]
                builder
                    .title_bar_style(tauri::TitleBarStyle::Visible)
                    .build()
                    .ok();
            });
        }

        #[cfg(any(debug_assertions, feature = "devtools"))]
        MainMenu::ViewDevTools => {
            main_window.open_devtools();
        }
        MainMenu::ViewHideMenuBar => {
            main_window.remove_menu()?;
        }
        MainMenu::ViewRefresh => {
            main_window
                .url()
                .map(|mut url| {
                    url.set_path("/");
                    main_window.navigate(url)
                })
                .ok();
        }
        MainMenu::ViewReload => {
            let menu = menu::build(app)?;
            app.set_menu(menu)?;

            #[cfg(debug_assertions)]
            let url = app.config().build.dev_url.clone();
            #[cfg(not(debug_assertions))]
            let url = Url::parse("tauri://localhost").ok();

            url.map(|u| main_window.navigate(u).ok());
        }
        MainMenu::HelpMemospotDocumentation => {
            open_link("https://memospot.github.io/");
        }
        MainMenu::HelpMemospotReleaseNotes => {
            let version = app.package_info().version.clone();
            let url = format!("https://github.com/memospot/memospot/releases/tag/v{version}",);
            open_link(url.as_str());
        }
        MainMenu::HelpMemospotReportIssue => {
            open_link("https://github.com/memospot/memospot/issues/new");
        }
        MainMenu::HelpMemosDocumentation => {
            open_link("https://usememos.com/docs");
        }
        MainMenu::HelpMemosReleaseNotes => {
            let current_version = MemosVersionStore::get();
            let changelog_version = current_version.replace(".", "-");
            let url = format!("https://www.usememos.com/changelog/{changelog_version}",);
            open_link(url.as_str());
        }
        _ => {
            error!("unhandled event: {}", menu_event.id().0)
        }
    }
    Ok(())
}

fn handle_window_event<R: Runtime>(app: &AppHandle<R>, run_event: RunEvent)
where
    WebviewWindow<R>: WebviewWindowExt,
{
    let RunEvent::WindowEvent { label, event, .. } = run_event else {
        return;
    };

    let Ok(window) = Window::from_str(&label) else {
        // Main window duplicates use UUID as label, and
        // they don't need to be handled individually.
        return;
    };

    match window {
        Window::Main => {
            match event {
                WindowEvent::Resized { .. } | WindowEvent::Moved { .. } => {
                    if let Some(w) = app.get_webview_window(Window::Main.into()) {
                        w.persist_window_state();
                    }
                }
                WindowEvent::CloseRequested { .. } => {
                    // Close all windows except `main` itself.
                    app.webview_windows()
                        .into_iter()
                        .skip(1)
                        .for_each(|(_, w)| {
                            w.close().ok();
                        });
                }
                _ => {
                    // Ignore other events.
                }
            }
        }
        _ => {
            // Currently, only `main` window events matter.
        }
    }
}
