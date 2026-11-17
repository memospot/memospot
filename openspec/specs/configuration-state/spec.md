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

A save SHALL report that a restart is required while the saved configuration differs in a startup-only setting from the values when the app started, except for window fields recently changed by the window itself, which compare against the live window state.

#### Scenario: Persisted update without restart impact

- **WHEN** the update is valid, persisted, and the saved configuration matches the comparison values in startup-only settings
- **THEN** the update reports success with no restart required

#### Scenario: Persisted update with restart impact

- **WHEN** the saved configuration differs in a startup-only setting from the comparison values, including after an earlier restart-required change
- **THEN** the update reports success with a restart required and the restart notice is presented

### Requirement: Live settings apply immediately

Live settings SHALL apply without a restart. Saving SHALL not automatically restart or switch the running Memos server.

#### Scenario: Locale is changed

- **WHEN** the user changes the locale through the settings UI and the update succeeds
- **THEN** the locale preference is persisted, the UI and application menu are updated, and no restart is required solely for the locale change

#### Scenario: The running server keeps startup values

- **WHEN** the user changes remote-server, Memos process, environment, or other startup-only configuration
- **THEN** the values are persisted and reported as restart-required while the current server process and active URL continue using their startup values

### Requirement: Window state is saved without losing settings

Window move and resize SHALL update the saved geometry without losing a concurrent settings change. Quitting SHALL persist changed window state.

#### Scenario: Window state moves during a settings change

- **WHEN** the window moves while a settings save is in progress
- **THEN** both the settings change and the window state are present in the saved configuration

### Requirement: Existing configuration remains compatible

An existing valid `memospot.yaml` SHALL load without the file being rewritten on load and without an implicit reset to defaults.

#### Scenario: Existing configuration is opened

- **WHEN** Memospot starts with an existing valid `memospot.yaml`
- **THEN** the file is accepted as-is and its fields are recognized
