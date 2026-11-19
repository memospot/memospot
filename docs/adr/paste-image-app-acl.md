# Linux paste-image app ACL manifest

## Status

Accepted.

## Context

The gated `request_paste_image` command needs an allow-list that remote
server pages can use while every other app command stays local-only. Tauri
creates an app ACL manifest as soon as any file exists under
`crates/memospot/permissions/`; from that point every app command invoked
without a matching `allow-*` permission is denied, on all platforms.

## Decision

Declare one `allow-<command>` permission per app command in
`crates/memospot/permissions/app.toml`, re-allow the seventeen existing
commands for local pages in `capabilities/default.toml`, and grant only
`allow-request-paste-image` to remote origins in the Linux-only
`capabilities/paste-image.toml` with explicit `https:` and loopback-`http:`
URL patterns. The Rust origin check against the configured server origin
remains the enforcement; the static patterns only express the eligible
scheme and host shape.

## Consequences

Remote pages lose all app commands except the Linux paste read; the retained
`core:event:allow-emit` shortcut path is unchanged. Any future command must
gain an `allow-*` permission and a capability entry, or local invocations
break; Task 4.1 guards this with a handler-versus-permission coverage check.
