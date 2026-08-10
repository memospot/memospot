//! Workspace and Cargo build-path discovery helpers.

use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
};

/// Walk upward from the current directory until the workspace markers are found.
pub fn find_workspace_root() -> Option<PathBuf> {
    find_workspace_root_from(env::current_dir().ok()?)
}

/// Walk upward from `start_dir` until both `.git` and `.gitattributes` are found.
pub fn find_workspace_root_from(start_dir: impl AsRef<Path>) -> Option<PathBuf> {
    let mut current_dir = start_dir.as_ref().to_path_buf();

    loop {
        if current_dir.join(".git").exists() && current_dir.join(".gitattributes").exists() {
            return Some(current_dir);
        }
        if !current_dir.pop() {
            return None;
        }
    }
}

/// Resolve Cargo's target directory from a build script's `OUT_DIR` path.
///
/// Returns `None` when `out_dir` has no ancestor named `build`.
pub fn find_target_dir(out_dir: &Path) -> Option<PathBuf> {
    out_dir
        .ancestors()
        .find(|path| path.file_name() == Some(OsStr::new("build")))
        .and_then(Path::parent)
        .map(Path::to_path_buf)
}
