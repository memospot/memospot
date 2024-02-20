use crate::runtime_config::RuntimeConfig;
use log::{debug, error};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection};
use std::io::{Error, ErrorKind, Result};

/// Get a database connection using SeaORM.
pub async fn get_database_connection(rtcfg: &RuntimeConfig) -> Result<DatabaseConnection> {
    let database_url = format!("sqlite://{}", rtcfg.paths.memos_db_file.to_string_lossy());
    let mut opt = ConnectOptions::new(&database_url);
    opt.sqlx_logging(false);

    Database::connect(opt).await.map_err(|err| {
        Error::new(
            ErrorKind::ConnectionRefused,
            format!("Failed to connect to database: {}", err),
        )
    })
}

/// Checkpoint database WAL.
///
/// Memos is currently not being gracefully shutdown, so we need to checkpoint the
/// database WAL manually right before closing the app to ensure that all new
/// data is commited to the main database and that it's properly closed.
pub async fn checkpoint(rtcfg: &RuntimeConfig) {
    debug!("Checkpointing database WAL...");

    let db = match get_database_connection(rtcfg).await {
        Ok(conn) => conn,
        Err(err) => {
            error!("Failed to connect to database: {}", err);
            return;
        }
    };

    match db
        .execute_unprepared("PRAGMA wal_checkpoint(TRUNCATE);")
        .await
    {
        Ok(_) => debug!("Database WAL checkpointed."),
        Err(e) => error!("Failed to checkpoint database WAL: {}", e),
    }

    let _ = db.close().await;
}
