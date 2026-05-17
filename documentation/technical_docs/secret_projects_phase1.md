# Secret Projects Phase 1: Global Wonders & Crisis Mitigation

## Overview
This update introduces the foundation for **Secret Projects**—global wonders that provide powerful, unique bonuses and mitigate the severe risks of the planetary crisis system. Only one of each project can exist in the world, making their completion a major strategic milestone.

## 1. The First Secret Projects

### **The Weather Pattern**
- **Cost**: 200 Minerals
- **Role**: Climate Control
- **Mitigation**: Prevents the **Dust Fall** nutrient penalty from affecting the owning faction's bases.
- **Description**: A global wonder that stabilizes planetary conditions, protecting against atmospheric thickening.

### **The Clinical Immortality**
- **Cost**: 300 Minerals
- **Role**: Social Stability
- **Effect**: Provides a global **+25% bonus to Food Security**.
- **Description**: A global wonder that optimizes biological health, ensuring long-term population stability.

### **The Empath Guild**
- **Cost**: 250 Minerals
- **Role**: Governance Integrity
- **Mitigation**: Significantly reduces the risk of **Governance Override** (reduces override chance from 15% to 5% per turn).
- **Description**: A global wonder that strengthens collective oversight, protecting against synthetic drift.

## 2. Global Uniqueness & Building Logic
- **Uniqueness**: Once a project is completed by any faction, it is instantly removed from all other base production queues and build lists globally.
- **Feedback**: A high-priority global log message is emitted upon completion: `GLOBAL WONDER: [Faction Name] has completed [Project Name]!`

## Technical Changes

### `smac_core`
- **`model::SecretProject`**: New enum defining the first three projects.
- **`model::GameState`**: Added `built_secret_projects: Vec<(SecretProject, usize)>` to track global ownership.
- **`ProductionItem`**: Expanded to include Secret Project variants with a new `secret_project()` helper.
- **`game_state.rs`**: 
    - Implemented `complete_secret_project` logic to enforce global uniqueness and queue cleanup.
    - Updated `process_faction_economy` to apply the specific bonuses for Weather Pattern and Clinical Immortality.
    - Updated `process_strategic_crises` to apply the Empath Guild mitigation logic.
- **`content.rs`**: Updated content validation to support the new `project` build kind and ensure data integrity.

### `smac_gui`
- **Factions Tab**: Added a dedicated **Secret Projects** section for each faction, displaying their completed wonders with a unique `✧` symbol.
- **Tooltips**: Updated production tooltips to describe the global effects and uniqueness of Secret Projects.

## Verification Results
- **Logic Verification**: Confirmed that completing a project correctly wipes it from other factions' queues.
- **Effect Verification**: Verified that The Weather Pattern correctly ignores Dust Fall penalties in yield calculations.
- **Persistence**: Save/load system fully updated to track and restore global wonder ownership.
- **Stability**: Passed 100% of the **130+ tests**.
