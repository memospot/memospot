use std::collections::HashMap;

use crate::log::Log;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Migrations {
    pub enabled: Option<bool>,
    pub history: Option<HashMap<String, i64>>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Memospot {
    pub migrations: Migrations,
    pub log: Log,
}
