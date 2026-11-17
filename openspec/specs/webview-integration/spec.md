# webview-integration Specification

## Purpose

Native Tauri webview hosting for the Memos frontend, including polyfill injection and window state.

## Requirements

### Requirement: Native Desktop Window

The system SHALL run the Memos frontend within a native desktop window powered by Tauri.

#### Scenario: Normal execution

- **WHEN** the app starts
- **THEN** the main webview loads an internal loader route first
- **AND** the loader redirects to the runtime Memos URL after readiness checks

### Requirement: Custom User Agent

The system SHALL configure a webview User-Agent for all server requests.

#### Scenario: API Requests

- **WHEN** remote mode is disabled or no remote user-agent override is configured
- **THEN** the User-Agent includes `Memospot/<version>`

#### Scenario: Remote user-agent override configured

- **WHEN** remote mode is enabled and `memospot.remote.user_agent` is non-empty
- **THEN** the configured override value is used as the webview User-Agent

### Requirement: Shortcut Polyfill Injection

The system SHALL inject a keyboard shortcut polyfill on each page load completion.

#### Scenario: Page load finished

- **WHEN** a webview page finishes loading
- **THEN** shortcut JavaScript is injected so app-level shortcuts continue to work even when native menu accelerators are unavailable

### Requirement: Main Window State Persistence

The system SHALL restore and persist main window state through configuration.

#### Scenario: Main window starts

- **WHEN** the main window is created
- **THEN** position, size, and maximized/fullscreen-related settings are restored from configuration

#### Scenario: Main window moved or resized

- **WHEN** the main window move/resize events occur
- **THEN** the current window position and size are persisted to runtime configuration for later save

### Requirement: Single-Instance Focus

The system SHALL enforce single-instance behavior for the main process.

#### Scenario: User launches a second instance

- **WHEN** another instance is started while one is already running
- **THEN** the existing main window is focused
