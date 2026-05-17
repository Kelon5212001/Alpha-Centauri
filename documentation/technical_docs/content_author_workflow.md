# Content Author Workflow

Last updated: 2026-05-15

This workflow is the current authoritative check path for bundled game content and workspace health.

## Active Content Files

The active runtime content root is `data/`.

Current runtime files:

- `factions.json`
- `facilities.json`
- `production.json`
- `runtime_rules.json`
- `start_scenario.json`
- `technology_tree.json`
- `ui_theme.json`
- `units.json`
- `localization.json`

Archived non-runtime files live under `data/_archived/`.

## Normal Verification Flow

Use the local toolchain and run:

```bash
scripts/verify_workspace.sh
```

That script runs:

1. `scripts/bootstrap_local_toolchain.sh`
2. `cargo check`
3. `cargo run -p smac_core --bin validate_content --quiet`
4. `cargo test --workspace --quiet`

## Makefile Shortcut

If `make` is available in the shell image:

```bash
make verify
```

## Bootstrap

If the local Rust/Zig toolchain is missing, prepare it with:

```bash
scripts/bootstrap_local_toolchain.sh
```

If `make` is available:

```bash
make bootstrap
```

## When To Run This

Run the workflow after:

- editing files under `data/`
- changing content mappings in `smac_core/src/content.rs`
- changing runtime enums or lookup logic in `smac_core`
- changing tests that assert content counts or content availability

## Policy

- New runtime content should land in the active `data/` root only if it has an active caller.
- Runtime content changes should keep `validate_content` green.
- Files without active callers should not be left in the active `data/` root.
