use std::{
    env::{self, current_dir},
    path::PathBuf,
};

pub fn find_workspace_root() -> Option<PathBuf> {
    const MAX_DEPTH: usize = 5;
    let mut current_dir = current_dir().ok()?;

    for _ in 0..=MAX_DEPTH {
        if current_dir.join(".git").exists() && current_dir.join(".gitattributes").exists() {
            return Some(current_dir);
        }
        if !current_dir.pop() {
            return None;
        }
    }
    None
}

/// Find the build target directory.
///
/// Workaround for <https://github.com/rust-lang/cargo/issues/5457>.
///
/// See <https://github.com/tauri-apps/tauri/blob/22edc65a/crates/tauri-build/src/lib.rs#L522>.
pub fn find_target_dir() -> Option<PathBuf> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap_or_default());
    if out_dir == PathBuf::default() {
        return None;
    }

    const MAX_DEPTH: usize = 5;
    let mut current_dir = env::current_dir().ok()?;

    for _ in 0..=MAX_DEPTH {
        if current_dir.join(".cargo-lock").exists() {
            return Some(current_dir);
        }
        if current_dir.join(".fingerprint").exists() && current_dir.join("build").exists() {
            return Some(current_dir);
        }
        if !current_dir.pop() {
            return None;
        }
    }
    None
}
