# settings-result-navigation Specification

## Purpose

Open the setting selected from search and make it easy to find.

## Requirements

### Requirement: Navigate to matching control with highlight

The search experience SHALL switch to the matching section, scroll the target setting into view, focus its control, and apply a short-lived highlight when the user selects a result.

#### Scenario: Selecting a result

- **WHEN** the user hits Enter on a highlighted dropdown entry
- **THEN** the sidebar activates the result's section, the content pane scrolls to the matching row, focus moves to the row's control, and a temporary highlight animation runs on the row before fading

#### Scenario: Rapid selection changes

- **WHEN** the user moves through results quickly
- **THEN** the previously highlighted row returns to normal before the new highlight starts and no stale highlight remains

### Requirement: Dropdown respects keyboard shortcuts

The dropdown SHALL honor ArrowDown/ArrowUp to change the selection, Enter to confirm, and Escape to close the dropdown.

#### Scenario: Keyboard navigation

- **WHEN** the dropdown is open and the user presses ArrowDown
- **THEN** the next result becomes highlighted without closing the dropdown

#### Scenario: Closing dropdown with Escape

- **WHEN** the user presses Escape while the search input or a search result has focus
- **THEN** the dropdown closes, focus moves to the search input, and no navigation occurs
