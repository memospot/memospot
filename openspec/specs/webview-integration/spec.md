# webview-integration Specification

## Purpose

Host the Memos frontend in the native desktop window, including startup loading, user agent, page-load behavior, window state, and single-instance handling.

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
