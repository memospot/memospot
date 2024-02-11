use crate::*;
use home::home_dir;
use std::env;
use std::path::{Path, PathBuf};

#[cfg(not(target_os = "windows"))]
use {super::getent, super::Error};

/// Test that `~` is expanded to the current user's home directory.
/// Uses some code from the `home` crate.
/// See: https://github.com/rust-lang/cargo/blob/master/crates/home/src/windows.rs
#[test]
fn test_expand() {
    let olduserprofile = env::var_os("USERPROFILE").unwrap();

    env::remove_var("HOME");
    env::remove_var("USERPROFILE");

    assert_eq!(home_dir(), Some(PathBuf::from(olduserprofile)));

    #[cfg(target_os = "windows")]
    static HOME: &str = r"C:\Users\foo tar baz";

    #[cfg(not(target_os = "windows"))]
    static HOME: &str = "/home/foo tar baz";

    let homepath = Path::new(HOME);

    env::set_var("HOME", homepath.as_os_str());
    assert_ne!(home_dir().as_deref(), Some(homepath));

    env::set_var("USERPROFILE", HOME);
    assert_eq!(home_dir().as_deref(), Some(homepath));

    let subpath = Path::new(homepath).join(".vimrc");

    #[cfg(not(target_os = "windows"))]
    assert_eq!("~/.vimrc".expand_home().unwrap(), subpath);

    #[cfg(target_os = "windows")]
    assert_eq!(r"~\.vimrc".expand_home().unwrap(), subpath);
}

/// Test that paths without `~` are returned as-is.
#[test]
fn test_expand_nonexpansion() {
    #[cfg(not(target_os = "windows"))]
    assert_eq!(
        "/etc/some.conf".expand_home().unwrap(),
        PathBuf::from("/etc/some.conf")
    );

    #[cfg(target_os = "windows")]
    assert_eq!(
        r"C:\Windows\explorer.exe".expand_home().unwrap(),
        PathBuf::from(r"C:\Windows\explorer.exe")
    );
}

/// Test that `~user` is expanded to the home directory of `user`.
#[cfg(not(target_os = "windows"))]
#[test]
fn test_root() {
    #[cfg(target_os = "macos")]
    const ROOT_DIR: &'static str = "/var/root";

    #[cfg(target_os = "linux")]
    const ROOT_DIR: &'static str = "/root";

    assert_eq!(getent("root").unwrap(), PathBuf::from(ROOT_DIR));
    assert_eq!("~root".expand_home().unwrap(), PathBuf::from(ROOT_DIR));
}

/// Test that a invalid `~user` returns an error.
#[cfg(not(target_os = "windows"))]
#[test]
fn test_missing() {
    assert!(matches!(
        getent("_foobar_").unwrap_err(),
        Error::MissingEntry(_)
    ));
}
