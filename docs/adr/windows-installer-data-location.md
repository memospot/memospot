# Windows installer data location

## Status

Accepted.

## Context

The NSIS `currentUser` installer installs Memospot to `%LOCALAPPDATA%\Memospot`, and the app resolves its data directory to `%LOCALAPPDATA%\memospot`. Windows paths are case-insensitive, so the install directory, the notes database, and the configuration file share one directory. The uninstaller offers a "Delete app data" checkbox that recursively removes the WebView cache under `%APPDATA%\com.memospot.app` and `%LOCALAPPDATA%\com.memospot.app`; notes must never enter either path.

## Decision

Keep the upstream Tauri installer layout and the app's data location. Do not add recursive deletion of `$INSTDIR` or `${PRODUCTNAME}`. Keep the only recursive deletes in the uninstaller limited to `$APPDATA\${BUNDLEID}` and `$LOCALAPPDATA\${BUNDLEID}`. Enforce the invariant with a Rust test and the vendored-vs-upstream diff.

## Consequences

Notes survive an uninstall by default. The "Delete app data" checkbox remains a WebView-cache cleanup, not a notes cleanup. The bundle identifier must never equal the product name or the data-directory name, and any future installer template update must preserve the non-recursive `$INSTDIR` removal.
