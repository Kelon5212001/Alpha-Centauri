# Implementation: Crisis Triggers

## Overview
As a direct extension of the "Strategic Indices" system, the game now features **Crisis Triggers**. These are gameplay penalties and narrative events that occur when a faction's global health meters (Food Security, Planet Toxicity, AI Dependence) cross critical thresholds. This system bridges the gap between resource management and the narrative "Moonfall" crisis.

## Crisis Mechanisms

### 1. Famine Unrest
- **Trigger**: Faction-wide `Food Security` drops below `-20`.
- **Effect**: All bases in the faction experience additional unrest.
- **Penalty Scale**: `+1` unrest for every 20 points below the `-20` threshold (e.g., `-60` food security results in `+2` unrest per base).
- **Impact**: This creates a negative feedback loop where famine leads to unrest, which further reduces mineral and energy output.

### 2. Narrative Alerts (Logging)
The system monitors strategic indices during the economy phase and emits high-priority warnings into the event log:
- **Global Famine**: Triggered when `Food Security < -50`. Warns of widespread starvation and instability.
- **Environmental Alert**: Triggered when `Planet Toxicity` is between `50` and `80`. Warns of high planetary strain.
- **Toxicity Crisis**: Triggered when `Planet Toxicity > 80`. Signals severe strain and warns of possible physical hazards (acid rain, debris).
- **Governance Warning**: Triggered when `AI Dependence > 70`. Warns of excessive reliance on synthetic governance.

## Changes

### `smac_core`
- **`game_state.rs`**: 
    - Updated `base_unrest` to factor in faction-wide food security penalties.
    - Implemented `process_strategic_crises` to monitor indices and emit log messages.
    - Integrated `process_strategic_crises` into the `process_faction_economy` loop.
- **`tests/crisis_triggers_test.rs`**: Added new integration tests to verify:
    - Famine correctly increases base unrest.
    - Strategic indices correctly trigger log messages during turn transitions.

## Verification Results
- **Functional Verification**: Confirmed that negative food security values (e.g., `-60`) correctly add `+2` unrest to bases.
- **Narrative Verification**: Confirmed that log messages appear in the `game.log` when thresholds are crossed.
- **Stability**: All 130+ tests passed.
