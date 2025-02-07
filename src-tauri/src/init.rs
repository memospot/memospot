/// Runtime checks and initialization code.
///
/// Functions in this module panics with native dialogs instead of returning errors.
/// Main purpose is to unclutter `main.rs`.
use crate::fl;
use crate::runtime_config::RuntimeConfig;
use crate::sqlite;
use crate::utils::*;
use crate::webview;
use crate::zip;
use config::Config;
use dialog::*;
use homedir::HomeDirExt;
use log::{debug, info, warn};
use migration::{Migrator, MigratorTrait};
use std::env;
use std::env::consts::OS;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;
use tokio::time::Instant;
use writable::PathExt;

/// Ensure that data directory exists and is writable.
pub fn data_path(app_name: &str) -> PathBuf {
    let data_path = get_app_data_path(app_name);
    if !data_path.exists() {
        if let Err(e) = fs::create_dir_all(&data_path) {
            panic_dialog!(
                "{}",
                fl!(
                    "panic-failed-to-create-data-directory",
                    dir = data_path.to_string_lossy(),
                    error = e.to_string()
                ),
            );
        }
    }

    if !&data_path.is_writable() {
        panic_dialog!(
            "{}",
            fl!(
                "panic-data-directory-is-not-writable",
                dir = data_path.to_string_lossy()
            )
        );
    }
    data_path
}

/// Ensure that Memos data directory exists and is writable.
///
/// Use Memospot data directory if user-provided path is empty or ".".
/// Optionally, resolve a user-provided data directory.
pub fn memos_data(rtcfg: &RuntimeConfig) -> PathBuf {
    let data_str = rtcfg
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
        return rtcfg.paths.memospot_data.to_path_buf();
    }

    let expanded_path = PathBuf::from(data_str).expand_home().unwrap_or_default();
    let path = absolute_path(expanded_path)
        .unwrap_or_else(|_| rtcfg.paths.memospot_data.to_path_buf());
    if path.exists() && path.is_dir() {
        return path;
    }

    panic_dialog!(
        "{}",
        fl!(
            "panic-unable-to-resolve-custom-data-directory",
            dir = path.to_string_lossy()
        )
    );
}

/// Ensure that backup directory exists and is writable.
///
/// Use Memospot data directory if user-provided path is empty or ".".
/// Optionally, resolve a user-provided directory.
pub fn ensure_backup_directory(rtcfg: &RuntimeConfig) -> PathBuf {
    let folder_name = "backups";
    let default_path = rtcfg.paths.memospot_data.join(folder_name);

    let cfg_path = rtcfg
        .yaml
        .memospot
        .backups
        .path
        .as_ref()
        .map(|s| s.as_str().trim())
        .unwrap_or("");

    // Use default directory if user-provided path is empty or ".".
    // Prevents resolving data path to a non-writable directory,
    // like /usr/local/bin or "Program Files".
    let path: PathBuf = if cfg_path.is_empty() || cfg_path == "." || cfg_path == folder_name {
        default_path
    } else {
        let expanded_path = PathBuf::from(cfg_path).expand_home().unwrap_or_default();
        absolute_path(expanded_path).unwrap_or(default_path)
    };

    if !path.exists() {
        if let Err(e) = std::fs::create_dir_all(&path) {
            panic_dialog!(
                "{}",
                fl!(
                    "panic-unable-to-create-backup-directory",
                    dir = path.to_string_lossy(),
                    error = e.to_string()
                )
            );
        }
    }

    if path.is_file() {
        panic_dialog!(
            "{}",
            fl!(
                "panic-backup-directory-is-a-file",
                dir = path.to_string_lossy()
            )
        );
    }

    if !&path.is_writable() {
        panic_dialog!(
            "{}",
            fl!(
                "panic-backup-directory-is-not-writable",
                dir = path.to_string_lossy()
            )
        );
    }

    path
}

/// Ensure that database files are writable, if they exist.
pub fn database(rtcfg: &RuntimeConfig) -> PathBuf {
    let db_file = &format!(
        "memos_{}.db",
        rtcfg.yaml.memos.mode.as_deref().unwrap_or_default()
    );
    let db_path = rtcfg.paths.memos_data.join(db_file);
    let files = vec![
        db_path.with_extension("db"),
        db_path.with_extension("db-wal"),
        db_path.with_extension("db-shm"),
    ];
    for file in files {
        if !file.exists() {
            continue;
        }
        // Remove demo database in dev/debug mode. Demo database is not handled by
        // migrations and can prevent Memos from starting if the model is outdated.
        if cfg!(debug_assertions) && rtcfg.yaml.memos.mode.as_deref() == Some("demo") {
            match std::fs::remove_file(&file) {
                Ok(_) => warn!(
                    "Demo database \"{}\" removed.",
                    file.file_name().unwrap_or_default().to_string_lossy()
                ),
                Err(e) => warn_dialog!("Failed to remove demo database:\n{}", e),
            }
            continue;
        }
        if !&file.is_writable() {
            panic_dialog!(
                "{}",
                fl!(
                    "panic-database-file-is-not-writable",
                    file = file.to_string_lossy()
                )
            );
        }
    }
    db_path
}

/// Run database migrations.
pub async fn migrate_database(rtcfg: &RuntimeConfig) {
    if !rtcfg.yaml.memospot.migrations.enabled.unwrap_or_default() {
        warn!("Database migrations were disabled via configuration.");
        return;
    }
    if !rtcfg.paths.memos_db_file.exists() {
        return;
    }

    let db_file = rtcfg.paths.memos_db_file.clone();
    let db_conn = sqlite::get_database_connection(&db_file)
        .await
        .unwrap_or_else(|e| {
            panic_dialog!(
                "{}",
                fl!("panic-failed-to-connect-to-database", error = e.to_string())
            );
        });
    let migration_amount = Migrator::get_pending_migrations(&db_conn)
        .await
        .unwrap_or_default()
        .len();
    let _ = db_conn.close().await;
    if migration_amount == 0 {
        debug!("No pending migrations found.");
        return;
    }

    if rtcfg.yaml.memospot.backups.enabled.unwrap_or_default() {
        let backup_path = ensure_backup_directory(rtcfg);
        let datetime = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
        let backup_name = format!("db-{}-pre-migration.zst.zip", datetime);
        let backup_path = backup_path.join(&backup_name);
        let start_time = Instant::now();
        let backup = zip::related_files(
            &rtcfg.paths.memos_db_file,
            &["db-wal", "db-shm"],
            &backup_path,
        );
        match backup.await {
            Ok(_) => {
                info!(
                    "Database backup completed successfully! Operation took {:?}. Backup file: {}",
                    start_time.elapsed(),
                    backup_path.to_string_lossy()
                );
            }
            Err(e) => {
                warn_dialog!(
                    "{}",
                    fl!("warn-failed-to-backup-database", error = e.to_string())
                );
            }
        }
    }

    let start_time = Instant::now();
    let db_file = rtcfg.paths.memos_db_file.clone();
    let db_conn = sqlite::get_database_connection(&db_file)
        .await
        .unwrap_or_else(|e| {
            panic_dialog!(
                "{}",
                fl!("panic-failed-to-connect-to-database", error = e.to_string())
            );
        });
    if let Err(e) = Migrator::up(&db_conn, None).await {
        warn_dialog!(
            "{}",
            fl!(
                "panic-failed-to-run-database-migrations",
                error = e.to_string()
            )
        );
    }
    db_conn.close().await.unwrap_or_else(|e| {
        panic_dialog!(
            "{}",
            fl!(
                "panic-failed-to-close-database-connection",
                error = e.to_string()
            )
        );
    });

    info!(
        "Database migrations took {:?}. Ran {} migrations.",
        start_time.elapsed(),
        migration_amount,
    );
}

/// Ensure that WebView is available.
pub fn ensure_webview() {
    if webview::is_available() {
        return;
    }

    let user_confirmed = confirm_dialog(
        fl("prompt-install-webview-title").as_str(),
        fl("prompt-install-webview-message").as_str(),
        MessageType::Error,
    );
    if !user_confirmed {
        warn!("User declined to setup WebView.");
        exit(1);
    }

    tauri::async_runtime::block_on(async move {
        if let Err(e) = webview::install().await {
            error_dialog!(
                "{}",
                fl!("error-failed-to-install-webview", error = e.to_string())
            );

            if let Err(e) = webview::open_install_website() {
                warn!("Failed to launch WebView download website:\n{}", e);
            }
            exit(1)
        }
    });

    if !webview::is_available() {
        panic_dialog!("{}", fl!("error-failed-to-install-webview", error = ""));
    }
}

/// Initialize application configuration.
///
/// - Ensure that configuration file exists and is writable.
/// - If configuration file is missing or malformed, optionally reset it to defaults.
pub fn config(config_path: &PathBuf) -> Config {
    if !config_path.exists() {
        if let Err(e) = Config::reset_file_blocking(config_path) {
            panic_dialog!(
                "{}",
                fl!(
                    "panic-config-unable-to-create",
                    file = config_path.to_string_lossy(),
                    error = e.to_string()
                )
            );
        }
    }

    if config_path.is_dir() {
        panic_dialog!(
            "{}",
            fl!(
                "panic-config-is-not-a-file",
                path = config_path.to_string_lossy()
            )
        );
    }

    if !config_path.is_writable() {
        panic_dialog!(
            "{}",
            fl!(
                "panic-config-is-not-writable",
                file = config_path.to_string_lossy()
            )
        );
    }

    let mut cfg_reader = Config::init(config_path);
    if let Err(e) = cfg_reader {
        let user_confirmed = confirm_dialog(
            fl!("prompt-config-error-title").as_str(),
            fl!("prompt-config-error-message", error = e.to_string()).as_str(),
            MessageType::Warning,
        );

        if !user_confirmed {
            panic_dialog!("{}", fl!("panic-config-error"));
        }

        let now = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
        if let Err(e) = fs::copy(
            config_path,
            config_path.with_extension(format!("{}.yaml", now)),
        ) {
            panic_dialog!(
                "{}",
                fl!("panic-config-unable-to-backup", error = e.to_string())
            );
        }

        if let Err(e) = Config::reset_file_blocking(config_path) {
            panic_dialog!(
                "{}",
                fl!("panic-config-unable-to-reset", error = e.to_string())
            );
        }
        cfg_reader = Ok(Config::default());
    }

    cfg_reader.unwrap_or_else(|e| {
        panic_dialog!("{}", fl!("panic-config-parse-error", error = e.to_string()));
    })
}

/// Ensure that Memos port is available.
///
/// Tries to find a free port if the configured one is already
/// in use and updates the referenced configuration in place.
pub fn memos_port(rtcfg: &RuntimeConfig) -> u16 {
    let preferred_port = rtcfg.yaml.memos.port.unwrap_or_default();
    if let Some(free_port) = portpicker::find_free_port(preferred_port) {
        return free_port;
    }

    panic_dialog!("{}", fl!("panic-portpicker-error"));
}

/// Memos URL.
///
/// It's ensured to end with a slash.
///
/// If remote server is enabled, return the configured URL.
/// Otherwise, return the default Memos address for the spawned server.
pub fn memos_url(rtcfg: &RuntimeConfig) -> String {
    if !rtcfg.yaml.memospot.remote.enabled.unwrap_or_default()
        || rtcfg.yaml.memospot.remote.url.is_none()
    {
        return format!(
            "http://localhost:{}/",
            rtcfg.yaml.memos.port.unwrap_or_default()
        );
    }

    let url = rtcfg.yaml.memospot.remote.url.as_deref().unwrap();
    if url.is_empty() || !url.starts_with("http") {
        error_dialog!("{}", fl!("error-invalid-server-url", url = url));
    }

    url.trim_end_matches('/').to_string() + "/"
}

/// Locate Memos server binary.
///
/// Look for Memos server binary in the following order:
/// 1. Provided Memos binary path from the configuration file.
/// 2. Memospot current working directory.
/// 3. Memospot data directory.
/// 4. ProgramData/memos (Windows only).
/// 5. /usr/local/bin, /var/opt/memos, /usr/local/memos (POSIX only).
pub fn find_memos(rtcfg: &RuntimeConfig) -> PathBuf {
    // Prefer path from the configuration file if it's valid.
    if let Some(binary_path) = &rtcfg.yaml.memos.binary_path {
        let yaml_bin = binary_path.as_str().trim();
        if !yaml_bin.is_empty() {
            let expanded_path = Path::new(yaml_bin).expand_home().unwrap_or_default();
            let path = absolute_path(expanded_path).unwrap_or_default();
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
        rtcfg.paths.memospot_cwd.clone(),
        rtcfg.paths.memospot_data.clone(),
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
            info!("Memos server found at: {}", memos_path.to_string_lossy());
            return memos_path;
        }
    }

    panic_dialog!("{}", fl!("panic-unable-to-find-memos-binary"));
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
pub fn setup_logger(rtcfg: &RuntimeConfig) -> bool {
    if !rtcfg.yaml.memospot.log.enabled.unwrap_or_default() {
        return false;
    }
    let log_config: PathBuf = rtcfg.paths.memospot_data.join("logging_config.yaml");

    // SAFETY: We're setting an environment variable, which is generally safe.
    // The unsafe block is required due to the potential for race conditions in
    // a multithreaded context.
    unsafe {
        // Allows using $ENV{MEMOSPOT_DATA} in log4rs config.
        env::set_var(
            "MEMOSPOT_DATA",
            rtcfg.paths.memospot_data.to_string_lossy().to_string(),
        );
    }
    if log4rs::init_file(&log_config, Default::default()).is_ok() {
        // logging is enabled and config is ok
        return true;
    }

    // Logging is enabled, but config is bad.
    if let Ok(mut file) = File::create(&log_config) {
        let config_template = LOGGING_CONFIG_YAML.replace("    ", "  ");
        if let Err(e) = file.write_all(config_template.as_bytes()) {
            panic_dialog!(
                "{}",
                fl!(
                    "panic-log-config-write-error",
                    file = log_config.to_string_lossy(),
                    error = e.to_string()
                )
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
            "{}",
            fl!(
                "panic-log-config-reset-error",
                file = log_config.to_string_lossy()
            )
        );
    }

    if log4rs::init_file(&log_config, Default::default()).is_ok() {
        // Logging is enabled and config was reset.
        return true;
    }

    panic_dialog!(
        "{}",
        fl!("panic-log-setup-error", file = log_config.to_string_lossy())
    );
}

#[cfg(target_os = "linux")]
/// Set up WebView hardware acceleration.
///
/// There are known issues with WebView hardware acceleration on Nvidia GPUs under X11.
/// See: https://github.com/tauri-apps/tauri/issues/9394.
///
/// This function mitigates those issues by preemptively setting the following environment variables heuristically:
/// - `WEBKIT_DISABLE_COMPOSITING_MODE=1`
/// - `WEBKIT_DISABLE_DMABUF_RENDERER=1`
///
/// The variables are only set for the current process, leaving the system untouched.
pub fn hw_acceleration() {
    use std::process::{Command, Stdio};

    let disable_compositing = || {
        warn!("Forcing software rendering preemptively with 'WEBKIT_DISABLE_COMPOSITING_MODE=1'. Should this cause you issues, override this heuristic by setting 'memospot.env.WEBKIT_DISABLE_COMPOSITING_MODE'='0' on `memospot.yaml`.");
        // SAFETY: There's potential for race conditions in a multi-threaded context.
        // Shouldn't be an issue here.
        unsafe {
            env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        }
    };
    let disable_dmabuf_renderer = || {
        warn!("Disabling DMABuf rendering preemptively with 'WEBKIT_DISABLE_DMABUF_RENDERER=1'. Should this cause you issues, override this heuristic by setting 'memospot.env.WEBKIT_DISABLE_DMABUF_RENDERER'='0' on `memospot.yaml`.");
        // SAFETY: There's potential for race conditions in a multi-threaded context.
        // Shouldn't be an issue here.
        unsafe {
            env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    };

    if !Path::new("/dev/dri").exists() {
        warn!("No GPU renderer was detected.");
        disable_compositing();
        return;
    }

    let is_x11 = env::var("WAYLAND_DISPLAY").is_err()
        && env::var("XDG_SESSION_TYPE").unwrap_or_default() == "x11";
    if !is_x11 {
        debug!("No X11 session detected. Leaving hardware acceleration as-is.");
        return;
    }

    let is_flatpak = env::var("FLATPAK_ID").is_ok();
    if is_flatpak {
        warn!("Running as a Flatpak container under X11. This may present issues with Nvidia GPUs.");
        disable_dmabuf_renderer();
        return;
    }

    // This will return true only if lshw runs and detect a GeForce GPU on the system.
    // lshw won't run inside a Flatpak sandbox.
    let is_nvidia = Command::new("lshw")
        .args([
            "-quiet", "-short", "-disable", "disk", "-disable", "volume", "-disable", "usb",
            "-disable", "scsi", "-disable", "pnp", "-c", "display",
        ])
        .stdout(Stdio::piped())
        .output()
        .map(|cmd| String::from_utf8_lossy(&cmd.stdout).contains("GeForce"))
        .unwrap_or_default();
    if is_nvidia {
        warn!("Detected Nvidia GPU under X11. This may present issues with WebView.");
        disable_dmabuf_renderer();
        return;
    }
    debug!("No Nvidia GPU was detected. Leaving hardware acceleration as-is.");
}

/// Set Memospot environment variables.
///
/// This is intended to configure the WebView on edge cases, like passing
/// WEBKIT_DISABLE_COMPOSITING_MODE=1 to disable hardware acceleration on Linux.
///
/// Should be called after init::hw_acceleration() to allow user-defined overrides.
pub fn set_env_vars(rtcfg: &RuntimeConfig) {
    if !rtcfg.yaml.memospot.env.enabled.unwrap_or_default() {
        return;
    }

    if let Some(memospot_env) = &rtcfg.yaml.memospot.env.vars {
        for (key, value) in memospot_env {
            // SAFETY: The unsafe block is required due to the potential for race conditions in a multithreaded context.
            // Shouldn't be an issue here.
            unsafe {
                env::set_var(key, value);
            }
        }
    }
}
