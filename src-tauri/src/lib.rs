use home::home_dir;
use log::{error, info, warn};
use native_dialog::{MessageDialog, MessageType};
use path_clean::PathClean;
use std::env;
use std::io::Result;
use std::path::{Path, PathBuf};

#[macro_export]
macro_rules! panic_dialog {
    ($($arg:tt)*) => {
        panic_dialog(&format!($($arg)*));
        panic!("Fatal error: {}", &format!($($arg)*));
    };
}

#[macro_export]
macro_rules! info_dialog {
    ($($arg:tt)*) => {
        info_dialog(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! warn_dialog {
    ($($arg:tt)*) => {
        warn_dialog(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! error_dialog {
    ($($arg:tt)*) => {
        error_dialog(&format!($($arg)*));
    };
}

pub fn panic_dialog(msg: &str) {
    const FATAL_ERROR: &str = "Fatal error";
    MessageDialog::new()
        .set_type(MessageType::Error)
        .set_title(FATAL_ERROR)
        .set_text(msg)
        .show_alert()
        .unwrap_or_default();
    error!("{}: {}", FATAL_ERROR, msg);
    panic!("{}: {}", FATAL_ERROR, msg);
}

pub fn info_dialog(msg: &str) {
    info!("{}", msg);
    MessageDialog::new()
        .set_type(MessageType::Info)
        .set_title("Info")
        .set_text(msg)
        .show_alert()
        .unwrap_or_default();
}

pub fn warn_dialog(msg: &str) {
    warn!("{}", msg);
    MessageDialog::new()
        .set_type(MessageType::Warning)
        .set_title("Warning")
        .set_text(msg)
        .show_alert()
        .unwrap_or_default();
}

pub fn error_dialog(msg: &str) {
    error!("{}", msg);
    MessageDialog::new()
        .set_type(MessageType::Error)
        .set_title("Error")
        .set_text(msg)
        .show_alert()
        .unwrap_or_default();
}

pub fn confirm_dialog(title: &str, msg: &str, icon: MessageType) -> bool {
    MessageDialog::new()
        .set_type(icon)
        .set_title(title)
        .set_text(msg)
        .show_confirm()
        .unwrap_or_default()
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

    let probe_paths = [
        home.join(".config").join(app_name),
        home.join(format!(".{}", app_name)),
    ];
    for path in probe_paths {
        if path.exists() {
            return path;
        }
    }

    if cfg!(windows) {
        if let Ok(path) = env::var("LOCALAPPDATA").or_else(|_| env::var("APPDATA")) {
            let path = PathBuf::from(path);
            return if path.ends_with("Local") {
                path.join(app_name)
            } else {
                path.parent().unwrap_or(&path).join("Local").join(app_name)
            };
        }
    }

    home.join(format!(".{}", app_name))
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
        // SAFETY: This is a test function, and we're removing an environment variable
        // which is generally safe. The unsafe block is required due to the potential
        // for race conditions in a multithreaded context, but this is a single-threaded test.
        unsafe {
            env::remove_var("HOME");
            env::remove_var("LOCALAPPDATA");
            env::remove_var("APPDATA");
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_get_data_path_windows() {
        remove_env_vars();
        env::set_var("LOCALAPPDATA", r"C:\Users\foo\AppData\Local");
        let data_path = get_app_data_path("memospot");
        assert_eq!(
            data_path,
            PathBuf::from(r"C:\Users\foo\AppData\Local\memospot")
        );

        remove_env_vars();
        env::set_var("APPDATA", r"C:\Users\foo\AppData\Roaming");
        let data_path = get_app_data_path("memospot");
        assert_eq!(
            data_path,
            PathBuf::from(r"C:\Users\foo\AppData\Local\memospot")
        );
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_get_data_path_posix() {
        remove_env_vars();
        // SAFETY: This is a test function, and we're setting an environment variable
        // which is generally safe. The unsafe block is required due to the potential
        // for race conditions in a multithreaded context, but this is a single-threaded test.
        unsafe {
            env::set_var("HOME", r"/home/foo");
        }
        let data_path = get_app_data_path("memospot");
        assert_eq!(data_path, PathBuf::from(r"/home/foo/.memospot"));
    }

    #[test]
    fn test_get_data_path() {
        remove_env_vars();
        let data_path = get_app_data_path("memospot");
        assert!(data_path.has_root());
        assert!(data_path.is_absolute());
        if cfg!(windows) {
            assert!(data_path.ends_with("memospot"));
        } else {
            assert!(data_path.ends_with(".memospot"));
        }
    }
}
