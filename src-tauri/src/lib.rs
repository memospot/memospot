use native_dialog::{MessageDialog, MessageType};

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

pub fn panic_dialog(msg: &str) {
    MessageDialog::new()
        .set_type(MessageType::Error)
        .set_title("Fatal error")
        .set_text(msg)
        .show_alert()
        .unwrap();
    panic!("Fatal error: {}", msg);
}

pub fn info_dialog(msg: &str) {
    MessageDialog::new()
        .set_type(MessageType::Info)
        .set_title("Info")
        .set_text(msg)
        .show_alert()
        .unwrap();
}

pub fn warn_dialog(msg: &str) {
    MessageDialog::new()
        .set_type(MessageType::Warning)
        .set_title("Warning")
        .set_text(msg)
        .show_alert()
        .unwrap();
}

pub fn confirm_dialog(title: &str, msg: &str, icon: MessageType) -> bool {
    MessageDialog::new()
        .set_type(icon)
        .set_title(title)
        .set_text(msg)
        .show_confirm()
        .unwrap()
}

// find an open port
pub fn find_open_port() -> u16 {
    let mut listener = std::net::TcpListener::bind("127.0.0.1:0");
    if listener.is_err() {
        listener = std::net::TcpListener::bind("::1:0");
    }

    if let Ok(listener) = listener {
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        return port;
    }

    panic_dialog!("Failed to find open port");
}
