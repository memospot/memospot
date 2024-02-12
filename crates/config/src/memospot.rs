use std::collections::HashMap;

use crate::log::Log;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Migrations {
    pub enabled: bool,
    pub history: Option<HashMap<String, i64>>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Memospot {
    pub migrations: Migrations,
    // pub unmanaged_server: UnmanagedServer,
    pub log: Log,
}
