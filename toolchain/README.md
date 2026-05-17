# Toolchain Artifacts

This directory contains in-repo bootstrap payloads for the local development toolchain.

Committed contents:

- `README.md`: retention/bootstrap policy for toolchain payloads
- `manifest.json`: pinned download sources for the active bootstrap path

Generated local contents after bootstrap:

- `rustup-init`: downloaded local Rust toolchain bootstrap binary
- `zig-x86_64-linux-0.16.0.tar.xz`: downloaded local Zig archive used to provision the linker/toolchain setup

These files were moved out of the repository root on 2026-05-15 during `Operation Clean Slate` so the active project path is easier to read.

Bootstrap entry point:

- `scripts/bootstrap_local_toolchain.sh`
- `make bootstrap`

Bootstrap extraction notes:

- the committed Zig archive is extracted into `.zig/`
- if `toolchain/rustup-init` is missing, bootstrap downloads it from the official Rust static distribution path and verifies the published SHA-256 file
- if `toolchain/zig-x86_64-linux-0.16.0.tar.xz` is missing, bootstrap downloads it from the pinned Zig 0.16.0 release URL and verifies the pinned SHA-256
- extraction prefers a system `xz` binary when available
- if `xz` is unavailable, bootstrap falls back to Python's standard-library `tarfile`/`lzma` support
- removable local generated toolchain/build state can be previewed with `scripts/clean_local_artifacts.sh`

Related active files:

- `.cargo/config.toml`: points the Linux linker at `zig-cc`
- `zig-cc`: wrapper script used by the active workspace
- `.zig/`: extracted local Zig payload used by `zig-cc`
- `.rustup-local/`: local Rust toolchain used by the workspace

This directory is part of the active bootstrap story, unlike `_archived/`.
