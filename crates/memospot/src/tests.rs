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
