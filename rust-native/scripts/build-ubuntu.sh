#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_DIR="${ROOT_DIR}/dist/ubuntu"

cd "${ROOT_DIR}"

cargo build --release -p alpha_centauri_platform_linux

mkdir -p "${OUTPUT_DIR}"
cp "${ROOT_DIR}/target/release/alpha-centauri" "${OUTPUT_DIR}/alpha-centauri"
chmod +x "${OUTPUT_DIR}/alpha-centauri"

cat <<MSG
Built Ubuntu executable:
  ${OUTPUT_DIR}/alpha-centauri
Run it with:
  ${OUTPUT_DIR}/alpha-centauri
MSG
