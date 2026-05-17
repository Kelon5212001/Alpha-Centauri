# Archived Root Artifacts

These files were moved out of the repository root on 2026-05-15 during `Operation Clean Slate`.

Reason:

- they no longer represent the active `smac_core` + `smac_gui` workspace
- they created ambiguity about the supported setup and project layout

Archived here:

- `install_and_run.sh`: old bootstrap/run script for an earlier crate layout
- `setup_universal_codex.sh`: old environment bootstrap script that scaffolds a different project structure
- `Cargo.toml.backup`: obsolete backup manifest from an older workspace phase
- `smac_core/Cargo.toml.backup`: obsolete core manifest backup from an older workspace phase
- `bk bk test Projects SMAC.txt`: old tree snapshot of a no-longer-current repository layout

Moved elsewhere instead of archiving:

- `rustup-init`: moved to `toolchain/` as an explicit active bootstrap artifact
- `zig-x86_64-linux-0.16.0.tar.xz`: moved to `toolchain/` as an explicit active bootstrap artifact
- `smac_new_game_units_facilities_upgrades_reference_v02.txt`: moved to `documentation/technical_docs/references/`

Intentionally kept in the repository root:

- `zig-cc`: active linker wrapper referenced by `.cargo/config.toml`

If any archived file becomes relevant again, it should come back with:

1. an explicit active owner,
2. documentation in the main workflow docs, and
3. a clear reason it belongs on the active path.
