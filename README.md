# SMAC Rust AI

This repository is a Rust-first SMAC-inspired strategy project with a playable `egui` front end, a large deterministic core crate, bundled JSON game content, and an embedded `glsmac/` reference codebase.

This README reflects the current repository state as of 2026-05-17 after the simulation-stabilization work, the Bevy client transition, advanced-warfare AI, the planetary-council diplomacy foundation, Sprint O stability hardening, and Sprint P sweep recovery.

## Current State

- Active Rust workspace: `smac_core` + `smac_gui` + `smac_bevy`
- Stable front end: `smac_gui` using `eframe/egui`
- Transition front end: `smac_bevy`
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

### `smac_bevy`

`smac_bevy` is the newer presentation client. It is the active visual-transition path for the project and now carries newer rendering, UI polish, audio hooks, and terrain-presentation work while still depending on `smac_core` for deterministic gameplay rules.

Current role:

- newer presentation path
- map rendering and visual polish
- themed UI work
- audio/narrative presentation hooks
- future home for terrain-transition work
- default verification path now uses a feature-gated headless configuration
- the desktop Bevy binary is behind the `desktop` feature, and audio remains optional behind `audio`

## Autoplay Demo And Graphics Reality

The repository now has a verified watchable autoplay path:

- default demo profile: `100` turns on a `20x20` map with seed `7`
- headless demo runner: `smac_core/src/bin/autoplay_demo.rs`
- multi-seed diagnostics runner: `smac_core/src/bin/autoplay_sweep.rs`
- live viewer: `smac_gui`, using the observer controls in the top bar

Current verified demo behavior on that profile:

- the sim no longer goes inert at turn 50
- the demo reaches turn 100 without a terminal outcome
- Sprint O fixed the seed-`7` Sparta collapse, and the current verified turn-100 result is `Gaia 3 bases / 14 units` and `Sparta 3 bases / 13 units`

Current known gameplay limitations on that same profile:

- the support-collapse regression has been removed again, but the AI side still finishes below `3` bases too often outside seed `7`
- the remaining sampled low-expansion seeds are now narrower colony/settlement-quality failures, not broad economic collapse
- the later turns are still more economy/research churn than warfare, captures, or project races

Current verified multi-seed sweep signal on `10` seeds (`1` through `10`) for the same `20x20` / `100`-turn profile:

- `0/10` runs terminate early; the proving harness now reaches turn `100` on all sampled seeds
- famine events are currently `0/10`, support-disband events are `0/10`, and starvation events are `0/10`
- the player side now clears `3` bases in all sampled runs
- the AI side now clears `3` bases in `7/10` runs
- the current sweep is exposing a much narrower remaining AI expansion problem, not terminal-collapse or support-economy instability

Current transcendence pacing rule:

- `Secrets of Planet` is no longer an instant-win tech by itself
- the tech now unlocks `Empath Guild`
- transcendence requires both `Secrets of Planet` and the `Empath Guild` secret project

About the current viewers:

- `smac_gui` remains the most reliable watch/debug client in constrained environments
- `smac_bevy` is the newer primary visual-transition client, but the desktop/audio path still has heavier system dependencies
- `glsmac/` is a separate C++ reference project with its own renderer, assets, and build system
- `glsmac/` is not wired into the Rust simulation/runtime path
- `glsmac/` is reference-only for this repository today

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
- 17 techs
- 14 units
- 19 facilities
- 41 production items

## Repository Layout

- `Cargo.toml`: root workspace manifest
- `Makefile`: wrapper targets for `check`, `build`, `test`, `validate`, `run-gui`, `fmt`, `clippy`
- `scripts/clean_local_artifacts.sh`: explicit cleanup helper for ignored local toolchain/build/save artifacts; dry-run by default
- `rust-toolchain.toml`: stable toolchain with `rustfmt`, `clippy`, and `rust-analyzer`
- `smac_core/`: active gameplay crate
- `smac_gui/`: active GUI crate
- `smac_bevy/`: active Bevy-based client/presentation path
- `data/`: bundled runtime content
- `data/_archived/`: non-runtime JSON files removed from the active content root
- `documentation/`: technical notes and status docs
- `documentation/project_history/`: older planning and assessment documents preserved for context
- `documentation/project_history/SPRINT_LOG_2026-05-17.md`: detailed sprint-by-sprint log and Gemini handoff
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
- locally reverified on 2026-05-17:
  - `cargo test --workspace --quiet`: passed
  - `cargo test -p smac_bevy --quiet`: passed
  - `cargo test -p smac_core --quiet`: passed
  - `cargo test -p smac_gui --quiet`: passed
  - `cargo run -p smac_core --bin validate_content --quiet`: passed

Current verified workspace test count:

- 276 tests passed

Current Bevy build notes:

- default `cargo test --workspace` no longer hard-requires desktop/audio system libraries
- the interactive Bevy desktop client now requires `cargo run -p smac_bevy --features desktop`
- desktop audio requires `cargo run -p smac_bevy --features "desktop audio"`
- Debian/Ubuntu desktop builds will typically need `apt install pkg-config libasound2-dev libwayland-dev libxkbcommon-dev libx11-dev libxcursor-dev libxi-dev libxrandr-dev libxinerama-dev libudev-dev`

Observed validation result:

```text
Content validation passed: 5 factions, 17 techs, 14 units, 19 facilities, 41 production items.
```

Observed autoplay result on the verified demo profile:

```text
Turn 100 | Mission Year 2201 | routes 4 | projects 0
  Gaia's Stepdaughters -> bases 3, units 14, energy 1, known techs 10, research Gene Splicing 33/50
  Spartan Federation -> bases 3, units 13, energy 8, known techs 9, research Secrets of the Human Brain 33/65
Demo completed 100 turns without a terminal outcome.
```

Observed autoplay sweep aggregate on the verified diagnostic profile:

```text
aggregate | terminal 0 / 10 | bankruptcies 0 | famines 0 | starvation 0 | support 0 | player low-expansion 0 | ai low-expansion 3 | player zero-unit 0 | ai zero-unit 0
```

## Current Focus

The visual-transition work is established, and the repo is now in `Phase 4: Advanced Strategy And World Mechanics`.

Immediate next sprint:

1. Preserve the new seed-`7` Sparta survival/expansion fix.
2. Preserve the restored `0/10` famine/support-collapse sweep baseline across the `10`-seed proving pass.
3. Reduce AI low-expansion from the current `3/10` by attacking the remaining colony/settlement outlier seeds (`2`, `7`, and `9` in the latest sampled sweep).
4. Only after the remaining expansion outliers are reduced further, move back to council-aware AI strategy and stronger midgame conflict generation.

Detailed sprint history and the Gemini handoff live in:

- [SPRINT_LOG_2026-05-17.md](/home/bk/Projects/SMAC_Rust_AI/documentation/project_history/SPRINT_LOG_2026-05-17.md)

## Bottom Line

What is true now:

- the workspace is green and currently verifies `276` passing tests
- this shell locally reverified the full workspace, including the default `smac_bevy` test path, on 2026-05-17
- the 100-turn seed-`7` demo still completes without a terminal outcome
- Sprint O fixed the seed-`7` Sparta collapse and Sprint P kept that fix intact
- the 10-seed proving sweep now reaches turn `100` without premature terminal outcomes
- multi-seed diagnostics now show `0/10` famine/support-collapse outcomes on the verified sweep
- multi-seed diagnostics now show `3/10` remaining AI low-expansion outcomes on the verified sweep
- `Stockpile Energy` no longer risks a zero-cost production-loop hang in the core economy pass
- transcendence now requires both `Secrets of Planet` and `Empath Guild`
- rich factions can now bridge mineral support shortfalls from energy reserves instead of immediately disbanding units
- support pressure ends in a healthier state on the verified seed than it did before this sprint
- deep systems coverage remains intact across AI, logistics, saves, research, diplomacy, workshop, crises, and tactical heuristics
- the Planetary Council foundation is now in place, including council state, weighted votes, save persistence, and autoplay/reporting integration
- **Repeatable Diagnostics**: Autoplay demo and multi-seed sweep binaries for continuous engine verification.

What is still not true:

- the repo is not yet trimmed down to only active code and assets
- preserved historical material is still intentionally present
- archive/reference material still increases repository scope even though its boundary is now explicit
- the Rust GUI is not yet a finished art/sprite presentation
- the Bevy client desktop path is not yet locally turnkey in minimal shells without system windowing/audio packages
- the current 100-turn demo is not yet balanced or dramatically interesting enough to count as polished gameplay
- the remaining sweep weakness is now concentrated in a smaller set of AI low-expansion seeds that still need better colony/settlement behavior
