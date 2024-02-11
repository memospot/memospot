use serde::{Deserialize, Serialize};

// https://github.com/estk/log4rs/blob/main/docs/Configuration.md

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Log {
    pub enabled: bool,
    pub file: String,
    pub level: String,
    pub pattern: String,
    pub rotation: LogRotation,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct LogRotation {
    pub enabled: bool,
    pub max_size: String,
    pub amount: u16,
    pub path_mask: String,
}
