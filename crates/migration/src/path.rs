use homedir::HomeDirExt;

use std::path::{Path, PathBuf};

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
