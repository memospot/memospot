use log::{error, info, warn};
use native_dialog::MessageDialog;

pub use native_dialog::MessageType;

#[macro_export]
macro_rules! panic_dialog {
    ($($arg:tt)*) => {
        panic_dialog(&format!($($arg)*));
        panic!("Fatal error: {}", &format!($($arg)*));
    };
}

#[macro_export]
macro_rules! info_dialog {
    ($($arg:tt)*) => {
        info_dialog(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! warn_dialog {
    ($($arg:tt)*) => {
        warn_dialog(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! error_dialog {
    ($($arg:tt)*) => {
        error_dialog(&format!($($arg)*));
    };
}

pub fn panic_dialog(msg: &str) {
    const FATAL_ERROR: &str = "Fatal error";
    MessageDialog::new()
        .set_type(MessageType::Error)
        .set_title(FATAL_ERROR)
        .set_text(msg)
        .show_alert()
        .unwrap_or_default();
    error!("{}: {}", FATAL_ERROR, msg);
    panic!("{}: {}", FATAL_ERROR, msg);
}

pub fn info_dialog(msg: &str) {
    info!("{}", msg);
    MessageDialog::new()
        .set_type(MessageType::Info)
        .set_title("Info")
        .set_text(msg)
        .show_alert()
        .unwrap_or_default();
}

pub fn warn_dialog(msg: &str) {
    warn!("{}", msg);
    MessageDialog::new()
        .set_type(MessageType::Warning)
        .set_title("Warning")
        .set_text(msg)
        .show_alert()
        .unwrap_or_default();
}

pub fn error_dialog(msg: &str) {
    error!("{}", msg);
    MessageDialog::new()
        .set_type(MessageType::Error)
        .set_title("Error")
        .set_text(msg)
        .show_alert()
        .unwrap_or_default();
}

pub fn confirm_dialog(title: &str, msg: &str, icon: MessageType) -> bool {
    MessageDialog::new()
        .set_type(icon)
        .set_title(title)
        .set_text(msg)
        .show_confirm()
        .unwrap_or_default()
}
