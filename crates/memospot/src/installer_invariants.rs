//! Installer data-safety invariants.
//!
//! Notes live in the app data directory. For the NSIS currentUser installer,
//! that directory is the same as `$INSTDIR`, so no template path may
//! recursively delete it or the product-name directory.

use std::fs;
use std::path::{Path, PathBuf};

use tauri_utils::config::parse::parse;
use tauri_utils::platform::Target;

use crate::APP_DATA_DIR_NAME;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root should resolve")
}

fn read_in_repo(relative: &str) -> String {
    let path = repo_root().join(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

#[test]
fn identifier_does_not_alias_app_data_directory() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let (config, _) = parse(Target::current(), manifest_dir.join("tauri.conf.json"))
        .expect("Tauri.toml should parse");

    let product_name = config
        .product_name
        .as_deref()
        .expect("productName should be configured");

    // GIVEN the Tauri config and the app data directory name
    // WHEN the identifier is compared with both names
    // THEN the identifier must not alias either one
    assert_ne!(
        config.identifier.to_lowercase(),
        product_name.to_lowercase(),
        "identifier must not equal the product name"
    );
    assert_ne!(
        config.identifier.to_lowercase(),
        APP_DATA_DIR_NAME.to_lowercase(),
        "identifier must not equal the app data directory name"
    );
}

#[test]
fn nsis_only_recursively_deletes_bundle_cache_directories() {
    // GIVEN the vendored NSIS template
    // WHEN every recursive RMDir target is collected
    // THEN only the two BUNDLEID cache deletes may appear
    let contents = read_in_repo("installer/nsis.nsi");
    let app_data_dir_name = APP_DATA_DIR_NAME.to_lowercase();
    let mut saw_appdata = false;
    let mut saw_localappdata = false;

    for line in contents.lines() {
        let line = line.trim();
        if line.starts_with(';') || line.starts_with('#') {
            continue;
        }

        let mut tokens = line.split_whitespace();
        let Some(command) = tokens.next() else {
            continue;
        };
        if !command.eq_ignore_ascii_case("rmdir") {
            continue;
        }

        let tokens = tokens.collect::<Vec<_>>();
        if !tokens.iter().any(|token| token.eq_ignore_ascii_case("/r")) {
            continue;
        }

        let target = tokens
            .iter()
            .filter(|token| !token.starts_with('/'))
            .copied()
            .collect::<Vec<_>>()
            .join(" ");
        let target = target.trim_matches('"');
        let lower = target.to_lowercase();

        let appdata = lower.starts_with("$appdata\\${bundleid}");
        let localappdata = lower.starts_with("$localappdata\\${bundleid}");

        assert!(
            appdata || localappdata,
            "recursive delete targets a non-cache path: {line}"
        );
        assert!(
            !lower.contains("$instdir")
                && !lower.contains("${productname}")
                && !lower.contains(app_data_dir_name.as_str()),
            "recursive delete must not target install or data paths: {line}"
        );

        saw_appdata |= appdata;
        saw_localappdata |= localappdata;
    }

    assert!(
        saw_appdata,
        "expected a recursive delete of $APPDATA\\${{BUNDLEID}}"
    );
    assert!(
        saw_localappdata,
        "expected a recursive delete of $LOCALAPPDATA\\${{BUNDLEID}}"
    );
}

#[test]
fn wix_does_not_recursively_remove_install_directory() {
    // GIVEN the vendored WiX template
    // WHEN RemoveFolderEx elements are inspected
    // THEN none may target INSTALLDIR
    let contents = read_in_repo("installer/wix.wxs");
    let contents = strip_xml_comments(&contents).to_lowercase();

    assert!(
        contents.contains("<removefolder id=\"installdir\""),
        "expected the non-recursive INSTALLDIR RemoveFolder entry"
    );

    let mut rest = contents.as_str();
    while let Some(start) = rest.find("removefolderex") {
        let after = &rest[start..];
        let end = after.find('>').map_or(after.len(), |index| index + 1);
        let element = &after[..end];
        assert!(
            !element.contains("installdir"),
            "RemoveFolderEx must not target INSTALLDIR: {element}"
        );
        rest = &after[end..];
    }
}

fn strip_xml_comments(contents: &str) -> String {
    let mut result = String::with_capacity(contents.len());
    let mut rest = contents;

    while let Some(start) = rest.find("<!--") {
        result.push_str(&rest[..start]);
        let Some(end) = rest[start + 4..].find("-->") else {
            return result;
        };
        rest = &rest[start + 4 + end + 3..];
    }

    result.push_str(rest);
    result
}
