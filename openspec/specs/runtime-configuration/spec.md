# runtime-configuration Specification

## Purpose

Prepare the startup environment: configuration file, data directories, logging, database preparation, and server environment.

## Requirements

### Requirement: Ensure Config File Exists

The system SHALL ensure `memospot.yaml` exists and is writable during startup.

#### Scenario: Config file missing

- **WHEN** startup initialization runs and the config file does not exist
- **THEN** Memospot creates a default configuration file

#### Scenario: Config path cannot be used

- **WHEN** the config path is a directory or the file is not writable
- **THEN** startup reports the error

### Requirement: Recover From Malformed Config

The system SHALL support user-mediated recovery when the config file cannot be parsed.

#### Scenario: Config parse error

- **WHEN** the config file is malformed
- **THEN** Memospot asks the user whether to reset configuration
- **AND** on confirmation, backs up the broken file with a timestamped extension and resets to defaults

### Requirement: Startup Directories Are Prepared

The system SHALL ensure the Memospot data directory exists and is writable; a missing directory is created and a non-writable directory is reported.

#### Scenario: Data directory is missing

- **WHEN** the Memospot data directory does not exist
- **THEN** it is created

#### Scenario: Data directory is not writable

- **WHEN** the Memospot data directory is not writable
- **THEN** startup reports the error

### Requirement: Optional Database Backup Before Migration

The system SHALL create a pre-migration database backup when migration and backup features are enabled.

#### Scenario: Pending migrations detected

- **WHEN** migrations are enabled, pending migrations exist, and backups are enabled
- **THEN** Memospot creates a timestamped backup archive of the database plus related `-wal`/`-shm` files before running migrations

### Requirement: Optional Logging With Self-Healing Log Config

The system SHALL initialize logging only when enabled and repair invalid logger configuration files.

#### Scenario: Logging enabled with invalid `log.yaml`

- **WHEN** logger initialization fails with logging enabled
- **THEN** Memospot rewrites `log.yaml` from the default configuration and retries initialization

### Requirement: Startup Server Environment Is Resolved

The system SHALL resolve the Memos data directory and server binary and prepare the server environment before starting the server. Port selection is specified in desktop-client (see desktop-client).

#### Scenario: Default data location

- **WHEN** the Memos data setting is empty or a dot
- **THEN** the Memospot data directory is used

#### Scenario: A configured data path is invalid

- **WHEN** the configured Memos data directory resolves to a location that is not an existing directory
- **THEN** startup reports the error

#### Scenario: Binary path is empty or invalid

- **WHEN** no valid Memos binary path is configured
- **THEN** the standard installation locations are searched and the first existing binary is used
