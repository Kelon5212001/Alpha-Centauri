# Sprint Log And Gemini Handoff

Last updated: 2026-05-17

This document is the detailed sprint-by-sprint summary for the current Rust workspace. It is intended to be the handoff file for future Codex or Gemini sessions.

## Verification Snapshot

Locally reverified in this shell on 2026-05-17:

- `cargo test -p smac_core --quiet`: passed
- `cargo test -p smac_gui --quiet`: passed
- `cargo run -p smac_core --bin validate_content --quiet`: passed

Verification caveats in this shell:

- `cargo test --workspace --quiet` could not be re-run end-to-end because `smac_bevy` needs crates that are not fully cached and this shell currently has no working DNS access to `crates.io`.
- `cargo test -p smac_bevy --quiet` is additionally blocked here by missing system tooling for `alsa-sys` (`pkg-config`, and likely ALSA development headers).
- The last reported full-workspace green state from the active repo docs and recent checkpoint flow is `266` passing tests.

## Sprint A: Cleanup And Workspace Recovery

Scope:

- restored the broken Rust workspace to a buildable/testable state
- archived stale root artifacts, dead JSON files, source snapshots, sample saves, and historical experiments
- added explicit bootstrap, cleanup, verification, and retention-policy docs/scripts

Key outcomes:

- active runtime data path was separated from archived content
- repository root was reduced to live crates, docs, assets, and intentional reference material
- verification scripts and cleanup scripts were added so repo state stopped depending on tribal knowledge

Next-sprint handoff that followed:

- stop treating cleanup as the primary milestone
- move immediately into simulation diagnostics and measurable stability work

## Sprint B: Diagnostics And Support Telemetry

Scope:

- added `autoplay_demo.rs` and `autoplay_sweep.rs`
- added support/upkeep telemetry so sweeps could distinguish famine, starvation, and support-disband collapse
- fixed the zero-cost `Stockpile Energy` loop hazard

Key outcomes:

- long-run behavior became measurable instead of anecdotal
- regressions could now be tracked across seeds instead of one manually observed run

Next-sprint handoff that followed:

- attack the most common mechanical failure first: support collapse and early-game economy instability

## Sprint C: Expansion And Victory-Pacing Pass

Scope:

- relaxed colony-pod behavior so mild support pressure could be solved by expansion instead of permanent turtling
- added a single-base hoard breaker
- slowed autoplay AI pursuit of `Secrets of Planet`
- changed transcendence pacing so `Secrets of Planet` alone no longer ends the proving harness

Key outcomes:

- support-related stagnation was reduced
- autoplay runs stopped getting distorted by premature transcendence outcomes

Next-sprint handoff that followed:

- fix the remaining two-base trap and production inertia around half-built infrastructure

## Sprint D: Native-Pressure And Production-Inertia Pass

Scope:

- allowed trapped two-base factions to interrupt half-built infrastructure for `ColonyPod`
- relaxed the stalled two-base colony thresholds
- split native psi pressure from faction frontline pressure
- preserved early psi-defense behavior while leaving the two-base escape hatch active

Key outcomes:

- AI low-expansion in the verified sweep dropped from `5/10` to `1/10`
- famine, starvation, support collapse, and terminal losses stayed at `0/10`
- the remaining sampled expansion failure narrowed down to the default seed `7` Sparta turtle case

Next-sprint handoff that followed:

- focus on the single remaining sampled Sparta outlier instead of broad economy triage

## Sprint E: World Mechanics And Generation

Scope:

- added biome-coherent procedural generation
- improved native-life spawning and toxicity scaling
- added oceanic native threats
- fixed tests that were too tightly coupled to earlier procedural assumptions

Key outcomes:

- maps became less uniform and more strategically distinct
- environmental systems started to matter beyond cosmetic terrain variety

Next-sprint handoff that followed:

- make the AI behave better on these richer maps instead of assuming isolated units act well alone

## Sprint F: Coordinated Combat AI

Scope:

- added `AiBattleGroup` and `AiObjective`
- implemented staging logic for attack groups
- added assembly-point behavior for idle combat units
- added escort behavior for colony pods and formers
- strengthened threatened-base defense logic

Key outcomes:

- combat units gained group-level intent
- frontier movement became less random and more concentration-driven

Next-sprint handoff that followed:

- improve strategic economy and victory pacing so the AI has something meaningful to fight over

## Sprint G: Economic Mastery

Scope:

- added orbital economy systems
- scaled commerce with connected-base population
- improved AI satellite launch and orbital defense priorities
- added symmetric victory handling and rebalanced late-game tech pacing

Key outcomes:

- economy systems expanded beyond local tile yields
- late-game progression became less one-sided and less hardcoded to a single faction path

Next-sprint handoff that followed:

- modernize AI unit design so strategic systems are backed by up-to-date military pieces

## Sprint H: AI Modernization

Scope:

- tracked weapon and armor component unlocks more explicitly
- implemented dynamic AI custom unit design
- added stricter modernization loops so units upgrade to legal current designs
- integrated plasma/resonance tiers into AI design decisions

Key outcomes:

- the AI stopped relying on outdated scout-era designs forever
- equipment tech started translating into fielded military changes

Next-sprint handoff that followed:

- expand diplomacy and threat evaluation so factions use their power politically instead of only tactically

## Sprint I: Strategic Refinement

Scope:

- implemented diplomatic demands
- added fear-based demand acceptance
- improved tech-brokering logic
- surfaced ultimatums in turn-summary alerts
- reverified save migration and 100-turn sim stability after diplomacy data changes

Key outcomes:

- diplomacy became more coercive and less binary
- strategic disparity started affecting negotiation outcomes

Next-sprint handoff that followed:

- improve presentation quality enough that longer simulations are easier to watch and debug

## Sprint J: Visual Polish

Scope:

- moved Bevy map rendering to persistent entities
- added unit facing
- expanded terrain/improvement visuals
- applied a stronger sci-fi UI theme
- reduced flicker and improved synchronization

Key outcomes:

- the Bevy client became more readable and less placeholder-like
- map state changes became visually more stable

Next-sprint handoff that followed:

- add bigger strategic set pieces that justify the improved presentation layer

## Sprint K: Project Power

Scope:

- implemented transports and cargo handling
- added load/unload actions and recursive cargo destruction rules
- enabled AI naval transport design
- added coordinated naval invasions and naval escorts

Key outcomes:

- water stopped being a hard stop for military projection
- invasion logic started to operate across map layers instead of only on contiguous land

Next-sprint handoff that followed:

- strengthen narrative and endgame feedback so the broader strategic layer feels more legible

## Sprint L: Sound And Narrative

Scope:

- added the "Voice of Planet" narrative event system
- integrated `bevy_audio`
- added mission-conclusion overlays
- verified cargo persistence through the newer client/runtime paths

Key outcomes:

- faction/system milestones gained more thematic framing
- the Bevy client gained the first pass of audio infrastructure

Next-sprint handoff that followed:

- push into high-impact strategic warfare systems and asymmetrical late-game pressure

## Sprint M: Advanced Warfare

Scope:

- implemented AI logic for air-superiority patrolling
- implemented AI logic for Planet Buster deployment

Key outcomes:

- the AI can reason about higher-tier offensive projection
- late-game threat posture is no longer limited to ground/naval movements alone

Commit checkpoint:

- `40e673d` `checkpoint: implemented AI logic for Air Superiority patrolling and Planet Buster deployment`

Next-sprint handoff that followed:

- build out multi-faction diplomacy so late-game pressure is not only military

## Sprint N: Multi-Faction Diplomacy Expansion

Scope:

- added `CouncilState` and `CouncilVote`
- activated the Planetary Council once `Empath Guild` is completed
- implemented `CallCouncil`, `VoteForGovernor`, and `VoteForSupremeLeader`
- added population-based vote weighting, with double weight for the `Empath Guild` owner
- added majority-based council resolution
- added diplomatic council-related game-over outcomes
- integrated council outcomes into autoplay diagnostics
- persisted council state through saves and migrations

Key outcomes:

- the repo now has a real planetary-governance foundation instead of only bilateral diplomacy
- save/load and diagnostics understand the new political layer

Commit checkpoint:

- `2ec723c` `checkpoint: implemented multi-faction diplomacy foundation and planetary council mechanics`

Next-sprint handoff that should happen now:

- stop broadening diplomacy for one sprint
- return to the remaining Sparta expansion outlier so the simulation baseline is strong before council AI policy gets more complex

## Immediate Gemini Handoff

Use this exact sprint order next.

### Sprint O: Fix Sparta Seed-7 Turtling

Goal:

- eliminate the remaining sampled `2`-base Sparta outcome on the default `20x20`, `100`-turn, seed `7` demo

Where to work:

- `smac_core/src/ai.rs`
- `smac_core/src/game_state.rs`
- `smac_core/src/bin/autoplay_demo.rs`
- `smac_core/src/bin/autoplay_sweep.rs`

What to inspect first:

- whether Sparta builds colony pods but fails to settle
- whether garrison/escort priorities crowd out expansion in seed `7`
- whether unrest recovery over-prioritizes static infrastructure once Sparta reaches `2` bases
- whether colony targeting is too conservative around local psi/native pressure

Acceptance criteria:

- default seed `7` reaches `3` Sparta bases by turn `100`
- verified 10-seed sweep stays at `0/10` terminal, famine, starvation, and support failures
- verified 10-seed sweep reaches `0/10` AI low-expansion if possible, or at minimum does not regress from `1/10`

### Sprint P: Council-Aware AI Strategy

Goal:

- make AI factions proactively use the new Planetary Council instead of merely supporting the mechanics

Where to work:

- `smac_core/src/ai.rs`
- `smac_core/src/game_state.rs`
- `smac_core/src/model.rs`
- council-related autoplay output in `smac_core/src/bin/autoplay_demo.rs` and `autoplay_sweep.rs`

What to add:

- AI logic for when to call council
- AI vote targeting based on relations, power, and victory posture
- strategic value for `Empath Guild` tied to council leverage

Acceptance criteria:

- autoplay logs show factions making non-random council choices
- new council decisions do not break save/load or 100-turn sim stability

### Sprint Q: Bevy Build Hardening

Goal:

- make `smac_bevy` easier to build in constrained environments

Where to work:

- `smac_bevy/Cargo.toml`
- any audio/bootstrap/build scripts that assume host ALSA tooling
- docs in `README.md` and `ROADMAP.md`

What to investigate:

- feature-gating audio so tests/builds do not hard-require ALSA
- documenting exact Linux build packages (`pkg-config`, ALSA dev headers)
- ensuring workspace verification commands fail clearly instead of opaquely

Acceptance criteria:

- `smac_bevy` build instructions are explicit
- if audio remains optional, headless/client verification paths should stop failing on missing ALSA tooling

### Sprint R: Terrain Transitions And Visual Readability

Goal:

- improve terrain transitions and border readability after the build-hardening pass

Where to work:

- `smac_bevy`
- theme/tileset asset pipeline
- map presentation docs

Acceptance criteria:

- terrain edges read more cleanly at gameplay zoom levels
- visuals improve readability without coupling new rules into presentation code

