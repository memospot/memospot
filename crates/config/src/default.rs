//! Default configuration for Memospot and Memos.

use crate::log::{Log, LogRotation};
use crate::memos::Memos;
use crate::memospot::{Memospot, Migrations};
use crate::Config;

impl Default for Config {
    fn default() -> Config {
        Config {
            memos: Memos {
                bin: "".into(),
                cwd: "".into(),
                mode: "prod".into(),
                addr: "127.0.0.1".into(),
                port: 0,
                data: "".into(),
                metric: true,
                env: None,
                // log: Log {
                //     enabled: false,
                //     file: "memos.log".into(),
                //     level: "info".into(),
                //     pattern: "{d(%Y-%m-%d %H:%M:%S)} - {h({l})}: {m}{n}".into(),
                //     rotation: LogRotation {
                //         enabled: true,
                //         max_size: "10 mb".into(),
                //         amount: 5,
                //         path_mask: "$ENV{MEMOSPOT_DATA}/memos.log.{}.gz".into(),
                //     },
                // },
            },
            memospot: Memospot {
                // unmanaged_server: UnmanagedServer {
                //     enabled: false,
                //     full_url: "http://server_addr:port/".into(),
                // },
                database_migrations: Migrations {
                    enabled: false,
                    history: None,
                },
                log: Log {
                    enabled: false,
                    file: "memospot.log".into(),
                    level: "info".into(),
                    pattern: "{d(%Y-%m-%d %H:%M:%S)} - {h({l})}: {m}{n}".into(),
                    rotation: LogRotation {
                        enabled: true,
                        max_size: "10 mb".into(),
                        amount: 5,
                        path_mask: "$ENV{MEMOSPOT_DATA}/memospot.log.{}.gz".into(),
                    },
                },
            },
        }
    }
}
