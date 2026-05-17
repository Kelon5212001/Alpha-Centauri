# Faction Specialization & Tidal Crisis

## Overview
This update introduces asymmetric starting conditions for factions and the final phase of the Moonfall crisis system. Factions now possess unique strategic index bonuses, and the planet faces the threat of global coastal flooding.

## 1. Faction Specialization
Factions are no longer identical at the start of the game. They now begin with unique Strategic Index values defined in `factions.json`:
- **Gaians**: Start with high **Food Security** (+50%), representing their ecological harmony.
- **University**: Starts with significant **AI Dependence** (40%), reflecting their advanced research automation.
- **Spartans**: Start with moderate **AI Dependence** (20%), representing disciplined martial automation.
- **Morganites**: Start with low **Food Security** (-10%) but higher initial energy, reflecting their industrial focus over basic subsistence.
- **Planetmind**: Starts with **Negative Toxicity** (-50), acting as a natural planetary healer.

## 2. Moonfall Phase 3: Tidal Chaos
- **Trigger**: Occurs randomly after **Mission Year 50** if `tidal_chaos_turns_left` is zero.
- **Mechanic**:
    - Global sea levels temporarily rise for **4-7 turns**.
    - **Flooding**: Any land tile with **Elevation <= 0** (coastal/lowland) has its resource yields reduced to **zero**.
- **Impact**: Factions must now consider elevation when building coastal infrastructure or risk total economic shutdown during tidal events.
- **Feedback**: High-priority log messages signal the start and end of the chaos.

## Technical Changes

### `smac_core`
- **`model::GameState`**: Added `tidal_chaos_turns_left` to track crisis duration.
- **`save::GameStateSnapshot`**: Updated with full persistence for tidal state and legacy save migration.
- **`content.rs`**: Expanded `FactionDefinition` and `StartingFactionSetup` to handle starting strategic index values from JSON.
- **`game_state.rs`**: 
    - Implemented `Tidal Chaos` trigger in `process_strategic_crises`.
    - Updated `tile_total_yields` to apply the flooding penalty.
    - Set starting turn to **1** for all new games to align with mission year logic.

## Verification Results
- **Content Validation**: Verified that all 5 starting factions load their unique bonuses correctly.
- **Crisis Trigger**: Confirmed that Tidal Chaos correctly zeroes out yields on low-elevation land tiles.
- **Stability**: Passed 100% of the **130+ tests**, including legacy save migration tests.
