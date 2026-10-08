/// Runtime checks and initialization code.
///
/// Functions in this module panics with native dialogs instead of returning errors.
mod environment;
mod logging;
mod setup;
mod webview;

pub use environment::set_env_vars;
pub use logging::setup_logger;
pub use setup::{
    config, data_path, database, find_memos, memos_data, memos_port, migrate_database,
};
pub use webview::ensure_webview;
