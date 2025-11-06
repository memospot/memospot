use crate::runtime_config::RuntimeConfig;
use tauri::WebviewWindow;
use tauri_utils::config::WindowConfig;
pub trait WebviewWindowExt {
    fn persist_window_state(&self);
}
impl WebviewWindowExt for WebviewWindow {
    /// Store the following Window attributes to the global store:
    ///
    /// - maximized
    /// - width
    /// - height
    /// - x
    /// - y
    fn persist_window_state(&self) {
        let mut config = RuntimeConfig::from_global_store();
        let window = &mut config.yaml.memospot.window;

        window.maximized = Some(self.is_maximized().unwrap_or_default());
        window.width = Some(self.inner_size().unwrap_or_default().width);
        window.height = Some(self.outer_size().unwrap_or_default().height);
        window.x = Some(self.outer_position().unwrap_or_default().x);
        window.y = Some(self.outer_position().unwrap_or_default().y);

        RuntimeConfig::to_global_store(&config);
    }
}

pub trait WindowConfigExt {
    fn restore_window_state(self) -> WindowConfig;
}
impl WindowConfigExt for WindowConfig {
    /// Restore the following Window attributes from the global store into a WindowConfig object:
    ///
    /// - center
    /// - fullscreen
    /// - maximized
    /// - resizable
    /// - width
    /// - height
    /// - x
    /// - y
    fn restore_window_state(mut self) -> WindowConfig {
        let config = RuntimeConfig::from_global_store();
        let window = &config.yaml.memospot.window;

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
