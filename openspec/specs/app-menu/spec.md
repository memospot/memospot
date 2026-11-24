# app-menu Specification

## Purpose

Provide the application menu and the window-level actions it triggers.

## Requirements

### Requirement: Application menu is presented

The main window SHALL present an application menu with settings, browse, update-check, open-in-browser, and quit actions, a view menu with new-window, refresh, reload, zoom (see desktop-client), fullscreen (where available), and (where available) menu-bar actions, and a help menu with version, documentation, release-notes, and report-issue actions for Memospot, plus Memos version, documentation, and release-notes entries.

#### Scenario: Menu is hidden by preference

- **WHEN** the menu bar is hidden by preference
- **THEN** the application menu bar is not shown

#### Scenario: Menu shows versions

- **WHEN** the application starts
- **THEN** the help menu shows the application version, including the short revision or `unknown` when unavailable, and the Memos version once known

### Requirement: Menu actions perform their function

Each menu action SHALL perform its observable function: opening or focusing settings, browsing the data or downloads directory, opening the current address in the browser, zooming (see desktop-client), fullscreen where available, checking for updates, quitting, opening a new window, refreshing to the server root, reloading the application route, toggling the menu bar where available, and opening help links in the browser.

#### Scenario: Settings menu action

- **WHEN** the settings action is chosen
- **THEN** the settings window opens and is focused, or the existing settings window receives focus

#### Scenario: New window action

- **WHEN** the new-window action is chosen
- **THEN** a fresh main window is opened

#### Scenario: Open in browser action

- **WHEN** the open-in-browser action is chosen
- **THEN** the current Memos address is opened in the default browser

#### Scenario: Update check action

- **WHEN** the check-for-updates action is chosen
- **THEN** an update check runs (see app-updater)

#### Scenario: Quit action

- **WHEN** the quit action is chosen
- **THEN** the application exits
