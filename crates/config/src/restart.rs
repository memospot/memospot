use crate::Config;

/// Restart impact of configuration edits.
///
/// Only theme, animation, and locale changes apply live. Every other
/// difference requires restarting Memospot, so unknown future fields
/// fail closed toward restart: blank them above only if they genuinely
/// take effect without one.
impl Config {
    pub fn restart_required(&self, other: &Self) -> bool {
        let mut before = self.clone();
        let mut after = other.clone();
        for config in [&mut before, &mut after] {
            config.memospot.window.theme = None;
            config.memospot.window.reduce_animation = None;
            config.memospot.window.locale = None;
        }
        before != after
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restart_classification_marks_restart_required_settings() {
        // GIVEN the default configuration as baseline.
        let before = Config::default();
        let mut after = before.clone();

        // WHEN nothing changes THEN no restart is required.
        assert!(!before.restart_required(&after));

        // WHEN only live settings change THEN no restart is required.
        after.memospot.window.theme = Some("dark".to_string());
        assert!(!before.restart_required(&after));
        after.memospot.window.locale = Some("es".to_string());
        assert!(!before.restart_required(&after));
        after.memospot.window.reduce_animation = Some(true);
        assert!(!before.restart_required(&after));

        // WHEN server/process settings change THEN a restart is required.
        after.memos.port = Some(9999);
        assert!(before.restart_required(&after));
        // AND remote settings.
        after = before.clone();
        after.memospot.remote.url = Some("https://example.com/".into());
        assert!(before.restart_required(&after));
        // AND environment settings.
        after = before.clone();
        after.memospot.env.enabled = Some(true);
        assert!(before.restart_required(&after));
        // AND updater settings.
        after = before.clone();
        after.memospot.updater.enabled = Some(false);
        assert!(before.restart_required(&after));

        // WHEN startup-only window settings change THEN a restart is required.
        after = before.clone();
        after.memospot.window.maximized = Some(true);
        assert!(before.restart_required(&after));
        // AND menu-bar visibility.
        after = before.clone();
        after.memospot.window.hide_menu_bar = Some(true);
        assert!(before.restart_required(&after));
    }
}
