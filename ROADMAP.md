# SMAC Rust Project Status

Last updated: 2026-05-17

This file is the live status tracker for the active Rust workspace.

## Current Phase

Current phase: `Phase 4: Advanced Strategy And World Mechanics`

The repository is past the initial cleanup and stabilization recovery work. The current emphasis is: keep the simulation baseline strong, fix the remaining Sparta low-expansion outlier, then expand strategic depth through council-aware AI and richer midgame conflict.

## Current Status

- Workspace focus: `smac_core` + `smac_gui` + `smac_bevy`
- Stable watch/debug client: `smac_gui`
- Transition presentation client: `smac_bevy`
- Deterministic gameplay authority: `smac_core`
- Reference implementation: `glsmac/`
- Detailed sprint history and Gemini handoff: [documentation/project_history/SPRINT_LOG_2026-05-17.md](/home/bk/Projects/SMAC_Rust_AI/documentation/project_history/SPRINT_LOG_2026-05-17.md)

## Build Health

Locally reverified in this shell on 2026-05-17:

- `cargo test -p smac_core --quiet`: passed
- `cargo test -p smac_gui --quiet`: passed
- `cargo run -p smac_core --bin validate_content --quiet`: passed

Last reported full-workspace green state:

- `266` passing tests

Current shell caveats:

- `cargo test --workspace --quiet` could not be re-run end-to-end because `smac_bevy` needs dependencies that are not fully cached here and this shell has no working DNS access to `crates.io`
- `cargo test -p smac_bevy --quiet` is also blocked here by missing host tooling for `alsa-sys` (`pkg-config`, and likely ALSA development headers)
- the local `smac_core` test pass surfaced a warning in `save.rs`; that warning has been cleaned up in the current working tree

Still true at the repo level:

- content validation count remains `5 factions, 17 techs, 14 units, 19 facilities, 41 production items`
- the default autoplay demo remains `100` turns on `20x20` with seed `7`
- the verified 10-seed sweep aggregate remains:
  - `terminal 0 / 10`
  - `famines 0`
  - `starvation 0`
  - `support 0`
  - `player low-expansion 0`
  - `ai low-expansion 1`

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
- the remaining measured sim weakness is the Sparta seed-`7` turtle case, not broad support or famine collapse

## Immediate Next Tasks

1. Eliminate the remaining Sparta seed-`7` low-expansion outlier.
2. Preserve the current `0/10` famine, starvation, and support-collapse sweep result while doing it.
3. After Sparta is fixed, implement council-aware AI strategy so the new political layer becomes a live strategic system instead of only persisted mechanics.
4. Harden the `smac_bevy` build path so missing host audio/system packages are documented or feature-gated.

## Recommended Near-Term Order

1. Use `autoplay_sweep` as the baseline diagnostic for gameplay changes instead of tuning against one seed.
2. Fix Sparta’s seed-`7` colony behavior before broadening diplomacy again.
3. Teach AI factions to call and vote in council based on power, relations, and victory posture.
4. Only then resume terrain-transition polish and broader Bevy presentation work.

## Current Milestone Slice

Current slice: `Simulation Hardening Before Deeper Strategy`

- keep the core deterministic and green
- maintain the no-famine/no-support-collapse sweep baseline
- remove the last measured expansion outlier
- then deepen diplomacy and council behavior from a stable base

## Prior History

Older detailed planning and preserved historical status material live under `documentation/project_history/` and `_archived/`.
