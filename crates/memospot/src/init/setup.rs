/// Configuration and path setup.
///
/// Ensures data directories, configuration file, database files,
/// Memos server port, and server binary location. Runs database migrations.
use crate::fl;
use crate::memos_process;
use crate::runtime_config::RuntimePaths;
use crate::sqlite;
use crate::utils::*;
use crate::zip;
use config::{Config, Memos};
use dialog::*;
use homedir::HomeDirExt;
use log::{debug, info, warn};
use migration::{Migrator, MigratorTrait};
use std::env;
use std::env::consts::OS;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::time::Instant;
use writable::PathExt;

/// Ensure that data directory exists and is writable.
pub fn data_path(app_name: &str) -> PathBuf {
    let data_path = get_app_data_path(app_name);
    if !data_path.exists() {
        fs::create_dir_all(&data_path).expect_dialog(fl!(
            "panic-failed-to-create-data-directory",
            dir = data_path.to_string_lossy()
        ));
    }

    if !&data_path.is_writable() {
        panic_dialog!(fl!(
            "panic-data-directory-is-not-writable",
            dir = data_path.to_string_lossy()
        ));
    }
    data_path
}

/// Ensure that Memos data directory exists and is writable.
///
/// Use Memospot data directory if user-provided path is empty or ".".
/// Optionally, resolve a user-provided data directory.
pub fn memos_data(memos: &Memos, memospot_data: &Path) -> PathBuf {
    let data_str = memos.data.as_ref().map(|s| s.as_str().trim()).unwrap_or("");

    // Use Memospot data directory if user-provided path is empty or ".".
    // Prevents resolving data path to a non-writable directory,
    // like /usr/local/bin or "Program Files".
    if data_str.is_empty() || data_str == "." {
        return memospot_data.to_path_buf();
    }

    let expanded_path = PathBuf::from(data_str).expand_home().unwrap_or_default();
    let path = absolute_path(expanded_path).unwrap_or_else(|_| memospot_data.to_path_buf());
    if path.exists() && path.is_dir() {
        return path;
    }

    panic_dialog!(fl!(
        "panic-unable-to-resolve-custom-data-directory",
        dir = path.to_string_lossy()
    ));
}

/// Ensure that backup directory exists and is writable.
///
/// Use Memospot data directory if user-provided path is empty or ".".
/// Optionally, resolve a user-provided directory.
pub fn ensure_backup_directory(config: &Config, memospot_data: &Path) -> PathBuf {
    let folder_name = "backups";
    let default_path = memospot_data.join(folder_name);

    let cfg_path = config
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
        std::fs::create_dir_all(&path).expect_dialog(fl!(
            "panic-unable-to-create-backup-directory",
            dir = path.to_string_lossy()
        ));
    }

    if path.is_file() {
        panic_dialog!(fl!(
            "panic-backup-directory-is-a-file",
            dir = path.to_string_lossy()
        ));
    }

    if !&path.is_writable() {
        panic_dialog!(fl!(
            "panic-backup-directory-is-not-writable",
            dir = path.to_string_lossy()
        ));
    }

    path
}

/// Ensure that database files are writable, if they exist.
pub fn database(memos: &Memos, memos_data: &Path) -> PathBuf {
    let db_file = &format!("memos_{}.db", memos.mode.as_deref().unwrap_or_default());
    let db_path = memos_data.join(db_file);
    let files = vec![
        db_path.with_extension("db"),
        db_path.with_extension("db-wal"),
        db_path.with_extension("db-shm"),
    ];
    for file in files {
        if !file.exists() {
            continue;
        }
        // Remove demo database.
        //
        // Demo database is not handled by
        // migrations and can prevent Memos from starting if the schema is outdated.
        if memos.demo == Some(true) || memos.mode.as_deref() == Some("demo") {
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
            panic_dialog!(fl!(
                "panic-database-file-is-not-writable",
                file = file.to_string_lossy()
            ));
        }
    }
    db_path
}

/// Run database migrations.
pub async fn migrate_database(config: &Config, paths: &RuntimePaths) {
    if !config.memospot.migrations.enabled.unwrap_or_default() {
        warn!("database migration: disabled via configuration");
        return;
    }
    if !paths.memos_db_file.exists() {
        return;
    }

    let db_file = paths.memos_db_file.clone();
    let db_conn = sqlite::get_database_connection(&db_file)
        .await
        .expect_dialog(fl!("panic-failed-to-connect-to-database"));

    let migration_amount = Migrator::get_pending_migrations(&db_conn)
        .await
        .unwrap_or_default()
        .len();
    let _ = db_conn.close().await;
    if migration_amount == 0 {
        debug!("database migration: no pending migrations found");
        return;
    }

    if config.memospot.backups.enabled.unwrap_or_default() {
        let backup_path = ensure_backup_directory(config, &paths.memospot_data);
        let datetime = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
        let backup_name = format!("db-{datetime}-pre-migration.zst.zip");
        let backup_path = backup_path.join(&backup_name);
        let start_time = Instant::now();
        let backup =
            zip::related_files(&paths.memos_db_file, &["db-wal", "db-shm"], &backup_path);
        match backup.await {
            Ok(_) => {
                info!(
                    "database migration: backup completed. Operation took {:?}. Backup file: {}.",
                    start_time.elapsed(),
                    backup_path.to_string_lossy()
                );
            }
            Err(e) => {
                warn_dialog!(fl!("warn-failed-to-backup-database", error = e.to_string()));
            }
        }
    }

    let start_time = Instant::now();
    let db_file = paths.memos_db_file.clone();
    let db_conn = sqlite::get_database_connection(&db_file)
        .await
        .expect_dialog(fl!("panic-failed-to-connect-to-database"));

    if let Err(e) = Migrator::up(&db_conn, None).await {
        warn_dialog!(fl!(
            "panic-failed-to-run-database-migrations",
            error = e.to_string()
        ));
    }
    db_conn
        .close()
        .await
        .expect_dialog(fl!("panic-failed-to-close-database-connection"));

    info!(
        "database migration: Ran {} migrations in {:?}.",
        migration_amount,
        start_time.elapsed(),
    );
}

/// Initialize application configuration.
///
/// - Ensure that configuration file exists and is writable.
/// - If configuration file is missing or malformed, optionally reset it to defaults.
pub fn config(config_path: &PathBuf) -> Config {
    if !config_path.exists() {
        Config::reset_file_blocking(config_path)
            .expect_dialog(fl!("panic-config-unable-to-create"));
    }

    if config_path.is_dir() {
        panic_dialog!(fl!(
            "panic-config-is-not-a-file",
            path = config_path.to_string_lossy()
        ));
    }

    if !config_path.is_writable() {
        panic_dialog!(fl!(
            "panic-config-is-not-writable",
            file = config_path.to_string_lossy()
        ));
    }

    let mut cfg_reader = Config::init(config_path);
    if let Err(e) = cfg_reader {
        let user_confirmed = confirm_dialog(
            fl!("prompt-config-error-title").as_str(),
            fl!("prompt-config-error-message", error = e.limit_width()).as_str(),
            MessageType::Warning,
        );

        if !user_confirmed {
            panic_dialog!(fl!("panic-config-error"));
        }

        let now = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
        fs::copy(
            config_path,
            config_path.with_extension(format!("{now}.yaml")),
        )
        .expect_dialog(fl!("panic-config-unable-to-backup"));

        Config::reset_file_blocking(config_path)
            .expect_dialog(fl!("panic-config-unable-to-reset"));

        cfg_reader = Ok(Config::default());
    }
    let mut cfg = cfg_reader.expect_dialog(fl!("panic-config-parse-error"));
    memos_process::sync_mode_demo_compat(&mut cfg.memos);
    cfg
}

/// Ensure that Memos port is available.
///
/// Tries to find a free port if the configured one is already
/// in use and updates the referenced configuration in place.
pub fn memos_port(memos: &Memos) -> u16 {
    let preferred_port = memos.port.unwrap_or_default();
    if let Some(free_port) = portpicker::find_free_port(preferred_port) {
        return free_port;
    }

    panic_dialog!(fl!("panic-portpicker-error"));
}

/// Locate Memos server binary.
///
/// Look for Memos server binary in the following order:
/// 1. Provided Memos binary path from the configuration file.
/// 2. Memospot current working directory.
/// 3. Memospot data directory.
/// 4. ProgramData/memos (Windows).
/// 5. /Applications/Memospot.app/Contents/MacOS/memos (macOS)
/// 6. /usr/local/bin, /var/opt/memos, /usr/local/memos (Linux).
pub fn find_memos(memos: &Memos, memospot_data: &Path, memospot_cwd: &Path) -> PathBuf {
    #[cfg(debug_assertions)]
    {
        // cwd is target/debug/ on dev.
        let server_dist_dir = memospot_cwd
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("server-dist");

        let current_triple = env::var("TARGET_TRIPLE").unwrap_or_default();
        let binary = server_dist_dir.join(format!("memos-{current_triple}"));
        if binary.exists() {
            warn!("Using Memos server from {binary:?}");
            return binary;
        }
    }

    // Prefer path from the configuration file if it's valid.
    if let Some(binary_path) = &memos.binary_path {
        let yaml_bin = binary_path.as_str().trim();
        if !yaml_bin.is_empty() {
            let expanded_path = Path::new(yaml_bin).expand_home().unwrap_or_default();
            let path = absolute_path(expanded_path).unwrap_or_default();
            if path.exists() && path.is_file() {
                return path;
            }
        }
    }

    let mut search_paths: Vec<PathBuf> =
        Vec::from([memospot_cwd.to_path_buf(), memospot_data.to_path_buf()]);

    // Fall backs.
    match OS {
        "windows" => {
            if let Ok(program_files) = env::var("PROGRAMFILES") {
                search_paths.push(PathBuf::from(program_files).join("Memospot"));
            }
            if let Ok(program_data) = env::var("PROGRAMDATA") {
                search_paths.push(PathBuf::from(program_data).join("memos"));
            }
        }
        "macos" => {
            search_paths.push(PathBuf::from("/Applications/Memospot.app/Contents/MacOS"));
        }
        "linux" => {
            search_paths.push(PathBuf::from("/usr/bin"));
            search_paths.push(PathBuf::from("/usr/local/bin"));
            search_paths.push(
                PathBuf::from("~/.local/bin")
                    .expand_home()
                    .unwrap_or_default(),
            );
        }
        _ => {}
    };

    let binary_name = match OS {
        "windows" => "memos.exe",
        _ => "memos",
    };

    debug!("Looking for Memos server at: {search_paths:?}");
    for path in search_paths {
        let memos_path = path.join(binary_name);
        if memos_path.exists() && memos_path.is_file() {
            info!("Memos server found at: {}", memos_path.to_string_lossy());
            return memos_path;
        }
    }

    panic_dialog!(fl!("panic-unable-to-find-memos-binary"));
}
