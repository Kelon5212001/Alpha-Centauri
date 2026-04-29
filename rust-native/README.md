# Rust Native Rewrite Workspace

This workspace is the new home for the Linux-native Rust rewrite.

## Crates
- `engine`: core gameplay/runtime domain model
- `platform-linux`: Linux bootstrap/runtime integration, builds the `alpha-centauri` executable

## Build an Ubuntu executable
From the repository root:

```bash
./rust-native/scripts/build-ubuntu.sh
```

The binary will be generated at:
- `rust-native/dist/ubuntu/alpha-centauri`

## Notes
Legacy C++ code is preserved under `legacy/cpp/` during migration.
