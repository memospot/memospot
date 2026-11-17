# settings-deep-search-index Specification

## Purpose

TBD

## Requirements

### Requirement: Build a dynamic settings index

The search subsystem SHALL construct its index from metadata emitted by each settings control so adding/removing settings does not require manual index updates.

#### Scenario: Indexing a new control

- **WHEN** a settings row renders with `data-setting-id`, `data-setting-label`, and optional `data-setting-aliases`
- **THEN** the shared index collector registers an entry with the row's label, aliases, owning section ID, and selector reference

#### Scenario: Removing a control

- **WHEN** a settings row unmounts because the user navigated away or the control is removed
- **THEN** the index automatically drops the entry so stale matches do not appear in future searches

### Requirement: Keep index scoped to visible sections (configurable)

The system SHALL allow building the index lazily (per section) to avoid indexing dozens of hidden rows while still supporting instant search once the section renders.

#### Scenario: Lazy indexing

- **WHEN** a section is never visited during a session
- **THEN** its rows are not added to the index until the section renders, but once visited the entries become available immediately
