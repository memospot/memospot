# settings-navigation-search Specification

## Purpose

The settings UI SHALL expose a search input at the top of the left sidebar that filters the visible section entries and quickly scrolls to matching controls when a query is submitted.

## Requirements

### Requirement: Sidebar search filters settings

The settings UI SHALL expose a search input at the top of the left sidebar that filters the visible section entries and quickly scrolls to matching controls when a query is submitted.

#### Scenario: Filtering sections by name

- **WHEN** the user types "Memos" into the sidebar search box
- **THEN** only the sections whose display name matches "Memos" remain visible in the sidebar and the first matching section scrolls into view when the user presses Enter or selects a suggested result

#### Scenario: Clearing the search restores all sections

- **WHEN** the search input is cleared
- **THEN** the sidebar immediately shows all sections again with their default ordering and focus returns to the previously selected item

### Requirement: Provide fuzzy search results grouped by section

The search dropdown SHALL perform fuzzy matching (e.g., Fuse.js) over the dynamically generated index, grouping matches by section while ranking results by relevance to the query.

#### Scenario: Abbreviated query

- **WHEN** the user types a partial term (e.g., "bind")
- **THEN** the dropdown surfaces matches from multiple sections ranked by label similarity and shows section headers above each group

#### Scenario: No matches

- **WHEN** the query yields no matches
- **THEN** the dropdown displays a "No results" message while still allowing the user to fall back to section filtering

### Requirement: Dropdown is anchored to the fixed search input

The dropdown SHALL appear below the fixed header's search input, follow it across screen sizes, and stay visible while the header is pinned.

#### Scenario: Sticky header scrolling

- **WHEN** the user scrolls the settings content
- **THEN** the dropdown stays attached to the header and the results panel remains within the viewport bounds
