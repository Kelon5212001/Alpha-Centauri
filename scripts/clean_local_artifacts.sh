#!/bin/bash
set -euo pipefail

PROJECT_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

paths=(
    "$PROJECT_ROOT/target"
    "$PROJECT_ROOT/.cargo-local"
    "$PROJECT_ROOT/.rustup-local"
    "$PROJECT_ROOT/.zig"
    "$PROJECT_ROOT/.zig-cache"
    "$PROJECT_ROOT/toolchain/rustup-init"
    "$PROJECT_ROOT/toolchain/zig-x86_64-linux-0.16.0.tar.xz"
)

save_globs=(
    "$PROJECT_ROOT/smac_gui/saves/*.json"
    "$PROJECT_ROOT/smac_gui/saves/*.json.meta"
)

mode="dry-run"
if [ "${1:-}" = "--yes" ]; then
    mode="apply"
elif [ "${1:-}" != "" ]; then
    echo "usage: $0 [--yes]"
    exit 1
fi

echo "[clean-local] mode: $mode"
echo "[clean-local] generated directories:"
for path in "${paths[@]}"; do
    if [ -e "$path" ]; then
        echo "  $path"
    fi
done

echo "[clean-local] manual GUI save artifacts:"
for pattern in "${save_globs[@]}"; do
    for match in $pattern; do
        if [ -e "$match" ]; then
            echo "  $match"
        fi
    done
done

if [ "$mode" != "apply" ]; then
    echo "[clean-local] dry run only; rerun with --yes to delete the listed artifacts"
    exit 0
fi

for path in "${paths[@]}"; do
    if [ -e "$path" ]; then
        rm -rf "$path"
    fi
done

for pattern in "${save_globs[@]}"; do
    for match in $pattern; do
        if [ -e "$match" ]; then
            rm -f "$match"
        fi
    done
done

echo "[clean-local] local generated artifacts removed"
