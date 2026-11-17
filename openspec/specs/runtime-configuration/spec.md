## ADDED Requirements

### Requirement: Ensure Config File Exists

The system SHALL ensure `memospot.yaml` exists and is writable during startup.

#### Scenario: Config file missing

- **WHEN** startup initialization runs and the config file does not exist
- **THEN** Memospot creates a default configuration file

### Requirement: Recover From Malformed Config

The system SHALL support user-mediated recovery when the config file cannot be parsed.

#### Scenario: Config parse error

- **WHEN** the config file is malformed
- **THEN** Memospot asks the user whether to reset configuration
- **AND** on confirmation, backs up the broken file with a timestamped extension and resets to defaults

### Requirement: Persist Runtime Config Changes

The system SHALL persist in-app configuration changes made through the settings UI.

#### Scenario: Settings update

- **WHEN** a valid config patch is applied through the command interface
- **THEN** runtime config is updated and saved back to `memospot.yaml`

### Requirement: Optional Database Backup Before Migration

The system SHALL create a pre-migration database backup when migration and backup features are enabled.

#### Scenario: Pending migrations detected

- **WHEN** migrations are enabled, pending migrations exist, and backups are enabled
- **THEN** Memospot creates a timestamped backup archive of the database plus related `-wal`/`-shm` files before running migrations

### Requirement: Optional Logging With Self-Healing Log Config

The system SHALL initialize logging only when enabled and repair invalid logger configuration files.

#### Scenario: Logging enabled with invalid `log.yaml`

- **WHEN** logger initialization fails with logging enabled
- **THEN** Memospot rewrites `log.yaml` from the embedded default template and retries initialization
