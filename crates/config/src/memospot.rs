use crate::log::Log;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Backups {
    /// Enable backups.
    pub enabled: Option<bool>,
    /// Directory where Memospot will store backups.
    pub path: Option<String>,
}
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Migrations {
    /// Enable database migrations.
    /// Can be disabled to use Memospot with an uncertified Memos version.
    pub enabled: Option<bool>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct RemoteServer {
    /// Enable remote server. This will disable spawning a local Memos server.
    pub enabled: Option<bool>,
    pub url: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Memospot {
    /// Backups settings.
    pub backups: Backups,
    /// Database migrations settings.
    pub migrations: Migrations,
    // Log settings.
    pub log: Log,
    /// Remote server settings.
    pub remote: RemoteServer,
}
