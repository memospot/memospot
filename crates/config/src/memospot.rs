use std::collections::HashMap;

use crate::log::Log;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Migrations {
    pub enabled: bool,
    pub history: Option<HashMap<String, u64>>,
    // pub last_run: String,
    // pub last_run_version: String,
}
// #[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
// pub struct UnmanagedServer {
//     pub enabled: bool,
//     pub full_url: String,
// }

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Memospot {
    pub database_migrations: Migrations,
    // pub unmanaged_server: UnmanagedServer,
    pub log: Log,
}
