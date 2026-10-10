//! Windows application manifest reuse for test binaries.
//!
//! Test binaries import GUI symbols such as `TaskDialogIndirect` from
//! `comctl32.dll`, which exist only in Common Controls v6. The Windows
//! loader binds v6 solely when the executable carries an application
//! manifest declaring it; otherwise the process dies before `main` with
//! `STATUS_ENTRYPOINT_NOT_FOUND` (`0xc0000139`). Tauri links its manifest
//! into the shipped binary alone, leaving the lib unit-test binary bare.
//! This module extracts Tauri's already generated manifest and directs the
//! linker to embed it into every artifact, including the test binaries.
//!
//! See <https://github.com/tauri-apps/tauri/issues/13419>.

use std::{env, fs, path::PathBuf};

/// Dependency name identifying Common Controls v6 in manifest text.
const COMCTL_V6_NAME: &str = "Microsoft.Windows.Common-Controls";
/// Common Controls version carrying the symbols test binaries import.
const COMCTL_V6_VERSION: &str = "6.0.0.0";

/// Dependency stanza merged into manifests missing Common Controls v6.
const COMCTL_V6_STANZA: &str = concat!(
    "  <dependency>\n",
    "    <dependentAssembly>\n",
    "      <assemblyIdentity\n",
    "        type=\"win32\"\n",
    "        name=\"Microsoft.Windows.Common-Controls\"\n",
    "        version=\"6.0.0.0\"\n",
    "        processorArchitecture=\"*\"\n",
    "        publicKeyToken=\"6595b64144ccf1df\"\n",
    "        language=\"*\"\n",
    "      />\n",
    "    </dependentAssembly>\n",
    "  </dependency>\n",
);

/// Standalone manifest used when Tauri's generated manifest cannot be reused.
const MINIMAL_V6_MANIFEST: &str = "\
<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
<assembly xmlns=\"urn:schemas-microsoft-com:asm.v1\" manifestVersion=\"1.0\">\n\
  <dependency>\n\
    <dependentAssembly>\n\
      <assemblyIdentity\n\
        type=\"win32\"\n\
        name=\"Microsoft.Windows.Common-Controls\"\n\
        version=\"6.0.0.0\"\n\
        processorArchitecture=\"*\"\n\
        publicKeyToken=\"6595b64144ccf1df\"\n\
        language=\"*\"\n\
      />\n\
    </dependentAssembly>\n\
  </dependency>\n\
</assembly>\n";

/// Reuse Tauri's generated manifest for binaries the linker leaves bare.
///
/// Reads the `resource.rc` that `tauri-build` generated earlier in the same
/// `build.rs` invocation, extracts its manifest, patches in the Common
/// Controls v6 dependency when absent, and directs the linker to embed the
/// result into every artifact, including the lib unit-test binary. The
/// shipped binary merges the identical manifest it already carries, so its
/// resources are unchanged.
pub fn ensure_test_manifest() {
    let out_dir = PathBuf::from(
        env::var("OUT_DIR").expect("OUT_DIR is required while running the build script"),
    );
    let manifest = fs::read_to_string(out_dir.join("resource.rc"))
        .ok()
        .and_then(|rc| extract_manifest_from_rc(&rc))
        .map(|manifest| ensure_comctl_v6(&manifest))
        .unwrap_or_else(|| {
            println!(
                "cargo:warning=failed to reuse Tauri's Windows manifest for test binaries; embedding a minimal Common Controls v6 manifest instead"
            );
            MINIMAL_V6_MANIFEST.to_string()
        });
    let manifest_path = out_dir.join("test_manifest.xml");
    fs::write(&manifest_path, manifest).expect("failed to write test manifest");
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!(
        "cargo:rustc-link-arg=/MANIFESTINPUT:{}",
        manifest_path.display()
    );
    // Turn linker warnings into errors so a skipped manifest fails loudly
    // instead of resurfacing as STATUS_ENTRYPOINT_NOT_FOUND at test time.
    println!("cargo:rustc-link-arg=/WX");
}

/// Extract the inline application manifest from a generated `.rc` file.
///
/// Returns [`None`] when no `1 24` manifest block is present, for example
/// when a custom Tauri configuration references a manifest file by path
/// instead of embedding manifest text.
pub fn extract_manifest_from_rc(rc: &str) -> Option<String> {
    let mut lines = rc.lines().map(str::trim).skip_while(|line| *line != "1 24");
    lines.next()?;
    if lines.next()? != "{" {
        return None;
    }
    let mut manifest = String::new();
    for line in lines {
        if line == "}" {
            break;
        }
        let inner = line.strip_prefix('"')?.strip_suffix('"')?;
        let inner = inner.strip_prefix(' ').unwrap_or(inner);
        let inner = inner.strip_suffix(' ').unwrap_or(inner);
        manifest.push_str(&unescape_rc_string(inner));
        manifest.push('\n');
    }
    Some(manifest)
}

/// Whether a manifest is missing the Common Controls v6 dependency.
pub fn manifest_needs_comctl_v6(manifest: &str) -> bool {
    !(manifest.contains(COMCTL_V6_NAME) && manifest.contains(COMCTL_V6_VERSION))
}

/// Insert the Common Controls v6 dependency into a manifest lacking it.
///
/// Falls back to a minimal standalone manifest when the input has no
/// `</assembly>` root to attach the stanza to.
pub fn ensure_comctl_v6(manifest: &str) -> String {
    if !manifest_needs_comctl_v6(manifest) {
        return manifest.to_string();
    }
    match manifest.rfind("</assembly>") {
        Some(index) => {
            let (head, tail) = manifest.split_at(index);
            format!("{head}{COMCTL_V6_STANZA}{tail}")
        }
        None => MINIMAL_V6_MANIFEST.to_string(),
    }
}

/// Inverse of the `escape_string` helper in `tauri-winres`.
fn unescape_rc_string(escaped: &str) -> String {
    let mut unescaped = String::with_capacity(escaped.len());
    let mut chars = escaped.chars();
    while let Some(char_) = chars.next() {
        if char_ != '\\' && char_ != '"' {
            unescaped.push(char_);
            continue;
        }
        match chars.next() {
            Some('"') if char_ == '"' => unescaped.push('"'),
            Some('\\') if char_ == '\\' => unescaped.push('\\'),
            Some('\'') if char_ == '\\' => unescaped.push('\''),
            Some('n') if char_ == '\\' => unescaped.push('\n'),
            Some('t') if char_ == '\\' => unescaped.push('\t'),
            Some('r') if char_ == '\\' => unescaped.push('\r'),
            Some(other) => {
                unescaped.push(char_);
                unescaped.push(other);
            }
            None => unescaped.push(char_),
        }
    }
    unescaped
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RC: &str = r#"#pragma code_page(65001)
1 VERSIONINFO
FILEVERSION 1, 0, 4, 0
{
BLOCK "StringFileInfo"
{
BLOCK "040004b0"
{
VALUE "ProductName", "memospot"
}
}
}
2 ICON "C:\icons\icon.ico"
1 24
{
" <assembly manifestVersion=""1.0""> "
" <dependency name=""Common-Controls"" version=""6.0.0.0""/> "
" </assembly> "
}
"#;

    #[test]
    fn extracts_manifest_from_generated_resource_file() {
        // GIVEN a resource file in the shape `tauri-winres` writes
        // WHEN the manifest is extracted
        let manifest = extract_manifest_from_rc(SAMPLE_RC).expect("manifest should extract");

        // THEN the manifest XML is recovered with resource escaping undone.
        assert_eq!(
            manifest,
            "<assembly manifestVersion=\"1.0\">\n\
             <dependency name=\"Common-Controls\" version=\"6.0.0.0\"/>\n\
             </assembly>\n"
        );
    }

    #[test]
    fn extraction_fails_without_manifest_block() {
        // GIVEN a resource file with no `1 24` manifest block
        let rc = "1 VERSIONINFO\nFILEVERSION 1, 0, 4, 0\n";

        // WHEN the manifest is extracted THEN no manifest is reported.
        assert_eq!(extract_manifest_from_rc(rc), None);
    }

    #[test]
    fn detects_missing_comctl_v6_dependency() {
        // GIVEN manifests with and without the v6 dependency
        let complete = format!("<assembly>\n{COMCTL_V6_STANZA}</assembly>\n");
        let missing_stanza = "<assembly><assemblyIdentity name=\"Other\"/></assembly>";
        let stale_version =
            format!("<assembly name=\"{COMCTL_V6_NAME}\" version=\"5.82.0.0\"/>");

        // WHEN inspected THEN only the complete manifest passes.
        assert!(!manifest_needs_comctl_v6(&complete));
        // AND a missing stanza or a stale version both need the patch.
        assert!(manifest_needs_comctl_v6(missing_stanza));
        assert!(manifest_needs_comctl_v6(&stale_version));
    }

    #[test]
    fn keeps_complete_manifest_untouched() {
        // GIVEN a manifest already declaring Common Controls v6
        let manifest = format!("<assembly>\n{COMCTL_V6_STANZA}</assembly>\n");

        // WHEN ensured THEN the manifest is returned unchanged.
        assert_eq!(ensure_comctl_v6(&manifest), manifest);
    }

    #[test]
    fn inserts_stanza_before_assembly_close() {
        // GIVEN a manifest missing the v6 dependency
        let manifest = "<assembly xmlns=\"urn:schemas-microsoft-com:asm.v1\">\n</assembly>\n";

        // WHEN ensured THEN the stanza lands inside the assembly root.
        let patched = ensure_comctl_v6(manifest);
        assert!(patched.contains(COMCTL_V6_NAME));
        assert!(patched.contains(COMCTL_V6_VERSION));
        assert!(patched.ends_with("</assembly>\n"));
        assert!(patched.starts_with("<assembly"));
    }

    #[test]
    fn falls_back_to_minimal_manifest_without_assembly_root() {
        // GIVEN text with no `</assembly>` root to attach the stanza to
        // WHEN ensured THEN a standalone v6 manifest is returned.
        assert_eq!(ensure_comctl_v6("not xml"), MINIMAL_V6_MANIFEST);
    }

    #[test]
    fn unescapes_resource_string_literals() {
        // GIVEN resource-escaped content
        // WHEN unescaped THEN quotes and backslashes are restored literally.
        assert_eq!(unescape_rc_string("a\"\"b\\\\c"), "a\"b\\c");
    }
}
