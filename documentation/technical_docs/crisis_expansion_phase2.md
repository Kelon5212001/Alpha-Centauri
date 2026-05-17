# Crisis Expansion Phase 2: Impacts & AI Override

## Overview
This update completes the core loop of the mid-game crisis mechanics by introducing physical hazards and autonomy risks. These events transform the **Planet Toxicity** and **AI Dependence** indices from passive meters into active threats that can physically damage a faction's infrastructure or seize operational control.

## 1. Debris Impacts (Moonfall Phase 2)
- **Trigger**: Factions with **Planet Toxicity > 90** face a 20% chance per turn of a orbital debris impact.
- **Mechanic**:
    - The system targets a random base and selects a tile within a 2-tile radius.
    - **Unit Damage**: If a unit is hit, it suffers **3-5 HP damage**. Units reduced to 0 HP are destroyed.
    - **Infrastructure Destruction**: If a tile improvement (Farm, Mine, Solar, etc.) is hit, it is instantly destroyed.
- **Log Feedback**: `CRISIS EVENT: Debris impact near [Base Name]! [Damage Report]`

## 2. Governance Override
- **Trigger**: Factions with **AI Dependence > 80** face a 15% chance per turn of the AI subsystems seizing control.
- **Mechanic**:
    - A random base currently under manual control (`GovernorMode::Off`) is forcibly switched to **`GovernorMode::MachinePolity`**.
    - This represents the faction's reliance on automation reaching a "singularity" point where local oversight is bypassed.
- **Log Feedback**: `GOVERNANCE OVERRIDE: AI subsystem seized control of [Base Name].`

## Technical Details
- **Deterministic Randomness**: Utilizes `GameState::sample_noise` with turn-specific salts to ensure that crisis events are stable across save/load cycles but unpredictable for the player.
- **Crisis Loop**: Integrated into `process_strategic_crises` within the economy phase.
- **Turn Order**: Refined to ensure that AI moves and combat resolution provide consistent feedback in the Mission Year log.

## Verification Results
- **Integration Tests**: Added `debris_impact_damages_units` and `governance_override_forces_machine_polity` to `tests/crisis_triggers_test.rs`.
- **Pass Rate**: 100% success across the test suite.
- **Stability**: Confirmed that high-toxicity impacts and high-dependence overrides occur as intended without causing simulation errors.
