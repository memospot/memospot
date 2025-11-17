//! Memos event log.
//!
//! This is split as a module so `log4rs` can filter out the logs.

use log::{error, info, warn};
use sidecar::{CommandEvent, Receiver};

pub async fn log_events(mut events: Receiver<CommandEvent>) {
    while let Some(event) = events.recv().await {
        match event {
            CommandEvent::Error(e) => {
                error!("{e}");
            }
            CommandEvent::Stderr(stderr) => {
                error!("{stderr}");
            }
            CommandEvent::Stdout(stdout) => {
                info!("{stdout}");
            }
            CommandEvent::Terminated(term) => {
                warn!(
                    "-- server exited with code {:?} --\n",
                    term.code.unwrap_or_default()
                );
            }
            _ => {}
        }
    }
}
