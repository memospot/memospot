//! Application-wide configuration state.
//!
//! [`AppState`] is the single Tauri-managed source of truth for the editable
//! configuration and the immutable runtime context derived from it at startup.

use config::Config;
use json_patch::Patch;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use ts_rs::TS;

/// Runtime paths used throughout the app.
#[derive(TS, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct RuntimePaths {
    /// Memos binary file path.
    pub memos_bin: PathBuf,
    /// Memos data directory path.
    pub memos_data: PathBuf,
    /// Memos database file path.
    ///
    /// File name can be one of:
    /// - memos_prod.db
    /// - memos_demo.db
    pub memos_db_file: PathBuf,
    /// Memospot backup directory path.
    pub memospot_bin: PathBuf,
    /// Memospot configuration file path.
    pub memospot_config_file: PathBuf,
    /// Memospot current working directory path.
    pub memospot_cwd: PathBuf,
    /// Memospot data directory path.
    pub memospot_data: PathBuf,
}

/// Active server connection details of the running process.
#[derive(TS, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ActiveServer {
    /// Memos URL.
    ///
    /// URL always ends with a slash.
    pub url: String,
    /// User-Agent header sent to Memos server.
    pub user_agent: String,
    /// Whether Memospot is managing a local Memos server.
    /// If false, Memospot is using a remote server.
    pub managed: bool,
}

/// Immutable, startup-derived view of the running process.
///
/// Created once during `run()` after the configuration file is loaded and all
/// derived values are calculated. It represents the active process, not every
/// subsequent edit to the current configuration.
#[derive(TS, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct RuntimeContext {
    /// Store paths used throughout the app.
    pub paths: RuntimePaths,
    /// Active server connection details.
    pub active_server: ActiveServer,
    /// Effective Memos server settings for the running process.
    ///
    /// Includes startup resolution, such as a free port, and debug-only
    /// overrides. Edits to the current configuration do not affect these
    /// values until the application restarts.
    pub memos: config::Memos,
}

/// A consistent, immutable view of the configuration store.
#[derive(Debug, PartialEq, Clone)]
pub struct ConfigSnapshot {
    /// The current configuration.
    pub current: Arc<Config>,
    /// The configuration as loaded at startup.
    ///
    /// Named baseline used to decide whether the current configuration
    /// changed and needs to be persisted. Do not modify.
    pub initial: Arc<Config>,
}

/// Result of a successful configuration update.
#[derive(TS, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ConfigUpdateResult {
    /// Whether the update only takes effect after restarting Memospot.
    pub restart_required: bool,
}

/// Internal result of a configuration update.
#[derive(Debug, PartialEq, Clone)]
pub struct ConfigUpdate {
    /// Result exposed to the frontend.
    pub result: ConfigUpdateResult,
    /// Whether the update changed the persisted locale preference.
    pub locale_changed: bool,
}

/// Errors that can occur while updating the managed configuration.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The JSON Patch could not be parsed or applied.
    #[error("invalid configuration patch: {0}")]
    InvalidPatch(String),
    /// The patched configuration is not a valid `Config`.
    #[error("patch produced an invalid configuration: {0}")]
    InvalidConfig(String),
    /// The configuration could not be persisted to disk.
    #[error("failed to persist configuration: {0}")]
    Persistence(String),
}

/// Synchronized configuration store.
///
/// Owns the current configuration and its startup baseline. All
/// mutations pass through one serialized read-modify-write path so that
/// settings updates, locale changes, and window-state updates cannot
/// overwrite one another.
#[derive(Clone)]
pub struct ConfigStore {
    current: Arc<RwLock<Arc<Config>>>,
    initial: Arc<Config>,
    restart_baseline: Arc<RwLock<Arc<Config>>>,
    writer: Arc<tokio::sync::Mutex<()>>,
    config_file: PathBuf,
}

impl ConfigStore {
    /// Create a store for the given current configuration and startup baseline.
    pub fn new(current: Config, initial: Config, config_file: PathBuf) -> Self {
        Self {
            restart_baseline: Arc::new(RwLock::new(Arc::new(current.clone()))),
            current: Arc::new(RwLock::new(Arc::new(current))),
            initial: Arc::new(initial),
            writer: Arc::new(tokio::sync::Mutex::new(())),
            config_file,
        }
    }

    /// Consistent read of the store: the current configuration and
    /// the startup baseline.
    pub fn snapshot(&self) -> ConfigSnapshot {
        ConfigSnapshot {
            current: self.current.read().expect("config lock poisoned").clone(),
            initial: self.initial.clone(),
        }
    }

    /// Validate, persist, and commit a user configuration patch.
    ///
    /// The patch is applied to a candidate, normalized, and classified for
    /// restart impact. Only after persistence succeeds is the candidate
    /// published as the synchronized current configuration, so a failed write
    /// is never reported as a successful update.
    pub async fn apply_patch_and_persist(
        &self,
        patch: &Patch,
    ) -> Result<ConfigUpdate, ConfigError> {
        let _writer = self.writer.lock().await;

        let current = self.snapshot().current;
        let candidate = normalize_config(apply_patch(&current, patch)?);
        let restart_baseline = self
            .restart_baseline
            .read()
            .expect("config lock poisoned")
            .clone();
        let restart_required = restart_baseline.restart_required(&candidate);
        let locale_changed = current.memospot.window.locale != candidate.memospot.window.locale;

        candidate
            .save_to_file(&self.config_file)
            .await
            .map_err(|e| ConfigError::Persistence(e.to_string()))?;

        *self.current.write().expect("config lock poisoned") = Arc::new(candidate);
        Ok(ConfigUpdate {
            result: ConfigUpdateResult { restart_required },
            locale_changed,
        })
    }

    /// Apply a typed, fully type-checked mutation to the configuration.
    ///
    /// Unlike `apply_patch_and_persist`, the field being changed is known at
    /// compile time — no stringly-typed JSON Pointer, no `serde_json::to_value`
    /// / `from_value` round-trip. The closure receives `&mut Config` directly.
    pub async fn update_and_persist<F>(&self, update: F) -> Result<ConfigUpdate, ConfigError>
    where
        F: FnOnce(&mut Config) + Send,
    {
        let _writer = self.writer.lock().await;

        let current = self.snapshot().current;
        let mut candidate = (*current).clone();
        update(&mut candidate);
        candidate = normalize_config(candidate);

        let restart_baseline = self
            .restart_baseline
            .read()
            .expect("config lock poisoned")
            .clone();
        let restart_required = restart_baseline.restart_required(&candidate);
        let locale_changed = current.memospot.window.locale != candidate.memospot.window.locale;

        candidate
            .save_to_file(&self.config_file)
            .await
            .map_err(|e| ConfigError::Persistence(e.to_string()))?;

        *self.current.write().expect("config lock poisoned") = Arc::new(candidate);
        Ok(ConfigUpdate {
            result: ConfigUpdateResult { restart_required },
            locale_changed,
        })
    }

    /// Merge runtime-owned fields into the current configuration.
    ///
    /// Used for in-session fields such as window state. The update is not
    /// persisted immediately; it is written to disk at shutdown by
    /// [`finalize_persistence`](Self::finalize_persistence), which keeps the
    /// deferred dirty state explicit.
    pub async fn update_runtime_owned_fields(&self, update: impl Fn(&mut Config) + Send) {
        let _writer = self.writer.lock().await;

        let mut current = (**self.current.read().expect("config lock poisoned")).clone();
        let mut restart_baseline =
            (**self.restart_baseline.read().expect("config lock poisoned")).clone();
        update(&mut current);
        update(&mut restart_baseline);
        *self.current.write().expect("config lock poisoned") = Arc::new(current);
        *self.restart_baseline.write().expect("config lock poisoned") =
            Arc::new(restart_baseline);
    }

    /// Persist the current configuration if it differs from the startup baseline.
    ///
    /// Called at shutdown after queued window-state updates have drained
    /// (see the shutdown path). The baseline comparison is explicit and
    /// never mutates the current configuration.
    pub async fn finalize_persistence(&self) -> Result<(), ConfigError> {
        let _writer = self.writer.lock().await;
        let snapshot = self.snapshot();
        if snapshot.current != snapshot.initial {
            snapshot
                .current
                .save_to_file(&self.config_file)
                .await
                .map_err(|e| ConfigError::Persistence(e.to_string()))?;
        }
        Ok(())
    }
}

/// Tauri-managed application state.
///
/// The single authoritative access path for the editable configuration and
/// the active runtime context. Commands receive it via `State`, event
/// handlers and owned background tasks access it through `AppHandle`.
#[derive(Clone)]
pub struct AppState {
    /// Immutable runtime context derived at startup.
    pub runtime: RuntimeContext,
    /// Synchronized editable configuration store.
    pub config: ConfigStore,
    /// Memos server version once the readiness probe delivers it.
    ///
    /// Written once by the single readiness task; read by the menu version
    /// entry and the changelog link. [`None`] until delivered.
    pub memos_version: Arc<RwLock<Option<String>>>,
    /// Current webview zoom factor, applied app-wide.
    ///
    /// Written by every zoom path; read back to step from the current level.
    pub zoom_level: Arc<RwLock<f64>>,
    /// Coalescing queue for runtime-owned window state.
    ///
    /// Merges through the configuration store's serialized path and drains
    /// before shutdown persistence.
    pub window_states: crate::window_state::WindowStateQueue,
}

/// Apply a JSON Patch to a configuration, producing a validated candidate.
pub fn apply_patch(config: &Config, patch: &Patch) -> Result<Config, ConfigError> {
    let mut value =
        serde_json::to_value(config).map_err(|e| ConfigError::InvalidConfig(e.to_string()))?;
    json_patch::patch(&mut value, patch)
        .map_err(|e| ConfigError::InvalidPatch(e.to_string()))?;
    serde_json::from_value(value).map_err(|e| ConfigError::InvalidConfig(e.to_string()))
}

/// Normalize a configuration candidate before it is committed.
///
/// Applies the existing demo-mode compatibility rules so persisted values
/// always match the normalized form used at startup.
pub fn normalize_config(mut config: Config) -> Config {
    config.memos.normalize();
    config
}

/// Apply debug-only Memos server mode and port overrides.
///
/// The overrides only affect the running process: they never enter the
/// current configuration, so shutdown compares the current configuration
/// against the startup baseline without restoring fields.
#[cfg(debug_assertions)]
pub fn apply_debug_overrides(memos: &mut config::Memos) {
    // ! `MEMOS_MODE` is retired from v0.26.0. Database is always in `prod` mode unless `MEMOS_DEMO=true` is set.
    // Use Memos in demo mode during development,
    // as it's already seeded with some data.
    memos.mode = Some("demo".to_string());
    memos.demo = Some(true);
    // Use an upper port to use a dedicated WebView cache for development.
    let dev_port = memos.port.unwrap_or_default() + 1;
    memos.port = Some(dev_port.clamp(1, 65535));
}

// Used only by ts-rs to discover exported bindings; never constructed at runtime.
#[allow(dead_code, clippy::large_enum_variant)]
#[derive(TS, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[ts(export)]
enum ExportTSBindings {
    Config(Config),
    ConfigUpdateResult(ConfigUpdateResult),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    fn store_with(dir: &TempDir, current: Config, initial: Config) -> ConfigStore {
        ConfigStore::new(current, initial, dir.path().join("memospot.yaml"))
    }

    fn default_store(dir: &TempDir) -> ConfigStore {
        let config = Config::default();
        store_with(dir, config.clone(), config)
    }

    fn patch(path: &str, value: serde_json::Value) -> Patch {
        serde_json::from_value(json!([{ "op": "replace", "path": path, "value": value }]))
            .expect("test patch should be valid")
    }

    #[test]
    fn fresh_state_starts_with_supplied_configuration_only() {
        let dir = TempDir::new().expect("tempdir");
        let mut current = Config::default();
        current.memospot.window.theme = Some("dark".to_string());
        let store = store_with(&dir, current.clone(), current.clone());

        let snapshot = store.snapshot();
        assert_eq!((*snapshot.current).clone(), current);
        assert_eq!((*snapshot.initial).clone(), current);

        // A fresh instance is not affected by previous tests.
        let other = default_store(&dir);
        assert_eq!(other.snapshot().current.memospot.window.theme, None);
    }

    #[test]
    fn desired_configuration_and_runtime_context_are_separate() {
        // Kept as historical name for test output compatibility.
        let mut current = Config::default();
        current.memos.port = Some(5230);
        let mut runtime_memos = current.memos.clone();
        runtime_memos.port = Some(5231);

        let runtime = RuntimeContext {
            paths: RuntimePaths {
                memos_bin: PathBuf::new(),
                memos_data: PathBuf::new(),
                memos_db_file: PathBuf::new(),
                memospot_bin: PathBuf::new(),
                memospot_config_file: PathBuf::new(),
                memospot_cwd: PathBuf::new(),
                memospot_data: PathBuf::new(),
            },
            active_server: ActiveServer {
                url: "http://localhost:5231/".into(),
                user_agent: "test".into(),
                managed: true,
            },
            memos: runtime_memos,
        };

        // The active runtime keeps the startup port while the current
        // configuration stays untouched.
        assert_eq!(runtime.memos.port, Some(5231));
        assert_eq!(current.memos.port, Some(5230));
        assert_eq!(runtime.active_server.url, "http://localhost:5231/");
    }

    #[cfg(debug_assertions)]
    #[test]
    fn debug_overrides_apply_to_runtime_memos_only() {
        let mut memos = config::Memos::default();
        let original_port = memos.port;

        apply_debug_overrides(&mut memos);

        assert_eq!(memos.mode.as_deref(), Some("demo"));
        assert_eq!(memos.demo, Some(true));
        assert_eq!(memos.port, Some(original_port.unwrap() + 1));
    }

    #[tokio::test]
    async fn baseline_comparison_decides_shutdown_persistence() {
        let dir = TempDir::new().expect("tempdir");
        let config_file = dir.path().join("memospot.yaml");

        // Unchanged current configuration is not persisted.
        let config = Config::default();
        let store = ConfigStore::new(config.clone(), config, config_file.clone());
        store.finalize_persistence().await.expect("finalize");
        assert!(!config_file.exists());

        // Changed current configuration is persisted.
        let mut current = Config::default();
        current.memospot.window.theme = Some("dark".to_string());
        let initial = Config::default();
        let store = ConfigStore::new(current, initial, config_file.clone());
        store.finalize_persistence().await.expect("finalize");
        assert!(config_file.exists());
    }

    #[tokio::test]
    async fn valid_patch_commits_the_complete_candidate() {
        let dir = TempDir::new().expect("tempdir");
        let store = default_store(&dir);

        let mut patch = patch("/memospot/window/theme", json!("dark"));
        patch.0.push(
            serde_json::from_value(
                json!({ "op": "replace", "path": "/memos/port", "value": 9999 }),
            )
            .expect("patch op"),
        );

        let result = store
            .apply_patch_and_persist(&patch)
            .await
            .expect("patch should succeed");

        assert!(result.result.restart_required);
        let snapshot = store.snapshot();
        assert_eq!(
            snapshot.current.memospot.window.theme.as_deref(),
            Some("dark")
        );
        assert_eq!(snapshot.current.memos.port, Some(9999));

        // The full candidate is on disk and round-trips without schema changes.
        let on_disk: Config = serde_saphyr::from_str(
            &std::fs::read_to_string(dir.path().join("memospot.yaml")).expect("config file"),
        )
        .expect("on-disk config should parse");
        assert_eq!(on_disk, *snapshot.current);
    }

    #[tokio::test]
    async fn invalid_patch_leaves_memory_and_disk_unchanged() {
        let dir = TempDir::new().expect("tempdir");
        let store = default_store(&dir);
        let config_file = dir.path().join("memospot.yaml");

        // Unparseable patch operations are rejected before touching state.
        let broken: Result<Patch, _> = serde_json::from_value(json!([{
            "op": "bogus",
            "path": "/memospot/window/theme",
            "value": "dark"
        }]));
        assert!(broken.is_err());

        // Valid patch ops pointing at a nonexistent path.
        let missing_path = patch("/does/not/exist", json!("x"));
        store
            .apply_patch_and_persist(&missing_path)
            .await
            .expect_err("patch should fail");

        // A patch that produces an invalid Config.
        let wrong_type = patch("/memos/port", json!("not-a-port"));
        let error = store
            .apply_patch_and_persist(&wrong_type)
            .await
            .expect_err("patch should fail");
        assert!(matches!(error, ConfigError::InvalidConfig(_)));

        // Memory and disk are untouched by all rejected patches.
        assert_eq!(store.snapshot().current.memospot.window.theme, None);
        assert!(!config_file.exists());
    }

    #[tokio::test]
    async fn persistence_failure_returns_error_without_false_success() {
        let dir = TempDir::new().expect("tempdir");
        // A directory cannot be written as a configuration file.
        let store = ConfigStore::new(
            Config::default(),
            Config::default(),
            dir.path().to_path_buf(),
        );
        let before = store.snapshot().current.clone();

        let error = store
            .apply_patch_and_persist(&patch("/memospot/window/theme", json!("dark")))
            .await
            .expect_err("persistence should fail");

        assert!(matches!(error, ConfigError::Persistence(_)));
        assert_eq!(store.snapshot().current, before);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_updates_to_different_fields_are_all_retained() {
        let dir = TempDir::new().expect("tempdir");
        let store = default_store(&dir);

        let patches = [
            ("/memospot/window/theme", json!("dark")),
            ("/memospot/window/reduce_animation", json!(true)),
            ("/memospot/window/locale", json!("es")),
            ("/memospot/window/hide_menu_bar", json!(true)),
            ("/memospot/window/maximized", json!(true)),
            ("/memos/port", json!(9999)),
            ("/memospot/remote/enabled", json!(true)),
            ("/memospot/updater/enabled", json!(false)),
            ("/memospot/log/enabled", json!(true)),
            ("/memospot/migrations/enabled", json!(false)),
            ("/memospot/backups/enabled", json!(false)),
            ("/memospot/env/enabled", json!(true)),
        ];

        let mut handles = Vec::new();
        for (path, value) in patches {
            let store = store.clone();
            handles.push(tokio::spawn(async move {
                store
                    .apply_patch_and_persist(&patch(path, value))
                    .await
                    .expect("patch should succeed");
            }));
        }
        for handle in handles {
            handle.await.expect("task should finish");
        }

        // No update was lost to a stale clone.
        let snapshot = store.snapshot();
        assert_eq!(
            snapshot.current.memospot.window.theme.as_deref(),
            Some("dark")
        );
        assert_eq!(
            snapshot.current.memospot.window.reduce_animation,
            Some(true)
        );
        assert_eq!(
            snapshot.current.memospot.window.locale.as_deref(),
            Some("es")
        );
        assert_eq!(snapshot.current.memospot.window.hide_menu_bar, Some(true));
        assert_eq!(snapshot.current.memospot.window.maximized, Some(true));
        assert_eq!(snapshot.current.memos.port, Some(9999));
        assert_eq!(snapshot.current.memospot.remote.enabled, Some(true));
        assert_eq!(snapshot.current.memospot.updater.enabled, Some(false));
        assert_eq!(snapshot.current.memospot.log.enabled, Some(true));
        assert_eq!(snapshot.current.memospot.migrations.enabled, Some(false));
        assert_eq!(snapshot.current.memospot.backups.enabled, Some(false));
        assert_eq!(snapshot.current.memospot.env.enabled, Some(true));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn runtime_owned_update_merges_with_concurrent_settings_patch() {
        let dir = TempDir::new().expect("tempdir");
        let store = default_store(&dir);

        let store_settings = store.clone();
        let settings = tokio::spawn(async move {
            store_settings
                .apply_patch_and_persist(&patch("/memospot/window/theme", json!("dark")))
                .await
                .expect("patch should succeed");
        });

        let store_window = store.clone();
        let window = tokio::spawn(async move {
            store_window
                .update_runtime_owned_fields(|config| {
                    config.memospot.window.x = Some(42);
                    config.memospot.window.y = Some(24);
                })
                .await;
        });

        settings.await.expect("settings task");
        window.await.expect("window task");

        let snapshot = store.snapshot();
        assert_eq!(
            snapshot.current.memospot.window.theme.as_deref(),
            Some("dark")
        );
        assert_eq!(snapshot.current.memospot.window.x, Some(42));
        assert_eq!(snapshot.current.memospot.window.y, Some(24));
    }

    #[tokio::test]
    async fn restart_requirement_is_measured_against_startup_baseline() {
        let dir = TempDir::new().expect("tempdir");
        let store = default_store(&dir);

        let changed = patch("/memos/port", json!(9999));
        assert!(
            store
                .apply_patch_and_persist(&changed)
                .await
                .expect("patch should succeed")
                .result
                .restart_required
        );

        let reverted = patch("/memos/port", json!(5230));
        assert!(
            !store
                .apply_patch_and_persist(&reverted)
                .await
                .expect("patch should succeed")
                .result
                .restart_required
        );
    }

    #[tokio::test]
    async fn runtime_owned_window_updates_do_not_create_restart_requirement() {
        let dir = TempDir::new().expect("tempdir");
        let store = default_store(&dir);

        store
            .update_runtime_owned_fields(|config| {
                config.memospot.window.x = Some(42);
                config.memospot.window.y = Some(24);
            })
            .await;

        let result = store
            .apply_patch_and_persist(&patch("/memospot/window/theme", json!("dark")))
            .await
            .expect("patch should succeed");

        assert!(!result.result.restart_required);
    }
}
