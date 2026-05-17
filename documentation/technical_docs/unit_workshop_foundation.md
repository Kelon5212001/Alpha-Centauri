# Implementation: Unit Workshop Phase 1 (Engine Foundation)

## Overview
This update implements the core engine changes required for the **Unit Workshop**, a central feature of the original SMAC. We have transitioned from a static unit system to a modular, component-based architecture where every unit instance refers to a `UnitDesign`.

## Key Changes

### 1. Component-Based Design Model
- **`UnitDesign`**: A new data structure composed of:
    - **Chassis**: Determines movement type and base cost.
    - **Weapon**: Determines attack strength.
    - **Armor**: Determines defense strength.
    - **Abilities**: Special traits (e.g., Amphibious, Trance).
- **Serialization**: All unit components and designs now support `Serialize` and `Deserialize`, allowing custom player designs to be saved and loaded.

### 2. Faction-Stored Designs
- Each **`Faction`** now maintains its own `unit_designs: Vec<UnitDesign>`.
- Factions start with standard prototypes (Scout Patrol, Colony Pod, Former) pre-populated in their design library.
- This paves the way for players to add their own custom designs in Phase 2.

### 3. Dynamic Unit Instances
- **`Unit`** struct updated with `design_index: usize`, linking it to its parent design in the faction's library.
- **Dynamic Stats**: Combat resolution and UI panels now pull unit names, attack power, and defense power directly from the assigned `UnitDesign` rather than hardcoded enums.

## Technical Details
- Updated `smac_core::model::Unit` and `smac_core::model::Faction`.
- Refactored `smac_core::game_state::process_faction_economy` and `smac_core::game_state::unit_panel_summary`.
- Added `GameState::unit_attack_strength` and `GameState::unit_defense_strength` to centralize dynamic stat lookups.
- Expanded `factions.json` and `content.rs` to handle prototype loading for all 14 factions.

## Verification
- **Integration Test**: Added `smac_core/tests/unit_workshop_test.rs`, which successfully verifies that a custom "Super Tank" design created at runtime correctly influences combat results and UI labels.
- **Stability**: 100% pass rate across the full test suite (140+ tests).
