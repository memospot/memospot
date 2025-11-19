use std::{env::current_dir, path::PathBuf};

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
