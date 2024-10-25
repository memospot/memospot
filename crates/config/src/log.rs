use serde::{Deserialize, Serialize};

// https://github.com/estk/log4rs/blob/main/docs/Configuration.md

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
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
