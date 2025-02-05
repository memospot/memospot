use std::{env, fs};

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};

/// Ensure that the build dependencies exist in some form.
///
/// This makes the linter happy when the repository is not fully set up.
///
/// - Create UI build directory.
/// - Create dummy server binaries in case they don't exist.
///   > The Tauri build hook will download the proper binaries later.
/// - Set the executable bit on valid UNIX server binaries.
fn ensure_deps() {
    let current_dir = env::current_dir().expect("Failed to get current directory!");

    let src_ui = current_dir.join("../src-ui/build");
    fs::create_dir_all(&src_ui).ok();

    let server_dist = current_dir.join("../server-dist");
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
        let target_bin = dist_path.join(format!("memos-{}", target));
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
