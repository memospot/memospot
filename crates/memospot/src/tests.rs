mod configuration_state_tests {
    use crate::cmd;
    use crate::i18n;
    use crate::runtime_config::{
        ActiveServer, AppState, ConfigStore, RuntimeContext, RuntimePaths,
    };
    use crate::window::Window;
    use config::Config;
    use i18n_embed::LanguageLoader;
    use serde_json::{Value, json};
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, RwLock};
    use tauri::ipc::{CallbackFn, InvokeBody, InvokeResponseBody};
    use tauri::test::{
        INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder, mock_context, noop_assets,
    };
    use tauri::webview::InvokeRequest;
    use tauri::{App, Manager, WebviewWindow, WebviewWindowBuilder};
    use tempfile::TempDir;
    use url::Url;

    fn runtime_context() -> RuntimeContext {
        RuntimeContext {
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
                url: "http://localhost:5230/".into(),
                user_agent: "test".into(),
                managed: true,
            },
            memos: config::Memos::default(),
        }
    }

    fn app_state(store: &ConfigStore) -> AppState {
        let maximized = store
            .snapshot()
            .current
            .memospot
            .window
            .maximized
            .unwrap_or_default();
        AppState {
            runtime: runtime_context(),
            config: store.clone(),
            memos_version: Arc::new(RwLock::new(None)),
            zoom_level: Arc::new(RwLock::new(1.0)),
            window_states: crate::window_state::WindowStateQueue::new(store.clone(), maximized),
        }
    }

    fn build_app(store: ConfigStore) -> App<MockRuntime> {
        mock_builder()
            .invoke_handler(tauri::generate_handler![cmd::set_config])
            .manage(app_state(&store))
            .build(mock_context(noop_assets()))
            .expect("mock app should build")
    }

    fn seed_config_file(config: &Config, path: &Path) {
        tauri::async_runtime::block_on(config.save_to_file(path)).expect("config should save");
    }

    fn create_webview(app: &App<MockRuntime>, label: &str) -> WebviewWindow<MockRuntime> {
        WebviewWindowBuilder::new(app, label, Default::default())
            .build()
            .expect("mock webview should build")
    }

    fn local_request_url() -> Url {
        let url = if cfg!(any(windows, target_os = "android")) {
            "https://tauri.localhost"
        } else {
            "tauri://localhost"
        };
        url.parse().expect("local URL should parse")
    }

    fn invoke_set_config(
        webview: &WebviewWindow<MockRuntime>,
        patch: Value,
    ) -> Result<InvokeResponseBody, Value> {
        get_ipc_response(
            webview,
            InvokeRequest {
                cmd: "set_config".into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: local_request_url(),
                body: InvokeBody::Json(json!({ "patch": patch })),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
    }

    /// The live locale path: persistence and localization reload,
    /// exercised through the real `set_locale` command on a mock app
    /// without constructing a full webview application.
    ///
    /// macOS menu construction panics off the main thread (muda requires
    /// `MainThreadMarker`), and the mock runtime runs `run_on_main_thread`
    /// inline on the calling test thread. The locale refresh therefore
    /// catches that construction panic and keeps the menu unset there instead
    /// of crashing the test.
    #[tokio::test]
    async fn set_locale_persists_and_applies_live() {
        let dir = TempDir::new().expect("tempdir");
        let config_file = dir.path().join("memospot.yaml");
        let config = Config::default();
        let store = ConfigStore::new(config.clone(), config, config_file.clone());

        let app = mock_builder()
            .manage(app_state(&store))
            .build(mock_context(noop_assets()))
            .expect("mock app should build");

        let result =
            cmd::set_locale(app.handle().clone(), app.state::<AppState>(), "es".into())
                .await
                .expect("set_locale should succeed");

        // Locale changes apply live: no restart required.
        assert!(!result.restart_required);

        // The preference is persisted and reflected in the managed store.
        assert_eq!(
            store.snapshot().current.memospot.window.locale.as_deref(),
            Some("es")
        );
        let on_disk: Config = serde_saphyr::from_str(
            &std::fs::read_to_string(&config_file).expect("config file should exist"),
        )
        .expect("on-disk config should parse");
        assert_eq!(on_disk.memospot.window.locale.as_deref(), Some("es"));

        // The backend localization is reloaded live.
        assert_eq!(i18n::LOCALE_LOADER.current_language().to_string(), "es");

        // The application menu was rebuilt, except on macOS where menu
        // construction panics off the main thread under the mock runtime.
        #[cfg(not(target_os = "macos"))]
        assert!(app.menu().is_some());
        #[cfg(target_os = "macos")]
        assert!(app.menu().is_none());
    }

    fn settings_patch(value: &str) -> Value {
        json!([{
            "op": "replace",
            "path": "/memospot/window/theme",
            "value": value,
        }])
    }

    fn assert_rejected(error: Value) {
        assert_eq!(
            error,
            json!("configuration updates are only accepted from the settings window")
        );
    }

    #[test]
    fn set_config_persists_from_settings_window() {
        // GIVEN a mock app with a settings webview and a seeded config file
        let dir = TempDir::new().expect("tempdir");
        let config_file = dir.path().join("memospot.yaml");
        let config = Config::default();
        seed_config_file(&config, &config_file);
        let store = ConfigStore::new(config.clone(), config, config_file.clone());
        let app = build_app(store);
        let webview = create_webview(&app, Window::Settings.as_str());

        // WHEN the settings window submits a theme update
        let response = invoke_set_config(&webview, settings_patch("dark"))
            .expect("settings update should succeed");

        // THEN the update is reported without a restart and persisted.
        let result: Value = response
            .deserialize()
            .expect("update result should deserialize");
        assert_eq!(result["restart_required"], json!(false));
        let on_disk: Config = serde_saphyr::from_str(
            &std::fs::read_to_string(&config_file).expect("config file should exist"),
        )
        .expect("on-disk config should parse");
        assert_eq!(on_disk.memospot.window.theme.as_deref(), Some("dark"));
    }

    fn assert_config_update_rejected(label: &str) {
        // GIVEN a mock app with a non-settings webview and a seeded config file
        let dir = TempDir::new().expect("tempdir");
        let config_file = dir.path().join("memospot.yaml");
        let config = Config::default();
        seed_config_file(&config, &config_file);
        let before_file =
            std::fs::read_to_string(&config_file).expect("config file should exist");
        let store = ConfigStore::new(config.clone(), config, config_file.clone());
        let before_config = store.snapshot().current;
        let app = build_app(store.clone());
        let webview = create_webview(&app, label);

        // WHEN the non-settings window submits a theme update
        let error = match invoke_set_config(&webview, settings_patch("dark")) {
            Ok(_) => panic!("configuration update from `{label}` should be rejected"),
            Err(error) => error,
        };

        // THEN the request is rejected and neither memory nor disk changed.
        assert_rejected(error);
        assert_eq!(*store.snapshot().current, *before_config);
        let after_file =
            std::fs::read_to_string(&config_file).expect("config file should still exist");
        assert_eq!(after_file, before_file);

        // AND removing the file proves rejection does not rewrite it.
        std::fs::remove_file(&config_file).expect("config file should be removable");
        let error = match invoke_set_config(&webview, settings_patch("dark")) {
            Ok(_) => panic!("configuration update from `{label}` should be rejected"),
            Err(error) => error,
        };
        assert_rejected(error);
        assert!(
            !config_file.exists(),
            "rejected update must not rewrite the config file"
        );
    }

    #[test]
    fn set_config_rejects_main_window() {
        assert_config_update_rejected(Window::Main.as_str());
    }

    #[test]
    fn set_config_rejects_cloned_main_window() {
        assert_config_update_rejected("3f0c9a2e-7b1d-4c8a-9e2f-1a2b3c4d5e6f");
    }
}

mod i18n_tests {
    use crate::i18n::resolve_supported_locale;
    use i18n_embed::unic_langid::LanguageIdentifier;

    fn lang(value: &str) -> LanguageIdentifier {
        value.parse().expect("test locale should be valid")
    }

    fn available_locales() -> Vec<LanguageIdentifier> {
        vec![
            lang("en"),
            lang("es"),
            lang("de-DE"),
            lang("fr-FR"),
            lang("ja-JP"),
            lang("pt-BR"),
            lang("ru-RU"),
            lang("zh-Hans"),
            lang("zh-Hant"),
        ]
    }

    #[test]
    fn resolve_supported_locale_normalizes_underscore_tags() {
        let available = available_locales();
        let resolved = resolve_supported_locale("pt_BR", &available);

        assert_eq!(resolved, Some(lang("pt-BR")));
    }

    #[test]
    fn resolve_supported_locale_maps_zh_hk_to_zh_hant() {
        let available = available_locales();
        let resolved = resolve_supported_locale("zh-HK", &available);

        assert_eq!(resolved, Some(lang("zh-Hant")));
    }

    #[test]
    fn resolve_supported_locale_falls_back_to_same_language_family() {
        let available = available_locales();
        let resolved = resolve_supported_locale("es-MX", &available);

        assert_eq!(resolved, Some(lang("es")));
    }

    #[test]
    fn resolve_supported_locale_returns_none_for_unavailable_language() {
        let available = available_locales();
        let resolved = resolve_supported_locale("it-IT", &available);

        assert_eq!(resolved, None);
    }
}

mod memos_tests {
    use crate::memos_api::query_version;

    /// The readiness probe fails fast against a refused connection.
    ///
    /// This is the [`None`] precondition of the version handoff: an
    /// unreachable server yields `Err` here, so `wait_api_ready` keeps
    /// polling until its timeout and then delivers [`None`]. The full
    /// 15-second timeout path is deliberately not exercised in tests.
    #[tokio::test]
    async fn query_version_fails_against_refused_connection() {
        // GIVEN a port nothing listens on, so the probe is refused
        // immediately instead of timing out.
        let url = "http://127.0.0.1:1/";

        // WHEN the version is queried THEN the probe reports failure.
        let result = query_version(url).await;

        // THEN the handoff precondition holds: no version to deliver.
        assert!(result.is_err());
    }
}

mod zoom_tests {
    use crate::event::{ZOOM_STEP, stepped_zoom};

    #[test]
    fn stepped_zoom_steps_from_current_level() {
        // GIVEN the default level THEN one step lands exactly on 1.1 / 0.9.
        assert_eq!(stepped_zoom(1.0, ZOOM_STEP), 1.1);
        assert_eq!(stepped_zoom(1.0, -ZOOM_STEP), 0.9);
    }

    #[test]
    fn stepped_zoom_clamps_at_bounds() {
        // GIVEN a level at the ceiling THEN stepping up stays clamped.
        assert_eq!(stepped_zoom(5.0, ZOOM_STEP), 5.0);
        // AND a level at the floor THEN stepping down stays clamped.
        assert_eq!(stepped_zoom(0.2, -ZOOM_STEP), 0.2);
        // AND overshoots clamp rather than wrap or exceed.
        assert_eq!(stepped_zoom(4.95, ZOOM_STEP), 5.0);
        assert_eq!(stepped_zoom(0.25, -ZOOM_STEP), 0.2);
    }
}
