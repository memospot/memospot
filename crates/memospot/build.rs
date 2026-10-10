use memospot_build_internals::{
    find_target_dir, find_workspace_root_from,
    git::emit_build_metadata,
    memos::{cleanup_dummy_dependency, ensure_dependencies},
    memospot::{ShortcutBinding, generate_shortcut_artifacts},
    windows_test_manifest::ensure_test_manifest,
};

use std::{env, path::PathBuf};

macro_rules! shortcut_bindings {
    (
        $(
            $name:ident => {
                command: $command:literal,
                accelerator: $accelerator:literal,
                codes: [$($code:literal),+ $(,)?],
            }
        ),+ $(,)?
    ) => {
        vec![
            $(
                ShortcutBinding::new(
                    stringify!($name),
                    $command,
                    $accelerator,
                    &[$($code),+],
                ),
            )+
        ]
    };
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is required while running the build script");
    let manifest_dir = PathBuf::from(manifest_dir);
    emit_build_metadata(&manifest_dir);

    let workspace_root = find_workspace_root_from(&manifest_dir)
        .expect("failed to detect workspace root from CARGO_MANIFEST_DIR");
    ensure_dependencies(&workspace_root, cfg!(feature = "unittest"));

    env::var("TARGET")
        .map(|t| println!("cargo:rustc-env=TARGET_TRIPLE={t}"))
        .ok();

    let out_dir = PathBuf::from(
        env::var("OUT_DIR").expect("OUT_DIR is required while running the build script"),
    );
    let bindings: Vec<ShortcutBinding> = include!("src/shortcut_bindings.in.rs");
    generate_shortcut_artifacts(&out_dir, &bindings);

    // Runs only on dev and release builds.
    if cfg!(not(feature = "unittest"))
        && let Some(target_dir) = find_target_dir(&out_dir)
    {
        let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
        cleanup_dummy_dependency(&target_dir, &target_os);
    }

    tauri_build::build();

    // Test binaries link GUI symbols that exist only in Common Controls v6
    // but carry none of Tauri's manifest, so the Windows loader kills them
    // before `main`. Reuse Tauri's generated manifest for every artifact.
    // Target-gated so cross-compiled and non-Windows builds never match it.
    if cfg!(feature = "unittest")
        && env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows"
        && env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default() == "msvc"
    {
        ensure_test_manifest();
    }
}
