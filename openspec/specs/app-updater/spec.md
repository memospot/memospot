# app-updater Specification

## Purpose

Keep the application current by checking for updates in the background and offering to install them.

## Requirements

### Requirement: Automatic update checks

The application SHALL check for updates in the background when automatic updates are enabled, the configured interval has elapsed, and the app is not running as a Flatpak.

#### Scenario: Update available

- **WHEN** a check finds a newer version
- **THEN** the user is prompted to install it, the managed server is shut down before installation, and the application restarts after installation

#### Scenario: Install fails

- **WHEN** downloading or installing an available update fails
- **THEN** the user is offered to open the release page manually

#### Scenario: Install fails and the manual prompt is declined

- **WHEN** the manual prompt is declined after a failed install
- **THEN** the application restarts with the current version still installed

#### Scenario: No update found

- **WHEN** a check finds no newer version
- **THEN** no prompt is shown

#### Scenario: User declines installation

- **WHEN** the user declines the installation prompt
- **THEN** the current version keeps running without downloading

### Requirement: Manual update check feedback

When no update is available, manually checking for updates SHALL show a no-update message.

#### Scenario: Manual check finds no update

- **WHEN** the user manually checks for updates and none is available
- **THEN** a no-update message is shown
