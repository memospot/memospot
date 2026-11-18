# configuration-state Specification

## Purpose

Define how settings changes are persisted, when they require a restart, and how live settings and window state are applied.

## Requirements

### Requirement: Settings changes persist atomically

A valid settings change SHALL be written to `memospot.yaml` before the UI reports success. An invalid change or a failed write SHALL leave the file unchanged and report failure.

#### Scenario: Concurrent changes are both kept

- **WHEN** two settings changes target different fields at the same time
- **THEN** both changes are retained and neither overwrites the other

#### Scenario: Invalid change

- **WHEN** a change cannot be parsed or produces an invalid configuration
- **THEN** it is rejected, the saved configuration is unchanged, and the file is not rewritten

#### Scenario: Write fails

- **WHEN** a valid change cannot be written to the file
- **THEN** the update is reported as failed and is not presented as successful

### Requirement: Restart impact is reported

A save SHALL report that a restart is required while the saved configuration differs in a startup-only setting from the comparison values (the configuration as established at startup, including startup-resolved values, updated by window state the window itself changed). Moving, resizing, or maximizing the window SHALL NOT by itself cause a later save to report a restart.

#### Scenario: Persisted update without restart impact

- **WHEN** the update is valid, persisted, and the saved configuration matches the comparison values in startup-only settings
- **THEN** the update reports success with no restart required

#### Scenario: Persisted update with restart impact

- **WHEN** the saved configuration differs in a startup-only setting from the comparison values, including after an earlier restart-required change
- **THEN** the update reports success with a restart required and the restart notice is presented

#### Scenario: Window changes do not force a restart

- **WHEN** the window moves, resizes, or maximizes and the user then saves an unrelated setting
- **THEN** the update reports success with no restart required for the window changes

### Requirement: Live settings apply immediately

Live settings SHALL apply without a restart. Saving SHALL not automatically restart or switch the running Memos server.

#### Scenario: Locale is changed

- **WHEN** the user changes the locale through the settings UI and the update succeeds
- **THEN** the locale preference is persisted, the UI and application menu are updated, and no restart is required solely for the locale change

#### Scenario: The running server keeps startup values

- **WHEN** the user changes remote-server, Memos process, environment, or other startup-only configuration
- **THEN** the values are persisted and reported as restart-required while the current server process and active URL continue using their startup values

### Requirement: Window state is saved without losing settings

Window move, resize, and maximize SHALL update the saved window state without losing a concurrent settings change. Geometry reported while maximized SHALL NOT overwrite the saved geometry, and events without a maximized toggle SHALL leave the saved maximized preference unchanged. Quitting SHALL persist changed window state.

#### Scenario: Window state moves during a settings change

- **WHEN** the window moves while a settings save is in progress
- **THEN** both the settings change and the window state are present in the saved configuration

#### Scenario: Maximizing is recorded and persisted

- **WHEN** the window toggles to maximized and the configuration is saved on shutdown
- **THEN** the saved configuration records the maximized state and the saved geometry is unchanged

#### Scenario: Stale maximized events keep the saved preference

- **WHEN** maximized window events arrive without a maximized toggle after the saved preference changed via settings
- **THEN** the saved maximized preference is unchanged

### Requirement: Existing configuration remains compatible

An existing valid `memospot.yaml` SHALL load without the file being rewritten on load and without an implicit reset to defaults.

#### Scenario: Existing configuration is opened

- **WHEN** Memospot starts with an existing valid `memospot.yaml`
- **THEN** the file is accepted as-is and its fields are recognized

### Requirement: Settings updates are accepted only from the settings window

A request to change persisted configuration through the generic settings-update interface SHALL be accepted only from the settings window. A request from the main Memos window, a cloned main window, or any other window SHALL be rejected without applying or persisting any part of the change.

#### Scenario: Settings window saves a change

- **WHEN** the settings window submits a valid settings update
- **THEN** the update is applied and persisted

#### Scenario: Main Memos window attempts a settings update

- **WHEN** the main Memos window submits a settings update
- **THEN** the request is rejected, the caller receives an error, and the saved configuration is unchanged

#### Scenario: Cloned main window attempts a settings update

- **WHEN** a cloned main window submits a settings update
- **THEN** the request is rejected and the saved configuration is unchanged

#### Scenario: Rejected update has no side effects

- **WHEN** a settings update is rejected because it came from a window other than the settings window
- **THEN** no locale change, restart notice, or configuration file rewrite occurs
