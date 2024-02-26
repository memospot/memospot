use crate::log::Log;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Migrations {
    pub enabled: Option<bool>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Backup {
    pub enabled: Option<bool>,
    pub path: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Memospot {
    /// Directory where Memospot will store backups.
    pub backup: Backup,
    pub migrations: Migrations,
    pub log: Log,
}
