use anyhow::{anyhow, Result};
use log::{debug, warn};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection};
use std::path::Path;

/// Get a database connection using SeaORM.
pub async fn get_database_connection(db: &Path) -> Result<DatabaseConnection, anyhow::Error> {
    let database_url = format!("sqlite://{}", db.to_string_lossy());
    let mut opt = ConnectOptions::new(&database_url);

    opt.sqlx_logging(
        cfg!(debug_assertions)
            && [
                std::env::var("RUST_LOG").is_ok_and(|x| x.to_lowercase().contains("sea_orm")),
                std::env::var("MEMOSPOT_DEBUG_SQL").is_ok(),
            ]
            .iter()
            .any(|&x| x),
    );

    let db = Database::connect(opt).await?;
    if database_url.starts_with("sqlite") {
        let pragmas = [
            "PRAGMA foreign_keys = 0;",
            "PRAGMA cache_size = -16000;",
            "PRAGMA busy_timeout = 10000;",
        ]
        .join("\n");

        db.execute_unprepared(pragmas.as_str()).await?;
    }
    Ok(db)
}

/// Checkpoint database WAL.
///
/// Memos' currently not being gracefully shutdown, so we're checkpointing the
/// database WAL manually right before closing the app to ensure that all new
/// data is commited to the main database and that it's properly closed.
pub async fn checkpoint(db: &Path) -> Result<(), anyhow::Error> {
    let connection = get_database_connection(db).await?;
    connection
        .execute_unprepared("PRAGMA wal_checkpoint(TRUNCATE);")
        .await?;
    connection.close().await?;
    Ok(())
}

/// Wait for database checkpoint.
///
/// This is a blocking function.
pub async fn wait_checkpoint(db_file: &Path, interval_millis: u64, timeout_millis: u128) {
    debug!("Checkpointing database WAL…");
    let time_start = tokio::time::Instant::now();

    let mut last_error: Option<anyhow::Error> = None;
    let mut interval =
        tokio::time::interval(tokio::time::Duration::from_millis(interval_millis));

    let wal = &db_file.with_extension("db-wal");
    while wal.exists() {
        if time_start.elapsed().as_millis() > timeout_millis {
            last_error = anyhow!("operation timed out").into();
            break;
        }
        interval.tick().await;
        if let Err(e) = checkpoint(db_file).await {
            last_error = e.into()
        }
    }

    match last_error {
        Some(e) => {
            warn!(
                "Failed to checkpoint database WAL: {}. Giving up after {} ms.",
                e.to_string(),
                timeout_millis
            );
        }
        None => {
            debug!(
                "Database checkpoint took <{} ms. Database closed.",
                time_start.elapsed().as_millis()
            );
        }
    }
}
