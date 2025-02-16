use log::{error, info, warn};
use native_dialog::{MessageDialog, MessageType};

/// Show a confirmation dialog with the given title, message, and icon.
///
/// # Failures
/// May fail on some platforms.
///
/// Using [`native-dialog`] crate version 0.7.0 will break this under GNOME.
pub fn confirm_dialog<T: std::fmt::Display>(title: T, msg: T, icon: MessageType) -> bool {
    match MessageDialog::new()
        .set_type(icon)
        .set_title(&title.to_string())
        .set_text(&msg.to_string())
        .show_confirm()
    {
        Ok(v) => v,
        Err(e) => {
            error!(
                "Unable to get result from confirm_dialog(): {}. Defaulting to false.",
                e
            );
            false
        }
    }
}

pub fn error_dialog<T: std::fmt::Display>(msg: T) {
    error!("{}", msg);
    MessageDialog::new()
        .set_type(MessageType::Error)
        .set_title("Error")
        .set_text(&msg.to_string())
        .show_alert()
        .ok();
}

pub fn info_dialog<T: std::fmt::Display>(msg: T) {
    info!("{}", msg);
    MessageDialog::new()
        .set_type(MessageType::Info)
        .set_title("Info")
        .set_text(&msg.to_string())
        .show_alert()
        .ok();
}

pub fn panic_dialog<T: std::fmt::Display>(msg: T) -> ! {
    const FATAL_ERROR: &str = "Fatal error";
    MessageDialog::new()
        .set_type(MessageType::Error)
        .set_title(FATAL_ERROR)
        .set_text(&msg.to_string())
        .show_alert()
        .ok();
    error!("{}: {}", FATAL_ERROR, msg);
    panic!("{}: {}", FATAL_ERROR, msg);
}

pub fn warn_dialog<T: std::fmt::Display>(msg: T) {
    warn!("{}", msg);
    MessageDialog::new()
        .set_type(MessageType::Warning)
        .set_title("Warning")
        .set_text(&msg.to_string())
        .show_alert()
        .ok();
}
