use crate::runtime_config::RuntimeConfig;
use log::{debug, error};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection};
use std::io::{Error, ErrorKind, Result};

/// Checkpoint database WAL.
///
/// Memos is currently not being gracefully shutdown, so we need to checkpoint the
/// database WAL manually right before closing the app to ensure that all new
/// data is commited to the main database and that it's properly closed.
pub async fn checkpoint(rconfig: &RuntimeConfig) {
    debug!("Checkpointing database WAL...");

    let db = match get_database_connection(rconfig).await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            return;
        }
    };

    let _ = db
        .execute_unprepared("PRAGMA wal_checkpoint(TRUNCATE);")
        .await;

    let _ = db.close().await;
}

/// Get a database connection using SeaORM.
pub async fn get_database_connection(rconfig: &RuntimeConfig) -> Result<DatabaseConnection> {
    let database_url = format!("sqlite://{}", rconfig.paths.memos_db_file.to_string_lossy());

    let mut opt = ConnectOptions::new(&database_url);
    opt.sqlx_logging(false);

    Database::connect(opt).await.map_err(|e| {
        Error::new(
            ErrorKind::ConnectionRefused,
            format!("Failed to connect to database: {}", e),
        )
    })
}

pub async fn count_pending_migrations(connection: DatabaseConnection) -> usize {
    if let Ok(pending_migrations) = Migrator::get_pending_migrations(&connection).await {
        return pending_migrations.len();
    }
    0
}
