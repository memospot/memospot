# settings-deep-search-index Specification

## Purpose

Make each setting discoverable by label and keywords without a manual index update.

## Requirements

### Requirement: Rendered settings become searchable

Every rendered settings row SHALL be findable through search by its label, and by its keywords when supplied, without a manual index update.

#### Scenario: New setting is searchable

- **WHEN** a section renders a settings row with a label and keywords
- **THEN** the row is findable through search using those terms

#### Scenario: Section content stays current

- **WHEN** the user returns to a section whose rendered settings changed
- **THEN** search reflects that section's current settings

### Requirement: Search covers all sections

Search results SHALL include settings from every section immediately after the settings screen loads, without visiting each section first.

#### Scenario: Unvisited section on load

- **WHEN** the user opens settings and searches for a setting in a section that was never visited during the session
- **THEN** the setting appears in the results immediately

#### Scenario: Switching sections keeps all results

- **WHEN** the user switches sections and searches again
- **THEN** settings from the other sections remain searchable
