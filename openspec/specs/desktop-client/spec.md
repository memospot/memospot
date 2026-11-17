## ADDED Requirements

### Requirement: Resolve Startup Server Mode

The system SHALL resolve the runtime Memos URL from configuration and determine whether to run a managed local server or client mode.

#### Scenario: Remote server not enabled

- **WHEN** `memospot.remote.enabled` is not `true` or `memospot.remote.url` is empty
- **THEN** Memospot uses `http://localhost:<memos.port>/` as its runtime URL and starts the managed local Memos server

#### Scenario: Remote server enabled with URL

- **WHEN** `memospot.remote.enabled` is `true` and `memospot.remote.url` is non-empty
- **THEN** Memospot uses that remote URL as its runtime URL
- **AND** treats the app as client mode unless the resolved URL matches the managed localhost pattern

### Requirement: Runtime Remote URL Handling

The system SHALL apply minimal runtime validation and normalization to the configured remote URL.

#### Scenario: Invalid URL provided

- **WHEN** a configured remote URL does not start with `http`
- **THEN** Memospot shows an invalid URL error dialog
- **AND** still returns the configured URL with a trailing slash as the runtime URL

### Requirement: Settings UI Remote URL Validation

The settings UI SHALL normalize and validate a remote URL before persisting it.

#### Scenario: User enters remote URL in settings

- **WHEN** the user edits the remote URL in settings
- **THEN** the UI ensures the URL uses `http` or `https` and ends with `/`
- **AND** pings the server before applying the change
- **AND** restores the previous value and shows an error toast when validation fails

### Requirement: Zoom In Hotkey

The system SHALL increase webview zoom when the user presses `Ctrl+=` (or `Cmd+=` on macOS), including numpad equivalents.

#### Scenario: User zooms in

- **WHEN** user presses the zoom-in hotkey
- **THEN** the webview zoom factor increases by a fixed step of `0.1`

### Requirement: Zoom Out Hotkey

The system SHALL decrease webview zoom when the user presses `Ctrl+-` (or `Cmd+-` on macOS), including numpad equivalents.

#### Scenario: User zooms out

- **WHEN** user presses the zoom-out hotkey
- **THEN** the webview zoom factor decreases by a fixed step of `0.1`

### Requirement: Reset Zoom Hotkey

The system SHALL reset the webview zoom level to the default when the user presses `Ctrl+0` (or `Cmd+0` on macOS).

#### Scenario: User resets zoom

- **WHEN** user presses the reset-zoom hotkey
- **THEN** the webview zoom factor returns to its base 100% scale

### Requirement: Zoom Bounds

The system SHALL clamp zoom changes to a safe range.

#### Scenario: Repeated zoom in/out

- **WHEN** zoom operations exceed configured limits
- **THEN** zoom is clamped to the range `0.2` through `5.0`
