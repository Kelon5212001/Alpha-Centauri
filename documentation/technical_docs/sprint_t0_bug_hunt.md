# Sprint T-0 Bug Hunt And Regression Lockdown

Last updated: 2026-06-17

## Scope

Sprint T-0 is a stabilization pass. It should not add broad new systems before the current diplomacy, combat, AI, save/load, content, and UI assumptions are covered by tests, validation, and autoplay sweeps.

## Bugs Fixed In This Pass

- Probe-team facility sabotage no longer removes whichever facility happens to be last in the base facility vector. It now deterministically removes the highest-production-cost sabotage target available on the base.
- Existing Sprint T diplomacy regression tests now also cover AI restraint around Treaty/Pact targets, escalation snapshot persistence, and mutual-defense response logging.
- Damaged-unit and stalled-attack AI decisions now emit typed diagnostics for strategic retreats and avoided hopeless attacks so autoplay sweeps can separate caution from combat failures.
- Content validation now catches production entries that map to runtime items missing from `ProductionItem::all`, and facility definitions must round-trip through their production-item mapping.
- Save/load regression coverage now preserves cascaded war states and typed event-kind counts across snapshot roundtrips, including legacy log migration for new retreat and avoided-attack event kinds.
- AI attack-legality regression coverage now includes base-capture targets, proving Treaty/Pact bases stay protected and Truce base captures require explicit escalation intent.
- Probe-team hostile actions now escalate through diplomacy for tech theft, facility sabotage, and unit subversion; council sessions and expanded Pact visibility downgrades have snapshot/regression coverage.
- Fixed-seed midrun save/load replay now has a deterministic continuation signature check so restored snapshots can be compared against uninterrupted play without relying on wall-clock profiler log text.

## Sprint T-0.1 Through T-0.3 Lockdown Work

- **T-0.1 Typed event classification**: Event log entries now carry a structured kind for wartime combat, first-strike escalation, Treaty violation, Pact betrayal, and defensive response instead of requiring tests to parse only free-form log text.
- **T-0.2 AI escalation intent and diagnostics**: AI offensive targeting can escalate from Truce only when explicit aggression and hostile-attitude gates are met; Treaty and Pact targets remain protected from accidental tactical attacks, and autoplay sweeps now report typed wartime/escalation/betrayal/defensive-response counters.
- **T-0.3 Pact visibility cleanup**: Pact-derived visibility/exploration now has regression coverage proving allied intelligence disappears when the Pact downgrades.

## Sprint T-0.4 Strategic AI Lockdown Follow-up

- **Strategic retreat diagnostics**: Damaged combat units and vulnerable non-combat units that successfully fall back now generate `STRATEGIC RETREAT` typed event logs.
- **Hopeless-attack avoidance diagnostics**: Small under-strength attack groups that stage instead of charging heavily defended targets now generate `AVOIDED ATTACK` typed event logs.
- **Autoplay telemetry**: The autoplay sweep event-kind report now includes strategic-retreat and avoided-attack totals alongside wartime combat and escalation categories.

## Sprint T-0.5 Content Validation Lockdown Follow-up

- **Production/runtime item coverage**: `validate_content` now rejects production entries whose IDs map to runtime items that are omitted from `ProductionItem::all`.
- **Facility production roundtrip**: Runtime facilities must have a production item, and that production item must map back to the same facility so sabotage, build queues, and content validation share the same source of truth.
- **Naval production coverage**: Sea Colony Pod and Sea Transport are now included in the runtime production-item inventory covered by validation and tests.

## Sprint T-0.6 Save/Load Determinism Follow-up

- **Cascaded diplomacy persistence**: Snapshot roundtrips now cover attack-triggered war escalation plus Pact mutual-defense cascades, preserving both relation status and typed event-kind counts.
- **Legacy event-kind migration**: Legacy save migration now has regression coverage for first-strike, defensive-response, strategic-retreat, and avoided-attack log classification.

## Sprint T-0.7 AI Attack-Legality Matrix Follow-up

- **Protected base targets**: Tactical AI now has regression coverage proving Treaty and Pact base targets are not captured or escalated accidentally.
- **Truce escalation gate for bases**: AI base captures from Truce now have coverage for both cautious restraint and high-aggression escalation intent.

## Sprint T-0.8 Five-Task Lockdown Follow-up

- **Probe tech theft escalation**: Successful tech theft now transitions non-war targets through the diplomacy escalation layer.
- **Probe sabotage escalation**: Successful facility sabotage now produces Treaty/Pact/Truce escalation semantics before destroying the target facility.
- **Probe subversion escalation**: Successful unit subversion now escalates diplomacy before ownership changes.
- **Council save/load coverage**: Active council sessions, governor state, meeting turn, and pending votes now roundtrip through snapshots.
- **Pact visibility downgrade matrix**: Pact-derived visibility/exploration now has downgrade coverage for Treaty, Truce, and War.

## Sprint T-0.9 Deterministic Replay Follow-up

- **Midrun replay parity**: Fixed-seed games now have regression coverage that saves after multiple turns, restores from snapshot JSON, advances both branches, and compares deterministic game-state signatures.
- **Profiler-log isolation**: The replay signature intentionally compares state and typed event-kind counts rather than wall-clock profiler messages.

## Features And Refinements To Add Or Harden Next

1. **Diplomacy/combat consistency**
   - Expand the typed event taxonomy beyond the current escalation/retreat categories into a dedicated combat-event payload with actor, defender, target tile, and diplomatic pre-state.
   - Add a dedicated diplomatic memory model for grievances, betrayals, and defensive-war legitimacy.

2. **AI attack legality**
   - Add remaining regression coverage for bombardment, transports/cargo, and naval offensive paths under War/Truce/Treaty/Pact.
   - Extend autoplay diagnostics from aggregate typed counters into per-faction legal-combat, first-strike, retreat, and avoided-attack trend lines.

3. **Pact/shared vision cleanup**
   - Add tests for Pact visibility loss when a Pact downgrades to Treaty/Truce/War.
   - Add UI affordances that explain which tiles are visible through allied intelligence.

4. **Strategic AI stuck states**
   - Extend current retreat/avoided-attack telemetry into stuck-group counters for raid groups that spend multiple turns staging without closing distance.
   - Add regression tests for regrouping and re-targeting after failed attacks, beyond the current retreat and avoided-attack event-kind coverage.

5. **Save/load determinism**
   - Add roundtrip tests for battle-group-relevant unit state beyond the current cascaded-war/event-kind/council coverage.
   - Extend deterministic replay checks from end-turn boundaries into true mid-action/mid-turn save points.

6. **Facility/project/effect coverage**
   - Add coverage ensuring every facility effect has at least one direct behavior test.
   - Add tests for sabotage target priority across all sabotage-eligible facilities.

7. **Unit ability behavior coverage**
   - Add matrix tests for transport, probe, artillery, air-superiority, drop-pod, and native-psi abilities across legal/illegal diplomatic states.
   - Add edge-case tests for cargo ownership and cargo survival during combat or capture.

8. **Terraforming/yield correctness**
   - Add golden tests for each terrain/improvement/facility yield combination.
   - Add tests that compare base-yield previews with actual end-turn production outcomes.

9. **Production/research overflow bugs**
   - Add overflow tests for very high mineral stock, research stock, rush build, and project completion values.
   - Add tests for queue rollover when a just-unlocked item becomes available mid-turn.

10. **GUI/core mismatch**
    - Remove remaining GUI-side `unwrap()` calls on mutable game entities where stale selections can occur.
    - Add presentation-state tests for selected unit/base IDs that disappear after combat, capture, or disbanding.

11. **Content validation gaps**
    - Extend validation to sabotage eligibility classes once some facilities become intentionally immune or partially protected.
    - Validate maintenance/upkeep balance bands for all facilities instead of only requiring non-negative values.

12. **Documentation drift**
    - Keep README, ROADMAP, sprint logs, and validation-count references aligned with the actual `validate_content` output.
    - Add a short release checklist for every future Sprint T stabilization block.
