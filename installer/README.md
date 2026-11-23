# Installer code for Windows

This directory contains the vendored Tauri installer templates for Windows.

The templates are pinned to `tauri-cli-v2.12.4` and are not replaced by Tauri's built-in templates at build time. Pinning prevents an unexpected upstream change from deleting user notes during an uninstall.

## Data locations

### `$APPDATA\${BUNDLEID}`

`%AppData%\com.memospot.app`

Roaming WebView cache. The uninstaller recursively deletes this under the same checkbox gate.

### `$LOCALAPPDATA\${BUNDLEID}`

`%LocalAppData%\com.memospot.app`

WebView cache. Safe to delete recursively. The uninstaller deletes this only when the "Delete app data" checkbox is selected.

### `$LOCALAPPDATA\${PRODUCTNAME}`

`%LocalAppData%\Memospot`

User data and application binaries for the NSIS `currentUser` installer. Notes live here. Never delete this directory recursively, and never make `${BUNDLEID}` equal `${PRODUCTNAME}` or `memospot`; the uninstaller's recursive cache cleanup would then delete notes.

The "Delete app data" checkbox deletes the WebView cache, not the notes. Passive uninstalls (`/P`) skip the confirm page and leave the checkbox unset.

## Invariants

- Recursive deletion is allowed only for `$APPDATA\${BUNDLEID}` and `$LOCALAPPDATA\${BUNDLEID}`.
- `$INSTDIR` is removed non-recursively.
- The Rust test `crates/memospot/src/installer_invariants.rs` checks the identifier/data-directory inequality and scans the vendored templates for recursive deletes.
- `bundle.resources` and `installer_hooks` must not target the data directory.

## Tweaks

- The main `memospot.exe` is handled by upstream `CheckIfAppIsRunning`, which uses the Windows Restart Manager and the installed executable path.
- `memos.exe` keeps a custom `KillMemosIfRunning` macro because the server can be started from a custom `memos.binary_path`, the current directory, Program Files, or ProgramData. It uses the global `FindProcess`/`KillProcess` pair, accepts "no matching process" as success, and replaces the `{{product_name}}` placeholder in dialogs.
- The 2.12.4 main-exe check aborts with a permission error when `memospot.exe` runs elevated; close it manually. The combined 2.12.5 bump adds the permission-denied message.
- The bundler writes its own `utils.nsh` into the output directory; this repository does not keep a local copy.

## MSI

The NSIS installer is `currentUser` and defaults to `%LOCALAPPDATA%\Memospot`. The MSI template keeps `InstallScope="perMachine"`. The 2.12.4 WiX search reads the unnamed `InstallDir` value that NSIS writes, so an MSI can reuse the NSIS per-user path. Test NSIS↔MSI transitions before shipping; if the perMachine MSI cannot safely reuse the path, scope the MSI out or fix it first.

## Sources

- `installer.nsi`: [tauri-cli-v2.12.4](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.12.4/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi), blob `75efe3da...`
- `main.wxs`: [tauri-cli-v2.12.4](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.12.4/crates/tauri-bundler/src/bundle/windows/msi/main.wxs), blob `dec39dad...`
- See `installer/UPSTREAM.md` for the normalized diff.
