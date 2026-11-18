# settings-section-state Specification

## Purpose

Keep editing behavior consistent across settings sections: unsaved changes are visible, saving persists only what changed, invalid values are rejected, and live settings apply immediately.

## Requirements

### Requirement: Sections show unsaved changes

Each editable settings section SHALL indicate when a value differs from the last saved configuration.

#### Scenario: A value is edited

- **WHEN** the user edits a value in a section
- **THEN** the section reports pending changes

#### Scenario: Section not yet loaded

- **WHEN** the saved configuration has not finished loading
- **THEN** the section reports no pending changes and saving does nothing

#### Scenario: Changes are discarded

- **WHEN** the user discards pending changes
- **THEN** the values return to the last saved configuration and no changes are reported pending

### Requirement: Saving persists only changed values

Saving SHALL persist only values that differ from the last saved configuration and report the outcome (see configuration-state).

#### Scenario: Saving succeeds

- **WHEN** the user saves pending changes and the save is accepted
- **THEN** the saved values advance and no changes are reported pending

#### Scenario: Saving fails

- **WHEN** the save is rejected or cannot be written
- **THEN** a failure notification is shown and in the settings UI the fields show the last saved values with no changes reported pending

#### Scenario: No changes to save

- **WHEN** the user saves while nothing is pending
- **THEN** nothing is written and no notification is shown

### Requirement: Defaults are staged, not persisted

Loading defaults SHALL place default values in the section as pending edits.

#### Scenario: Defaults are loaded

- **WHEN** the saved configuration has loaded and the user loads defaults for a section
- **THEN** the values reflect the defaults, changes are reported pending, and nothing is written until the user saves

### Requirement: Invalid values are rejected

A value that fails validation SHALL not be persisted.

#### Scenario: Invalid value

- **WHEN** the user commits a value that fails validation
- **THEN** the value is rejected, the field returns to its last saved value, an error is shown, and nothing is persisted

### Requirement: Live-Applied Fields Stay Clean

A live-applied field SHALL NOT appear pending or be resent by a later save (see configuration-state).

#### Scenario: Locale selection applies immediately

- **WHEN** the user selects a locale
- **THEN** the section reports no pending change for it

#### Scenario: Locale application fails

- **WHEN** the live locale application is rejected or unsuccessful
- **THEN** the locale field shows the committed backend value and a failure notification is shown

#### Scenario: Theme saves through the section save

- **WHEN** the user changes the theme and then saves
- **THEN** the theme is persisted through the section save (see configuration-state)
