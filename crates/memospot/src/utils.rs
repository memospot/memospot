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
    let fallback = home.join(format!(".{app_name}"));
    let xdg_config_path = env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home.join(".config"))
        .join(app_name);

    if xdg_config_path.exists() {
        return xdg_config_path;
    }
    if fallback.exists() {
        return fallback;
    }
    #[cfg(target_os = "windows")]
    if let Ok(app_data) = env::var("LOCALAPPDATA").or_else(|_| env::var("APPDATA")) {
        let app_data = PathBuf::from(app_data);
        return if app_data.ends_with("Roaming") {
            app_data.parent().unwrap().join("Local").join(app_name)
        } else {
            app_data.join(app_name)
        };
    }

    fallback
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
    let user_dirs = env::var("XDG_CONFIG_HOME")
        .ok()
        .filter(|dir| !dir.trim().is_empty() && PathBuf::from(dir).is_absolute())
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"))
        .join("user-dirs.dirs");
    if let Ok(contents) = std::fs::read_to_string(user_dirs)
        && let Some(dir) = resolve_downloads_from_user_dirs(&contents, &home)
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

    fn remove_env_vars() {
        // SAFETY: This is a test function, and removing a process environment
        // variable is generally safe. The unsafe block is required due to the
        // potential for race conditions in a multithreaded context.
        unsafe {
            env::remove_var("APPDATA");
            env::remove_var("HOME");
            env::remove_var("LOCALAPPDATA");
            env::remove_var("XDG_CONFIG_HOME");
        }
    }

    fn ensure_env_vars() {
        let home = std::env::var("HOME").unwrap_or_default();
        let app_data = std::env::var("APPDATA").unwrap_or_default();
        let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();

        // SAFETY: This is a test function, and setting a process environment
        // variable is generally safe. The unsafe block is required due to the
        // potential for race conditions in a multithreaded context.
        unsafe {
            if home.is_empty() {
                env::set_var(
                    "HOME",
                    if cfg!(windows) {
                        r"C:\Users\foo"
                    } else {
                        r"/home/foo"
                    },
                );
            }
            if app_data.is_empty() {
                env::set_var("APPDATA", r"C:\Users\foo\AppData\Roaming");
            }
            if local_app_data.is_empty() {
                env::set_var("LOCALAPPDATA", r"C:\Users\foo\AppData\Local");
            }
        }
    }

    #[cfg(windows)]
    #[test]
    fn test_get_data_path_windows() {
        remove_env_vars();

        // Test fallback to USERPROFILE (via home crate).
        assert!(
            get_app_data_path("memospot")
                .to_string_lossy()
                .ends_with("memospot")
        );

        let ci = env::var("CI").unwrap_or_default() == "true";
        if ci {
            assert!(
                get_app_data_path("memospot")
                    .to_string_lossy()
                    .ends_with("memospot")
            );
        } else {
            // Test fallback via APPDATA (ancient Windows versions).
            unsafe {
                env::set_var("APPDATA", r"C:\Users\foo\AppData\Roaming");
            }
            assert_eq!(
                get_app_data_path("memospot"),
                PathBuf::from(r"C:\Users\foo\AppData\Local\memospot")
            );

            // Test a standard system with LOCALAPPDATA set.
            unsafe {
                env::set_var("LOCALAPPDATA", r"C:\Users\foo\AppData\Local");
            }
            assert_eq!(
                get_app_data_path("memospot"),
                PathBuf::from(r"C:\Users\foo\AppData\Local\memospot")
            );
        }
    }

    #[test]
    fn test_get_data_path() {
        ensure_env_vars();
        let data_path = get_app_data_path("memospot");
        assert!(data_path.has_root());
        assert!(data_path.is_absolute());
        assert!(data_path.to_string_lossy().ends_with("memospot"));
    }

    #[test]
    fn test_xdg_config_home() -> Result<()> {
        remove_env_vars();
        let tmp_dir = tempfile::tempdir()?;
        let xdg_config_home = tmp_dir.path().join(".config");
        unsafe {
            env::set_var("XDG_CONFIG_HOME", &xdg_config_home);
        }
        assert_eq!(
            env::var("XDG_CONFIG_HOME").unwrap(),
            xdg_config_home.to_string_lossy()
        );

        // Test fallback to HOME/.memospot.
        assert!(
            get_app_data_path("memospot")
                .to_string_lossy()
                .ends_with("memospot")
        );

        // Create XDG_CONFIG_HOME/memospot.
        std::fs::create_dir_all(xdg_config_home.join("memospot"))?;
        assert!(
            get_app_data_path("memospot")
                .to_string_lossy()
                .ends_with("memospot")
        );
        Ok(())
    }

    #[test]
    fn user_dirs_download_resolves_home_relative_and_absolute_entries() {
        let home = Path::new("/home/alice");
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
        let home = Path::new("/home/alice");
        assert_eq!(
            resolve_downloads_from_user_dirs("XDG_DOCUMENTS_DIR=\"$HOME/Documents\"\n", home),
            None
        );
        assert_eq!(
            resolve_downloads_from_user_dirs("XDG_DOWNLOAD_DIR=\"\"\n", home),
            None
        );
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
        let expanded =
            expand_windows_env(r"%A%-%B%", |name| (name == "A").then(|| "a".to_string()));
        assert_eq!(expanded, None);
    }
}
