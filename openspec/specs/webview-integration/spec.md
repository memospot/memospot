# webview-integration Specification

## Purpose

Host the Memos frontend in the native desktop window, including startup loading, user agent, page-load behavior, window state, single-instance handling, the Linux clipboard image-paste path, and remote app-command denial.

## Requirements

### Requirement: Native Desktop Window

The system SHALL run the Memos frontend within a native desktop window.

#### Scenario: Normal execution

- **WHEN** the app starts
- **THEN** the app shows a loading screen
- **AND** loads the runtime Memos URL once the server is ready (see memos-server-management)
- **AND** in release builds the window becomes visible only after theme and locale initialization

### Requirement: Custom User Agent

The system SHALL configure a webview User-Agent for all server requests.

#### Scenario: API Requests

- **WHEN** remote mode is disabled or no remote user-agent override is configured
- **THEN** the User-Agent includes `Memospot/<version>`

#### Scenario: Remote user-agent override configured

- **WHEN** remote mode is enabled and `memospot.remote.user_agent` is non-empty
- **THEN** the configured override value is used as the webview User-Agent

### Requirement: Navigation behavior

The system SHALL keep app-level shortcuts and the reduce-animation preference working after navigation to the Memos frontend.

#### Scenario: Navigation finished

- **WHEN** a webview page finishes loading
- **THEN** app-level shortcuts continue to work even when native menu accelerators are unavailable
- **AND** the reduce-animation preference is applied when enabled

### Requirement: Main Window State Restore

The system SHALL restore main window state from configuration at startup (see configuration-state for shutdown persistence).

#### Scenario: Main window starts

- **WHEN** the main window is created
- **THEN** position, size, resizable, centered, maximized, and fullscreen settings are restored from configuration

### Requirement: Single-Instance Focus

The system SHALL enforce single-instance behavior for the main process.

#### Scenario: User launches a second instance

- **WHEN** another instance is started while one is already running
- **THEN** the existing main window is focused

### Requirement: Clipboard Image Paste (Linux)

On Linux, when an image-only paste yields an empty transfer and the read yields an image, the image SHALL be attached to the captured editor unless a gate, size, timeout, read-failure, or replay-target rule below applies. Native pasting of text, HTML, or files with a non-empty transfer is unaffected.

#### Scenario: Image paste attaches the clipboard image at the editor

- **WHEN** a trusted image-only paste reaches the editor with an empty transfer and the read yields an image
- **THEN** exactly one clipboard image is attached to the editor that received the paste

#### Scenario: Image-only types listed without files still read

- **WHEN** an image-only paste lists types (e.g. `["Files"]`) but carries no files
- **THEN** the app read fires and exactly one clipboard image is attached if the read yields an image

#### Scenario: Untrusted pastes start no clipboard read

- **WHEN** an untrusted paste reaches the editor
- **THEN** no clipboard read occurs for that paste

#### Scenario: Text paste proceeds natively

- **WHEN** a paste carries text in any `text/*` type (including alongside HTML) with a non-empty transfer
- **THEN** the editor receives the native text paste unchanged
- **AND** no clipboard image is attached

#### Scenario: Empty text item counts as empty

- **WHEN** a paste carries only an empty (zero-length) text item alongside image data
- **THEN** the app read fires and exactly one clipboard image is attached if the read yields an image

#### Scenario: Mixed image and text drops the image

- **WHEN** a paste carries clipboard image data with any text and the transfer is non-empty
- **THEN** the editor receives the native text paste unchanged
- **AND** no clipboard image is attached

#### Scenario: Empty transfer loses pasted text

- **WHEN** an image-and-text paste yields an empty transfer that exposes neither text nor files
- **THEN** exactly one clipboard image is attached and the text is lost (known limitation)

#### Scenario: HTML paste proceeds natively

- **WHEN** a paste carries `text/html` without `text/plain`
- **THEN** the editor receives the native HTML paste unchanged
- **AND** no clipboard image is attached

#### Scenario: Empty clipboard is a quiet no-op

- **WHEN** a paste is empty and the clipboard holds no image
- **THEN** the editor content is unchanged
- **AND** no clipboard image is attached and no error is shown

#### Scenario: Wayland keeps its native paste path

- **WHEN** an image-only paste reaches the editor on Wayland and the native transfer is non-empty
- **THEN** the editor receives the native paste unchanged

#### Scenario: Wayland empty transfer uses the app path

- **WHEN** an image-only paste on Wayland yields an empty transfer and the read yields an image
- **THEN** exactly one clipboard image is attached to the editor

#### Scenario: Wayland read failure is a quiet no-op

- **WHEN** the clipboard yields no image on Wayland
- **THEN** the editor content is unchanged

#### Scenario: The captured editor receives the replay

- **WHEN** the user moves focus between the paste and the clipboard reply and the captured editor is still connected and editable
- **THEN** exactly one clipboard image is attached to the captured editor

#### Scenario: Disconnected editor drops the replay

- **WHEN** the captured editor is no longer connected at reply time
- **THEN** no clipboard image is attached

#### Scenario: Non-editable target drops the replay

- **WHEN** the captured target is no longer editable at reply time
- **THEN** no clipboard image is attached

#### Scenario: Clipboard read timeout is a quiet no-op

- **WHEN** an accepted read takes longer than 2000 ms after gate acceptance to complete
- **THEN** the editor content is unchanged and any late reply is discarded

#### Scenario: Denied reads are quiet on the page

- **WHEN** a clipboard request is rejected for any reason
- **THEN** the page shows no error and logs no unhandled rejection

#### Scenario: Reload attaches exactly once

- **WHEN** the page reloads and one image-only paste occurs
- **THEN** exactly one clipboard image is attached

#### Scenario: Double load completion attaches once

- **WHEN** the page signals load completion twice for one document and one image-only paste occurs
- **THEN** exactly one clipboard image is attached

#### Scenario: Non-editor pastes paste natively

- **WHEN** an image-only paste reaches a non-editor field
- **THEN** no image is attached and the field pastes natively

#### Scenario: Other platforms keep native behavior

- **WHEN** an image-only paste occurs on macOS or Windows
- **THEN** native paste behavior is unchanged

#### Scenario: URI-list paste proceeds natively

- **WHEN** a paste carries a non-empty `text/uri-list`
- **THEN** the editor receives the native paste unchanged
- **AND** no clipboard image is attached

#### Scenario: Malformed nonce is rejected

- **WHEN** a clipboard request carries a malformed page-load nonce
- **THEN** no clipboard read occurs and no clipboard image is attached

#### Scenario: File paste proceeds natively

- **WHEN** a paste carries files, including an image transfer with files present
- **THEN** the editor receives the native paste unchanged
- **AND** the app read does not fire

#### Scenario: Replay does not double-attach

- **WHEN** the replayed paste is delivered at the captured editor
- **THEN** no further clipboard image is attached for that paste

### Requirement: Clipboard read gate (Linux)

The app's clipboard-image read SHALL occur only for an eligible request from the runtime server origin (see desktop-client "Resolve Startup Server Mode" and Clipboard-eligible origins), bearing a fresh timestamp and a nonce bound to the current page load. The webview-origin match is the trust anchor; the request origin must be present and match.

#### Scenario: Webview origin does not match, regardless of request origin

- **WHEN** a clipboard request arrives from a webview whose origin does not match the runtime server origin, whether or not a request origin is present
- **THEN** no clipboard read occurs and no clipboard image is attached

#### Scenario: Mismatched request origin is rejected

- **WHEN** the clipboard request carries a request origin that does not match the runtime server origin
- **THEN** no clipboard read occurs and no clipboard image is attached

#### Scenario: Unparseable origin is rejected

- **WHEN** the runtime, webview, or request origin cannot be parsed
- **THEN** no clipboard read occurs and no clipboard image is attached

#### Scenario: Missing request origin is rejected

- **WHEN** a clipboard request arrives without a request origin
- **THEN** no clipboard read occurs and no clipboard image is attached

#### Scenario: Opaque request origin is rejected

- **WHEN** a clipboard request arrives with an opaque request origin
- **THEN** no clipboard read occurs and no clipboard image is attached

#### Scenario: Null transfer fails closed

- **WHEN** `clipboardData` is null
- **THEN** no clipboard read occurs and no clipboard image is attached

#### Scenario: Subframe invocation is rejected

- **WHEN** a clipboard request originates from a cross-origin subframe
- **THEN** no clipboard read occurs and no clipboard image is attached

#### Scenario: Origin is the sole trust boundary

- **WHEN** a script in the configured origin holds the current nonce and mints a fresh timestamp
- **THEN** the read is accepted

#### Scenario: Loader route may not read the clipboard

- **WHEN** a webview is still on the loader route before the Memos page loads
- **THEN** no clipboard read occurs and no clipboard image is attached

#### Scenario: Stale requests are rejected

- **WHEN** a clipboard request arrives with a timestamp older than 2000 ms (exactly 2000 ms old is accepted)
- **THEN** no clipboard read occurs and no clipboard image is attached

#### Scenario: Overlapping reads drop the second paste

- **WHEN** a second paste occurs while a read for the same webview is still running
- **THEN** no image is attached for the second paste

#### Scenario: Future requests are rejected

- **WHEN** a clipboard request arrives with a timestamp more than 200 ms in the future (exactly 200 ms early is accepted)
- **THEN** no clipboard read occurs and no clipboard image is attached

#### Scenario: Requests from another page load are rejected

- **WHEN** a clipboard request arrives with a nonce from a previous page load after reload
- **THEN** no clipboard read occurs and no clipboard image is attached

#### Scenario: Destroyed webview invalidates its nonce

- **WHEN** a webview is destroyed and a later request carries its nonce
- **THEN** no clipboard read occurs and no clipboard image is attached

#### Scenario: Cross-window nonces are isolated

- **WHEN** two webview windows each hold their own page-load nonce and each requests a read with its own nonce
- **THEN** each request is accepted independently and a nonce from one window is rejected in the other

#### Scenario: Cleared clipboard reads nothing

- **WHEN** the clipboard is cleared after a paste and another read is requested
- **THEN** no clipboard image is attached

### Requirement: Clipboard-eligible origins (Linux)

Eligibility SHALL be derived from the runtime server origin (see desktop-client "Resolve Startup Server Mode"): only an `https:` origin or a loopback `http:` origin is eligible, compared as canonical `scheme + host + port` (default ports folded) with `localhost` and `127.0.0.1` distinct. Loopback SHALL mean exactly `localhost`, `127.0.0.1`, or `::1`.

#### Scenario: HTTPS remote attaches

- **WHEN** an HTTPS remote Memos server (including a non-default port) is the runtime origin and an image-only paste yields an empty transfer
- **THEN** exactly one clipboard image is attached to the editor

#### Scenario: Loopback HTTP attaches

- **WHEN** a loopback `http:` Memos server is the runtime origin and an image-only paste yields an empty transfer
- **THEN** exactly one clipboard image is attached to the editor

#### Scenario: Loopback means exactly three hosts

- **WHEN** the runtime origin is any of `localhost`, `127.0.0.1`, or `::1`
- **THEN** it counts as loopback for the eligibility rule

#### Scenario: Loopback alias mismatch is rejected

- **WHEN** the runtime origin is `localhost` but the webview origin is `127.0.0.1`, or vice versa
- **THEN** no clipboard read occurs and no clipboard image is attached

#### Scenario: Plain-HTTP remote is rejected

- **WHEN** a plain-HTTP non-loopback remote is the configured origin and an image-only paste yields an empty transfer
- **THEN** no clipboard read occurs and no clipboard image is attached by the app

#### Scenario: Scheme mismatch is rejected

- **WHEN** the runtime origin is HTTPS but the webview origin is plain HTTP on the same host
- **THEN** no clipboard read occurs and no clipboard image is attached

### Requirement: Remote app-command denial

A remote page SHALL NOT invoke app commands. The paste read is allowed on Linux where it is registered, and denied elsewhere along with the other commands. Local app commands keep working. Page shortcut events are out of scope for this change; the retained event path can still trigger shortcut actions, which is accepted.

#### Scenario: Remote origin is denied app commands

- **WHEN** a remote page invokes an app command through the invoke surface
- **THEN** the invocation is denied

#### Scenario: Remote denial applies on macOS

- **WHEN** a remote page invokes an app command on macOS
- **THEN** the invocation is denied

#### Scenario: Local app commands keep working

- **WHEN** a local app command is invoked from the app
- **THEN** the command succeeds

### Requirement: Clipboard bytes are not retained (Linux)

Clipboard bytes SHALL be returned only to the requesting page load; a second request SHALL reflect the current clipboard content.

#### Scenario: Bytes reflect the current clipboard

- **WHEN** two sequential reads occur with changed clipboard content between them
- **THEN** each attached image matches the clipboard content at its own read time

### Requirement: Clipboard diagnostic hygiene (Linux)

Diagnostics for clipboard requests SHALL NOT contain clipboard bytes, request URLs, request origins, or page-load nonces.

#### Scenario: Rejections emit one reason-coded warning

- **WHEN** a clipboard request is rejected or dropped
- **THEN** one warn diagnostic with a reason code is emitted, without clipboard bytes, request URLs, request origins, or page-load nonces

#### Scenario: Rejection errors carry no sensitive values

- **WHEN** a clipboard request is rejected
- **THEN** the returned error contains no clipboard bytes, request URLs, request origins, or page-load nonces

#### Scenario: Served reads are logged without bytes

- **WHEN** a clipboard read is served
- **THEN** one info diagnostic is emitted without clipboard bytes, request URLs, request origins, or page-load nonces

### Requirement: Clipboard target preference (Linux)

When the clipboard offers more than one supported image target, the PNG target SHALL be preferred, then JPEG, then WebP.

#### Scenario: PNG wins over JPEG when both are offered

- **WHEN** the clipboard offers both PNG and JPEG targets
- **THEN** the PNG bytes are attached

#### Scenario: JPEG wins over WebP when both are offered

- **WHEN** the clipboard offers both JPEG and WebP targets
- **THEN** the JPEG bytes are attached

### Requirement: Clipboard bytes match the detected type (Linux)

When the clipboard read returns PNG, JPEG, or WebP bytes, the attached file type, name, and bytes SHALL match the detected bytes (`image.png`, `image.jpg`, `image.webp`). Bytes whose type cannot be detected SHALL be dropped with no image attached. The decoded fallback SHALL attach `image.png`.

#### Scenario: MIME bytes are preserved

- **WHEN** the clipboard offers a PNG, JPEG, or WebP target and the clipboard read returns those bytes
- **THEN** the attached file type, name, and bytes match the detected bytes

#### Scenario: Sniff failure drops the image

- **WHEN** the clipboard read returns bytes whose type cannot be detected
- **THEN** no clipboard image is attached

#### Scenario: Empty preferred target falls through to the next

- **WHEN** the clipboard advertises PNG and JPEG but the PNG target yields no bytes
- **THEN** the JPEG bytes are attached

#### Scenario: Detected type wins over advertised target

- **WHEN** the clipboard advertises a PNG target but the returned bytes detect as JPEG
- **THEN** the JPEG type, name, and bytes are attached

### Requirement: Clipboard fallback decoding (Linux)

When the clipboard holds an image but no supported target yields bytes and no size limit rejects it (see Clipboard image size limits), the fallback SHALL attach exactly one valid PNG.

#### Scenario: Fallback decoding yields a valid PNG

- **WHEN** the clipboard offers no supported image target that yields bytes, the fallback decoding succeeds within size limits, and no other rejection applies
- **THEN** exactly one PNG image is attached

#### Scenario: GIF-only clipboard attaches a static PNG

- **WHEN** the clipboard offers only GIF with no PNG, JPEG, or WebP target
- **THEN** exactly one static PNG is attached and animation is lost

#### Scenario: Fallback decoding failure is a quiet no-op

- **WHEN** no supported image target yields bytes and the fallback decoding fails, times out, or exceeds the encoded cap
- **THEN** no clipboard image is attached

### Requirement: Clipboard image size limits (Linux)

Images larger than 10 MiB (10,485,760 bytes of image data as returned by the clipboard read) SHALL be dropped with no image attached. Fallback-decoded images larger than 67,108,864 decoded pixel bytes SHALL be dropped with no image attached. The 10 MiB cap also applies to fallback output. Both caps are arbitrary guards on the app-assisted path only, set below the Memos server's own upload limit; native pastes that bypass the app read are not subject to them.

#### Scenario: Encoded image exceeds the size limit

- **WHEN** the clipboard image is larger than 10 MiB of image data
- **THEN** no clipboard image is attached

#### Scenario: Fallback decoded image exceeds the memory limit

- **WHEN** the fallback decoded image exceeds 67,108,864 decoded pixel bytes
- **THEN** no clipboard image is attached
