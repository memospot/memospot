mod configuration_state_tests {
    use crate::cmd;
    use crate::i18n;
    use crate::runtime_config::{
        ActiveServer, AppState, ConfigStore, RuntimeContext, RuntimePaths,
    };
    use config::Config;
    use i18n_embed::LanguageLoader;
    use serde_json::json;
    use std::path::PathBuf;
    use tauri::Manager;
    use tauri::test::{mock_builder, mock_context, noop_assets};
    use tempfile::TempDir;

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

    /// The live locale path: persistence, localization reload, and menu
    /// refresh, exercised through the real `set_locale` command on a mock app
    /// without constructing a full webview application.
    #[tokio::test]
    async fn set_locale_persists_and_applies_live() {
        let dir = TempDir::new().expect("tempdir");
        let config_file = dir.path().join("memospot.yaml");
        let config = Config::default();
        let store = ConfigStore::new(config.clone(), config, config_file.clone());
        let app_state = AppState {
            runtime: runtime_context(),
            config: store.clone(),
        };

        let app = mock_builder()
            .manage(app_state)
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

        // The application menu was rebuilt.
        assert!(app.menu().is_some());

        let patch = serde_json::to_string(&json!([{
            "op": "replace",
            "path": "/memospot/window/locale",
            "value": "de-DE",
        }]))
        .expect("locale patch should serialize");
        cmd::set_config(app.handle().clone(), app.state::<AppState>(), patch)
            .await
            .expect("generic locale patch should succeed");

        assert_eq!(i18n::LOCALE_LOADER.current_language().to_string(), "de-DE");
        assert_eq!(
            store.snapshot().current.memospot.window.locale.as_deref(),
            Some("de-DE")
        );
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
    use crate::memos::sync_mode_demo_compat;

    #[test]
    fn sync_mode_demo_compat_sets_demo_for_legacy_mode() {
        let mut memos = config::Memos {
            mode: Some("demo".to_string()),
            demo: Some(false),
            ..Default::default()
        };

        sync_mode_demo_compat(&mut memos);

        assert_eq!(memos.demo, Some(true));
    }

    #[test]
    fn sync_mode_demo_compat_disables_demo_for_non_demo_modes() {
        let mut memos = config::Memos {
            mode: Some("prod".to_string()),
            demo: Some(true),
            ..Default::default()
        };

        sync_mode_demo_compat(&mut memos);

        assert_eq!(memos.demo, Some(false));
    }

    #[test]
    fn sync_mode_demo_compat_defaults_unknown_mode_to_prod() {
        let mut memos = config::Memos {
            mode: Some("staging".to_string()),
            demo: Some(true),
            ..Default::default()
        };

        sync_mode_demo_compat(&mut memos);

        assert_eq!(memos.mode, Some("prod".to_string()));
        assert_eq!(memos.demo, Some(false));
    }
}
