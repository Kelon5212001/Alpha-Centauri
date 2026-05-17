# Gameplay & Crisis Expansion: Combat, Moonfall, and Machine Polity

## Overview
This update introduces a trio of major features that deepen the strategic simulation and enhance player feedback. These changes bridge the gap between early-game expansion and mid-game industrial/environmental challenges.

## 1. Combat Visualization & Feedback
- **Enhanced Combat Logs**: Combat encounters now report the exact **Attack** and **Defense** scores calculated during the resolution.
- **Example**: `COMBAT: Spartan Scout Patrol (atk: 5) destroyed Gaians Speeder (def: 3).`
- **Impact**: Players can now understand why they won or lost a fight, making the influence of terrain, experience, and HP much more transparent.

## 2. Moonfall Phase 1: Atmospheric Dust Fall
- **Trigger**: High **Planet Toxicity** (> 80) increases the chance of a global Dust Fall event.
- **Effect**: During a Dust Fall, **global nutrient yields are halved**.
- **Duration**: Events last between 3 and 6 turns.
- **Narrative**: The system emits logs when the event begins and ends, signaling the transition between normal conditions and crisis management.
- **Impact**: High industrial output now carries the direct risk of triggering global famines, forcing a balance between rapid expansion and environmental stability.

## 3. Governor Expansion: Machine Polity
- **New Governor Mode**: `Machine Polity`.
- **Logic**: This advanced mode prioritizes pure industrial output (`Mineral Refinery`) and logistical standardization (`Transit Hub`) above social or morale concerns.
- **AI Dependence Bonus**: Scaling with the `AI Dependence` meter, factions now receive a **global yield bonus** (up to +25% at 100% dependence).
- **Faster Accumulation**: Using the `Machine Polity` mode increases a faction's AI Dependence three times faster than standard governor modes.
- **Impact**: Offers a high-reward, high-risk "Synthetic Governance" path for players who want to maximize economic throughput at the cost of surrendering autonomy.

## Technical Refinements
- **Turn Order Stabilization**: Refined the `end_turn` sequence to ensure that economies, AI moves, and interdictions resolve in a stable, predictable order. This fixed several edge-case test failures.
- **State Persistence**: Added `dust_fall_turns_left` to the game state and save snapshots with full backwards compatibility.
- **AI Module Exposure**: Refactored `smac_core::ai` to expose individual turn phases (Strategy, Economy, Tactics) for more granular control.

## Verification Results
- **Tests**: 100% pass rate across 130+ integration and unit tests.
- **Stability**: Verified that save files from previous versions load correctly and migrate to the new state format.
- **Build**: `smac_gui` and `smac_core` compile cleanly without warnings.
