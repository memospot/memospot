# Vendored Tauri installer templates

Pinned to `tauri-cli-v2.12.4`.

| File                 | Upstream path                                                | Blob SHA                                   |
| -------------------- | ------------------------------------------------------------ | ------------------------------------------ |
| `installer/nsis.nsi` | `crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi` | `75efe3da953fb6914a08359705d94d9ee8732542` |
| `installer/wix.wxs`  | `crates/tauri-bundler/src/bundle/windows/msi/main.wxs`       | `dec39dadb4bc9160d108d19094987ada2a6eb996` |

## Normalized diff

`installer/wix.wxs` is byte-identical to upstream.

`installer/nsis.nsi` differs only in the block marked
`; BEGIN MEMOSPOT CUSTOM` through `; END MEMOSPOT CUSTOM` (the
`KillMemosIfRunning` macro and its `Var`s) and the two
`!insertmacro KillMemosIfRunning "memos.exe" "Memos"` call sites. The macro keeps
the name-based `memos.exe` shutdown because the server can run from a custom
`memos.binary_path`, the current directory, Program Files, or ProgramData.

Compare against the pinned tag with the normalized diff described above.
