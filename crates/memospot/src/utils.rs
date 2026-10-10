use home::home_dir;
use path_clean::PathClean;
use std::env;
use std::io::Result;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri::Runtime;

/// Run a function on the main thread and block until it completes.
///
/// macOS requires UI operations to be performed on the main thread.
#[allow(dead_code)]
pub fn run_on_main_thread_blocking<R, T, F>(app: &AppHandle<R>, f: F) -> Option<T>
where
    R: Runtime,
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    app.run_on_main_thread(move || {
        let _ = tx.send(f());
    })
    .ok()?;
    rx.recv().ok()
}

/// Resolve the data path from injected environment inputs.
///
/// `xdg_config_home`, `local_app_data`, and `app_data` are the raw values of
/// `XDG_CONFIG_HOME`, `LOCALAPPDATA`, and `APPDATA`. `exists` stands in for
/// filesystem probes so tests stay hermetic. Production callers pass the real
/// environment and `Path::exists`.
fn resolve_app_data_path(
    home: &Path,
    app_name: &str,
    xdg_config_home: Option<&str>,
    local_app_data: Option<&str>,
    app_data: Option<&str>,
    exists: impl Fn(&Path) -> bool,
) -> PathBuf {
    let fallback = home.join(format!(".{app_name}"));
    let xdg_config_path = xdg_config_home
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"))
        .join(app_name);

    if exists(&xdg_config_path) {
        return xdg_config_path;
    }
    if exists(&fallback) {
        return fallback;
    }
    if let Some(app_data) = local_app_data.or(app_data) {
        let app_data = PathBuf::from(app_data);
        return if app_data.ends_with("Roaming") {
            app_data.parent().unwrap().join("Local").join(app_name)
        } else {
            app_data.join(app_name)
        };
    }

    fallback
}

/// Get the data path to supplied application name.
///
/// Probe paths:
///   - ~/.config/{app_name}
///   - ~/.{app_name}
///
/// Default path:
///   - Windows: `%LOCALAPPDATA%\{app_name}`
///   - POSIX-compliant systems: `~/.{app_name}`.
///
/// Fallback:
///   - `%APPDATA%\..\Local\{app_name}` (Windows)
///   - `~/.{app_name}`
///
/// Home directory is determined by the [`home`](https://docs.rs/home) crate.
pub fn get_app_data_path(app_name: &str) -> PathBuf {
    let home = home_dir().unwrap_or_default();
    let xdg_config_home = env::var("XDG_CONFIG_HOME").ok();
    #[cfg(target_os = "windows")]
    let (local_app_data, app_data) = (env::var("LOCALAPPDATA").ok(), env::var("APPDATA").ok());
    #[cfg(not(target_os = "windows"))]
    let (local_app_data, app_data): (Option<String>, Option<String>) = (None, None);

    resolve_app_data_path(
        &home,
        app_name,
        xdg_config_home.as_deref(),
        local_app_data.as_deref(),
        app_data.as_deref(),
        |path| path.exists(),
    )
}

/// Get the user's Downloads directory.
///
/// - Windows: Downloads Known Folder from the registry, falling back to
///   `%USERPROFILE%\Downloads`.
/// - Linux: `XDG_DOWNLOAD_DIR` from `user-dirs.dirs`, falling back to
///   `~/Downloads`.
/// - macOS: `~/Downloads`.
///
/// Home directory resolution uses the [`home`](https://docs.rs/home) crate.
pub fn get_downloads_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    return windows_downloads_dir();
    #[cfg(target_os = "linux")]
    return linux_downloads_dir();
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    return home_dir().unwrap_or_default().join("Downloads");
}

#[cfg(target_os = "windows")]
fn windows_downloads_dir() -> PathBuf {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    const DOWNLOADS_GUID: &str = "{374DE290-123F-4565-9164-39C4925E467B}";
    const SHELL_FOLDERS: &str =
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders";
    const USER_SHELL_FOLDERS: &str =
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders";

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) = hkcu.open_subkey(USER_SHELL_FOLDERS)
        && let Ok(value) = key.get_value::<String, _>(DOWNLOADS_GUID)
        && let Some(expanded) = expand_windows_env(value.trim(), |name| env::var(name).ok())
        && !expanded.trim().is_empty()
    {
        return PathBuf::from(expanded);
    }
    if let Ok(key) = hkcu.open_subkey(SHELL_FOLDERS)
        && let Ok(value) = key.get_value::<String, _>(DOWNLOADS_GUID)
        && !value.trim().is_empty()
    {
        return PathBuf::from(value.trim());
    }
    if let Ok(profile) = env::var("USERPROFILE")
        && !profile.trim().is_empty()
    {
        return PathBuf::from(profile).join("Downloads");
    }

    home_dir().unwrap_or_default().join("Downloads")
}

#[cfg(target_os = "linux")]
fn linux_downloads_dir() -> PathBuf {
    let home = home_dir().unwrap_or_default();
    let xdg_config_home = env::var("XDG_CONFIG_HOME").ok();
    resolve_linux_downloads_dir(&home, xdg_config_home.as_deref(), |path| {
        std::fs::read_to_string(path).ok()
    })
}

/// Resolve the Linux Downloads directory from injected inputs.
///
/// `xdg_config_home` is the raw `XDG_CONFIG_HOME` value. `read_user_dirs`
/// stands in for reading `user-dirs.dirs` so tests stay hermetic. Production
/// callers pass the real environment and filesystem.
#[cfg(any(target_os = "linux", test))]
fn resolve_linux_downloads_dir(
    home: &Path,
    xdg_config_home: Option<&str>,
    read_user_dirs: impl Fn(&Path) -> Option<String>,
) -> PathBuf {
    let user_dirs = xdg_config_home
        .filter(|dir| !dir.trim().is_empty() && PathBuf::from(dir).is_absolute())
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"))
        .join("user-dirs.dirs");
    if let Some(contents) = read_user_dirs(&user_dirs)
        && let Some(dir) = resolve_downloads_from_user_dirs(&contents, home)
    {
        return dir;
    }

    home.join("Downloads")
}

/// Resolve the Downloads directory from `user-dirs.dirs` file contents.
///
/// Returns `None` when `XDG_DOWNLOAD_DIR` is absent, so the caller falls back
/// to `~/Downloads`. Relative values resolve against `home`.
#[cfg(any(target_os = "linux", test))]
fn resolve_downloads_from_user_dirs(contents: &str, home: &Path) -> Option<PathBuf> {
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() != "XDG_DOWNLOAD_DIR" {
            continue;
        }
        let mut value = value.trim();
        if value.len() >= 2 {
            let bytes = value.as_bytes();
            if (bytes[0] == b'"' && bytes[value.len() - 1] == b'"')
                || (bytes[0] == b'\'' && bytes[value.len() - 1] == b'\'')
            {
                value = value[1..value.len() - 1].trim();
            }
        }
        if value.is_empty() {
            return None;
        }
        if value == "$HOME" || value == "${HOME}" || value == "$HOME/" {
            return Some(home.to_path_buf());
        }
        if let Some(rest) = value.strip_prefix("$HOME/") {
            return Some(home.join(rest));
        }
        if let Some(rest) = value.strip_prefix("${HOME}/") {
            return Some(home.join(rest));
        }
        let path = PathBuf::from(value);
        if path.is_absolute() {
            return Some(path);
        }
        return Some(home.join(path));
    }

    None
}

/// Expand `%NAME%` variables using `lookup`.
///
/// Returns `None` when a referenced variable has no value, so the caller
/// falls through to the next candidate instead of opening a mangled path.
#[cfg(any(target_os = "windows", test))]
fn expand_windows_env(value: &str, lookup: impl Fn(&str) -> Option<String>) -> Option<String> {
    let mut expanded = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '%' {
            expanded.push(c);
            continue;
        }
        let mut name = String::new();
        let mut closed = false;
        for next in chars.by_ref() {
            if next == '%' {
                closed = true;
                break;
            }
            name.push(next);
        }
        if closed && !name.is_empty() {
            expanded.push_str(&lookup(&name)?);
        } else {
            expanded.push('%');
            expanded.push_str(&name);
        }
    }

    Some(expanded)
}

/// Get the absolute path to supplied path.
pub fn absolute_path(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = path.as_ref();

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    }
    .clean();

    Ok(absolute_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_data_path_returns_fallback_when_nothing_exists() {
        // GIVEN a home directory with no probed paths on disk
        let home = Path::new("/home/alice");
        // WHEN resolving with no environment inputs and nothing existing
        let resolved = resolve_app_data_path(home, "memospot", None, None, None, |_| false);
        // THEN the home dot-directory fallback is returned
        assert_eq!(resolved, PathBuf::from("/home/alice/.memospot"));
    }

    #[test]
    fn app_data_path_prefers_existing_xdg_config_path() {
        // GIVEN an XDG config path that exists alongside a missing fallback
        let home = Path::new("/home/alice");
        let xdg = PathBuf::from("/etc/xdg/memospot");
        let fallback = PathBuf::from("/home/alice/.memospot");
        // WHEN resolving with the XDG input present
        let resolved =
            resolve_app_data_path(home, "memospot", Some("/etc/xdg"), None, None, |path| {
                path == xdg
            });
        // THEN the existing XDG path wins over the fallback
        assert_eq!(resolved, xdg);
        assert_ne!(resolved, fallback);
    }

    #[test]
    fn app_data_path_uses_fallback_when_only_it_exists() {
        // GIVEN a missing XDG path and an existing home fallback
        let home = Path::new("/home/alice");
        let fallback = PathBuf::from("/home/alice/.memospot");
        // WHEN resolving with an XDG input that does not exist
        let resolved =
            resolve_app_data_path(home, "memospot", Some("/etc/xdg"), None, None, |path| {
                path == fallback
            });
        // THEN the fallback is returned
        assert_eq!(resolved, fallback);
    }

    #[test]
    fn app_data_path_resolves_windows_inputs_without_process_env() {
        // GIVEN no probed paths on disk and Windows-style env inputs
        let home = Path::new("/home/alice");
        let no_disk = |_: &Path| false;
        // WHEN LOCALAPPDATA is present it wins over APPDATA
        // THEN the local path is returned
        assert_eq!(
            resolve_app_data_path(
                home,
                "memospot",
                None,
                Some("/local"),
                Some("/roaming"),
                no_disk,
            ),
            PathBuf::from("/local/memospot")
        );
        // WHEN only a Roaming APPDATA is present
        // THEN it maps to the sibling Local directory
        assert_eq!(
            resolve_app_data_path(home, "memospot", None, None, Some("/foo/Roaming"), no_disk,),
            PathBuf::from("/foo/Local/memospot")
        );
        // WHEN only a plain APPDATA is present
        // THEN the app name is appended directly
        assert_eq!(
            resolve_app_data_path(home, "memospot", None, None, Some("/foo/Plain"), no_disk,),
            PathBuf::from("/foo/Plain/memospot")
        );
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn linux_downloads_uses_injected_config_home_and_contents() {
        // GIVEN an injected XDG_CONFIG_HOME pointing at a user-dirs file
        let home = Path::new("/home/alice");
        let expected_file = PathBuf::from("/custom/config/user-dirs.dirs");
        // WHEN the injected file contains a home-relative download dir
        let resolved = resolve_linux_downloads_dir(home, Some("/custom/config"), |path| {
            assert_eq!(path, expected_file);
            Some("XDG_DOWNLOAD_DIR=\"$HOME/Downloads\"\n".to_string())
        });
        // THEN the resolved path expands against the injected home
        assert_eq!(resolved, PathBuf::from("/home/alice/Downloads"));
    }

    #[test]
    fn linux_downloads_falls_back_for_blank_or_missing_inputs() {
        // GIVEN a home directory with no readable user-dirs file
        let home = Path::new("/home/alice");
        // WHEN the config home is blank
        // THEN the default Downloads directory is returned
        assert_eq!(
            resolve_linux_downloads_dir(home, Some("   "), |_| None),
            PathBuf::from("/home/alice/Downloads")
        );
        // WHEN the config home is relative
        // THEN it is ignored in favor of the default
        assert_eq!(
            resolve_linux_downloads_dir(home, Some("relative/config"), |_| None),
            PathBuf::from("/home/alice/Downloads")
        );
        // WHEN the user-dirs file cannot be read
        // THEN the default Downloads directory is returned
        assert_eq!(
            resolve_linux_downloads_dir(home, Some("/custom/config"), |_| None),
            PathBuf::from("/home/alice/Downloads")
        );
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn user_dirs_download_resolves_home_relative_and_absolute_entries() {
        // GIVEN user-dirs contents with home-relative and absolute entries
        let home = Path::new("/home/alice");
        // WHEN resolving each entry
        // THEN each maps to the expected absolute path
        assert_eq!(
            resolve_downloads_from_user_dirs("XDG_DOWNLOAD_DIR=\"$HOME/Downloads\"\n", home),
            Some(PathBuf::from("/home/alice/Downloads"))
        );
        assert_eq!(
            resolve_downloads_from_user_dirs("XDG_DOWNLOAD_DIR=${HOME}/dl\n", home),
            Some(PathBuf::from("/home/alice/dl"))
        );
        assert_eq!(
            resolve_downloads_from_user_dirs("XDG_DOWNLOAD_DIR=/srv/shared\n", home),
            Some(PathBuf::from("/srv/shared"))
        );
        assert_eq!(
            resolve_downloads_from_user_dirs(
                "# comment\nXDG_DOCUMENTS_DIR=\"$HOME/Documents\"\nXDG_DOWNLOAD_DIR=\"$HOME/data/downloads\"\n",
                home
            ),
            Some(PathBuf::from("/home/alice/data/downloads"))
        );
    }

    #[test]
    fn user_dirs_download_returns_none_when_missing_or_empty() {
        // GIVEN user-dirs contents without a usable download entry
        let home = Path::new("/home/alice");
        // WHEN resolving entries that are missing or empty
        // THEN none is returned so the caller falls back
        assert_eq!(
            resolve_downloads_from_user_dirs("XDG_DOCUMENTS_DIR=\"$HOME/Documents\"\n", home),
            None
        );
        assert_eq!(
            resolve_downloads_from_user_dirs("XDG_DOWNLOAD_DIR=\"\"\n", home),
            None
        );
        // GIVEN entries with surrounding whitespace or a bare home reference
        // WHEN resolving them
        // THEN they expand against the injected home
        assert_eq!(
            resolve_downloads_from_user_dirs("XDG_DOWNLOAD_DIR = \"$HOME/Downloads\"\n", home),
            Some(PathBuf::from("/home/alice/Downloads"))
        );
        assert_eq!(
            resolve_downloads_from_user_dirs("XDG_DOWNLOAD_DIR=$HOME\n", home),
            Some(PathBuf::from("/home/alice"))
        );
    }

    #[test]
    fn windows_env_expansion_replaces_known_variables() {
        // GIVEN a lookup with known Windows variables
        // WHEN expanding paths that reference them
        // THEN the placeholders are replaced
        let expanded = expand_windows_env(r"%USERPROFILE%\Downloads", |name| {
            (name == "USERPROFILE").then(|| r"C:\Users\alice".to_string())
        });
        assert_eq!(expanded.as_deref(), Some(r"C:\Users\alice\Downloads"));

        let expanded = expand_windows_env(r"%A%\%B%", |name| match name {
            "A" => Some("a".to_string()),
            "B" => Some("b".to_string()),
            _ => None,
        });
        assert_eq!(expanded.as_deref(), Some(r"a\b"));
    }

    #[test]
    fn windows_env_expansion_fails_on_unknown_variables() {
        // GIVEN a lookup missing one referenced variable
        // WHEN expanding a path that needs it
        // THEN expansion fails so the caller tries the next candidate
        let expanded =
            expand_windows_env(r"%A%-%B%", |name| (name == "A").then(|| "a".to_string()));
        assert_eq!(expanded, None);
    }
}
