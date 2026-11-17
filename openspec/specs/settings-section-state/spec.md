## Requirements

### Requirement: Settings sections SHALL share one state-orchestration module

Each editable settings section SHALL manage its baseline configuration, working input, pending state, defaults loading, and save through one shared frontend module instead of duplicating orchestration per page. A section SHALL supply only a field mapping describing how its fields translate between the application configuration and its input shape (`mapToInput`), how input fields are written back onto the working configuration (`mapFromInput`), optional per-field validation (`validate`), and which fields are owned by a live-application path (`live`).

#### Scenario: A settings section declares its fields

- **WHEN** a settings page creates a section state with a field mapping for its config subtree (for example the `memos` subtree or the `memospot.window` subtree)
- **THEN** the module derives the input shape, pending-change detection, defaults loading, reset, and save behavior from that mapping, and the page does not reimplement any of them

#### Scenario: Section initialization

- **WHEN** a section state is created and initialized against the application configuration returned by the backend
- **THEN** the module stores the configuration as the committed baseline, maps it to the section input, and exposes the input for direct page binding

### Requirement: Pending changes SHALL be derived from the section input

The section SHALL report `hasPendingChanges` from a structural comparison between the current input and the committed baseline input. It SHALL be `false` until initialization completes, and SHALL return to `false` after a successful save, after `reset()`, and after defaults are loaded.

#### Scenario: A field is edited

- **WHEN** the user edits any mapped input field
- **THEN** `hasPendingChanges` becomes `true` while the edit differs from the committed baseline

#### Scenario: Section not yet initialized

- **WHEN** a page has created the section state but initialization has not completed
- **THEN** `hasPendingChanges` is `false` and no save or defaults operation can run before initialization finishes

#### Scenario: Changes are discarded

- **WHEN** `reset()` is invoked
- **THEN** the input is restored from the committed baseline and `hasPendingChanges` is `false`

### Requirement: Section save SHALL persist a JSON Patch with debounce, rollback, and notifications

`save()` SHALL write the mapped input onto the working configuration, compute a JSON Patch (RFC 6902) between the committed baseline and the working configuration, debounce repeated save calls to a single persisted write, and serialize the patch through the backend `set_app_config` command.

- On success: the committed baseline SHALL advance to the working configuration, a success notification SHALL be shown, and a restart notice SHALL be shown when the backend reports `restart_required`.
- On failure: the input and the working configuration SHALL be rolled back to the previously committed baseline, and a failure notification SHALL be shown.
- An empty diff SHALL NOT produce a backend write or any notification.
- Fields declared `live` in the field mapping (committed through a live-application path) SHALL be excluded from the generated patch.
- `save()` SHALL return whether the save succeeded.

#### Scenario: Saving pending changes succeeds

- **WHEN** the user invokes save with pending changes and the backend accepts the patch
- **THEN** a single success notification is shown, the baseline advances, `hasPendingChanges` becomes `false`, and `save()` resolves truthy

#### Scenario: Saving changes that require a restart

- **WHEN** the saved patch changes a server/process or startup-only setting and the backend reports `restart_required`
- **THEN** the success notification is accompanied by a restart notice, and the committed baseline advances

#### Scenario: Saving fails

- **WHEN** the backend rejects the patch or persistence fails
- **THEN** the input and the working configuration are restored to the previously committed baseline, a failure notification is shown, and `save()` resolves falsy

#### Scenario: No changes to save

- **WHEN** save is invoked while `hasPendingChanges` is `false`
- **THEN** no backend write and no notification occur, and `save()` resolves falsy

#### Scenario: Rapid consecutive saves

- **WHEN** save is invoked several times within the debounce interval
- **THEN** the backend receives a single persisted write reflecting the last invocation's state, and every caller receives the outcome of that write

### Requirement: Defaults loading and reset SHALL not persist

`loadDefaults()` SHALL fetch the backend default configuration, map it onto the input and the committed baseline, and SHALL NOT write anything to the configuration file or show save notifications. `reset()` SHALL restore the input from the committed baseline and SHALL NOT persist.

#### Scenario: Defaults are loaded

- **WHEN** the user invokes the defaults action on a section
- **THEN** the section input reflects the backend default configuration, the baseline becomes those defaults, `hasPendingChanges` becomes `false`, and no configuration write occurs

### Requirement: Field validation SHALL gate commits

Validation supplied through the field mapping SHALL gate persistence: a field value that fails validation MUST NOT be written to the configuration. The module SHALL revert the field's input to its committed value and surface the mapping's validation message.

#### Scenario: A path field does not exist

- **WHEN** the user commits an input value for a path field that fails the mapping's validation (for example the Memos data/binary/working-directory paths)
- **THEN** the value is rejected, the field input reverts to the committed value, a validation notification is shown, and nothing is persisted

### Requirement: Live-application settings SHALL keep applying live

Settings that apply without a restart (locale through the live locale command, theme through the theme-mode watcher, reduce-animation preference storage) SHALL keep applying immediately and SHALL NOT be re-persisted by the generic section save. When such a setting is committed live, the section SHALL advance its committed baseline for that field so the setting neither appears pending nor is included in a later save's patch.

#### Scenario: Locale selection applies immediately

- **WHEN** the user selects a locale
- **THEN** the locale is applied through the live locale path (rendering, preference, and page reload when the active locale changes), and the section baseline advances so the locale is not reported pending and is excluded from any subsequent save

#### Scenario: Locale application fails

- **WHEN** the live locale application is rejected or unsuccessful
- **THEN** the locale input and baseline revert to the committed backend value and a failure notification is shown

#### Scenario: Theme selection saves through the generic path

- **WHEN** the user changes the theme and then invokes save
- **THEN** the theme is persisted through the generic section save and the theme-mode watcher is applied as part of the section's save flow, without the module reimplementing theme-mode behavior
