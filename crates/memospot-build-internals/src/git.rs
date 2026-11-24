//! Git metadata helpers for Cargo build scripts.

use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Output},
};

/// Emit Cargo rerun hints and export the selected revision as `GIT_HASH` and
/// `GIT_SHORT_HASH` rustc environment variables.
///
/// `hash_override` takes precedence when it is a valid hex revision, even when
/// Git is available. Otherwise, Git HEAD is used. Git rerun hints are emitted
/// only when resolving through Git, so container builds can pass the host
/// worktree revision when Git metadata is unavailable inside the build context.
///
/// A non-empty override that is not 7..=64 ASCII hex digits is ignored with a
/// Cargo warning.
///
/// Returns `Err` if writing the Cargo directives fails.
pub fn emit_build_metadata<W: Write>(
    manifest_dir: &Path,
    hash_override: Option<&str>,
    output: &mut W,
) -> io::Result<()> {
    writeln!(output, "cargo:rerun-if-env-changed=GIT_HASH")?;

    if let Some(raw) = hash_override {
        if let Some(hash) = normalize_hash(raw) {
            return emit_hash(output, hash);
        }
        if !raw.trim().is_empty() {
            writeln!(
                output,
                "cargo:warning=Ignoring invalid GIT_HASH; expected 7..=64 ASCII hex characters"
            )?;
        }
    }

    emit_rerun_hints(manifest_dir, output)?;

    let git_hash = git_output(manifest_dir, &["rev-parse", "--verify", "HEAD"])
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned());
    if let Some(hash) = git_hash.as_deref().and_then(normalize_hash) {
        emit_hash(output, hash)?;
    }
    Ok(())
}

fn normalize_hash(value: &str) -> Option<&str> {
    let value = value.trim();
    ((7..=64).contains(&value.len()) && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .then_some(value)
}

fn emit_hash<W: Write>(output: &mut W, hash: &str) -> io::Result<()> {
    // Callers pass a hash validated by normalize_hash (>= 7 ASCII hex chars).
    let short_hash = &hash[..7];
    writeln!(output, "cargo:rustc-env=GIT_SHORT_HASH={short_hash}")?;
    writeln!(output, "cargo:rustc-env=GIT_HASH={hash}")
}

fn emit_rerun_hints<W: Write>(manifest_dir: &Path, output: &mut W) -> io::Result<()> {
    let Some(head_path) = git_path(manifest_dir, "HEAD") else {
        return Ok(());
    };
    writeln!(output, "cargo:rerun-if-changed={}", head_path.display())?;

    if let Some(packed_refs_path) = git_path(manifest_dir, "packed-refs") {
        writeln!(
            output,
            "cargo:rerun-if-changed={}",
            packed_refs_path.display()
        )?;
    }

    let Some(head_ref_output) = git_output(manifest_dir, &["symbolic-ref", "-q", "HEAD"])
    else {
        return Ok(());
    };
    let head_ref = String::from_utf8_lossy(&head_ref_output.stdout);
    if let Some(head_ref_path) = git_path(manifest_dir, head_ref.trim()) {
        writeln!(output, "cargo:rerun-if-changed={}", head_ref_path.display())?;
    }
    Ok(())
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
        .env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES")
        .env_remove("GIT_CEILING_DIRECTORIES")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_DIR")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_OBJECT_DIRECTORY")
        .env_remove("GIT_WORK_TREE")
        .output()
        .ok()
        .filter(|output| output.status.success())
}

#[cfg(test)]
mod tests {
    use super::emit_build_metadata;
    use std::path::Path;

    const RERUN_HINT: &str = "cargo:rerun-if-env-changed=GIT_HASH\n";
    const INVALID_WARNING: &str =
        "cargo:warning=Ignoring invalid GIT_HASH; expected 7..=64 ASCII hex characters\n";

    fn emit(hash_override: Option<&str>) -> String {
        let mut output = Vec::new();
        emit_build_metadata(
            Path::new("/nonexistent/memospot-build-metadata-test"),
            hash_override,
            &mut output,
        )
        .expect("build metadata output should be writable");
        String::from_utf8(output).expect("build metadata output should be UTF-8")
    }

    #[test]
    fn prefers_a_valid_environment_hash() {
        // GIVEN a valid override
        let hash = "a".repeat(40);

        // WHEN emitting build metadata
        let output = emit(Some(hash.as_str()));

        // THEN both hash environment variables are emitted without a warning
        assert_eq!(
            output,
            format!(
                "{RERUN_HINT}cargo:rustc-env=GIT_SHORT_HASH=aaaaaaa\ncargo:rustc-env=GIT_HASH={hash}\n"
            )
        );
    }

    #[test]
    fn warns_and_ignores_an_invalid_environment_hash() {
        // GIVEN an invalid non-empty override
        // WHEN emitting build metadata
        let output = emit(Some("not-a-hash"));

        // THEN the warning is emitted and no hash is exported
        assert_eq!(output, format!("{RERUN_HINT}{INVALID_WARNING}"));
    }

    #[test]
    fn ignores_blank_environment_hashes_without_warning() {
        // GIVEN a whitespace-only override
        // WHEN emitting build metadata
        let output = emit(Some("   \n"));

        // THEN no warning or hash is emitted
        assert_eq!(output, RERUN_HINT);
    }

    #[test]
    fn trims_and_accepts_abbreviated_and_sha256_hashes() {
        // GIVEN an abbreviated hash with whitespace and a 64-character hash
        let abbreviated = "  abc1234\n";
        let sha256 = "e".repeat(64);

        // WHEN emitting build metadata
        let abbreviated_output = emit(Some(abbreviated));
        let sha256_output = emit(Some(sha256.as_str()));

        // THEN both are exported
        assert_eq!(
            abbreviated_output,
            format!(
                "{RERUN_HINT}cargo:rustc-env=GIT_SHORT_HASH=abc1234\ncargo:rustc-env=GIT_HASH=abc1234\n"
            )
        );
        assert_eq!(
            sha256_output,
            format!(
                "{RERUN_HINT}cargo:rustc-env=GIT_SHORT_HASH=eeeeeee\ncargo:rustc-env=GIT_HASH={sha256}\n"
            )
        );
    }

    #[test]
    fn rejects_short_and_overlong_hashes() {
        // GIVEN 6- and 65-character hashes
        let short = "a".repeat(6);
        let overlong = "a".repeat(65);

        // WHEN emitting build metadata
        let short_output = emit(Some(short.as_str()));
        let overlong_output = emit(Some(overlong.as_str()));

        // THEN both are rejected with warnings
        assert_eq!(short_output, format!("{RERUN_HINT}{INVALID_WARNING}"));
        assert_eq!(overlong_output, format!("{RERUN_HINT}{INVALID_WARNING}"));
    }

    #[test]
    fn emits_only_the_rerun_hint_without_a_hash() {
        // GIVEN no override and no git repository
        // WHEN emitting build metadata
        let output = emit(None);

        // THEN only the environment rerun hint is emitted
        assert_eq!(output, RERUN_HINT);
    }
}
