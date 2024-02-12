use serde::{Deserialize, Serialize};

// https://github.com/estk/log4rs/blob/main/docs/Configuration.md

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Log {
    pub enabled: bool,
}
