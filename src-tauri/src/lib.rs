use homedir::HomeDirExt;
use path_clean::PathClean;

use log::error;
use native_dialog::{MessageDialog, MessageType};
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
    MessageDialog::new()
        .set_type(MessageType::Info)
        .set_title("Info")
        .set_text(msg)
        .show_alert()
        .unwrap_or_default();
}

pub fn warn_dialog(msg: &str) {
    MessageDialog::new()
        .set_type(MessageType::Warning)
        .set_title("Warning")
        .set_text(msg)
        .show_alert()
        .unwrap_or_default();
}

pub fn error_dialog(msg: &str) {
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
/// On Windows, it's `%LOCALAPPDATA%\{app_name}`.
///
/// On POSIX-compliant systems, path is always `~/.{app_name}`.
///
/// Fall backs:
///   - `%APPDATA%/../Local/{app_name}`
///   - `~/.{app_name}`
///
/// Home directory is underlyingly determined by [`home`](https://docs.rs/home) crate.
pub fn get_app_data_path(app_name: &str) -> PathBuf {
    if std::env::consts::OS == "windows" {
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            return PathBuf::from(local_appdata).join(app_name);
        }
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata)
                .parent()
                .unwrap_or(Path::new("."))
                .join("Local")
                .join(app_name);
        }
    }

    ["~/", ".", app_name]
        .concat()
        .expand_home()
        .unwrap_or_default()
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
    use std::env;
    use std::path::PathBuf;

    fn remove_envvars() {
        env::remove_var("HOME");
        env::remove_var("LOCALAPPDATA");
        env::remove_var("APPDATA");
    }

    #[test]
    fn test_get_data_path() {
        remove_envvars();
        env::set_var("LOCALAPPDATA", r"C:\Users\foo\AppData\Local");

        let data_path = get_app_data_path("memospot");
        assert_eq!(
            data_path,
            PathBuf::from(r"C:\Users\foo\AppData\Local\memospot")
        );

        remove_envvars();
        env::set_var("APPDATA", r"C:\Users\foo\AppData\Roaming");
        let data_path = get_app_data_path("memospot");
        assert_eq!(
            data_path,
            PathBuf::from(r"C:\Users\foo\AppData\Local\memospot")
        );

        remove_envvars();
        let data_path = get_app_data_path("memospot");
        assert!(data_path.has_root());
        assert!(data_path.is_absolute());
        assert!(data_path.ends_with(".memospot"));
    }
}
