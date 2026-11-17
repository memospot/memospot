## ADDED Requirements

### Requirement: Fixed top action bar with search

The settings screen SHALL pin a top action bar that contains the global action buttons (save, reload, defaults) next to the sidebar search input so the actions remain visible while scrolling through the content area.

#### Scenario: Scrolling content keeps buttons visible

- **WHEN** the user scrolls down a long settings page
- **THEN** the top action bar stays fixed at the top of the viewport and the action buttons remain clickable without needing to scroll back to the bottom

#### Scenario: Action buttons indicate dirty state

- **WHEN** the user makes changes that require saving
- **THEN** the Save button becomes enabled (and optionally shows a badge) while the action bar stays visible, guiding the user toward persistence before leaving the page
