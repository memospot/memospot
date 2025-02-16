#[macro_export]
macro_rules! confirm_dialog {
    ($title:expr, $msg:expr, $icon:expr) => {
        confirm_dialog($title, $msg, $icon)
    };
}

#[macro_export]
macro_rules! error_dialog {
    // Single argument.
    ($arg:expr) => {
        error_dialog(&$arg.to_string());
    };
    // Multiple arguments.
    ($($arg:tt)+) => {
        error_dialog(&format!($($arg)+));
    };
}

#[macro_export]
macro_rules! info_dialog {
    ($arg:expr) => {
        info_dialog(&$arg.to_string());
    };
    ($($arg:tt)+) => {
        info_dialog(&format!($($arg)+));
    };
}

#[macro_export]
macro_rules! panic_dialog {
    ($arg:expr) => {
        panic_dialog(&$arg.to_string());
    };
    ($($arg:tt)+) => {
        panic_dialog(&format!($($arg)+));
    };
}

#[macro_export]
macro_rules! warn_dialog {
    ($arg:expr) => {
        warn_dialog(&$arg.to_string());
    };
    ($($arg:tt)+) => {
        warn_dialog(&format!($($arg)+));
    };
}
