use home_dir::HomeDirExt;
use native_dialog::{MessageDialog, MessageType};
use serde::Deserialize;
use serde::Serialize;
use std::io;
use std::io::ErrorKind;
use std::path::PathBuf;

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

pub fn panic_dialog(msg: &str) {
    MessageDialog::new()
        .set_type(MessageType::Error)
        .set_title("Fatal error")
        .set_text(msg)
        .show_alert()
        .unwrap();
    panic!("Fatal error: {}", msg);
}

pub fn info_dialog(msg: &str) {
    MessageDialog::new()
        .set_type(MessageType::Info)
        .set_title("Info")
        .set_text(msg)
        .show_alert()
        .unwrap();
}

pub fn warn_dialog(msg: &str) {
    MessageDialog::new()
        .set_type(MessageType::Warning)
        .set_title("Warning")
        .set_text(msg)
        .show_alert()
        .unwrap();
}

pub fn confirm_dialog(title: &str, msg: &str, icon: MessageType) -> bool {
    MessageDialog::new()
        .set_type(icon)
        .set_title(title)
        .set_text(msg)
        .show_confirm()
        .unwrap()
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct MemosCfg {
    pub port: u16,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct MemospotCfg {
    pub memos: MemosCfg,
}

impl Default for MemospotCfg {
    fn default() -> Self {
        MemospotCfg {
            memos: MemosCfg { port: 0 },
        }
    }
}

impl MemospotCfg {
    pub fn new() -> Self {
        MemospotCfg::default()
    }
}

/// Read configuration from supplied path.
pub fn read_memospot_config(cfg_path: &PathBuf) -> io::Result<MemospotCfg> {
    let cfg_file = std::fs::File::open(cfg_path)?;
    let yaml: Result<MemospotCfg, serde_yaml::Error> = serde_yaml::from_reader(&cfg_file);
    drop(cfg_file);

    yaml.map_err(|e| io::Error::new(ErrorKind::InvalidData, e))
}

/// Save configuration to supplied file.
///
/// - If the file does not exist, it will be created.
/// - If the file exists, it will be overwritten.
/// - Comments are not preserved.
pub fn save_memospot_config(cfg_path: &PathBuf, cfg: &MemospotCfg) -> io::Result<()> {
    if cfg_path.exists() {
        if cfg_path.is_dir() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "provided path is a directory",
            ));
        }

        if !writable(cfg_path) {
            return Err(io::Error::new(
                ErrorKind::PermissionDenied,
                "file is not writable",
            ));
        }
    }

    let mut last_error = io::Error::new(ErrorKind::Other, "unknown error");
    for retry in 0..10 {
        if retry > 0 {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        let file = std::fs::File::create(cfg_path);
        let Ok(writer) = file else {
            last_error = io::Error::new(ErrorKind::Other, file.unwrap_err());
            continue;
        };

        let yaml_writer = serde_yaml::to_writer(&writer, cfg);
        if let Err(e) = yaml_writer {
            last_error = io::Error::new(ErrorKind::InvalidData, e);
            continue;
        };
        return Ok(());
    }
    Err(last_error)
}

/// Reset Memospot config to default.
pub fn reset_memospot_config(cfg_path: &PathBuf) -> io::Result<()> {
    let default_config = MemospotCfg::default();
    save_memospot_config(cfg_path, &default_config)
}

/// Find an open port
pub fn find_open_port(preferred_port: u16) -> io::Result<u16> {
    if preferred_port != 0 && portpicker::is_free(preferred_port) {
        return Ok(preferred_port);
    }

    let free_port = portpicker::pick_unused_port();
    if let Some(port) = free_port {
        return Ok(port);
    }

    Err(io::Error::new(
        ErrorKind::AddrNotAvailable,
        "no free port found",
    ))
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
                .unwrap()
                .join("Local")
                .join(app_name);
        }
    }

    ["~/", ".", app_name]
        .concat()
        .expand_home()
        .unwrap_or_default()
}

/// Check if a path is writable.
///
/// If the path is a file, it will be opened with write permissions.
///
/// If the path is a directory, a temporary file will be created in it.
pub fn writable(path: &PathBuf) -> bool {
    if path.is_file() {
        for retry in 0..10 {
            if retry > 0 {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            if let Ok(file) = std::fs::OpenOptions::new().write(true).open(path) {
                drop(file);
                return true;
            }
        }
        return false;
    }

    if path.is_dir() {
        let mut testfile = path.to_owned();
        testfile = testfile.join("write_test");

        let mut count = 0;
        while testfile.exists() {
            testfile.set_extension(&count.to_string());
            count += 1;

            if count > 100 {
                return false;
            }
        }

        for retry in 0..10 {
            if retry > 0 {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            if let Ok(file) = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&testfile)
            {
                drop(file);
                if std::fs::remove_file(&testfile).is_err() {
                    continue;
                }
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use fs::read_to_string;
    use std::env;
    use std::fs;
    use std::io::Write;
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

    #[test]
    fn test_writable() {
        let tmp_dir = env::temp_dir();
        let unwritable = PathBuf::from("/");

        // test directory
        assert!(writable(&tmp_dir));
        assert!(!writable(&unwritable));

        let file = tmp_dir.join("memospot_writable_test");
        if let Ok(mut f) = std::fs::File::create(&file) {
            static TEST_CONTENT: &str = "test content";

            f.write_all(TEST_CONTENT.as_bytes()).unwrap();
            drop(f);

            assert!(writable(&file));
            assert_eq!(read_to_string(&file).unwrap(), TEST_CONTENT);

            fs::remove_file(&file).unwrap();
        } else {
            panic!("Failed to create file for testing");
        }
    }
}
