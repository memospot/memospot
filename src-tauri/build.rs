/// Create dummy server binaries in case they don't exist.
///
/// This makes the linter happy when the repository is not fully set up.
fn ensure_dummy_deps() {
    let server_dist = std::env::current_dir()
        .expect("Failed to get current directory!")
        .join("../server-dist");
    std::fs::create_dir_all(&server_dist).ok();

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
            std::fs::File::create(&target_bin).ok();
        }
    }
}

fn main() {
    ensure_dummy_deps();
    tauri_build::build()
}
