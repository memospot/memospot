use crate::runtime_config::RuntimeConfig;
use chrono::prelude::*;
use itertools::Itertools;
use log::{debug, error, info};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use tokio::time::Instant;

/// Create a database connection pool and return an open connection.
pub async fn create_pool(db: &Path) -> Result<SqlitePool> {
    let connection_options = SqliteConnectOptions::new()
        .filename(db)
        .create_if_missing(false)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .foreign_keys(false)
        // .log_statements(log::LevelFilter::Debug)
        .busy_timeout(std::time::Duration::from_secs(30));

    SqlitePoolOptions::new()
        .connect_with(connection_options)
        .await
        .map_err(|e| Error::new(ErrorKind::ConnectionRefused, e))
}

/// Checkpoint database WAL.
///
/// Memos is currently not being gracefully shutdown, so we need to checkpoint the
/// database WAL manually right before closing the app to ensure that all new
/// data is commited to the main database and that it's properly closed.
pub async fn checkpoint(rconfig: &RuntimeConfig) {
    debug!("Checkpointing database WAL...");

    let db = &rconfig.paths.memos_db_file;
    let pool = match create_pool(db).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to connect to database `{}`: {}", &db.display(), e);
            return;
        }
    };

    let query_result = sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
        .execute(&pool)
        .await;
    match query_result {
        Ok(r) => debug!("Database WAL checkpointed: {:?}", r),
        Err(e) => error!("Failed to checkpoint database WAL: {}", e),
    };

    let _ = pool.close().await;
    let _ = pool.close_event().await;
}

/// Register migration history.
pub async fn write_migration_history(
    migration_fn: &str,
    rconfig: &mut RuntimeConfig,
) -> Result<()> {
    let migration_record: HashMap<String, i64> = HashMap::from_iter(vec![(
        migration_fn.to_string(),
        chrono::offset::Utc::now().timestamp(),
    )]);

    // Add timestamp to memospot.database_migrations.history.
    match rconfig.yaml.memospot.migrations.history {
        Some(ref mut migration_history) => {
            migration_history.extend(migration_record);
            rconfig.yaml.memospot.migrations.history = Some(migration_history.clone());
        }
        None => {
            rconfig.yaml.memospot.migrations.history = Some(migration_record);
        }
    };
    Ok(())
}

pub async fn get_migration_ts(rconfig: &RuntimeConfig, name: &str) -> Option<i64> {
    if let Some(migration) = rconfig.yaml.memospot.migrations.history.as_ref().cloned() {
        if migration.contains_key(name) {
            return Some(*migration.get(name).unwrap());
        }
    }
    None
}

/// Run programmatic database migrations.
///
/// Stores migration history in the configuration file. History is used to prevent
/// running the same migration multiple times, and also makes it possible to update
/// a migration code and invalidate a previous run.
///
/// Receives a mutable reference to `RuntimeConfig` to write back migration history.
pub async fn migrate(rconfig: &mut RuntimeConfig) -> Result<()> {
    if !rconfig.yaml.memospot.migrations.enabled.unwrap_or_default() {
        debug!("Database migrations are disabled.");
        return Ok(());
    }

    const KEY_LOCAL_RESOURCE_PATHS: &str = "db_local_resource_paths";
    let migrate_local_resource_paths_ts = get_migration_ts(rconfig, KEY_LOCAL_RESOURCE_PATHS)
        .await
        .unwrap_or(0);

    const TS_2024_02_11: i64 = 1707682533;
    if TS_2024_02_11 < migrate_local_resource_paths_ts {
        let dt: DateTime<Utc> =
            DateTime::from_timestamp(migrate_local_resource_paths_ts, 0).unwrap();
        debug!(
            "Database migration `{}` has already been run on {}.",
            KEY_LOCAL_RESOURCE_PATHS,
            dt.format("%Y-%m-%d %H:%M:%S UTC")
        );
    } else {
        info!("Running database migration `{}`.", KEY_LOCAL_RESOURCE_PATHS);
        migration_db_local_resource_paths(rconfig).await?;
        write_migration_history(KEY_LOCAL_RESOURCE_PATHS, rconfig).await?;
    }

    Ok(())
}

/// Migrate absolute resource paths to relative paths.
///
/// This may be needed for Memos upgrades that skipped version 0.18.2 and initial 0.19.0.
/// The path migrator is not present in 0.19.2 and later, so we need to run it manually.
///
/// Triggering paths:
/// - /var/opt/memos/
/// - Memos data path
/// - Memospot data path
/// - Windows paths containing backslashes
///
/// Notes:
/// - Migrating 300k resources takes about 10 seconds on a modern NVMe SSD and a decent CPU.
/// - Migration will first check if any path is absolute and then update the database in a single transaction.
///
/// Returns `true` if any paths were migrated, otherwise `false`.
pub async fn migration_db_local_resource_paths(rconfig: &RuntimeConfig) -> Result<bool> {
    fn norm_path(path: &str) -> String {
        path.replace(r#"\\"#, r#"\"#).replace("//", "/")
    }
    fn norm_prefix(path: &str) -> String {
        let mut p = path.to_string();
        if p.contains('/') && !p.ends_with('/') {
            p += "/";
        }
        if p.contains('\\') && !p.ends_with('\\') {
            p += "\\";
        }
        p
    }
    fn to_slash(path: &str) -> String {
        path.replace('\\', "/")
    }

    let memos_docker = "/var/opt/memos/".to_string();
    let memospot_data: String = norm_prefix(&rconfig.paths.memospot_data.to_string_lossy());
    let memos_data: String = norm_prefix(&rconfig.paths.memos_data.to_string_lossy());

    let db_pool = create_pool(&rconfig.paths.memos_db_file).await?;

    let migration_check = sqlx::query(
        "
                SELECT id FROM resource
                WHERE
                    internal_path LIKE '%\\%'
                    OR internal_path LIKE ?
                    OR internal_path LIKE ?
                    OR internal_path LIKE ?
                LIMIT 1;
            ",
    )
    .bind(format!("{}%", memos_docker))
    .bind(format!("{}%", &memos_data))
    .bind(format!("{}%", &memospot_data))
    .fetch_optional(&db_pool)
    .await
    .map_err(|e| Error::new(ErrorKind::Other, e))?;
    match migration_check {
        None => {
            debug!("Resource internal path migration is not required.");
            return Ok(false);
        }
        Some(_) => {
            info!("Migrating resource internal paths from absolute to relative.");
        }
    }

    #[derive(sqlx::FromRow)]
    struct Resource {
        id: i64,
        internal_path: String,
    }

    let list_resources_stmt = "
    SELECT id, internal_path FROM resource
    WHERE
        internal_path IS NOT ''
        AND internal_path IS NOT NULL
        AND internal_path LIKE '%\\%'
        OR internal_path LIKE ?
        OR internal_path LIKE ?
        OR internal_path LIKE ?;
    ";
    let resources = sqlx::query_as::<_, Resource>(list_resources_stmt)
        .bind(format!("{}%", memos_docker))
        .bind(format!("{}%", &memos_data))
        .bind(format!("{}%", &memospot_data))
        .fetch_all(&db_pool)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e))?;

    let mut tx = db_pool
        .begin()
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e))?;

    let update_stmt = "UPDATE resource SET internal_path = ? WHERE id = ?";
    let start_time = Instant::now();

    let mut migrated_count = 0;
    let total_resources = resources.len();
    let log_step = match total_resources {
        0..=100 => 1,
        101..=1000 => 10,
        1001..=10000 => 20,
        10001..=100000 => 50,
        100001..=1000000 => 100,
        _ => 100000,
    };
    let log_interval = total_resources / log_step;

    let migrate_paths: Vec<String> = [memos_docker, memos_data, memospot_data]
        .into_iter()
        .unique()
        .collect();

    for resource in resources.iter() {
        let internal_path = norm_path(&resource.internal_path);

        // Find the first matching path and strip it from the internal path.
        let mut new_path = migrate_paths
            .iter()
            .find_map(|p| internal_path.strip_prefix(p))
            .map(|s| s.to_string())
            .unwrap_or_default();
        if new_path.is_empty() {
            // Skip unmatched paths that also doesn't contain backslashes.
            if !internal_path.contains('\\') {
                continue;
            }
            // Allows later conversion of relative Windows paths to Unix paths.
            new_path = internal_path;
        }

        // Prevent internal_path from starting with a slash.
        new_path = to_slash(&new_path).trim_start_matches('/').to_string();

        sqlx::query(update_stmt)
            .bind(&new_path)
            .bind(resource.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, e))?;

        migrated_count += 1;
        if migrated_count < 100 || migrated_count % log_interval == 0 {
            match log::max_level() {
                log::LevelFilter::Info => {
                    info!(
                        "[Running] Migrated {}/{} paths.",
                        migrated_count, total_resources,
                    );
                }
                log::LevelFilter::Debug => {
                    debug!(
                        "[Running] Migrated {}/{} paths.\nLast: {} => {}",
                        migrated_count, total_resources, &resource.internal_path, new_path
                    );
                }
                _ => {}
            }
        }
    }

    tx.commit()
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e))?;

    info!(
        "[Completed] Migrated {} local resource paths in {:?}.",
        migrated_count,
        start_time.elapsed()
    );

    let _ = db_pool.close().await;
    let _ = db_pool.close_event().await;
    Ok(true)
}
