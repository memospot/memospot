# home-dir

## Example tilde notation in paths

This crate provides a function to expand tilde notation, for referring to home directories, in paths.

For example, `~/foo` would be expanded to something like `/home/johndoe/foo
`

if the current user's home directory is `/home/johndoe`.

Examples:

```rust
use home_dir::HomeDirExt;

let public_html = "~/public_html".expand_home().unwrap();
```

```rust
use home_dir::home_dir;

let other_user_home = "~otheruser".expand_home().unwrap();
```

> Note: Named user expansion is currently not supported on Windows, so this will return an error.

## Original package

<https://github.com/eulegang/home-dir>
