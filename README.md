# SMAC Rust AI

This repository is a Rust-first SMAC-inspired strategy project with a playable `egui` front end, a large deterministic core crate, bundled JSON game content, and an embedded `glsmac/` reference codebase.

This README reflects the current repository state verified on 2026-05-16 after cleanup, GUI observer work, and the current simulation-stabilization sprint.

## Current State

- Active Rust workspace: `smac_core` + `smac_gui`
- Active front end: `smac_gui` using `eframe/egui`
- Active gameplay authority: `smac_core`
- Reference implementation kept in-tree: `glsmac/`
- Archived experiments and preserved history: `_archived/`, `*.backup`, `*.bak`
- Project-local generated toolchain/bootstrap state may exist after bootstrap: `.rustup-local/`, `.cargo-local/`, `.zig/`, `zig-cc`, and downloaded payloads under `toolchain/`

This is not a minimal source-only repo. It contains the active Rust game, historical experiments, imported reference code, technical notes, and local toolchain state.

## Active Workspace

### `smac_core`

`smac_core` is the main implementation crate. Most deterministic logic lives in `smac_core/src/game_state.rs`.

Implemented systems visible in the active codebase:

- deterministic `GameState` and turn progression
- map generation, tile state, visibility, and exploration
- unit movement, combat, repairs, upgrades, and base founding
- terraforming, yields, and environmental/crisis effects
- base production, queues, research, and governor planning
- diplomacy, tech trading, social engineering, and secret projects
- convoy/logistics routes, interception, repair, rebuild, and escort logic
- save/load snapshots, legacy migration support, and save browser state
- content loading and validation from bundled JSON
- AI economy, diplomacy, tactics, probing, and terraforming behavior
- autoplay/demo execution for long-running simulation passes
- typed display state and presentation helpers consumed by `smac_gui`

Main core files:

- `smac_core/src/game_state.rs`
- `smac_core/src/ai.rs`
- `smac_core/src/content.rs`
- `smac_core/src/save.rs`
- `smac_core/src/model.rs`
- `smac_core/src/presentation.rs`
- `smac_core/src/bin/validate_content.rs`
- `smac_core/src/bin/autoplay_demo.rs`
- `smac_core/src/bin/autoplay_sweep.rs`
- `smac_core/src/bin/pack_assets.rs`

### `smac_gui`

`smac_gui/src/main.rs` is the current playable UI entry point.

It currently provides:

- top status bar
- sidebar tabs and hotkeys
- world map and minimap
- observer controls for `Restart Demo`, `Watch Sim`, `Step Sim`, turns-per-second, and turn cap
- unit/tile selection views
- base management
- research, faction, diplomacy, logistics, workshop, save, and log panels
- event popups for crisis and secret-project events

The GUI remains monolithic in file layout, but the rule-heavy logic has been pushed down into `smac_core`.

The active Rust GUI is functional, not art-complete. It renders the simulation with colored tiles, glyphs, overlays, minimap, convoy lines, and side panels rather than polished sprite/animation presentation.

## Autoplay Demo And Graphics Reality

The repository now has a verified watchable autoplay path:

- default demo profile: `100` turns on a `20x20` map with seed `7`
- headless demo runner: `smac_core/src/bin/autoplay_demo.rs`
- multi-seed diagnostics runner: `smac_core/src/bin/autoplay_sweep.rs`
- live viewer: `smac_gui`, using the observer controls in the top bar

Current verified demo behavior on that profile:

- the sim no longer goes inert at turn 50
- the demo reaches turn 100 without a terminal outcome
- the default seed-`7` profile still leaves Sparta at `2` bases while Gaia reaches `3`

Current known gameplay limitations on that same profile:

- the default seed-`7` demo is now the remaining low-expansion outlier
- unrest is still high enough to produce strained economies even though famine/support-disband failures are no longer firing in the sampled sweep
- the later turns are still more economy/research churn than warfare, captures, or project races

Current verified multi-seed sweep signal on `10` seeds (`1` through `10`) for the same `20x20` / `100`-turn profile:

- `0/10` runs terminate early; the proving harness now reaches turn `100` on all sampled seeds
- famine/support-disband events are currently `0/10`, and starvation events are also `0/10`
- the player side now clears `3` bases in all sampled runs
- the AI side now clears `3` bases in `9/10` runs, with seed `7` as the remaining sampled low-expansion outlier
- the sweep is now exposing a much narrower expansion edge case plus midgame pacing, not broad survival failures

Current transcendence pacing rule:

- `Secrets of Planet` is no longer an instant-win tech by itself
- the tech now unlocks `Empath Guild`
- transcendence requires both `Secrets of Planet` and the `Empath Guild` secret project

About the "original GUI":

- `smac_gui` is the active GUI for the Rust game
- `glsmac/` is a separate C++ reference project with its own renderer, assets, and build system
- `glsmac/` is not wired into the Rust simulation/runtime path
- for this repository today, the Rust GUI is the best live viewer

## Data And Content

Bundled runtime content lives under `data/`.

Files on the active runtime/content-validation path:

- `factions.json`
- `facilities.json`
- `production.json`
- `runtime_rules.json`
- `start_scenario.json`
- `technology_tree.json`
- `ui_theme.json`
- `units.json`
- `localization.json`

Archived non-runtime JSON files now live under `data/_archived/`:

- `buildings.json`
- `societies.json`
- `unit_runtime.json`

Current validated bundled content counts:

- 5 factions
- 15 techs
- 14 units
- 19 facilities
- 38 production items

## Repository Layout

- `Cargo.toml`: root workspace manifest
- `Makefile`: wrapper targets for `check`, `build`, `test`, `validate`, `run-gui`, `fmt`, `clippy`
- `scripts/clean_local_artifacts.sh`: explicit cleanup helper for ignored local toolchain/build/save artifacts; dry-run by default
- `rust-toolchain.toml`: stable toolchain with `rustfmt`, `clippy`, and `rust-analyzer`
- `smac_core/`: active gameplay crate
- `smac_gui/`: active GUI crate
- `data/`: bundled runtime content
- `data/_archived/`: non-runtime JSON files removed from the active content root
- `documentation/`: technical notes and status docs
- `documentation/project_history/`: older planning and assessment documents preserved for context
- `documentation/technical_docs/repository_retention_policy.md`: explicit policy for what stays active, archived, generated, or in-tree as reference material
- `documentation/technical_docs/references/`: large design/reference documents moved out of the repo root
- `glsmac/`: separate C++ reference project
- `_archived/`: retired Bevy prototype and other older subsystems/tooling
- `_archived/README.md`: top-level archive inventory for preserved non-active material
- `_archived/bundles/`: compressed preserved experiment trees that no longer need expanded source directories
- `_archived/root_artifacts/`: obsolete root-level setup scripts, backup manifests, and tree snapshots removed from the active path
- `_archived/source_snapshots/`: preserved one-off source patch artifacts removed from active crate directories
- `_archived/sample_saves/`: preserved manual save examples removed from the active GUI save path
- `assets/` and `assets.pack`: static/packed assets
- `toolchain/`: committed manifest/docs plus ignored downloaded bootstrap payloads such as `rustup-init` and the Zig archive
- `_archived/workspace_backups/`: preserved Rust workspace snapshots moved out of the repository root

Repository notes:

- `glsmac/` is independent and large, with its own build system and assets.
- `glsmac/` remains intentionally in-tree as a nested reference checkout; only its generated build/output directories are treated as local noise.
- `glsmac/` is reference material, not the active GUI/runtime path for `smac_core`
- obsolete top-level setup/bootstrap artifacts were moved to `_archived/root_artifacts/`
- `zig-cc` is still active and is referenced by `.cargo/config.toml`
- bootstrap sources are pinned in `toolchain/manifest.json`; downloaded payloads under `toolchain/` are ignored local artifacts
- local bootstrap/build directories are intentionally ignored via `.gitignore`: `.cargo-local/`, `.rustup-local/`, `.zig/`, `.zig-cache/`
- manual GUI save files are intentionally ignored via `.gitignore`; `smac_gui/saves/.gitkeep` only preserves the directory
- `documentation/api_documentation/` and `documentation/player_guides/` are reserved placeholder directories and now include marker READMEs.
- `_archived/README.md` is now the index for preserved non-active code and historical material.
- several preserved experiment trees under `_archived/` are now bundled as compressed archives instead of expanded source directories.
- `repository_retention_policy.md` is the authoritative policy for active vs archived vs generated repository material.

## Verified Build And Validation State

Using the local toolchain in `.rustup-local/`:

- `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- `cargo run -p smac_core --bin validate_content --quiet`: passed
- `cargo run -p smac_core --bin autoplay_demo --quiet -- --turns 100 --width 20 --height 20 --seed 7 --summary-every 100`: passed
- `cargo run -p smac_core --bin autoplay_sweep --quiet -- --turns 100 --width 20 --height 20 --start-seed 1 --count 10`: passed
- `cargo test --workspace --quiet`: passed

Current verified workspace test count:

- 266 tests passed

Observed validation result:

```text
Content validation passed: 5 factions, 15 techs, 14 units, 19 facilities, 38 production items.
```

Observed autoplay result on the verified demo profile:

```text
Turn 100 | Mission Year 2201 | routes 5 | projects 0
  Gaia's Stepdaughters -> bases 3, units 5, energy 658, known techs 13, research Singularity Physics 30/200
  Spartan Federation -> bases 2, units 9, energy 122, known techs 11, research Orbital Mechanics 83/100
Demo completed 100 turns without a terminal outcome.
```

Observed autoplay sweep aggregate on the verified diagnostic profile:

```text
aggregate | terminal 0 / 10 | bankruptcies 0 | famines 0 | starvation 0 | support 0 | player low-expansion 0 | ai low-expansion 1 | player zero-unit 0 | ai zero-unit 0
```

## Current Focus

The `Visual Transition` (Phase 3) core integration is complete. `smac_bevy` is now the primary presentation layer.

Current focus: `Phase 4: Advanced Strategy & World Mechanics`

1. **Procedural Biomes**: (Complete) Implemented coherent map generation and environmental scaling.
2. **Native Dynamics**: (Complete) Enhanced native life spawning and oceanic threats.
3. **Combat AI**: (Complete) Implemented Battle Groups, coordinated staging, and escort heuristics.
4. **Economic Mastery**: (Complete) Implemented Orbital Economy, population-scaled trade, and symmetric victories.
5. **AI Modernization**: (Complete) Implemented dynamic AI custom unit design and component-aware upgrades.
6. **Strategic Refinement**: (Complete) Implemented diplomatic demands, ultimatums, and tech brokering.
7. **Visual Polish**: (Complete) Implemented persistent map entities, unit facing, and themed UI.
8. **Project Power**: (Complete) Implemented naval invasions, transports, and fleet escorts.
9. **Sound & Narrative**: (Complete) Implemented "Voice of Planet" lore system and integrated `bevy_audio`.
10. **Advanced Warfare**: Transition to implementing Air Superiority and mass destruction mechanics.

## Bottom Line

What is true now:

- the workspace is green and currently verifies `266` passing tests
- the 100-turn seed-`7` demo still completes without a terminal outcome
- the 10-seed proving sweep now reaches turn `100` without premature terminal outcomes
- multi-seed diagnostics now show `0/10` famine/support-collapse outcomes on the verified sweep
- multi-seed diagnostics now show only `1/10` remaining AI low-expansion outcomes on the verified sweep
- `Stockpile Energy` no longer risks a zero-cost production-loop hang in the core economy pass
- transcendence now requires both `Secrets of Planet` and `Empath Guild`
- rich factions can now bridge mineral support shortfalls from energy reserves instead of immediately disbanding units
- support pressure ends in a healthier state on the verified seed than it did before this sprint
- deep systems coverage remains intact across AI, logistics, saves, research, diplomacy, workshop, crises, and tactical heuristics
- **Repeatable Diagnostics**: Autoplay demo and multi-seed sweep binaries for continuous engine verification.

What is still not true:

- the repo is not yet trimmed down to only active code and assets
- preserved historical material is still intentionally present
- archive/reference material still increases repository scope even though its boundary is now explicit
- the Rust GUI is not yet a finished art/sprite presentation
- the current 100-turn demo is not yet balanced or dramatically interesting enough to count as polished gameplay
- the default seed-`7` demo is still a turtle-heavy outlier and the sim still needs stronger midgame conflict
