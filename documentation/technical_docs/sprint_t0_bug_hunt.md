# Sprint T-0 Bug Hunt And Regression Lockdown

Last updated: 2026-06-16

## Scope

Sprint T-0 is a stabilization pass. It should not add broad new systems before the current diplomacy, combat, AI, save/load, content, and UI assumptions are covered by tests, validation, and autoplay sweeps.

## Bugs Fixed In This Pass

- Probe-team facility sabotage no longer removes whichever facility happens to be last in the base facility vector. It now deterministically removes the highest-production-cost sabotage target available on the base.
- Existing Sprint T diplomacy regression tests now also cover AI restraint around Treaty/Pact targets, escalation snapshot persistence, and mutual-defense response logging.

## Sprint T-0.1 Through T-0.3 Lockdown Work

- **T-0.1 Typed event classification**: Event log entries now carry a structured kind for wartime combat, first-strike escalation, Treaty violation, Pact betrayal, and defensive response instead of requiring tests to parse only free-form log text.
- **T-0.2 AI escalation intent**: AI offensive targeting can escalate from Truce only when explicit aggression and hostile-attitude gates are met; Treaty and Pact targets remain protected from accidental tactical attacks.
- **T-0.3 Pact visibility cleanup**: Pact-derived visibility/exploration now has regression coverage proving allied intelligence disappears when the Pact downgrades.

## Features And Refinements To Add Or Harden Next

1. **Diplomacy/combat consistency**
   - Add a typed combat-event enum instead of relying on log-string matching for wartime combat, first strikes, betrayals, and defensive responses.
   - Add a dedicated diplomatic memory model for grievances, betrayals, and defensive-war legitimacy.

2. **AI attack legality**
   - Add explicit AI escalation intent before any non-war offensive move.
   - Add autoplay diagnostics that count legal wartime combats separately from first-strike escalations and Pact betrayals.

3. **Pact/shared vision cleanup**
   - Add tests for Pact visibility loss when a Pact downgrades to Treaty/Truce/War.
   - Add UI affordances that explain which tiles are visible through allied intelligence.

4. **Strategic AI stuck states**
   - Add stuck-group telemetry for raid groups that spend multiple turns staging without closing distance.
   - Add regression tests for retreating, regrouping, and re-targeting after failed attacks.

5. **Save/load determinism**
   - Add roundtrip tests for active council sessions, battle-group-relevant unit state, and diplomacy logs after cascaded wars.
   - Add deterministic replay checks for fixed seeds after save/load at mid-turn boundaries.

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
    - Validate that every facility referenced by code has a production definition and every production facility has a runtime facility definition.
    - Validate sabotage eligibility and maintenance/upkeep fields for all facilities.

12. **Documentation drift**
    - Keep README, ROADMAP, sprint logs, and validation-count references aligned with the actual `validate_content` output.
    - Add a short release checklist for every future Sprint T stabilization block.
