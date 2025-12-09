//! Webview windows.

use std::convert::AsRef;
use std::fmt;
use strum::{AsRefStr, EnumString, IntoStaticStr};

#[derive(AsRefStr, EnumString, IntoStaticStr)]
pub enum Window {
    #[strum(serialize = "main")]
    Main,
    #[strum(serialize = "settings")]
    Settings,
}

impl Window {
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl fmt::Display for Window {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
