/// Logging setup.
use crate::fl;
use config::Config;
use dialog::*;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

static LOG_CONFIG_YAML: &str = include_str!("../log.default.yaml");

/// Setup logging if it's enabled.
///
/// - Validates `log.yaml`.
///
/// Return true if logging is enabled.
pub fn setup_logger(config: &Config, memospot_data: &Path) -> bool {
    if !config.memospot.log.enabled.unwrap_or_default() {
        return false;
    }

    let log_config: PathBuf = memospot_data.join("log.yaml");

    // SAFETY: There's potential for race conditions when setting environment
    // variables in a multithreaded context. Shouldn't be an issue here.
    unsafe {
        // Allows using $ENV{MEMOSPOT_DATA} in `log4rs` config.
        env::set_var("MEMOSPOT_DATA", memospot_data.to_string_lossy().to_string());
    }

    if log4rs::init_file(&log_config, Default::default()).is_ok() {
        return true;
    }

    // Logging is enabled, but config is bad. Try to reset it.

    let mut file = File::create(&log_config).expect_dialog(fl!(
        "panic-log-config-write-error",
        file = log_config.to_string_lossy()
    ));

    file.write_all(LOG_CONFIG_YAML.as_bytes())
        .expect_dialog(fl!(
            "panic-log-config-write-error",
            file = log_config.to_string_lossy()
        ));
    file.flush().expect_dialog(fl!(
        "panic-log-config-write-error",
        file = log_config.to_string_lossy()
    ));

    if log4rs::init_file(&log_config, Default::default()).is_ok() {
        return true;
    }

    panic_dialog!(fl!(
        "panic-log-config-reset-error",
        file = log_config.to_string_lossy()
    ));
}
