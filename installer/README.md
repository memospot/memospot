# Installer code

This directory contains the code for Windows installers.

Tauri's built-in templates are not used directly to avoid unexpected behavior changes between versions that could wipe the user's database during an uninstall.

- `$LOCALAPPDATA\${BUNDLEID}` (`%LocalAppData%\com.memospot.app`)
  > [!TIP]
  > WebView cache
  >
  > Safe to delete recursively.
- `$LOCALAPPDATA\${PRODUCTNAME} `(`%LocalAppData%\Memospot`)
  User data and application binaries.

  > [!CAUTION]
  > Stores user data
  >
  > Only memos.exe and memospot.exe (if using the NSIS/EXE installer) can be safely deleted.

## Sources

- [nsis_template.nsi](https://github.com/tauri-apps/tauri/tree/dev/tooling/bundler/src/bundle/windows/templates/installer.nsi)

- [wix_template.wxs](https://github.com/tauri-apps/tauri/tree/dev/tooling/bundler/src/bundle/windows/templates/main.wxs)
