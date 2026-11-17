# configuration-state Specification

## Purpose

TBD - created by archiving change refactor-configuration-managed-state. Update Purpose after archive.

## Requirements

### Requirement: Configuration SHALL have one application-owned state

Memospot SHALL register one Tauri-managed application state that owns the configuration and the immutable runtime context derived from it. Configuration consumers SHALL read through that state or receive an explicit startup snapshot; they MUST NOT read from a process-global default-initialized configuration store.

#### Scenario: State is initialized before runtime consumers

- **WHEN** startup has loaded and validated the YAML configuration and derived the runtime paths and active server details
- **THEN** the application registers the complete state with the Tauri builder before commands, event handlers, or managed background work can consume it

#### Scenario: Pre-builder startup work needs configuration

- **WHEN** startup performs work before the Tauri builder exists, such as orphan detection or server path preparation
- **THEN** the work receives the required configuration or runtime snapshot explicitly and does not access application-global state

#### Scenario: State access is attempted without initialization

- **WHEN** a configuration consumer has not been given initialized application state
- **THEN** the call fails explicitly or is impossible through the typed interface; it MUST NOT silently receive an empty default configuration

### Requirement: Desired configuration and active runtime context SHALL be distinct

The managed state SHALL distinguish the user-editable desired configuration from the immutable-or-snapshot active runtime context used by the current process. The initial configuration used for change comparison SHALL have a named, private responsibility rather than being represented by a magic `__yaml__` field.

#### Scenario: A restart-required setting changes

- **WHEN** a valid update changes a server/process setting or another startup-only setting
- **THEN** the desired configuration is validated and persisted, the active runtime context remains unchanged for the current process, and the result identifies that a restart is required

#### Scenario: The current server URL is queried

- **WHEN** the frontend requests the Memos URL during the current process
- **THEN** the response comes from the active runtime context, not from a newly edited desired configuration or a stale duplicate value

#### Scenario: Application shutdown compares configuration

- **WHEN** shutdown needs to persist runtime-managed configuration such as window state
- **THEN** it compares the current desired configuration with the named startup baseline without mutating the desired configuration to hide runtime-only overrides

### Requirement: Configuration updates SHALL be atomic and serialized

All configuration mutations, including JSON Patch updates, locale updates, and window-state persistence, SHALL use one serialized read-modify-write path. A mutation SHALL be applied to a validated candidate and SHALL publish the candidate only according to the persistence policy for that mutation.

#### Scenario: Concurrent updates target different fields

- **WHEN** two commands or event handlers update configuration concurrently
- **THEN** both successful updates are retained in the final state and neither update can overwrite the other using a stale clone

#### Scenario: A JSON Patch is invalid

- **WHEN** a patch cannot be parsed, contains an invalid operation, or produces an invalid `Config`
- **THEN** the update is rejected, the in-memory state is unchanged, and the configuration file is not rewritten

#### Scenario: Persistence fails

- **WHEN** a valid update cannot be written to the configuration file
- **THEN** the command returns a failure, the failed update is not reported as successful, and the state transition follows one documented policy consistently without leaving memory and disk falsely represented as synchronized

### Requirement: Configuration command results SHALL describe outcome and restart impact

Configuration commands SHALL return a result that distinguishes a successful persisted update from a rejected update and identifies whether the update requires a restart. Frontend callers SHALL handle failure results as failures rather than displaying a success message for `false` or ignored error values.

#### Scenario: A persisted update succeeds without restart impact

- **WHEN** the update is valid, persisted, and affects only settings that are already live or do not affect the active runtime
- **THEN** the command reports success with `restart_required` false

#### Scenario: A persisted update succeeds with restart impact

- **WHEN** the update changes a restart-required setting
- **THEN** the command reports success with `restart_required` true and the frontend presents the restart requirement through the existing settings flow

#### Scenario: An update fails

- **WHEN** validation or persistence fails
- **THEN** the command rejects the operation and the frontend presents the existing save-failure behavior; it MUST NOT show a save-success notification

### Requirement: Existing live and restart-required behavior SHALL be preserved

The refactor SHALL preserve the current configuration file schema and the current lifecycle contract. Server/process settings and startup-only window settings SHALL remain restart-required; settings currently applied live, notably locale and frontend theme/animation behavior, SHALL continue to apply live. The refactor MUST NOT automatically restart or switch the running Memos server as a side effect of saving settings.

#### Scenario: Locale is changed

- **WHEN** the user changes the locale through the settings UI and the update succeeds
- **THEN** the locale preference is persisted, the backend localization and application menu are updated as they are today, and no restart is required solely for the locale change

#### Scenario: Server configuration is changed

- **WHEN** the user changes remote-server, Memos process, environment, or other startup-only configuration
- **THEN** the desired values are persisted and reported as restart-required while the current server process and active URL continue using their startup snapshot

#### Scenario: Existing configuration is opened after upgrade

- **WHEN** Memospot starts with an existing valid YAML configuration file
- **THEN** the same configuration fields and values are accepted without a file-format migration or implicit reset

### Requirement: Runtime consumers SHALL use the managed state consistently

Commands, menus, event handlers, window-state persistence, server shutdown, and configuration serialization SHALL use the managed application state or an explicitly captured immutable snapshot. Separate Tauri-managed `MemosURL` and `Locale` configuration holders and startup closures that capture an independent configuration copy SHALL not remain as competing sources of truth.

#### Scenario: Window state is persisted during movement

- **WHEN** the main window emits a resize or move event
- **THEN** the window state is merged through the managed configuration update path without losing a concurrent settings update

#### Scenario: Menu visibility is rebuilt

- **WHEN** a menu is built or rebuilt
- **THEN** it reads the desired configuration through the managed state and uses the active runtime context for active-server details

#### Scenario: Server shutdown runs from an event or updater

- **WHEN** shutdown is requested from the Tauri event loop or updater lifecycle
- **THEN** shutdown receives the active runtime snapshot explicitly or through the managed state and does not depend on a process-global configuration lookup

### Requirement: Configuration state SHALL be testable without a running Tauri application

The configuration state SHALL expose a small internal interface that permits unit testing of validation, update serialization, restart classification, baseline comparison, and persistence outcomes without constructing webviews or relying on process-global mutable state.

#### Scenario: State behavior is tested in isolation

- **WHEN** a unit test applies valid, invalid, concurrent, and failing-persistence updates
- **THEN** it can assert the resulting snapshot and outcome deterministically without test-order dependence

#### Scenario: A new application instance is created in a test

- **WHEN** a test constructs a fresh application state
- **THEN** it starts with only the supplied configuration and runtime context and is not affected by a previous test's state
