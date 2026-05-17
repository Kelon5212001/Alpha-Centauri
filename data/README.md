# Data Directory Status

Last verified: 2026-05-15

This directory contains the active runtime content for the current Rust workspace.

## Active Runtime Files

These files are on the current `smac_core` runtime and validation path:

- `factions.json`
- `facilities.json`
- `production.json`
- `runtime_rules.json`
- `start_scenario.json`
- `technology_tree.json`
- `ui_theme.json`
- `units.json`
- `localization.json`

These are the files exercised by `cargo run -p smac_core --bin validate_content`.

## Archived Files

Older non-runtime JSON files were moved to [`data/_archived/`](./_archived/README.md):

- `buildings.json`
- `societies.json`
- `unit_runtime.json`

## Notes

- `assets.pack` can be built from this directory via `smac_core/src/bin/pack_assets.rs`.
- The loose JSON files remain the current default content source for the active runtime.
