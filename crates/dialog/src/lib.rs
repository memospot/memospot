use log::{error, info, warn};
use native_dialog::MessageDialog;

pub use native_dialog::MessageType;

#[macro_export]
macro_rules! panic_dialog {
    ($($arg:tt)*) => {
        panic_dialog(&format!($($arg)*));
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

pub fn panic_dialog<T: std::fmt::Display>(msg: T) -> ! {
    const FATAL_ERROR: &str = "Fatal error";
    MessageDialog::new()
        .set_type(MessageType::Error)
        .set_title(FATAL_ERROR)
        .set_text(&msg.to_string())
        .show_alert()
        .unwrap_or_default();
    error!("{}: {}", FATAL_ERROR, msg);
    panic!("{}: {}", FATAL_ERROR, msg);
}

pub fn info_dialog<T: std::fmt::Display>(msg: T) {
    info!("{}", msg);
    MessageDialog::new()
        .set_type(MessageType::Info)
        .set_title("Info")
        .set_text(&msg.to_string())
        .show_alert()
        .unwrap_or_default();
}

pub fn warn_dialog<T: std::fmt::Display>(msg: T) {
    warn!("{}", msg);
    MessageDialog::new()
        .set_type(MessageType::Warning)
        .set_title("Warning")
        .set_text(&msg.to_string())
        .show_alert()
        .unwrap_or_default();
}

pub fn error_dialog<T: std::fmt::Display>(msg: T) {
    error!("{}", msg);
    MessageDialog::new()
        .set_type(MessageType::Error)
        .set_title("Error")
        .set_text(&msg.to_string())
        .show_alert()
        .unwrap_or_default();
}

pub fn confirm_dialog<T: std::fmt::Display>(title: T, msg: T, icon: MessageType) -> bool {
    MessageDialog::new()
        .set_type(icon)
        .set_title(&title.to_string())
        .set_text(&msg.to_string())
        .show_confirm()
        .unwrap_or_default()
}

/// Port of https://crates.io/crates/dialog-expect.
/// A trait with 1 function, `dialog_expect`, which is implemented for [`std::option::Option<T>`] and [`std::result::Result<T, E>`].
pub trait DialogExpect<T> {
    /// Takes `self` and returns the contained value, or shows an error message in a dialog box.
    /// ### Failures
    /// This function can fail (nothing will happen, but the program will still panic with the message to stdout) if there is no way to show a dialog box (most likely unsupported platform).
    fn dialog_expect<M: std::fmt::Display>(self, msg: M) -> T;
}

impl<T> DialogExpect<T> for std::option::Option<T> {
    fn dialog_expect<M: std::fmt::Display>(self, msg: M) -> T {
        match self {
            Some(v) => v,
            None => {
                panic_dialog(msg);
            }
        }
    }
}

impl<T, E> DialogExpect<T> for std::result::Result<T, E> {
    fn dialog_expect<M: std::fmt::Display>(self, msg: M) -> T {
        match self {
            Ok(v) => v,
            Err(_) => {
                panic_dialog(msg);
            }
        }
    }
}
