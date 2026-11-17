## ADDED Requirements

### Requirement: Render settings in split shell layout

The settings screen SHALL render inside a two-pane shell where a persistent sidebar of sections sits to the left and the currently selected section content renders in a scrollable right-hand pane whose width flexes with the viewport.

#### Scenario: Desktop layout renders without clipping

- **WHEN** the user opens the settings screen on a wide desktop viewport
- **THEN** the sidebar remains pinned at the left, the right pane fills the remaining width, and the right-pane content scrolls independently without overlaying the sidebar

#### Scenario: Responsive collapse preserves content

- **WHEN** the viewport shrinks below 900px width or the height is short
- **THEN** the sidebar collapses into a narrower column or drawer (depending on space) while the right pane continues to render the settings items and allows scrolling without hiding the top header
