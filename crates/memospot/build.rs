use build_utils::find_workspace_root;

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::{env, fs, path::PathBuf};

/// Find the build target directory.
///
/// Workaround for <https://github.com/rust-lang/cargo/issues/5457>.
///
/// See <https://github.com/tauri-apps/tauri/blob/22edc65a/crates/tauri-build/src/lib.rs#L522>.
fn find_target_dir() -> Option<PathBuf> {
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

/// Ensure that the build dependencies exist in some form.
///
/// This makes the linter happy when the repository is not fully set up.
///
/// - Create UI build directory.
/// - Create dummy server binaries in case they don't exist.
///   > The Tauri build hook will download the proper binaries later.
/// - Set the executable bit on valid UNIX server binaries.
fn ensure_deps() {
    let workspace_root = find_workspace_root().expect("Failed to detect workspace root!");

    let src_ui = workspace_root.join("src-ui/build");
    fs::create_dir_all(&src_ui).ok();

    let server_dist = workspace_root.join("server-dist");
    fs::create_dir_all(&server_dist).ok();

    let dist_path = server_dist
        .canonicalize()
        .expect("Failed to canonicalize server-dist path!");

    const TARGETS: [&str; 5] = [
        "x86_64-unknown-linux-gnu",
        "aarch64-apple-darwin",
        "x86_64-apple-darwin",
        "x86_64-pc-windows-msvc.exe",
        "aarch64-pc-windows-msvc.exe",
    ];
    for target in TARGETS {
        let target_bin = dist_path.join(format!("memos-{target}"));

        if !target_bin.exists() {
            #[cfg(feature = "unittest")]
            {
                std::fs::File::create(&target_bin).expect("Failed to create dummy binary!");
            }
            #[cfg(not(feature = "unittest"))]
            {
                continue;
            }
        }

        #[cfg(unix)]
        {
            let meta = fs::metadata(&target_bin).expect("Failed to get file metadata!");
            if meta.size() < 1024 {
                continue;
            }
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&target_bin, perms).expect("Failed to set file permissions!");
        }
    }
}

/// Cleanup dummy server binaries.
///
/// Prevents dev mode to start with a dummy server binary.
fn cleanup_dummy_deps() {
    let Some(target_dir) = find_target_dir() else {
        return;
    };

    let is_windows = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows";
    let memos_bin = target_dir.join(if is_windows { "memos.exe" } else { "memos" });
    if !memos_bin.exists() {
        return;
    }
    if let Ok(meta) = memos_bin.metadata() {
        if meta.len() == 0 {
            fs::remove_file(memos_bin).ok();
        }
    }
}

fn main() {
    ensure_deps();
    // Runs only on dev and release builds.
    if cfg!(not(feature = "unittest")) {
        cleanup_dummy_deps();
    }
    tauri_build::build()
}
