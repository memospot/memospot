# app-shortcuts Specification

## Purpose

Make the menu-bar, settings, and search shortcuts available.

## Requirements

### Requirement: Menu-bar and settings shortcuts

The application SHALL trigger the menu-bar action (see app-menu) on `Ctrl+H` (or `Cmd+H` on macOS) where the toggle is available, and trigger the settings action (see app-menu) on `Ctrl+,` (or `Cmd+,` on macOS). Zoom shortcuts are owned by desktop-client (see desktop-client); persistence after navigation is owned by webview-integration (see webview-integration).

#### Scenario: Toggle menu bar shortcut

- **WHEN** the user presses the menu-bar shortcut where the toggle is available
- **THEN** the menu-bar action runs (see app-menu)

#### Scenario: Open settings shortcut

- **WHEN** the user presses the settings shortcut
- **THEN** the settings action runs (see app-menu)

### Requirement: Search focus shortcut

Pressing `F3` or `Ctrl+F` (or `Cmd+F` on macOS) or `Ctrl+K` (or `Cmd+K` on macOS) SHALL focus the settings search input when the settings screen is shown.

#### Scenario: Find shortcut

- **WHEN** the user presses a find shortcut while viewing settings
- **THEN** focus moves to the settings search input
