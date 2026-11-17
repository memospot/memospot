# settings-deep-search-index Specification

## Purpose

Make each setting discoverable by label and keywords without a manual index update.

## Requirements

### Requirement: Rendered settings become searchable

Every rendered settings row SHALL be findable through search by its label, and by its keywords when supplied, without a manual index update.

#### Scenario: New setting is searchable

- **WHEN** a section renders a settings row with a label and keywords
- **THEN** the row is findable through search using those terms

#### Scenario: Returning to a visited section

- **WHEN** the user returns to a previously visited section
- **THEN** search reflects that section's current settings

### Requirement: Search covers visited sections

Search results SHALL include only settings from sections visited in the session.

#### Scenario: Unvisited section

- **WHEN** a section has never been visited during a session
- **THEN** its settings are not searchable until the section is visited, after which they become findable immediately
