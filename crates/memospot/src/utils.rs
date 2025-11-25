use home::home_dir;
use path_clean::PathClean;
use std::env;
use std::io::Result;
use std::path::{Path, PathBuf};

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
        assert!(get_app_data_path("memospot")
            .to_string_lossy()
            .ends_with("memospot"));

        let ci = env::var("CI").unwrap_or_default() == "true";
        if ci {
            assert!(get_app_data_path("memospot")
                .to_string_lossy()
                .ends_with("memospot"));
        } else {
            // Test fallback via APPDATA (ancient Windows versions).
            env::set_var("APPDATA", r"C:\Users\foo\AppData\Roaming");
            assert_eq!(
                get_app_data_path("memospot"),
                PathBuf::from(r"C:\Users\foo\AppData\Local\memospot")
            );

            // Test a standard system with LOCALAPPDATA set.
            env::set_var("LOCALAPPDATA", r"C:\Users\foo\AppData\Local");
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
        assert!(get_app_data_path("memospot")
            .to_string_lossy()
            .ends_with("memospot"));

        // Create XDG_CONFIG_HOME/memospot.
        std::fs::create_dir_all(xdg_config_home.join("memospot"))?;
        assert!(get_app_data_path("memospot")
            .to_string_lossy()
            .ends_with(if cfg!(windows) {
                r".config\memospot"
            } else {
                ".config/memospot"
            }));
        Ok(())
    }
}
