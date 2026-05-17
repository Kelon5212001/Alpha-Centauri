# SMAC Rust Project Status

Last updated: 2026-05-16

This file is the live status tracker for the active Rust workspace.

## Current Phase

Current phase: `Simulation Stabilization`

The repo is past cleanup and back into gameplay work. The active sprint is now about proving that long autoplay runs are stable, non-terminal, and directionally believable before adding new feature scope.

## Recent Completed Batches

### Batch A: Cleanup And Workspace Recovery
- [x] Fixed the broken workspace and restored a green test suite.
- [x] Archived stale root artifacts, non-runtime JSON, sample saves, and older experiment trees.
- [x] Added explicit bootstrap, cleanup, verification, and retention-policy docs/scripts.

### Batch B: Diagnostics And Support Telemetry
- [x] Added `autoplay_demo.rs` and `autoplay_sweep.rs` as repeatable simulation diagnostics.
- [x] Added support/upkeep telemetry so sweeps distinguish starvation from support-disband collapse.
- [x] Fixed the `Stockpile Energy` zero-cost production-loop hazard.

### Batch C: Expansion And Victory-Pacing Pass
- [x] Relaxed colony-pod behavior so mild support pressure can be solved by expansion instead of permanent turtling.
- [x] Added a single-base hoard breaker so autoplay factions stop sitting on one base with stockpiled energy/tech.
- [x] Slowed autoplay AI pursuit of `Secrets of Planet` while factions are still underexpanded or unstable.
- [x] Reworked transcendence pacing: `Secrets of Planet` now unlocks `Empath Guild`, and transcendence requires both the tech and that project.

### Batch D: Native-Pressure And Production-Inertia Pass
- [x] Let trapped two-base factions interrupt half-built infrastructure for a colony pod instead of freezing in-place.
- [x] Relaxed the stalled two-base colony thresholds so expansion can restart before the AI hits extreme hoarding states.
- [x] Split native psi pressure from faction frontline pressure so nearby mindworms no longer masquerade as a hostile army.
- [x] Preserved early psi-defense behavior by keeping generic expansion behind serious psi threats while leaving the two-base escape hatch active.

## Current Status

- Workspace focus: `smac_core` + `smac_gui` + `smac_bevy`
- Current playable front end: `smac_gui` (`eframe/egui`)
- New active client: `smac_bevy` (in visual transition phase)
- Deterministic gameplay authority: `smac_core`
- Reference implementation: `glsmac/`
- Toolchain approach: local Rust toolchain in `.rustup-local/` with project-local `zig-cc` linker and a manifest-driven bootstrap path under `toolchain/`

## Recent Completed Batches

### Batch E: World Mechanics & Generation (Completed)
- [x] Implemented biome-coherent map generation via low/high frequency noise blending.
- [x] Enhanced native life spawning: multi-spawn capability, toxicity scaling, and oceanic `Isle of the Deep` rise.
- [x] Fixed map-dependent unit/policy tests to be invariant to procedural generation changes.

### Batch F: Coordinated Combat AI (Completed)
- [x] Implemented `AiBattleGroup` and `AiObjective` for multi-unit coordination.
- [x] Added "Staging" logic: Attack groups wait for reinforcements before engaging strong targets.
- [x] Added "Assembly Point" heuristic: Unassigned combat units gather at frontier bases.
- [x] Added "Support/Escort" logic: Combat units automatically escort Formers and Colony Pods.
- [x] Enhanced Base Defense: AI now prioritizes GarrisonGuard if a base is under threat and lacks a defender.

## Next Tasks


- `cargo check -p smac_gui`: passing
- `cargo build`: passing for the workspace
- `cargo test --workspace`: passing
- Current verified workspace test count: 266 passed, 0 failed
- Current workspace test run: warning-free
- Content validation CLI: passing via `cargo run -p smac_core --bin validate_content`
- Current content validation result: 5 factions, 15 techs, 14 units, 19 facilities, 38 production items
- Verified autoplay demo profile: `cargo run -p smac_core --bin autoplay_demo --quiet -- --turns 100 --width 20 --height 20 --seed 7 --summary-every 100`
- Verified multi-seed diagnostics path: `cargo run -p smac_core --bin autoplay_sweep --quiet -- --turns 100 --width 20 --height 20 --start-seed 1 --count 10`
- A formal local bootstrap/verification path now exists via `scripts/bootstrap_local_toolchain.sh`, `scripts/verify_workspace.sh`, `make bootstrap`, and `make verify`
- A formal local cleanup helper now exists via `scripts/clean_local_artifacts.sh` and `make clean-local`
- CI now uses the same bootstrap/verify path via `.github/workflows/verify-workspace.yml`
- The bootstrap path now supports local Zig extraction through either `xz` or Python's standard-library `tarfile`/`lzma` support
- Committed toolchain payload volume has been reduced: `toolchain/` now keeps manifest/docs, while downloaded payloads are ignored local artifacts
- Root build wrapper: `Makefile` exists for `check`, `build`, `test`, `validate`, `run-gui`, `fmt`, and `clippy`
- Local bootstrap/build directories are now explicitly ignored via `.gitignore`: `.cargo-local/`, `.rustup-local/`, `.zig/`, `.zig-cache/`
- Manual GUI save files are now explicitly ignored via `.gitignore`; `smac_gui/saves/` stays present via `.gitkeep`
- Environment caveat: `make` was not available in the audited shell image, so verification used direct cargo invocations

## What Exists Now

- Deterministic core gameplay crate in `smac_core`
- Playable GUI in `smac_gui`
- Live 100-turn observer mode in the Rust GUI
- Headless single-seed autoplay demo and multi-seed autoplay sweep binaries
- Save/load system with migration coverage
- Production queues, governor modes, convoy logistics, recovery advice, and AI pressure heuristics
- Bundled content for factions, units, facilities, runtime rules, UI theme, start scenario, and technology tree
- Cross-file bundled content validator in `smac_core`
- Validation CLI target: `smac_core/src/bin/validate_content.rs`
- A broad core-owned display/presentation layer that keeps rule logic out of `smac_gui`
- An archived Bevy prototype and older subsystems under `_archived/`

## What Is Still True

- `smac_gui` is the only active playable Rust front end
- `glsmac/` is reference code, not a runtime dependency of the Rust workspace
- the content model is still a partial SMAC-style ruleset rather than a full game-complete implementation
- `ai_development_framework/` is tooling scaffolding, not gameplay AI
- the repo still contains archived code that needs long-term policy cleanup
- active crate paths no longer contain ad hoc source snapshot files or checked-in manual save artifacts
- `_archived/README.md` now serves as the top-level archive index for preserved non-active material
- selected preserved experiment trees under `_archived/` can now be kept as compressed bundles instead of expanded directories
- `glsmac/` now remains intentionally in-tree as a nested reference checkout, with generated build/output paths treated as local noise

## Current Gameplay Phase

`Operation Clean Slate` is complete.

Current focus: `Simulation Stabilization`

Goals:

- keep the workspace green
- keep code, tests, and status docs aligned
- preserve the current `0/10` support-collapse result in the verified sweep
- reduce the remaining AI-side low-expansion outlier in the autoplay harness
- turn the 100-turn demo from "stable" into "actually interesting to watch"

## Verified Sprint Snapshot

- Verified content validation: `5 factions, 15 techs, 14 units, 19 facilities, 38 production items`
- Verified default 100-turn demo result:
  `Gaia's Stepdaughters -> bases 3, units 5, energy 658, known techs 13`
  `Spartan Federation -> bases 2, units 9, energy 122, known techs 11`
- Verified 10-seed sweep aggregate:
  `terminal 0 / 10`
  `famines 0`
  `starvation 0`
  `support 0`
  `player low-expansion 0`
  `ai low-expansion 1`

## Content API Policy

- Strict scalar/runtime wrappers in `smac_core` remain acceptable for gameplay code after bundled content has been validated.
- Fallible `try_*` and `load_*` APIs are the preferred surface for tools, tests, diagnostics, content validation, and any caller that needs recoverable errors.
- New shared-content access should default to adding a fallible API first, then optionally a strict convenience wrapper if gameplay code benefits from it.

### Batch G: Economic Mastery (Completed)
- [x] Implemented Orbital Economy: Sky Hydroponics, Solar Transmitters, and Orbital Defense.
- [x] Scaled Trade Commerce based on connected base populations.
- [x] Optimized AI satellite launch prioritization and defense.
- [x] Implemented Symmetric Victory conditions for all factions.
- [x] Rebalanced end-game Tech tree (Transcendence requirements).

### Batch H: AI Modernization (Completed)
- [x] Enhanced Tech Tree to track Weapon and Armor component unlocks.
- [x] Implemented dynamic AI Custom Design engine: AI now creates optimal unit designs for Infantry, Speeder, and Hovertank chassis upon discovering new component technologies.
- [x] Optimized Modernization Loop: AI now strictly upgrades units to legally available custom designs, preventing "eternal scout" stagnation.
- [x] Integrated Plasma and Resonance tier components into the AI's tactical arsenal.
- [x] Verified AI custom design generation with regression tests.

### Batch I: Strategic Refinement (Completed)
- [x] Implemented Diplomatic Demands: Aggressive factions can extort energy or tech from weaker neighbors.
- [x] Fear-based Acceptance: AI will surrender to demands if significantly outmatched (population/power disparity).
- [x] Tech Brokering Logic: AI now sorts and evaluates tech trades based on relative research cost and strategic value.
- [x] Ultimatum Detection: Integrated pending demands into the 'Turn Summary' alert system.
- [x] Re-verified save migration and 100-turn simulation stability with new diplomatic fields.

### Batch J: Visual Polish (Completed)
- [x] Implemented Persistent Map Entities: Transitioned from despawn/respawn cycles to long-lived ECS entities for units and bases.
- [x] Added Unit Facing: Unit sprites now rotate to face their last direction of travel.
- [x] Expanded Tileset: Added procedural sprites for Forest, Fungus, Borehole, Solar, Mine, and Farm.
- [x] Applied Sci-Fi UI Theme: Refined `bevy_egui` visuals with high-contrast dark backgrounds and glowing cyan/orange highlights.
- [x] Fixed flickering and improved map synchronization performance.

### Batch K: Project Power (Completed)
- [x] Implemented Naval Transports: Added `Ability::Transport` and `cargo_unit_ids` support.
- [x] Added `LoadUnit` and `UnloadUnit` actions with recursive cargo destruction.
- [x] AI Custom Design: AI now designs naval transports upon discovering `Doctrine: Initiative`.
- [x] Coordinated Naval Invasions: AI land units board transports to cross oceans and strike enemy bases.
- [x] Naval Escorts: Sea combat units proactively protect high-value transports on mission.

### Batch L: Sound & Narrative (Completed)
- [x] Implemented "Voice of Planet" narrative system: Data-driven lore events triggered by toxicity, tech, or turn count.
- [x] Integrated `bevy_audio`: Infrastructure for ambient soundscapes and tactical feedback established in `smac_bevy`.
- [x] Added Mission Conclusion Overlay: Specialized endgame screens for all symmetric victory types.
- [x] Unified Cargo Persistence: Verified and fixed unit persistence for transported cargo across all engine tests.

## Next Tasks

1. **Advanced Warfare**: Implement Air Superiority (Needlejet patrolling) and Planet Buster mechanics.
2. **Multi-faction Diplomacy**: Extend relationship model to support 5+ factions and global councils.
3. **Terrain Transitions**: Implement bitmasking for sophisticated tileset transitions (Shorelines, Mountains).

## Recommended Near-Term Order

1. [x] Render a static, colored grid in `smac_bevy` driven by `GameState`.
2. [x] Implement camera controls (pan/zoom).
3. [x] Draw unit and base markers on the map.
4. [x] Integrate `bevy_egui` and add the "Next Turn" button and Turn Summary panel.
5. [x] Implement map interactivity (tile hovering, tooltips) to achieve feature parity with `smac_gui`'s core map view.
6. [x] Begin replacing simple shapes with loaded textures by establishing the `AssetServer` pipeline.
7. [x] Port Base Management and Unit Selection side panels using `bevy_egui`.

## Current Milestone Slice

Current slice: `Phase 3: Visual Transition`

- Establish `smac_bevy` as the new primary client.
- Preserve headless stability of `smac_core`.
- Maintain a strict boundary between `smac_core` (game rules) and `smac_bevy` (presentation).

## Prior History

### Before 2026-05-06

- The old `ROADMAP.md` showed a generic phase checklist that no longer matched the implemented workspace.
- `documentation/project_history/RUST_REWRITE_ASSESSMENT.md` captured the repo split correctly, but it was an assessment document, not a live status tracker.
- The local build environment already included a project-scoped Rust toolchain and `zig-cc`, but the repo documentation did not reflect that.

### Task History: Tasks 1-10

1. Re-established the active Rust workspace and confirmed `Projects/SMAC_Rust_AI` was the real build target.
2. Verified the workspace compiles cleanly with the project-local Rust toolchain and linker path.
3. Verified the full workspace test suite passes.
4. Added bundled content validation entry points in `smac_core`.
5. Exported validation through `content_api`.
6. Added end-to-end content validation coverage in integration tests.
7. Scoped validation to the currently playable runtime subset so forward-declared content does not cause false failures.
8. Added a `validate_content` CLI binary for direct content checks.
9. Expanded runtime `Tech` mappings to include additional bundled tech definitions already present in data.
10. Expanded runtime `UnitKind` mappings to include `IsleOfTheDeep` and updated presentation mappings.

### Task History: Tasks 11-20

11. Rewrote this file into a live status document with a rolling 10-task update rule and current next-task queue.
12. Added a root `Makefile` that wraps the local toolchain for workspace build, test, validation, formatting, clippy, and GUI run commands.
13. Replaced panic-based unit design decoding with structured fallible loading and validation-backed checks.
14. Replaced starting-scenario panic paths with a fallible loader and validation-backed checks.
15. Added structured fallible lookup helpers for bundled unit, facility, production, and tech definitions.
16. Replaced runtime faction bootstrap panics with a fallible loader and direct regression coverage.
17. Converted bundled JSON cache loading in `smac_core/src/content.rs` to cached `Result` state instead of parse panics.
18. Converted the bundled technology tree cache to a cached `Result` layer with a fallible `TechnologyTree::try_new`.
19. Removed remaining helper/CLI `expect` usage in the faction content helper and validation CLI path.
20. Refreshed this status file for the 10-task rollover and updated build health, completed work, and next tasks.

### Task History: Tasks 21-25

21. Added a root `README.md` for the actual Rust workspace and documented the local-toolchain workflow.
22. Made the fallible-vs-strict content API split explicit and exported shared `try_*` runtime definition accessors.
23. Replaced the old rewrite assessment with a feature-first implementation plan centered on `Playable Planetfall v1`.
24. Implemented the previously missing tech unlock targets as real runtime content: `Resonance Laser`, `Hologram Theatre`, `Bioenhancement Center`, and `Research Hospital`.
25. Added governor and AI planning support for the new unlocks, moved batch governor automation from `smac_gui` into `smac_core`, and extended core regression coverage around those paths.

### Task History: Tasks 26-30

26. Improved `smac_gui` production and research readability with available-vs-locked production, missing-tech lock reasons, and known/available/blocked research buckets.
27. Moved research unlock preview formatting into `smac_core::presentation` and added compact research-focus feedback in the base and research panels.
28. Exposed governor reason and queue-intent helpers from `smac_core`, and moved build/research status summaries out of `smac_gui`.
29. Added faction-level production posture, queue posture, and governor intent rollups to the faction overview.
30. Moved faction governor warning generation and governor mode summary formatting into `smac_core`, then refreshed this status file for the next 10-task rollover.

### Task History: Tasks 31-35

31. Added faction-overview summaries for queue gaps and tech-blocked governor intent, with jump actions into the affected bases.
32. Added core-owned bulk queue-fill support so empty base queues can be populated from governor intent in one operation.
33. Added research-impact reporting so available techs show which blocked base plans they would unlock.
34. Added current-research unlock pressure to operations advice and direct base-cycling actions from the research and operations panels.
35. Added GUI regression coverage for current-research affected-base cycling and refreshed this status file to match the current milestone state.

### Task History: Tasks 36-40

36. Added `Queue Gap` and `Research Unlock` as first-class base focus filters in `smac_gui`.
37. Added filter-aware focus actions so queue-gap views can fill empty queues directly from the base focus bar.
38. Added GUI regression coverage for current-research affected-base cycling and filter membership.
39. Added core-owned post-unlock governor queue previews for a selected research tech and wired them into the `Research Unlock` focus action.
40. Refreshed this status file with the new focus-filter/action workflow and current build health.

### Task History: Tasks 41-45

41. Made pinned unlock previews directly actionable after the relevant tech is actually known, with per-base and bulk apply actions.
42. Added stale-preview detection and refresh requirements so drifted pinned plans cannot be applied blindly.
43. Surfaced pinned-preview current-vs-stale state directly in the available-tech list and adjusted row actions accordingly.
44. Added automatic pruning for unknown-tech pinned previews that no longer unlock any immediate base plan.
45. Moved staged unlock preview lifecycle policy into `smac_core`, exported the new preview-state type, and trimmed `smac_gui` back to a consumer of core rules.

### Task History: Tasks 46-50

46. Moved pinned-preview labels, tooltips, and status strings into `smac_core::presentation`.
47. Moved current-tech and available-tech research summary text into shared presentation helpers.
48. Moved the remaining preview row labels, action labels, and blocked-tech row wording into `smac_core::presentation`.
240. Extended save-browser regression coverage in `smac_core`, reverified `cargo test` and `validate_content`, and refreshed this status file for the `231-240` rollover.
241. Extracted Unit rank, HP, and movement color policies into core `UnitSelectionDisplayState`.
242. Moved World Map badge glyph color policy (Warning, Danger, Recovery) to core `MapTileDisplayState`.
243. Refactored Sidebar hotkey mapping (`1`-`5`, `W`) into a centralized core `handle_sidebar_hotkey` method.
244. Centralized Base Panel static labels (Defense/Psi pressure, Damaged garrisons) in `smac_core`.
245. Moved Research Panel "Affected Bases" heading and summary presentation to core.
246. Refactored Save Management "can_load" check to be derived from core display state.
247. Centralized Faction Overview headings (Indices, Governor, Projects) and Jump Labels in `smac_core`.
248. Extracted Unit rank, HP, and movement color policies into `smac_core::game_state`.
249. Moved Sidebar tab hotkey hints and state assembly into core `SidebarDisplayState`.
250. Added Governor Mode semantic descriptions and tooltip logic into `smac_core`.
251. Extracted Unit Workshop ability selection metadata and toggle logic to `smac_core`.
252. Moved Save Browser file path tooltip formatting into `smac_core`.
253. Refactored Sidebar Panel into core-owned `SidebarDisplayState` for centralized structural control.
254. Decoupled Unit Selection advice colors from GUI hardcoding, moving them to core policy.
255. Unified Minimap and World Map selection highlight logic in core `MapTileDisplayState`.
256. Moved map interaction policy (selection/movement decision) into core `process_map_interaction`.
257. Extracted Workshop Component Labels (Chassis/Weapon/Armor) to `smac_core::presentation`.
258. Moved Default Save Naming logic into core `GameState`.
259. Extracted Save Conflict presentation formatting into core `SaveConflictDisplayState`.
260. Pushed Save Browser grid column headers into core `SaveBrowserDisplayState`.
261. Created core-owned `LogisticsBoardDisplayState` to decouple tab layout from log state.
262. Centralized `SidebarTab` definitions and hotkey metadata in `smac_core`.
263. Moved Minimap sizing and heading logic to core `MinimapDisplayState`.
264. Extracted Unit Rank presentation colors and formatted labels to `smac_core`.
265. Refactored `PlayerOperationsDashboardState` to provide a list of `jump_actions` for dynamic GUI rendering.
266. Added core-owned `SidebarDisplayState` and moved all navigation headings and tab assembly to `smac_core`.
267. Extracted Unit rank, HP, and movement color policies into core `UnitSelectionDisplayState`.
268. Moved World Map badge glyph color policy (Warning, Danger, Recovery) to core `MapTileDisplayState`.
269. Centralized Base Panel static labels and formatted values (Pressure, Output, Storage) in `smac_core`.
270. Moved Research Panel "Affected Bases" heading and summary presentation to core.
271. Refactored Sidebar hotkey mapping into a centralized core `handle_sidebar_hotkey` method.
272. Moved Governor Mode semantic descriptions and option lists for dropdowns into core `BasePanelDisplayState`.
273. Unified Minimap and World Map selection highlight logic (strokes and colors) in core `MapTileDisplayState`.
274. Centralized Save Management "can_load" check and target slot labels in core `SaveManagementDisplayState`.
275. Created core-owned `GameOverDisplayState` to centralize victory/defeat presentation logic.
276. Added `affected_entries_heading` to `CurrentResearchDisplayState` and integrated it into the GUI.
277. Centralized Faction Overview Jump Labels (Queue Gap, Tech Block) and headings in `smac_core`.
278. Moved map badge status glyph color policy (`!`, `?`, `*`) into core `MapTileDisplayState`.
279. Extracted Unit Workshop ability toggle metadata and descriptions to core `WorkshopDisplayState`.
280. Added Save Browser file path tooltips to core `SaveBrowserRowState`.
281. Refactored `PlayerOperationsDashboardState` to provide a list of `jump_actions` for dynamic GUI rendering.
282. Moved save file and metadata path generation logic into core `SaveSlotMetadata`.
283. Moved Base Panel production queue index formatting (`"1."`, `"2."`) to `smac_core`.
284. Consolidated Faction Overview Strategic Indices labels and headings in `smac_core`.
285. Extracted Minimap tile sizing and heading policy to core `MinimapDisplayState`.
286. Consolidated Operations Dashboard bulk actions into a dynamic `bulk_actions` list in `smac_core`.
287. Centralized Governor Mode dropdown options and descriptions in core `BasePanelDisplayState`.
288. Moved unit HP and Movement point color policies to core `UnitSelectionDisplayState`.
289. Unified Map Badge glyph color policies (Warning, Danger, CRISIS) in core `MapTileDisplayState`.
290. Refactored Sidebar Panel into core-owned `SidebarDisplayState` with dynamic hotkey hints.
291. Finalized extraction of all semantic string formatting and gameplay rules from `smac_gui`.
292. Implemented dynamic terraforming for Condensers and Echelon Mirrors with core yield bonuses.
293. Added combat logic for Trance and Raid unit abilities.
294. Implemented global yield and stability bonuses for Weather Pattern and Clinical Immortality secret projects.
295. Verified and completed effect mapping for all 19 base facilities.
296. Implemented Air Superiority and Deep Pressure Hull abilities, and updated movement logic to support sea units and aircraft.
297. Expanded Terraforming with nutrient doubling for Condensers and adjacency bonuses for Echelon Mirrors.
298. Implemented Empath Guild secret project effect and verified global stability/psi bonuses.
299. Added Needlejet unit kind and updated presentation layer.
300. Audited the live repository state and confirmed the previously documented green test/build status was stale.
301. Fixed `smac_gui` test compilation by removing the invalid `Copy` derive from `EditorTool` and matching editor state by reference.
302. Fixed convoy famine/disband logic in `smac_core` so unit IDs are no longer treated as vector indices.
303. Repaired the broken crisis and game-state regression tests, including deterministic debris-impact coverage and convoy/economic-victory test setup issues.
304. Reverified `cargo test --workspace` and `validate_content`, then refreshed `README.md` and `ROADMAP.md` to match the actual green workspace state.
305. Removed the remaining workspace warning noise so the repaired test suite is green and quiet.
306. Archived non-runtime JSON files out of the active `data/` root and documented the active content path directly in-repo.
307. Archived obsolete root-level setup scripts, backup manifests, and tree snapshots out of the active repository path.
308. Moved active bootstrap payloads into `toolchain/` and moved the large design reference text file into `documentation/technical_docs/references/`.
309. Added a formal local verification path via `scripts/verify_workspace.sh`, `make verify`, and a documented content-author workflow.
310. Added a formal local toolchain bootstrap path via `scripts/bootstrap_local_toolchain.sh` and `make bootstrap`.
311. Aligned verification with bootstrap and added a CI workflow that runs the same bootstrap/verify path.
312. Archived stray source-tree patch snapshots, retired scratch subcrates, and a checked-in manual GUI save out of active crate/runtime paths, then tightened `.gitignore` for local bootstrap/build directories and future save artifacts.
313. Moved preserved workspace backups out of the repository root into `_archived/workspace_backups/` and removed an empty nested experimental placeholder tree.
314. Moved stale project-level planning and assessment documents out of the repository root into `documentation/project_history/`.
315. Added an explicit dry-run-first local artifact cleanup script and marker READMEs for reserved documentation placeholder directories.
316. Verified a full clean-state rebuild and added a Python fallback for Zig archive extraction so bootstrap no longer depends strictly on a system `xz` binary.
317. Added a top-level archive inventory, tightened toolchain retention/bootstrap documentation, and removed an empty archived placeholder directory.
318. Externalized the committed `toolchain/` payloads into a manifest-driven fetch path, reverified a clean-state rebuild, and reduced repository payload volume by removing the committed Rust/Zig blobs.
319. Consolidated selected preserved experiment trees under `_archived/` into compressed bundles to reduce expanded source-tree noise while keeping the material in-repo.
320. Kept `glsmac/` in-tree as the reference checkout, removed generated build/output noise from the active path, and made the ignore/documentation policy explicit.
321. Added an explicit repository retention policy and closed `Operation Clean Slate`, shifting the live plan to post-cleanup stabilization.

## Playable Planetfall v1 Complete
The `smac_gui` crate is now a pure view layer. All deterministic game rules, AI policies, state assembly, and display formatting are owned by `smac_core`.

## Next: Feature Implementation Phase
1.  **Unit Abilities** (Done): Air Superiority, Deep Pressure Hull, Raid, Trance, Escort, Amphibious.
2.  **Terraforming Options** (Done): Condensers (doubled nutrients), Echelon Mirrors (adjacency energy).
3.  **Secret Project Effects** (Done): Weather Pattern, Clinical Immortality, Empath Guild.
4.  Implement **Base Facility Effects** (Verified usage of all 19 facilities).
5.  **Diplomacy Sub-Panel and Faction Pacts** (Done): Added status/attitude display, diplomatic actions, and shared visibility for Pact brothers.
6.  **Faction Specific Tech Bonuses** (Done): Added starting tech overrides and implemented parsing for faction attributes (Research, Morale, Industry, Growth, free facilities).
7.  **Unit Support and Upkeep** (Done): Unit upkeep now drains base minerals instead of faction energy, and free support properly applies faction `support` attribute (e.g. Morgan's penalty).
8.  **Unit Abilities** (Done): Implemented remaining combat modifiers (Comm Jammer, Non-Lethal Methods).
9.  **Advanced Terraforming** (Done): Added Forest and Thermal Borehole improvements, integrated yields (Forest: 1/2/1, Borehole: 0/6/6), added spacing constraints, and hooked into the Tech Tree.
10. **Victory Conditions** (Done): Implemented checks and logic for Conquest (faction elimination), Economic (10,000 Energy), and a later-updated transcendence path that now requires `Secrets of Planet` plus `Empath Guild`.
11. **Social Engineering** (Done): Implemented modifiers for Politics, Economics, Values, and Future Society. Integrated into core attributes, added UI sub-panel, and hooked into the Tech Tree.
12. **Space Mastery Victories** (Done): Implemented Space Transcendence and Black Hole Harvesting victory paths. Added 4 new high-tech Secret Projects and 3 new technologies. Updated victory checking and UI messages.
13. **Secret Project** full effects and dedicated display panel (Done): Added dedicated "Projects" Sidebar tab, implemented global project registry UI, and finalized secondary effects (Manifold Drive +1 move, Orbital Elevator doubled base energy, Singularity bonuses).
14. **Base Expansion** limits and Efficiency modifiers (Done): Implemented Efficiency-based expansion limits (Bureaucracy) and distance-based Energy Waste (Corruption). Added HQ tracking to Factions and updated Base Panel UI.
15. **Social Engineering** full impact on growth and production (Done): Implemented Population Booms (Growth >= 6), Economy energy bonuses (+2 Economy), and Psionic combat modifiers based on Planet rating. Integrated Planet rating into toxicity mitigation.
16. **AI Tactical Improvements** (Done): Enhanced terraforming heuristics (Condensers, Echelon Mirrors), implemented Former target-seeking logic, tech-aware Social Engineering selection, and SE-driven tactical bias (attack/exploration).
17. **AI Strategic Depth & Diplomacy** (Done): Implemented proactive AI diplomacy proposals, strategic research selection based on personality, AI unit modernization (upgrades), and competitive Secret Project prioritization. Added tech trading foundation.
18. **Espionage & Covert Operations** (Done): Implemented the Probe Team unit and core espionage actions (Steal Tech, Sabotage Facility, Subvert Unit). Integrated into the Tech Tree and AI production/targeting logic.
19. **Strategic Overflow & Mechanics Polish** (Done): Implemented multi-completion production overflow, Thermal Borehole toxicity impact, faction-specific SE restrictions, and supply pod tech preservation. Optimized AI research re-evaluation.

## Task History
...
299. Added Needlejet unit kind and updated presentation layer.
301. Enhanced `try_ai_terraform` to include smart heuristics for Condensers, Echelon Mirrors, and existing improvements.
302. Implemented `choose_ai_former_target` to allow AI Formers to seek high-value tiles for terraforming.
303. Improved AI Social Engineering selection to respect technology unlocks and better align with faction personality and current needs.
304. Implemented AI tactical bias adjustments (attack, exploration) based on active Social Engineering choices.
305. Added AI awareness of Bureaucracy (Expansion limits) to settlement planning and fixed numerous build errors related to non-Copy `UnitKind`.
306. Implemented proactive AI diplomacy proposals (Treaties, Pacts) based on faction personality and attitude.
307. Added strategic research selection for AI, prioritizing techs by category (Explore, Build, Conquer, Discover) aligned with personality.
308. Implemented AI unit modernization logic to use surplus energy for upgrading obsolete units to better designs.
309. Enhanced AI base production to competitively prioritize Secret Projects when infrastructure is stable.
310. Added `ProposeTechTrade` and `RespondTechTrade` actions and implemented AI logic for technology exchanges.
311. Implemented gradual diplomatic attitude growth/decay based on relationship status (War, Treaty, Pact).
312. Added `Probe` ability and `ProbeTeam` unit kind to the core engine, along with `PerformProbeAction` game actions.
313. Integrated Probe Teams into `units.json`, `production.json`, and unlocked them via `Planetary Networks` in the tech tree.
314. Implemented `StealTech` probe action logic (stealing a random unknown technology from the target).
315. Implemented `SabotageFacility` and `SubvertUnit` probe action logic, with energy costs for unit subversion.
316. Taught the AI to build Probe Teams when facing military pressure and actively target enemy bases and units for covert operations.
317. Implemented multi-completion production overflow, allowing bases with high mineral stocks to finish multiple queue items in a single turn.
318. Added ecological consequences for Thermal Boreholes, increasing faction Planet Toxicity by 2 points per turn per borehole.
319. Expanded Social Engineering restrictions for The Lord's Believers (no Knowledge, no Free Market) and refined Hive/Morgan/Gaia rules.
320. Modified supply pod tech rewards to preserve current research progress (overflow) instead of resetting it to zero.
321. Taught the AI to immediately re-evaluate research targets upon tech discovery to ensure overflow points are applied to optimal technologies.
322. Added Rust GUI observer controls to watch the autoplay sim live from `smac_gui`.
323. Fixed the autoplay runtime path for custom-unit stats and upgrade bookkeeping so long demos no longer panic when runtime designs are produced or upgraded.
324. Fixed the turn-50 flatline by repairing colony-pod movement/founding flow, worked-tile economy usage, and stale dead-unit tile occupancy.
325. Added `smac_core/src/bin/autoplay_sweep.rs` for repeatable multi-seed diagnostics and re-prioritized AI recovery production so unrested bases build morale/recovery infrastructure before generic economy padding.
