# Implementation: Strategic Indices

## Overview
As part of the mid-game crisis and narrative expansion for `Playable Planetfall v1`, four primary "Strategic Indices" have been integrated into the core game state and UI. These meters provide a high-level view of a faction's long-term stability and strategic posture.

## Strategic Indices Added
1.  **Food Security (-100 to +100)**: Reflects the balance between nutrient production and population requirements.
2.  **AI Dependence (0 to 100)**: Measures the extent of automation and AI governance within the faction.
3.  **Orbital Index (0 to 10)**: Tracks progress in off-world infrastructure and space-based capabilities.
4.  **Planet Toxicity (0 to 100)**: Measures the environmental strain caused by industrial and military activities.

## Changes

### `smac_core`
- **`model::Faction`**: Added `food_security`, `ai_dependence`, `orbital_index`, and `planet_toxicity` fields.
- **`model::Faction`**: Applied `#[serde(default)]` to these fields to ensure backwards compatibility with existing save files.
- **`content::build_runtime_factions`**: Initialized all indices to 0 for new games.
- **`game_state::FactionOverviewDisplayState`**: Added formatted text fields for all four indices.
- **`GameState::faction_overview_display_states`**: Implemented logic to populate these text fields from the faction state.

### `smac_gui`
- **`main.rs`**: Added a new "Strategic Indices" group to the faction overview panel to display the new meters.

## Verification Results
- **Save Compatibility**: Verified that existing saves load correctly by adding `serde(default)` to the new fields. Save roundtrip tests passed.
- **UI Integration**: The new indices are correctly rendered in the Factions panel, providing immediate visibility into these global meters.
- **Tests**: All 17 unit tests and 90 integration tests passed, ensuring no regressions in existing game systems.
