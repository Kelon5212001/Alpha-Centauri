# SMAC Rust Implementation Plan

Last updated: 2026-05-06

This file replaces the earlier rewrite assessment with an implementation plan tied to the code that exists now.

## Current Direction

- Keep `smac_gui` as the active playable front end for near-term feature delivery.
- Treat the root Bevy prototype in `src/` as an architectural candidate, not the active game.
- Keep `smac_core` as the source of deterministic game truth.
- Prioritize feature-complete gameplay slices before engine migration work.

## Why This Direction

- `smac_gui` is already playable and test-backed.
- `smac_core` already holds most of the deterministic logic worth preserving.
- A front-end migration before the next playable milestone would slow feature delivery.
- The repo still needs clearer gameplay scope more than it needs a rendering rewrite.

## Feature-First Priority Order

1. Complete the first real gameplay milestone in the existing workspace.
2. Move more gameplay logic from `smac_gui` into `smac_core`.
3. Improve production and research feedback for the existing playable front end.
4. Stabilize content-author workflows and validation in normal development.
5. Only then decide whether Bevy replaces `smac_gui` or remains an experiment.

## First Gameplay Milestone

Milestone name: `Playable Planetfall v1`

Goal:

- A player can start a game, found a base, move units, end turns, save/load, build core production, research tech, and play against one functioning AI opponent through the current GUI.

Definition of done:

- New game flow reaches a stable playable map without manual setup.
- Player can found at least one base and expand normally.
- Base production, research, and turn progression work without known blockers.
- One AI faction expands, produces, and pressures the player credibly.
- Save/load is stable for the milestone feature set.
- Content validation passes.
- Workspace build and test remain green.

Non-goals for this milestone:

- Full diplomacy
- Full SMAC facility/unit catalog
- Final rendering stack decision
- Adaptive/learned AI
- Audio/polish-heavy presentation work
- Multiplayer

## Milestone Features By Priority

### P0: Must Ship For `Playable Planetfall v1`

- Stable new-game start using bundled factions and scenario data
- Colony founding and early expansion loop
- Unit movement and turn-ending loop
- Core base production loop
- Core research loop
- Basic combat resolution already in the current slice
- One active AI opponent with expansion and pressure behavior
- Stable save/load for the supported slice
- Content validation integrated into development workflow

### P1: Strongly Preferred In The Same Milestone

- Better production/research feedback in `smac_gui`
- Clear player warnings for unrest, support pressure, and convoy stress
- Scenario and content errors surfaced cleanly through fallible APIs instead of runtime panics
- More gameplay-rule ownership moved from `smac_gui` into `smac_core`

### P2: Defer Until After `Playable Planetfall v1`

- Full Bevy migration
- Diplomacy layer
- Council/commerce systems
- Advanced native ecology
- Menu polish and splash/main-menu rework
- Adaptive AI experiments

## Crate-By-Crate Plan

### `smac_core`

Priority: highest

Owns:

- deterministic rules
- runtime content access
- turn progression
- combat, production, research, logistics, saves
- AI-facing state and helper APIs

Next work:

1. Continue moving feature logic out of `smac_gui`.
2. Keep adding fallible accessors where tools and validation benefit.
3. Preserve strict wrappers only where gameplay code should assume validated content.
4. Expand core-owned automation, planning, and advisory APIs as the GUI gets thinner.

### `smac_gui`

Priority: high

Owns:

- current playable UI
- player interaction layer
- presentation of saves, selection, alerts, and planning tools

Next work:

1. Keep it as the delivery vehicle for the first gameplay milestone.
2. Remove any remaining gameplay-rule ownership that belongs in `smac_core`.
3. Improve production and research clarity before adding broader presentation polish.

### Root `src/` Bevy prototype

Priority: low until milestone completion

Status:

- architectural experiment

Next work:

1. Do not expand this in parallel with milestone gameplay work.
2. Reassess after `Playable Planetfall v1`.
3. Either promote it into a real renderer path or archive it decisively.

### `glsmac/`

Priority: reference only

Use for:

- data/reference behavior
- comparison during rules/content expansion

Do not use as:

- runtime dependency
- source of active project architecture

### `networking/`, `rendering_engine/`, `testing_framework/`

Priority: undecided / inactive

Current rule:

- Treat them as inactive until explicitly folded into the milestone or archived.

### `ai_development_framework/`

Priority: tooling only

Current rule:

- Keep separate from gameplay AI decisions.
- Do not let this crate family drive runtime architecture.

## Front-End Decision Rule

Do not switch the active front end before the first gameplay milestone unless one of these becomes true:

- `smac_gui` blocks milestone features in a way that is materially slower than migration
- a real Bevy front end exists with equivalent playability
- the team explicitly chooses engine migration over near-term playable progress

Until then:

- `smac_gui` is the active front end
- Bevy is a deferred architecture decision

## Immediate Next Implementation Queue

1. Improve production and research feedback in `smac_gui` for the current playable loop.
2. Audit `smac_gui` and move the next batch of rule ownership down into `smac_core`.
3. Add a CI-friendly content validation/build/test workflow.
4. Mark inactive crates/directories as deferred or archive candidates.
5. Reassess the Bevy path only after `Playable Planetfall v1` scope is stable.

## Active Slice

Current milestone slice: `Production And Research Control`

Deliverables:

- clearer production unlock state and queue intent in the GUI
- clearer research choice and post-unlock feedback
- more automation and planning behavior owned by `smac_core`
- no regression in save/load, validation, or AI turn flow
