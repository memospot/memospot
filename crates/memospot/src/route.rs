//! Front-end routes.

use std::convert::AsRef;
use std::path::PathBuf;
use strum_macros::AsRefStr;
use strum_macros::FromRepr;

#[derive(AsRefStr, FromRepr, Clone, Copy)]
pub enum Route {
    #[strum(serialize = "/loader")]
    Loader,
    #[strum(serialize = "/settings")]
    Settings,
}
impl Route {
    pub fn id(&self) -> u8 {
        *self as u8
    }
    pub fn text(self) -> String {
        self.as_ref().to_string()
    }
    pub fn path(self) -> PathBuf {
        self.as_ref().into()
    }
}
impl From<Route> for PathBuf {
    fn from(r: Route) -> Self {
        r.path()
    }
}
