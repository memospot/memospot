use crate::runtime_config::{ConfigStore, WindowState};
use config::Config;
use tauri::WebviewWindow;
use tauri_utils::config::WindowConfig;

pub trait WebviewWindowExt {
    fn persist_window_state(&self, config_store: &ConfigStore);
}
impl WebviewWindowExt for WebviewWindow {
    /// Store the following Window attributes into the managed configuration store:
    ///
    /// - maximized
    /// - width
    /// - height
    /// - x
    /// - y
    ///
    /// The update merges with concurrent settings updates and is persisted
    /// on shutdown rather than immediately.
    fn persist_window_state(&self, config_store: &ConfigStore) {
        // `WindowConfig` restores inner size and position in logical pixels,
        // so store them in the same unit. Physical pixels would grow the
        // window by the scale factor on every restart.
        let scale_factor = self.scale_factor().unwrap_or(1.0);
        let size = self
            .inner_size()
            .unwrap_or_default()
            .to_logical::<u32>(scale_factor);
        let position = self
            .outer_position()
            .unwrap_or_default()
            .to_logical::<i32>(scale_factor);

        config_store.queue_runtime_owned_window_state(WindowState {
            maximized: self.is_maximized().unwrap_or_default(),
            width: size.width,
            height: size.height,
            x: position.x,
            y: position.y,
        });
    }
}

pub trait WindowConfigExt {
    fn restore_window_state(self, config: &Config) -> WindowConfig;
}
impl WindowConfigExt for WindowConfig {
    /// Restore the following Window attributes from the configuration
    /// into a WindowConfig object:
    ///
    /// - center
    /// - fullscreen
    /// - maximized
    /// - resizable
    /// - width
    /// - height
    /// - x
    /// - y
    fn restore_window_state(mut self, config: &Config) -> WindowConfig {
        let window = &config.memospot.window;

        self.center = window.center.unwrap_or_default();
        self.fullscreen = window.fullscreen.unwrap_or_default();
        self.maximized = window.maximized.unwrap_or_default();
        self.resizable = window.resizable.unwrap_or_default();
        self.width = window.width.unwrap_or_default() as f64;
        self.height = window.height.unwrap_or_default() as f64;
        self.x = Some(window.x.unwrap_or_default() as f64);
        self.y = Some(window.y.unwrap_or_default() as f64);
        self
    }
}
