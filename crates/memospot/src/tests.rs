#[cfg(test)]
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
