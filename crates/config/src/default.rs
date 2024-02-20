//! Default configuration for Memospot and Memos.
//!
//! This allows creating a default configuration file and also
//! calling the method `.unwrap_or_default()` on optional configuration fields.

use crate::log::Log;
use crate::memos::Memos;
use crate::memospot::{Memospot, Migrations};
use crate::Config;

impl Default for Config {
    fn default() -> Config {
        Config {
            memos: Memos {
                binary_path: None,
                working_dir: None,
                data: None,
                mode: Some("prod".to_string()),
                addr: Some("127.0.0.1".to_string()),
                port: Some(0),
                metric: Some(true),
                env: None,
                log: Log {
                    enabled: Some(false),
                },
            },
            memospot: Memospot {
                migrations: Migrations {
                    enabled: Some(false),
                },
                log: Log {
                    enabled: Some(false),
                },
            },
        }
    }
}
