# Activation: Strategic Indices Computation

## Overview
The four "Strategic Indices" (Food Security, AI Dependence, Orbital Index, and Planet Toxicity) are now dynamically computed during each faction's economy phase. This turns these fields from static data points into live indicators of a faction's health, stability, and development path.

## Computation Logic

### 1. Food Security (-100 to +100)
- **Formula**: `((total_nutrients_produced / total_population) * 100) - 100`
- **Meaning**: 
    - `0`: Faction is exactly self-sufficient (1 nutrient produced per citizen).
    - `100`: Faction has a 2x nutrient surplus.
    - `-100`: Faction is in total famine.
- **Implementation**: Calculated in `process_faction_economy` by summing nutrients and population across all bases.

### 2. AI Dependence (0 to 100)
- **Accumulation**: Increases each turn based on the proportion of bases using AI governors.
- **Formula**: `(ai_governed_bases / total_bases) * 5` points per turn.
- **Meaning**: Reflects the cumulative reliance on automated governance. High values will eventually trigger risks or unlock unique "Machine Polity" efficiencies.

### 3. Planet Toxicity (0 to 100)
- **Accumulation**: Increases with industrial output; recovers slowly over time.
- **Formula**: `+ (total_minerals_produced / 20)` per turn, `- 1` (natural decay).
- **Meaning**: Replaces the legacy "Ecological Damage" index with a more readable global meter. High toxicity will trigger environmental crises.

### 4. Orbital Index (0 to 10)
- **Formula**: Based on the count of active space-capable facilities (e.g., `Transit Hubs`).
- **Meaning**: Tracks the faction's physical footprint in orbit and deep space.

## Changes

### `smac_core`
- **`game_state.rs`**: Modified `process_faction_economy` to accumulate the necessary metrics and apply the formulas above at the end of the economy phase.
- **`tests/strategic_indices_test.rs`**: Added a new integration test to verify that indices update correctly when bases are founded, governors are enabled, and facilities are built.

## Verification Results
- **Dynamic Updates**: Confirmed via tests that `AI Dependence` increments only when governors are active.
- **Facility Integration**: Verified that building a `Transit Hub` immediately updates the `Orbital Index`.
- **Food Metrics**: Verified that starting bases correctly report `Food Security` based on local yields and population.
- **Stability**: All 130+ tests in the suite passed after these changes.
