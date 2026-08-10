//! Git metadata helpers for Cargo build scripts.

use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

/// Emit Cargo rerun hints for Git state and export the current revision as
/// `GIT_HASH` and `GIT_SHORT_HASH` rustc environment variables.
pub fn emit_build_metadata(manifest_dir: &Path) {
    emit_rerun_hints(manifest_dir);

    let Some(output) = git_output(manifest_dir, &["rev-parse", "--verify", "HEAD"]) else {
        return;
    };

    let hash = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if hash.len() < 7 {
        return;
    }

    println!("cargo:rustc-env=GIT_SHORT_HASH={}", &hash[..7]);
    println!("cargo:rustc-env=GIT_HASH={hash}");
}

fn emit_rerun_hints(manifest_dir: &Path) {
    let Some(head_path) = git_path(manifest_dir, "HEAD") else {
        return;
    };
    println!("cargo:rerun-if-changed={}", head_path.display());

    if let Some(packed_refs_path) = git_path(manifest_dir, "packed-refs") {
        println!("cargo:rerun-if-changed={}", packed_refs_path.display());
    }

    let Some(head_ref_output) = git_output(manifest_dir, &["symbolic-ref", "-q", "HEAD"])
    else {
        return;
    };
    let head_ref = String::from_utf8_lossy(&head_ref_output.stdout);
    if let Some(head_ref_path) = git_path(manifest_dir, head_ref.trim()) {
        println!("cargo:rerun-if-changed={}", head_ref_path.display());
    }
}

fn git_path(manifest_dir: &Path, path: &str) -> Option<PathBuf> {
    let output = git_output(manifest_dir, &["rev-parse", "--git-path", path])?;
    let path = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    (!path.is_empty()).then(|| manifest_dir.join(path))
}

/// Run git command and return output if successful.
fn git_output(manifest_dir: &Path, args: &[&str]) -> Option<Output> {
    Command::new("git")
        .args(args)
        .current_dir(manifest_dir)
        .output()
        .ok()
        .filter(|output| output.status.success())
}
