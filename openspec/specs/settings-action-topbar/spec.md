# settings-action-topbar Specification

## Purpose

Keep the settings search and global actions visible while the user scrolls through the settings content.

## Requirements

### Requirement: Persistent settings header

The settings screen SHALL pin the search input, the window-close control, and the global action buttons (save, reload, defaults) in a header that remains visible while the content scrolls.

#### Scenario: Scrolling content keeps actions visible

- **WHEN** the user scrolls down a long settings page
- **THEN** the header stays at the top of the viewport and the action buttons remain clickable without scrolling back

#### Scenario: Action buttons indicate pending changes

- **WHEN** the user makes changes that require saving
- **THEN** the Save button becomes enabled and visually distinguished while the header stays visible

#### Scenario: No pending changes

- **WHEN** no changes require saving
- **THEN** the Save button is disabled while the header stays visible
