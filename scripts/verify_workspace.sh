#!/bin/bash
set -euo pipefail

PROJECT_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TOOLCHAIN_BIN="$PROJECT_ROOT/.rustup-local/toolchains/stable-x86_64-unknown-linux-gnu/bin"

echo "[verify] bootstrap local toolchain"
"$PROJECT_ROOT/scripts/bootstrap_local_toolchain.sh" >/dev/null

export PATH="$TOOLCHAIN_BIN:$PATH"
export RUSTC="$TOOLCHAIN_BIN/rustc"

cd "$PROJECT_ROOT"

echo "[verify] cargo check"
"$TOOLCHAIN_BIN/cargo" check

echo "[verify] cargo run -p smac_core --bin validate_content"
"$TOOLCHAIN_BIN/cargo" run -p smac_core --bin validate_content --quiet

echo "[verify] cargo test --workspace"
"$TOOLCHAIN_BIN/cargo" test --workspace --quiet

echo "[verify] workspace verification complete"
