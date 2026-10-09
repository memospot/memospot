//! Coalesced runtime-owned window-state merging.
//!
//! Window resize/move events fire far more often than the configuration
//! should be rewritten. [`WindowStateQueue`] keeps only the latest pending
//! [`WindowState`], merges it through the store's serialized
//! read-modify-write path, and drains before shutdown persistence.

use crate::runtime_config::ConfigStore;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Runtime-owned window state queued from the Tauri event loop.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowState {
    pub maximized: bool,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
}

/// Coalescing queue for runtime-owned window state.
///
/// Owns a [`ConfigStore`] clone and merges through its
/// `update_runtime_owned_fields`, so window updates serialize with
/// settings and locale writes instead of clobbering them.
#[derive(Clone)]
pub struct WindowStateQueue {
    store: ConfigStore,
    pending: Arc<Mutex<Option<WindowState>>>,
    scheduled: Arc<AtomicBool>,
    notify: Arc<tokio::sync::Notify>,
    /// Last maximized state observed on the live window. `maximized` is only
    /// written when the window actually toggles, so routine resize/move events
    /// (frequent on GNOME) don't clobber the preference saved from settings.
    observed_maximized: Arc<AtomicBool>,
}

impl WindowStateQueue {
    pub fn new(store: ConfigStore, maximized: bool) -> Self {
        Self {
            store,
            pending: Arc::new(Mutex::new(None)),
            scheduled: Arc::new(AtomicBool::new(false)),
            notify: Arc::new(tokio::sync::Notify::new()),
            observed_maximized: Arc::new(AtomicBool::new(maximized)),
        }
    }

    /// Queue the latest runtime-owned window state without blocking the event loop.
    pub fn queue(&self, window_state: WindowState) {
        *self.pending.lock().expect("window state lock poisoned") = Some(window_state);

        if self.scheduled.swap(true, Ordering::AcqRel) {
            return;
        }

        let queue = self.clone();
        tauri::async_runtime::spawn(async move {
            queue.process().await;
        });
    }

    async fn process(&self) {
        loop {
            let window_state = self
                .pending
                .lock()
                .expect("window state lock poisoned")
                .take();

            let Some(window_state) = window_state else {
                self.scheduled.store(false, Ordering::Release);
                let has_pending = self
                    .pending
                    .lock()
                    .expect("window state lock poisoned")
                    .is_some();
                if has_pending && !self.scheduled.swap(true, Ordering::AcqRel) {
                    continue;
                }
                self.notify.notify_waiters();
                return;
            };

            let maximized_toggled = self
                .observed_maximized
                .swap(window_state.maximized, Ordering::AcqRel)
                != window_state.maximized;
            self.store
                .update_runtime_owned_fields(|config| {
                    if maximized_toggled {
                        config.memospot.window.maximized = Some(window_state.maximized);
                    }
                    // Keep the restored geometry while maximized. A screen-sized
                    // window gets auto-maximized by GNOME on the next start, which
                    // would override `maximized: false`.
                    if window_state.maximized {
                        return;
                    }
                    config.memospot.window.width = Some(window_state.width);
                    config.memospot.window.height = Some(window_state.height);
                    config.memospot.window.x = Some(window_state.x);
                    config.memospot.window.y = Some(window_state.y);
                })
                .await;
        }
    }

    /// Wait until queued runtime-owned window updates have been merged.
    pub async fn flush(&self) {
        loop {
            let notified = self.notify.notified();
            let has_pending = self
                .pending
                .lock()
                .expect("window state lock poisoned")
                .is_some();
            let is_scheduled = self.scheduled.load(Ordering::Acquire);
            if !has_pending && !is_scheduled {
                return;
            }
            notified.await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime_config::ConfigStore;
    use config::Config;
    use serde_json::json;
    use tempfile::TempDir;

    fn store_with(dir: &TempDir, current: Config, initial: Config) -> ConfigStore {
        ConfigStore::new(current, initial, dir.path().join("memospot.yaml"))
    }

    fn default_store(dir: &TempDir) -> ConfigStore {
        let config = Config::default();
        store_with(dir, config.clone(), config)
    }

    fn queue_for(store: &ConfigStore) -> WindowStateQueue {
        WindowStateQueue::new(
            store.clone(),
            store
                .snapshot()
                .current
                .memospot
                .window
                .maximized
                .unwrap_or_default(),
        )
    }

    fn patch(path: &str, value: serde_json::Value) -> json_patch::Patch {
        serde_json::from_value(json!([{ "op": "replace", "path": path, "value": value }]))
            .expect("test patch should be valid")
    }

    fn saved_window_state(store: &ConfigStore) -> WindowState {
        let window = store.snapshot().current.memospot.window.clone();
        WindowState {
            maximized: window.maximized.unwrap_or_default(),
            width: window.width.unwrap_or_default(),
            height: window.height.unwrap_or_default(),
            x: window.x.unwrap_or_default(),
            y: window.y.unwrap_or_default(),
        }
    }

    #[tokio::test]
    async fn queued_window_updates_are_flushed_before_persistence() {
        // GIVEN a store with a window-state queue.
        let dir = TempDir::new().expect("tempdir");
        let store = default_store(&dir);

        // WHEN a window state is queued and flushed THEN it is merged.
        let queue = queue_for(&store);
        queue.queue(WindowState {
            maximized: false,
            width: 1440,
            height: 900,
            x: 42,
            y: 24,
        });
        queue.flush().await;

        // THEN the geometry is in the current configuration.
        let snapshot = store.snapshot();
        let window = &snapshot.current.memospot.window;
        assert_eq!(window.maximized, Some(false));
        assert_eq!(window.width, Some(1440));
        assert_eq!(window.height, Some(900));
        assert_eq!(window.x, Some(42));
        assert_eq!(window.y, Some(24));
    }

    #[tokio::test]
    async fn window_events_keep_saved_maximized_preference_and_restored_geometry() {
        // GIVEN a maximized store with a window-state queue.
        let dir = TempDir::new().expect("tempdir");
        let mut config = Config::default();
        config.memospot.window.maximized = Some(true);
        let store = store_with(&dir, config.clone(), config);
        let maximized = WindowState {
            maximized: true,
            width: 3904,
            height: 2304,
            x: 0,
            y: 0,
        };
        let restored = WindowState {
            maximized: false,
            width: 1440,
            height: 900,
            x: 42,
            y: 24,
        };

        // WHEN settings unmaximize THEN window events without a toggle
        // keep the saved preference and restored geometry.
        // NOTE: the queue is created before the patch, as in production
        // (startup construction seeds the observed state; later settings
        // patches intentionally do not reseed it).
        let queue = queue_for(&store);
        store
            .apply_patch_and_persist(&patch("/memospot/window/maximized", json!(false)))
            .await
            .expect("patch should succeed");
        let before = saved_window_state(&store);
        queue.queue(maximized);
        queue.flush().await;
        assert_eq!(saved_window_state(&store), before);

        // WHEN a restore-then-maximize sequence arrives THEN the toggle
        // writes maximized while geometry stays restored.
        queue.queue(restored);
        queue.flush().await;
        queue.queue(maximized);
        queue.flush().await;
        assert_eq!(
            saved_window_state(&store),
            WindowState {
                maximized: true,
                ..restored
            }
        );
    }
}
