//! Build-time management of the bundled Memos server artifacts.

use std::{fs, path::Path};

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};

const TARGETS: [&str; 5] = [
    "x86_64-unknown-linux-gnu",
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-pc-windows-msvc.exe",
    "aarch64-pc-windows-msvc.exe",
];

/// Create required build directories and prepare known Memos server artifacts.
///
/// When `create_dummy_binaries` is true, missing target binaries are seeded with
/// zero-byte placeholders for test and lint builds.
///
/// Existing binaries at least 1 KiB in size receive executable permissions on Unix.
pub fn ensure_dependencies(workspace_root: &Path, create_dummy_binaries: bool) {
    let src_ui = workspace_root.join("src-ui/build");
    fs::create_dir_all(&src_ui).expect("failed to create src-ui/build");

    let server_dist = workspace_root.join("server-dist");
    fs::create_dir_all(&server_dist).expect("failed to create server-dist");

    let dist_path = server_dist
        .canonicalize()
        .expect("failed to canonicalize server-dist path");
    println!("cargo:rerun-if-changed={}", dist_path.display());

    for target in TARGETS {
        let target_bin = dist_path.join(format!("memos-{target}"));

        if !target_bin.exists() {
            if create_dummy_binaries {
                fs::File::create(&target_bin).expect("failed to create dummy binary");
            } else {
                continue;
            }
        }

        #[cfg(unix)]
        {
            let meta = fs::metadata(&target_bin).expect("failed to get file metadata");
            if meta.size() < 1024 {
                continue;
            }
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&target_bin, perms).expect("failed to set file permissions");
        }
    }
}

/// Remove a zero-byte `memos` sidecar from `target_dir`.
///
/// `target_os` determines whether the sidecar name includes the Windows `.exe`
/// suffix. This removes stale placeholders before a normal build uses the target
/// directory.
pub fn cleanup_dummy_dependency(target_dir: &Path, target_os: &str) {
    let memos_bin = target_dir.join(if target_os == "windows" {
        "memos.exe"
    } else {
        "memos"
    });
    if !memos_bin.exists() {
        return;
    }

    memos_bin
        .metadata()
        .map(|meta| meta.len() == 0 && fs::remove_file(memos_bin).is_ok())
        .ok();
}
