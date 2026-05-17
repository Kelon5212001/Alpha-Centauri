# SMAC Rust Project Status

Last updated: 2026-05-17

This file is the live status tracker for the active Rust workspace.

## Current Phase

Current phase: `Phase 4: Advanced Strategy And World Mechanics`

The repository is past the initial cleanup and stabilization recovery work. The current emphasis is: preserve the now-clean multi-seed simulation baseline from Sprint Q, then broaden strategy through council-aware AI and stronger midgame conflict generation.

## Current Status

- Workspace focus: `smac_core` + `smac_gui` + `smac_bevy`
- Stable watch/debug client: `smac_gui`
- Transition presentation client: `smac_bevy`
- Deterministic gameplay authority: `smac_core`
- Reference implementation: `glsmac/`
- Detailed sprint history and Gemini handoff: [documentation/project_history/SPRINT_LOG_2026-05-17.md](/home/bk/Projects/SMAC_Rust_AI/documentation/project_history/SPRINT_LOG_2026-05-17.md)

## Build Health

Locally reverified in this shell on 2026-05-17:

- `cargo test --workspace --quiet`: passed
- `cargo test -p smac_bevy --quiet`: passed
- `cargo test -p smac_core --quiet`: passed
- `cargo test -p smac_gui --quiet`: passed
- `cargo run -p smac_core --bin validate_content --quiet`: passed

Current workspace test count:

- `281` passing tests

Current Bevy verification notes:

- default workspace verification now passes because `smac_bevy` desktop/audio dependencies are feature-gated off the default test path
- the interactive Bevy desktop binary still needs the `desktop` feature and host windowing/audio packages

Still true at the repo level:

- content validation count remains `5 factions, 17 techs, 14 units, 19 facilities, 41 production items`
- the default autoplay demo remains `100` turns on `20x20` with seed `7`
- the currently verified 10-seed sweep aggregate is:
  - `terminal 0 / 10`
  - `famines 0`
  - `starvation 0`
  - `support 0`
  - `player low-expansion 0`
  - `ai low-expansion 0`

## Completed Sprint Batches

- `Batch A`: Cleanup And Workspace Recovery
- `Batch B`: Diagnostics And Support Telemetry
- `Batch C`: Expansion And Victory-Pacing Pass
- `Batch D`: Native-Pressure And Production-Inertia Pass
- `Batch E`: World Mechanics And Generation
- `Batch F`: Coordinated Combat AI
- `Batch G`: Economic Mastery
- `Batch H`: AI Modernization
- `Batch I`: Strategic Refinement
- `Batch J`: Visual Polish
- `Batch K`: Project Power
- `Batch L`: Sound And Narrative
- `Batch M`: Advanced Warfare
- `Batch N`: Multi-Faction Diplomacy Expansion

For the detailed per-sprint breakdown, use the sprint log instead of this status file.

## Current Verified Snapshot

- Planetary Council foundation now exists:
  - `CouncilState`
  - weighted council voting
  - `CallCouncil`, `VoteForGovernor`, and `VoteForSupremeLeader`
  - council-related game-over outcomes
  - council save/load persistence
  - council reporting in autoplay tools
- advanced-warfare AI groundwork exists for:
  - air-superiority patrolling
  - Planet Buster deployment logic
- the remaining measured sim weakness is no longer broad support collapse or low expansion; the next gap is turning the stable economic baseline into more active diplomacy and conflict

## Immediate Next Tasks

1. Preserve the `0/10` multi-seed baseline across terminal, famine, starvation, support, and low-expansion metrics.
2. Implement council-aware AI strategy so the new political layer becomes a live strategic system instead of only persisted mechanics.
3. Add stronger midgame conflict triggers so the sim produces more raids, border tension, and contested expansion.
4. Keep documenting the `smac_bevy` desktop/audio host-package path while leaving default verification lightweight.

## Recommended Near-Term Order

1. Use `autoplay_sweep` as the baseline diagnostic for gameplay changes instead of tuning against one seed.
2. Hold the current no-famine/no-support/no-low-expansion baseline while expanding strategic behavior.
3. Teach AI factions to call and vote in council based on power, relations, and victory posture.
4. Then add stronger conflict drivers before resuming broader terrain-transition polish and Bevy presentation work.

## Current Milestone Slice

Current slice: `Simulation Baseline Stabilized After Sprint Q`

- keep the core deterministic and green
- preserve the seed-`7` Sparta fix and the clean 10-seed proving baseline
- deepen diplomacy and council behavior from that stable base
- increase strategic pressure so the midgame is more eventful
- then continue terrain-transition and presentation work without destabilizing the sim

## Prior History

Older detailed planning and preserved historical status material live under `documentation/project_history/` and `_archived/`.
