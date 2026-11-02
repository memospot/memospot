use itertools::Itertools;

use std::{env::consts::OS, sync::OnceLock};

/// Caller-supplied path prefixes, set once at startup by the app, which knows
/// the resolved data directory. Migrations must not re-derive it: a second
/// resolver with divergent semantics previously omitted the real data
/// directory on XDG-based systems.
static PREFIXES: OnceLock<Vec<String>> = OnceLock::new();

/// Set the path prefixes known to contain absolute resource paths.
///
/// Must be called once before running migrations. Subsequent calls keep the
/// original prefixes and log a warning.
pub fn set_prefixes(prefixes: Vec<String>) {
    if PREFIXES.set(prefixes).is_err() {
        log::warn!(
            "resource_path::set_prefixes called more than once; keeping original prefixes"
        );
    }
}

/// Normalize the suffix of the path, ensuring it ends with a directory separator.
#[inline]
pub fn norm_suffix(path: &str) -> String {
    let p = path.to_string();
    if p.contains('/') && !p.ends_with('/') {
        return p + "/";
    }
    if p.contains('\\') && !p.ends_with('\\') {
        return p + "\\";
    }
    p
}

/// Convert all backslashes to slashes.
///
/// Also replaces duplicated slashes with a single slash.
#[inline]
pub fn to_slash(path: &str) -> String {
    path.replace('\\', "/").replace("//", "/")
}

/// Build a list of known paths to check for absolute resource paths.
pub fn build_path_list() -> Vec<String> {
    let injected: Vec<String> = match PREFIXES.get() {
        Some(prefixes) => prefixes.clone(),
        None => {
            log::warn!(
                "resource_path::set_prefixes was not called; \
                 resource path migration will only strip legacy system prefixes"
            );
            Vec::new()
        }
    };
    build_path_list_with(&injected)
}

/// Build a list of known paths from explicitly supplied prefixes.
///
/// Pure construction helper so prefix assembly stays testable without global state.
fn build_path_list_with(injected: &[String]) -> Vec<String> {
    let mut paths: Vec<String> = injected.to_vec();

    // Legacy constants: absolute paths written by historical install layouts.
    paths.push("/var/opt/memos/".to_string());
    if let Ok(bin) = std::env::current_exe()
        && let Some(cwd) = bin.parent()
    {
        paths.push(norm_suffix(cwd.to_string_lossy().as_ref()));
    }
    if OS == "windows"
        && let Ok(program_data) = std::env::var("PROGRAMDATA")
    {
        paths.push(format!("{program_data}\\memos\\"));
    }
    paths.into_iter().unique().collect()
}

/// Strip known path prefixes from an absolute resource path, then normalize
/// separators and any leading slash. Shared by the resource-path migrations.
pub fn strip_known_prefix(path: &str, paths: &[String]) -> String {
    let mut new_path = path.to_string();
    for p in paths {
        new_path = new_path.trim_start_matches(p).to_string();
    }
    to_slash(&new_path).trim_start_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_path_list_with_combines_injected_prefixes_and_legacy_constants() {
        // GIVEN explicitly supplied prefixes containing the resolved data directory
        let injected = vec!["/home/alice/.config/memospot/".to_string()];

        // WHEN building the path list from those prefixes
        let paths = build_path_list_with(&injected);

        // THEN the injected prefix is present
        assert!(paths.contains(&"/home/alice/.config/memospot/".to_string()));
        // AND legacy constants are preserved
        assert!(paths.contains(&"/var/opt/memos/".to_string()));
    }

    #[test]
    fn first_set_prefixes_call_wins() {
        // GIVEN injected prefixes supplied once
        set_prefixes(vec!["/home/alice/.config/memospot/".to_string()]);
        // AND a later attempt to overwrite them
        set_prefixes(vec!["/elsewhere/".to_string()]);

        // WHEN building the path list
        let paths = build_path_list();

        // THEN the first injected prefix is retained
        assert!(paths.contains(&"/home/alice/.config/memospot/".to_string()));
        assert!(!paths.contains(&"/elsewhere/".to_string()));
    }

    #[test]
    fn strip_known_prefix_relativizes_xdg_style_absolute_path() {
        // GIVEN a path list with an XDG-style data directory
        let paths = vec!["/home/alice/.config/memospot/".to_string()];
        // AND an absolute resource path written under that directory
        let absolute = "/home/alice/.config/memospot/assets/2024/05/image.png";

        // WHEN stripping known prefixes
        let stripped = strip_known_prefix(absolute, &paths);

        // THEN the result is a relative assets path
        assert_eq!(stripped, "assets/2024/05/image.png");
    }

    #[test]
    fn strip_known_prefix_normalizes_windows_separators_and_leading_slash() {
        // GIVEN a path list with a Windows data directory
        let paths = vec!["C:\\Users\\alice\\AppData\\Local\\memospot\\".to_string()];
        // AND an absolute resource path under it
        let absolute =
            "C:\\Users\\alice\\AppData\\Local\\memospot\\assets\\2024\\05\\image.png";

        // WHEN stripping known prefixes
        let stripped = strip_known_prefix(absolute, &paths);

        // THEN separators are normalized and the path is relative
        assert_eq!(stripped, "assets/2024/05/image.png");
    }
}
