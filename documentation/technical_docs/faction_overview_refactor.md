# Refactor: Faction Overview Logic Moved to Core

## Overview
As part of the `Playable Planetfall v1` milestone, significant logic for computing faction status summaries, base roles, and production postures has been moved from `smac_gui` to `smac_core`. This aligns with the project's architectural goal of keeping the GUI layer "thin" and deterministic rules centralized in the core.

## Changes

### `smac_core`
- **`BaseSortMode`**: Moved from GUI to core. Added `base_sort_mode_label` helper.
- **`FactionOverviewDisplayState`**: New struct containing pre-computed strings and data for the faction overview panel.
- **`GameState::faction_overview_display_states`**: New method that computes the entire display state for all active factions, including:
    - Base and unit counts.
    - Upkeep breakdown.
    - Research progress.
    - Alert summaries (unrest, recovery, frontier).
    - Base area role distributions.
    - Logistics and convoy capacity status.
    - Governor mode mix.
    - Production and queue posture/role summaries.
    - Governor intent and queue gap summaries.
    - Tech-blocked production summaries.

### `smac_gui`
- Removed local `BaseSortMode` and `base_sort_mode_label`.
- Refactored `draw_faction_status` to use the new `faction_overview_display_states` from core.
- Removed several hundred lines of logic from the GUI update loop.
- Cleaned up unused methods (`production_items_for_owner`, `filtered_player_base_ids`, etc.) that were superseded by core functionality.

## Verification Results
- `cargo check -p smac_gui` passed.
- `cargo test -p smac_core` passed (17 core tests, 90 integration tests).
- All existing game systems (Logistics, Governor, Research) remain functional as verified by integration tests.
