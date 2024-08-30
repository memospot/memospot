//! Default configuration for Memospot and Memos.
//!
//! This allows creating a default configuration file and also calling
//! the method `.unwrap_or_default()` on optional configuration fields.

use crate::log::Log;
use crate::memos::Memos;
use crate::memospot::{Backups, Memospot, Migrations, RemoteServer};
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
                env: None,
            },
            memospot: Memospot {
                backups: Backups {
                    enabled: Some(true),
                    path: None,
                },
                log: Log {
                    enabled: Some(false),
                },
                migrations: Migrations {
                    enabled: Some(true),
                },
                remote: RemoteServer {
                    enabled: Some(false),
                    url: None,
                },
            },
        }
    }
}
