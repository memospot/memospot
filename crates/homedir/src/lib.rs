//! Enable expansion of tildes in paths
//!
//! Patched to work on Windows. Lacks resolving of `~user` on Windows.
//!
//! - ~ expands using the HOME environmental variable.
//! - if HOME does not exist, lookup current user in the user database
//!
//! - ~`user` will expand to the user's home directory from the user database
//!
//! Example:
//!
//! ```
//! use homedir::HomeDirExt;
//!
//! let public_html = "~/public_html".expand_home().unwrap();
//! ```

use home::home_dir;
use std::path::{Component, Path, PathBuf};

#[cfg(not(target_os = "windows"))]
use nix::unistd::{Uid, User};

#[cfg(test)]
mod tests;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
/// Error while expanding path
pub enum Error {
    /// The user being looked up is not in the user database
    #[error("the user {0} does not exist in the user database")]
    MissingEntry(String),

    /// Can't find name for current user.
    #[error("the current user (numeric id {0}) does not exist in the user database")]
    CurrentUserDoesNotExist(u32),
}

/// The expansion trait extension
pub trait HomeDirExt {
    /// Expands a users home directory signified by a tilde.
    ///
    /// Note: This function currently does not resolve `~user` on Windows.
    ///
    /// Examples:
    /// ```
    /// # use homedir::HomeDirExt;
    /// # use std::env::var;
    /// # use std::path::PathBuf;
    /// let mut path = PathBuf::from(var("HOME").unwrap());
    /// path.push(".vimrc");
    ///
    /// assert_eq!("~/.vimrc".expand_home().unwrap(), path);
    ///
    /// # #[cfg(target_os = "macos")]
    /// # const ROOT_VIMRC: &'static str = "/var/root/.vimrc";
    /// # #[cfg(target_os = "linux")]
    /// # const ROOT_VIMRC: &'static str = "/root/.vimrc";
    /// # #[cfg(not(target_os = "windows"))]
    /// assert_eq!("~root/.vimrc".expand_home().unwrap(), PathBuf::from(ROOT_VIMRC));
    ///
    /// let public_html = "~/public_html".expand_home().unwrap();
    ///
    /// // Resolves to `/home/john_doe/public_html`,
    /// // if john_doe exists as a user
    /// let john_doe_html = "~john_doe/public_html".expand_home().unwrap();
    ///
    /// // Resolves to /root
    /// let root_home = "~root".expand_home().unwrap();
    /// ```
    fn expand_home(&self) -> Result<PathBuf, Error>;
}

impl HomeDirExt for Path {
    fn expand_home(&self) -> Result<PathBuf, Error> {
        let mut path = PathBuf::new();
        let mut comps = self.components();

        match comps.next() {
            Some(Component::Normal(os)) => {
                if let Some(s) = os.to_str() {
                    match s {
                        "~" => {
                            let p = getenv()
                                .ok_or(Error::MissingEntry)
                                .or_else(|_| getent_current())?;
                            path.push(p);
                        }
                        s if s.starts_with('~') => {
                            path.push(getent(&s[1..])?);
                        }
                        s => path.push(s),
                    }
                } else {
                    path.push(os)
                }
            }
            Some(comp) => path.push(comp),
            None => return Ok(path),
        };

        for comp in comps {
            path.push(comp);
        }

        Ok(path)
    }
}

impl<T> HomeDirExt for T
where
    T: AsRef<Path>,
{
    fn expand_home(&self) -> Result<PathBuf, Error> {
        self.as_ref().expand_home()
    }
}

pub(crate) fn getenv() -> Option<PathBuf> {
    home_dir().map(Into::into)
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn getent(name: &str) -> Result<PathBuf, Error> {
    let usr = User::from_name(name).map_err(|_| Error::MissingEntry(name.to_string()))?;
    let usr = usr.ok_or_else(|| Error::MissingEntry(name.to_string()))?;

    Ok(usr.dir)
}
#[cfg(target_os = "windows")]
pub(crate) fn getent(name: &str) -> Result<PathBuf, Error> {
    // Not implemented. Always return error on Windows.
    Err(Error::MissingEntry(name.to_string()))
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn getent_current() -> Result<PathBuf, Error> {
    let uid = Uid::current();
    let usr = User::from_uid(uid).map_err(|_| Error::CurrentUserDoesNotExist(uid.as_raw()))?;
    let usr = usr.ok_or_else(|| Error::CurrentUserDoesNotExist(uid.as_raw()))?;

    Ok(usr.dir)
}
#[cfg(target_os = "windows")]
pub(crate) fn getent_current() -> Result<PathBuf, Error> {
    // Not implemented. Always return error on Windows.
    Err(Error::CurrentUserDoesNotExist(0))
}
