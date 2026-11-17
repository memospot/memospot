# memos-server-management Specification

## Purpose

Run the managed local Memos server, keep it clean across restarts, gate loading on API readiness, and update the server binary safely.

## Requirements

### Requirement: Spawn Managed Memos Server

When running in standalone mode (no remote server configured), the system SHALL spawn the resolved Memos server binary as a child process (see runtime-configuration).

#### Scenario: App launch

- **WHEN** Memospot is launched
- **THEN** it starts the Memos binary with appropriate environment variables and working directory

### Requirement: Graceful Server Shutdown

The system SHALL stop managed Memos processes during app shutdown.

#### Scenario: App exits

- **WHEN** Memospot exits
- **THEN** it terminates child processes for the managed Memos server
- **AND** checkpoints SQLite WAL state and removes the PID file when possible

### Requirement: Kill Orphaned Processes

The system SHALL ensure no orphaned Memos processes remain running from previous unclean shutdowns.

#### Scenario: Unclean previous shutdown

- **WHEN** Memospot starts and detects an existing PID file for a Memos server
- **THEN** it pings the server and kills the process if it is an orphaned instance

### Requirement: Check API Readiness

The system SHALL probe API readiness during startup and before loader redirect.

#### Scenario: Background readiness probe

- **WHEN** Memospot starts
- **THEN** it polls the Memos version endpoints in the background and stores the reported version when available

#### Scenario: Loader redirect gate

- **WHEN** the loader page is waiting to redirect to the runtime URL
- **THEN** it retries API ping checks against `/healthz`
- **AND** redirects only after a successful ping or shows an error state after retries are exhausted

### Requirement: Default to Latest Version

The updater scripts (`memos-server-updater.sh` and `memos-server-updater.ps1`) SHALL download and install the latest stable release of the Memos server when executed without any arguments.

#### Scenario: Running scripts without arguments

- **WHEN** the user executes either updater script without passing a version argument
- **THEN** the script fetches the latest tag from the GitHub API and proceeds to install that version

### Requirement: Specific Version Installation

Both updater scripts SHALL accept an optional command-line argument to explicitly specify the Memos version to install.

#### Scenario: Running scripts with a target version argument

- **WHEN** the user executes the script and explicitly provides a version string argument (e.g., `v0.25.0`)
- **THEN** the script downloads and installs that exact version rather than the latest version

### Requirement: Server Binary and Database Backups

Both updater scripts SHALL back up the existing server binary and database files before replacement.

#### Scenario: Updating an existing installation

- **WHEN** the script detects an existing Memos server binary or database file before extraction
- **THEN** the script creates timestamped backup archives in `server-updater-backups`
- **AND** database backups include `memos_prod.db` plus `-wal`/`-shm` files when present

### Requirement: Safe Execution for Partially Streamed Scripts

Both updater scripts SHALL NOT perform installation side effects when the script download is incomplete.

#### Scenario: Partially downloaded script

- **WHEN** a copy of either script truncated at a line boundary before its last line is executed directly
- **THEN** no server files are replaced or deleted

### Requirement: Unthrottled Download Progress

The Windows PowerShell script SHALL NOT let progress display throttle download throughput.

#### Scenario: Downloading the Memos binary via PowerShell

- **WHEN** `memos-server-updater.ps1` downloads the binary
- **THEN** the download completes at full network speed regardless of progress display

### Requirement: Installation Guardrails

Both updater scripts SHALL abort without replacing the current binary if a critical step fails.

#### Scenario: Download or extraction failure

- **WHEN** the download of the Memos asset fails, the SHA256 sum is mismatched, or extraction validation fails
- **THEN** the script aborts the update process and does not continue with replacement steps

### Requirement: Verify Download Integrity

Both updater scripts SHALL verify release checksums before installation.

#### Scenario: Release asset downloaded

- **WHEN** the release artifact and SHA256 sums are downloaded
- **THEN** the script compares the expected and computed SHA256 values
- **AND** aborts if they do not match
