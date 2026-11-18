# runtime-configuration Specification

## Purpose

Prepare the startup environment: configuration file, data directories, logging, database backup, WebView availability, data and binary resolution, and process environment.

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

#### Scenario: Rewritten log config still fails

- **WHEN** logger initialization fails again after the rewrite
- **THEN** startup reports the error and does not continue

### Requirement: Server Paths Are Resolved

The system SHALL resolve the Memos data directory and server binary before starting the server. Port selection is specified in desktop-client (see desktop-client).

#### Scenario: Default data location

- **WHEN** the Memos data setting is empty or a dot
- **THEN** the Memospot data directory is used

#### Scenario: Configured data path does not exist or is a file

- **WHEN** the configured Memos data directory does not exist or is a file
- **THEN** startup reports the error

#### Scenario: Binary path is empty or invalid

- **WHEN** no valid Memos binary path is configured
- **THEN** the application directory, data directory, and standard installation locations are searched and the first existing binary is used

#### Scenario: No binary found

- **WHEN** no Memos binary is found in any searched location
- **THEN** startup reports the error

### Requirement: Process Environment Is Applied

When environment-variable configuration is enabled, its entries SHALL be applied to the Memospot process and inherited by the managed server; an empty value removes the corresponding variable.

#### Scenario: Environment variables enabled

- **WHEN** environment variables are enabled and an entry has a value
- **THEN** the variable is set for the Memospot process and inherited by the managed server

#### Scenario: Empty value removes the variable

- **WHEN** the entry value is empty
- **THEN** the variable is removed from the Memospot process

### Requirement: WebView Availability Is Ensured

On Windows, startup SHALL ensure a WebView runtime is available before building the interface.

#### Scenario: WebView runtime is missing

- **WHEN** no WebView runtime is available
- **THEN** Memospot offers to install it and does not continue until it is available

#### Scenario: User declines installation

- **WHEN** the user declines the installation offer
- **THEN** startup exits without building the interface

#### Scenario: Installation fails

- **WHEN** installation fails
- **THEN** an error is shown, the install page is opened, and startup does not continue

### Requirement: Linux Rendering Tuning Preserves User Overrides

On Linux, the application SHALL apply rendering-related environment values for its own process without overriding user-defined values.

#### Scenario: User override exists

- **WHEN** a rendering-related environment value is already defined
- **THEN** the user value is kept unchanged

### Requirement: Environment Exposure Is Allowlisted

The frontend SHALL only be able to read environment variables whose names start with `MEMOSPOT_`.

#### Scenario: Non-prefixed name requested

- **WHEN** the frontend requests an environment variable without the `MEMOSPOT_` prefix
- **THEN** access is denied and no value is returned
