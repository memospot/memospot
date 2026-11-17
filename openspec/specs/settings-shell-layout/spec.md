# settings-shell-layout Specification

## Purpose

Present settings as a two-pane shell with a persistent section list and a scrollable content pane.

## Requirements

### Requirement: Render settings in a split shell layout

The settings screen SHALL render inside a two-pane shell: a persistent section list and the selected section content in a scrollable pane whose width flexes with the viewport.

#### Scenario: Desktop layout renders without clipping

- **WHEN** the user opens the settings screen on a wide desktop viewport
- **THEN** the section list remains at the left, the right pane fills the remaining width, and the right-pane content scrolls independently without overlaying the section list

#### Scenario: Narrow viewport preserves content

- **WHEN** the viewport is narrow (including after zooming in)
- **THEN** the section list stacks above the content pane, the settings header (see settings-action-topbar) remains visible, and the content remains scrollable
