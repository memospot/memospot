use serde::{Deserialize, Serialize};
use ts_rs::TS;

// https://github.com/estk/log4rs/blob/main/docs/Configuration.md

#[derive(TS, Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Log {
    pub enabled: Option<bool>,
}
impl Default for Log {
    fn default() -> Self {
        Self {
            enabled: Some(false),
        }
    }
}
