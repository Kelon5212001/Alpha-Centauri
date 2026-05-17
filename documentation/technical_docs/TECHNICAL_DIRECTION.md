# SMAC Rust AI: Technical Direction (Handoff Update)

**Current Version**: Platinum Simulation State (2026-05-17)
**Last Major Milestone**: Phase 4 - Sound, Narrative, and Naval Power

## 1. Project Context
The engine is now "Platinum" stable: 271 tests passing, hardware-accelerated persistent map entities, coordinated naval invasions, and a data-driven narrative system. The AI successfully designs and deploys custom units, manages orbital assets, and handles diplomatic ultimatums.

## 2. Handoff Objectives (Next 3 Rounds)

### Round 4: Advanced Air Superiority & Orbital Warfare
*   **Goal**: Full air-domain dominance and offensive orbital capabilities.
*   **Tasks**:
    *   **Air Patrolling**: Implement "Intercept" and "Scramble" missions for Needlejets to automatically engage nearby hostiles.
    *   **Planet Busters**: Add a new `SecretProject` or high-tier weapon that permanently alters terrain (Nuclear Crater) and resets pollution logic.
    *   **Orbital Insertion**: Enable `DropPod` units to teleport to any explored land tile, driven by a new AI "Shock Group" heuristic.

### Round 5: Multi-Faction Diplomacy & Council
*   **Goal**: Transition from bilateral relations to a global geopolitical stage.
*   **Tasks**:
    *   **Planetary Council**: Implement a periodic voting system where factions vote on global laws (e.g., Trade Embargos, Salvage Rights).
    *   **Coalition AI**: Update `AiBattleGroup` to support "Pact Support"—AI will now move units to defend an ally's base if they have a Pact.
    *   **Espionage Overhaul**: Add "Infiltrate Datalinks" to Probe Teams, allowing the AI to see the map and research progress of target factions.

### Round 6: World Finishing & Modularity
*   **Goal**: Polish the world generation and enable community extension.
*   **Tasks**:
    *   **Terrain Bitmasking**: Implement a rules-based bitmasking system in `smac_bevy` to render smooth transitions between different terrain types.
    *   **Climate Change**: Implement rising sea levels triggered by high global toxicity, turning land tiles into ocean.
    *   **Modding API**: Extract hardcoded unit and tech logic into external `yaml` or `toml` manifests to allow for easy balancing without re-compilation.

## 3. Engineering Mandates
*   **Safety**: Always verify `cargo_unit_ids` persistence when modifying unit movement or destruction.
*   **Determinism**: Maintain zero non-determinism in `smac_core`.
*   **Validation**: Every new `Ability` or `Tech` must be added to `validate_content` in `content.rs`.
