use crate::log::Log;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Migrations {
    pub enabled: Option<bool>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Memospot {
    pub migrations: Migrations,
    pub log: Log,
}
