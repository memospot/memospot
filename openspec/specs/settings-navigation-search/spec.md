# settings-navigation-search Specification

## Purpose

Find individual settings across sections from the settings header search.

## Requirements

### Requirement: Fuzzy search results grouped by section

The search SHALL perform approximate matching over all settings, grouping matches by section while ranking results by relevance to the query (see settings-deep-search-index).

#### Scenario: Abbreviated query

- **WHEN** the user types a partial term (e.g., "bind")
- **THEN** the dropdown surfaces matches from multiple sections ranked by label similarity and shows section headers above each group

#### Scenario: No matches

- **WHEN** the query yields no matches
- **THEN** the dropdown displays a "No results" message while the section list remains available

### Requirement: Dropdown is anchored to the fixed search input

The dropdown SHALL appear below the fixed header's search input, follow it across screen sizes, and stay visible while the header is pinned (see settings-action-topbar).

#### Scenario: Sticky header scrolling

- **WHEN** the user scrolls the settings content
- **THEN** the dropdown stays attached to the header and the results panel remains within the viewport bounds
