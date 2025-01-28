# Installer code for Windows

This directory contains the code for Windows installers.

Tauri built-in templates are not used directly to avoid unexpected behavior changes between versions that could wipe the user's database during an uninstallation.

## `$LOCALAPPDATA\${BUNDLEID}`

`%LocalAppData%\com.memospot.app`

> [!TIP]
> WebView cache
>
> Safe to delete recursively.

## `$LOCALAPPDATA\${PRODUCTNAME}`

`%LocalAppData%\Memospot`

User data and application binaries.

> [!CAUTION]
> Stores user data
>
> Only memos.exe and memospot.exe (if using the NSIS/EXE installer) can be safely deleted.

## Sources

- [nsis.nsi](https://github.com/tauri-apps/tauri/tree/dev/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi)

- [wix.wxs](https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/windows/msi/main.wxs)
