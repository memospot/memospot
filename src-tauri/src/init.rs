/// Runtime checks and initialization code.
///
/// Functions in this module panics with native dialogs instead of returning errors.
/// Main purpose is to unclutter `main.rs`.
use crate::{webview, RuntimeConfig};
use config::Config;
use log::{debug, info, warn};
use memospot::*;
use native_dialog::MessageType;
use std::env;
use std::env::consts::OS;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;
use writable::PathExt;

/// Ensure that data directory exists and is writable.
pub fn data_path(app_name: &str) -> PathBuf {
    let data_path = get_app_data_path(app_name);
    if !data_path.exists() {
        if let Err(e) = std::fs::create_dir_all(&data_path) {
            panic_dialog!(
                "Failed to create data directory `{}`:\n{}",
                data_path.to_string_lossy(),
                e.to_string()
            );
        }
    }

    if !&data_path.is_writable() {
        panic_dialog!(
            "Data directory is not writable:\n{}",
            data_path.to_string_lossy()
        );
    }
    data_path
}

/// Ensure that Memos's data directory exists and is writable.
///
/// Use Memospot's data directory if user-provided path is empty or ".".
/// Optionally, resolve a user-provided data directory.
pub fn memos_data(rconfig: &RuntimeConfig) -> PathBuf {
    let data_str = rconfig
        .yaml
        .memos
        .data
        .as_ref()
        .map(|s| s.as_str().trim())
        .unwrap_or("");

    // Use Memospot data directory if user-provided path is empty or ".".
    // Prevents resolving data path to a non-writable directory,
    // like /usr/local/bin or "Program Files".
    if data_str.is_empty() || data_str == "." {
        return rconfig.paths.memospot_data.to_path_buf();
    }

    let path = absolute_path(PathBuf::from(data_str))
        .unwrap_or_else(|_| rconfig.paths.memospot_data.to_path_buf());
    if path.exists() && path.is_dir() {
        return path;
    }

    panic_dialog!(
        "Failed to resolve custom Memos data directory!\n{}\n\nEnsure it exists and is a directory, or remove the setting `memos.data` to use the default data path.",
        path.to_string_lossy()
    );
}

/// Ensure that database files are writable, if they exist.
pub fn database(rconfig: &RuntimeConfig) -> PathBuf {
    let db_file = &format!(
        "memos_{}.db",
        rconfig.yaml.memos.mode.clone().unwrap_or_default()
    );
    let db_path = rconfig.paths.memos_data.join(db_file);
    let files = vec![
        db_path.with_extension("db"),
        db_path.with_extension("db-wal"),
        db_path.with_extension("db-shm"),
    ];
    for file in files {
        if file.exists() && !&file.is_writable() {
            panic_dialog!("Database file is not writable:\n{}", file.to_string_lossy());
        }
    }
    db_path
}

/// Ensure that WebView is available.
pub fn ensure_webview() {
    if webview::is_available() {
        return;
    }

    let user_confirmed = confirm_dialog(
            "WebView Error",
            "WebView is *required* for this application to work and it's not available on this system!\
            \n\nDo you want to install it?",
            MessageType::Error,
        );
    if !user_confirmed {
        warn!("User declined to setup WebView.");
        exit(1);
    }

    tauri::async_runtime::block_on(async move {
        if let Err(e) = webview::install().await {
            error_dialog!(
                "Failed to install WebView:\n{}\n\nPlease install it manually.",
                e
            );

            if let Err(e) = webview::open_install_website() {
                warn!("Failed to launch WebView download website:\n{}", e);
            }
            exit(1)
        }
    });

    if !webview::is_available() {
        panic_dialog!(
            "Unable to setup WebView!\n\n\
                Please install it manually and relaunch the application."
        );
    }
}

/// Initialize application configuration.
///
/// - Ensure that configuration file exists and is writable.
/// - If configuration file is missing or malformed, optionally reset it to defaults.
pub fn config(config_path: &PathBuf) -> Config {
    if !config_path.exists() {
        if let Err(e) = Config::reset_file(config_path) {
            panic_dialog!(
                "Failed to create configuration file:\n{}\n\n{}",
                config_path.to_string_lossy(),
                e.to_string()
            );
        }
    }

    if config_path.is_dir() {
        panic_dialog!(
            "Provided configuration path is a directory! It must be a file.\n{}",
            config_path.to_string_lossy()
        );
    }

    if !config_path.is_writable() {
        panic_dialog!(
            "Configuration file is not writable:\n{}",
            config_path.to_string_lossy()
        );
    }

    let mut cfg_reader = Config::init(config_path);
    if let Err(e) = cfg_reader {
        let user_confirmed = confirm_dialog(
            "Configuration Error",
            &format!(
                "Failed to parse configuration file:\n\n{}\n\n\
                Do you want to reset the configuration file and start the application with default settings?",
                e
            ),
            MessageType::Warning
        );

        if !user_confirmed {
            panic_dialog!("You must fix the config file manually and restart the application.");
        }

        if let Err(e) = Config::reset_file(config_path) {
            panic_dialog!(
                "Failed to reset configuration file `{}`:\n{}",
                config_path.to_string_lossy(),
                e.to_string()
            );
        }
        cfg_reader = Ok(Config::default());
    }

    let mut config = cfg_reader.unwrap_or_else(|e| {
        panic_dialog!("Failed to parse configuration file:\n{}", e.to_string());
    });

    if cfg!(dev) {
        // Use Memos in demo mode during development,
        // as it's already seeded with some data.
        config.memos.mode = Some("demo".to_string());
        let current_port = config.memos.port.unwrap_or_default();
        if current_port != 0 {
            config.memos.port = Some(current_port + 1);
        }
    }
    config
}

/// Ensure that Memos port is available.
///
/// Tries to find a free port if the configured one is already
/// in use and updates the referenced configuration in place.
pub fn memos_port(rconfig: &RuntimeConfig) -> u16 {
    if let Some(free_port) =
        portpicker::find_free_port(rconfig.yaml.memos.port.unwrap_or_default())
    {
        return free_port;
    }

    panic_dialog!("Failed to find an open port!");
}

/// Locate Memos's server binary.
///
/// Look for Memos's server binary in the following order:
/// 1. Provided Memos's binary path from the configuration file.
/// 2. Memospot's current working directory.
/// 3. Memospot's data directory.
/// 4. ProgramData/memos (Windows only).
/// 5. /usr/local/bin, /var/opt/memos, /usr/local/memos (POSIX only).
pub fn find_memos(rconfig: &RuntimeConfig) -> PathBuf {
    // Prefer path from the configuration file if it's valid.
    if let Some(binary_path) = &rconfig.yaml.memos.binary_path {
        let yaml_bin = binary_path.as_str().trim();
        if !yaml_bin.is_empty() {
            let path = absolute_path(Path::new(yaml_bin)).unwrap_or_default();
            if path.exists() && path.is_file() {
                return path;
            }
        }
    }

    let binary_name = match OS {
        "windows" => "memos.exe",
        _ => "memos",
    };

    let mut search_paths: Vec<PathBuf> = Vec::from([
        rconfig.paths.memospot_cwd.clone(),
        rconfig.paths.memospot_data.clone(),
    ]);

    // Windows fall back.
    if OS == "windows" {
        if let Ok(program_data) = env::var("PROGRAMDATA") {
            search_paths.push(PathBuf::from(program_data).join("memos"));
        }
    } else {
        // POSIX fall back.
        search_paths.push(PathBuf::from("/usr/local/bin"));
        search_paths.push(PathBuf::from("/var/opt/memos"));
        search_paths.push(PathBuf::from("/usr/local/memos"));
    }

    debug!("Looking for Memos server at: {:?}", search_paths);
    for path in search_paths {
        let memos_path = path.join(binary_name);
        if memos_path.exists() && memos_path.is_file() {
            info!("Memos's server found at: {}", memos_path.to_string_lossy());
            return memos_path;
        }
    }

    panic_dialog!("Unable to find Memos server!");
}

static LOGGING_CONFIG_YAML: &str = r#"
# Log4rs configuration file.
# https://github.com/estk/log4rs#quick-start
#
# Use absolute paths for file appender. Otherwise, it'll try to write next to the application binary.
# Data directory is available as: $ENV{MEMOSPOT_DATA}
appenders:
    file:
        encoder:
            pattern: "{d(%Y-%m-%d %H:%M:%S)} - {h({l})}: {m}{n}"
        path: $ENV{MEMOSPOT_DATA}/memospot.log
        kind: rolling_file
        policy:
            trigger:
                kind: size
                limit: 10 mb
            roller:
                kind: fixed_window
                pattern: $ENV{MEMOSPOT_DATA}/memospot.log.{}.gz
                count: 5
                base: 1
root:
    # debug | info | warn | error | off
    level: info
    appenders:
        - file
"#;

/// Setup logging if it's enabled.
///
/// - Validates `logging_config.yaml`.
///
/// Return true if logging is enabled.
pub fn setup_logger(rconfig: &RuntimeConfig) -> bool {
    if !rconfig.yaml.memospot.log.enabled.unwrap_or_default() {
        return false;
    }
    let log_config: PathBuf = rconfig.paths.memospot_data.join("logging_config.yaml");

    // Allows using $ENV{MEMOSPOT_DATA} in log4rs config.
    std::env::set_var(
        "MEMOSPOT_DATA",
        rconfig.paths.memospot_data.to_string_lossy().to_string(),
    );
    if log4rs::init_file(&log_config, Default::default()).is_ok() {
        // logging is enabled and config is ok
        return true;
    }

    // Logging is enabled, but config is bad.
    if let Ok(mut file) = File::create(&log_config) {
        let config_template = LOGGING_CONFIG_YAML.replace("    ", "  ");
        if let Err(e) = file.write_all(config_template.as_bytes()) {
            panic_dialog!(
                "Failed to write to `{}`:\n{}",
                log_config.to_string_lossy(),
                e.to_string()
            );
        }
        if let Err(e) = file.flush() {
            panic_dialog!(
                "Failed to flush `{}` to disk:\n{}",
                log_config.to_string_lossy(),
                &e
            );
        }
    } else {
        panic_dialog!(
            "Failed to truncate `{}`. Please delete it and restart the application.",
            log_config.to_string_lossy()
        );
    }

    if log4rs::init_file(&log_config, Default::default()).is_ok() {
        // Logging is enabled and config was reset.
        return true;
    }

    panic_dialog!(
        "Failed to setup logging!\nPlease delete `{}` and restart the application.",
        log_config.to_string_lossy()
    );
}
