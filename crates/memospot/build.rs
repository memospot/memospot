use std::{env, fs, path::PathBuf};

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};

/// Detect the workspace root.
fn detect_workspace_root() -> Option<PathBuf> {
    const MAX_DEPTH: usize = 5;
    let mut current_dir = env::current_dir().ok()?;

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

/// Ensure that the build dependencies exist in some form.
///
/// This makes the linter happy when the repository is not fully set up.
///
/// - Create UI build directory.
/// - Create dummy server binaries in case they don't exist.
///   > The Tauri build hook will download the proper binaries later.
/// - Set the executable bit on valid UNIX server binaries.
fn ensure_deps() {
    let workspace_root = detect_workspace_root().expect("Failed to detect workspace root!");

    let src_ui = workspace_root.join("src-ui/build");
    fs::create_dir_all(&src_ui).ok();

    let server_dist = workspace_root.join("server-dist");
    fs::create_dir_all(&server_dist).ok();

    let dist_path = server_dist
        .canonicalize()
        .expect("Failed to canonicalize server-dist path!");

    const TARGETS: [&str; 4] = [
        "x86_64-unknown-linux-gnu",
        "aarch64-apple-darwin",
        "x86_64-apple-darwin",
        "x86_64-pc-windows-msvc.exe",
    ];
    for target in TARGETS {
        let target_bin = dist_path.join(format!("memos-{target}"));
        if !target_bin.exists() {
            std::fs::File::create(&target_bin).expect("Failed to create dummy binary!");
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

fn main() {
    ensure_deps();
    tauri_build::build()
}
