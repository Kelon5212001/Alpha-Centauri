# SMAC Rust Project Status

Last updated: 2026-05-17

This file is the live status tracker for the active Rust workspace.

## Current Phase

Current phase: `Phase 4: Advanced Strategy And World Mechanics`

The repository is past the initial cleanup, stabilization recovery, and Sprint S midgame conflict-pressure work. The current emphasis is Sprint T: Strategic Reversals And Alliances, with diplomacy/combat consistency fixed before any additional visual or unrelated feature expansion.

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

- `283` passing tests

Current Bevy verification notes:

- default workspace verification now passes because `smac_bevy` desktop/audio dependencies are feature-gated off the default test path
- the interactive Bevy desktop binary still needs the `desktop` feature and host windowing/audio packages

Still true at the repo level:

- content validation count remains `5 factions, 17 techs, 16 units, 20 facilities, 45 production items`
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
- council-aware AI strategy now exists:
  - coalition-aware council-call logic
  - relation-weighted governor candidate selection
  - weighted vote logging in council sessions
- transcendence now requires:
  - `Secrets of Planet`
  - `Empath Guild`
  - active Planetary Governorship
- advanced-warfare AI groundwork exists for:
  - air-superiority patrolling
  - Planet Buster deployment logic
- the remaining measured sim weakness is no longer broad support collapse, low expansion, or inert council mechanics; the next gap is turning the stable economic baseline into more active conflict

## Immediate Next Tasks

1. Treat `Sprint S: Midgame Conflict Pressure` as complete and keep its active-conflict gains covered by autoplay sweeps.
2. Implement `Sprint T: Strategic Reversals And Alliances`, starting with war declaration on attack and legal categorization of combat events as wartime combat, escalation, betrayal, or defensive response.
3. Make tactical AI diplomacy-aware so raid, threat, and attack selection respects current relations or deliberately escalates through the diplomacy layer.
4. Strengthen Pact mechanics with shared vision/intelligence, mutual defense, ally-defense behavior, and council-voting affinity.
5. Add strategic retreat and regrouping behavior for damaged, elite, or outmatched units.
6. Add late-game economy support only after diplomacy correctness is stable.
7. Keep `smac_bevy` secondary: default verification stays lightweight/headless, gameplay authority remains in deterministic `smac_core`, and `smac_gui` remains the stable playable/debug view layer.

## Recommended Near-Term Order

1. Use `autoplay_sweep` as the baseline diagnostic for gameplay changes instead of tuning against one seed.
2. Preserve the no-famine/no-support/no-low-expansion baseline while validating that new combat events are legally categorized.
3. Fix diplomacy/combat consistency first: war on attack, first-strike escalation logs, Pact betrayal penalties, and mutual-defense responses.
4. Then improve Pact value, ally-defense behavior, and strategic retreats.
5. Only after diplomacy correctness is stable, add late-game economy support and revisit broader terrain-transition or Bevy presentation work.

Sprint T-0 bug-hunt findings and the stabilization backlog live in
`documentation/technical_docs/sprint_t0_bug_hunt.md`.

## Current Milestone Slice

Current slice: `Sprint T: Strategic Reversals And Alliances`

- keep the core deterministic and green
- preserve the seed-`7` Sparta fix, clean sweep baselines, council-governorship gate, and Sprint S conflict activity
- make all hostile attacks transition through diplomacy when the factions are not already at war
- distinguish wartime combat, first-strike escalation, Pact betrayal, mutual defense, and defensive response events in logs
- make tactical AI diplomacy-aware before adding unrelated systems
- keep Bevy presentation work secondary until diplomacy/combat consistency is stable

## Prior History

Older detailed planning and preserved historical status material live under `documentation/project_history/` and `_archived/`.
