# Repository Retention Policy

Last updated: 2026-05-15

This document defines what stays in the repository, what is treated as generated local state, and what belongs in archival storage rather than on the active workspace path.

## Active In-Repo Material

These paths are intentionally kept in the repository because they are part of the active development and verification flow:

- `smac_core/`
- `smac_gui/`
- `data/`
- `assets/`
- `assets.pack`
- `scripts/`
- `toolchain/manifest.json`
- `toolchain/README.md`
- `zig-cc`
- `.cargo/config.toml`
- `README.md`
- `ROADMAP.md`
- `documentation/technical_docs/`

## Kept In-Repo As Reference Material

These paths are intentionally retained even though they are not part of the active Rust runtime path:

- `glsmac/`
  - kept as an in-tree nested reference checkout
  - treated as read-only reference material for this repository unless explicitly working on `glsmac`
  - generated `glsmac` build/output directories are local noise and should not be treated as active workspace content
- `_archived/`
  - kept for preserved historical material that still has local reference value
  - preferred form is compact and clearly labeled

## Preferred Archive Forms

Use these rules when preserving non-active material:

- Keep small, directly readable historical artifacts unpacked when they are likely to be inspected manually.
- Prefer compressed bundles under `_archived/bundles/` for whole experiment trees that no longer need expanded source-directory presence.
- Keep source snapshots under `_archived/source_snapshots/` only when the exact file layout matters for later comparison.
- Keep project-spec and planning artifacts under `documentation/project_history/` or `_archived/project_spec/`, not in the repository root.

## Generated Local State

These paths are generated or downloaded during local work and are not part of the committed repository surface:

- `target/`
- `.cargo-local/`
- `.rustup-local/`
- `.zig/`
- `.zig-cache/`
- `toolchain/rustup-init`
- `toolchain/*.tar.xz`
- `smac_gui/saves/*.json`
- `smac_gui/saves/*.json.meta`
- `glsmac/build/`
- `glsmac/build_release/`
- `glsmac/out/`
- `glsmac/.vs/`
- `glsmac/.idea/`
- `glsmac/.cmake/`
- `glsmac/cmake-build-debug/`
- `glsmac/src/tmp/`

Use `scripts/clean_local_artifacts.sh` to remove the active workspace's generated local state. `glsmac` generated outputs are handled separately through ignore policy and manual deletion.

## Root-Level Policy

The repository root should stay limited to:

- active workspace manifests and wrappers
- live status docs
- active crates and assets
- intentionally retained top-level reference or archive directories

Do not reintroduce:

- backup files
- one-off patch snapshots
- setup scraps
- ambiguous project notes
- inactive JSON content

Those belong under `_archived/`, `documentation/project_history/`, or `documentation/technical_docs/references/`.

## Phase Exit Rule

`Operation Clean Slate` is considered complete when all of the following remain true:

- `scripts/verify_workspace.sh` passes from a clean local state
- active crate paths do not contain ad hoc backups or snapshot files
- runtime content files are separated from inactive content
- root-level clutter has been moved to documented archive/reference locations
- toolchain bootstrap is reproducible without committed binary payloads
- `README.md` and `ROADMAP.md` match the actual repository state

After that point, future cleanup should be incremental policy enforcement rather than broad repository reshaping.
