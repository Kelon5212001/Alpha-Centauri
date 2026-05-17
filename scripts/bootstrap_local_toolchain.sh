#!/bin/bash
set -euo pipefail

PROJECT_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
CARGO_HOME_LOCAL="$PROJECT_ROOT/.cargo-local"
RUSTUP_HOME_LOCAL="$PROJECT_ROOT/.rustup-local"
RUSTUP_INIT_BIN="$PROJECT_ROOT/toolchain/rustup-init"
RUSTUP_INIT_URL="https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init"
RUSTUP_INIT_SHA256_URL="${RUSTUP_INIT_URL}.sha256"
ZIG_ARCHIVE="$PROJECT_ROOT/toolchain/zig-x86_64-linux-0.16.0.tar.xz"
ZIG_ARCHIVE_URL="https://ziglang.org/download/0.16.0/zig-x86_64-linux-0.16.0.tar.xz"
ZIG_ARCHIVE_SHA256="70e49664a74374b48b51e6f3fdfbf437f6395d42509050588bd49abe52ba3d00"
ZIG_DIR="$PROJECT_ROOT/.zig"
ZIG_EXTRACTED_DIR="$ZIG_DIR/zig-x86_64-linux-0.16.0"

download_file() {
    local url="$1"
    local output="$2"

    if command -v curl >/dev/null 2>&1; then
        curl --proto '=https' --tlsv1.2 -fsSL "$url" -o "$output"
        return
    fi

    if command -v wget >/dev/null 2>&1; then
        wget -qO "$output" "$url"
        return
    fi

    if ! command -v python3 >/dev/null 2>&1; then
        echo "[bootstrap] unable to download $url: need curl, wget, or python3"
        exit 1
    fi

    python3 - "$url" "$output" <<'PY'
import sys
import urllib.request

url, output = sys.argv[1], sys.argv[2]
with urllib.request.urlopen(url) as response, open(output, "wb") as handle:
    handle.write(response.read())
PY
}

compute_sha256() {
    local path="$1"

    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$path" | awk '{print $1}'
        return
    fi

    python3 - "$path" <<'PY'
import hashlib
import sys

path = sys.argv[1]
digest = hashlib.sha256()
with open(path, "rb") as handle:
    for chunk in iter(lambda: handle.read(1024 * 1024), b""):
        digest.update(chunk)
print(digest.hexdigest())
PY
}

download_rustup_init() {
    local tmp_bin="$RUSTUP_INIT_BIN.tmp"
    local tmp_sha="$RUSTUP_INIT_BIN.sha256.tmp"
    local expected_sha
    local actual_sha

    echo "[bootstrap] downloading rustup-init"
    download_file "$RUSTUP_INIT_URL" "$tmp_bin"
    download_file "$RUSTUP_INIT_SHA256_URL" "$tmp_sha"

    expected_sha=$(awk '{print $1}' "$tmp_sha")
    actual_sha=$(compute_sha256 "$tmp_bin")
    if [ "$actual_sha" != "$expected_sha" ]; then
        echo "[bootstrap] rustup-init checksum mismatch"
        rm -f "$tmp_bin" "$tmp_sha"
        exit 1
    fi

    chmod +x "$tmp_bin"
    mv "$tmp_bin" "$RUSTUP_INIT_BIN"
    rm -f "$tmp_sha"
}

download_zig_archive() {
    local tmp_archive="$ZIG_ARCHIVE.tmp"
    local actual_sha

    echo "[bootstrap] downloading zig archive"
    download_file "$ZIG_ARCHIVE_URL" "$tmp_archive"
    actual_sha=$(compute_sha256 "$tmp_archive")
    if [ "$actual_sha" != "$ZIG_ARCHIVE_SHA256" ]; then
        echo "[bootstrap] zig archive checksum mismatch"
        rm -f "$tmp_archive"
        exit 1
    fi

    mv "$tmp_archive" "$ZIG_ARCHIVE"
}

extract_zig_archive() {
    if command -v xz >/dev/null 2>&1; then
        tar -xJf "$ZIG_ARCHIVE" -C "$ZIG_DIR"
        return
    fi

    if ! command -v python3 >/dev/null 2>&1; then
        echo "[bootstrap] neither xz nor python3 is available to unpack $ZIG_ARCHIVE"
        exit 1
    fi

    python3 - "$ZIG_ARCHIVE" "$ZIG_DIR" <<'PY'
import sys
import tarfile

archive_path, output_dir = sys.argv[1], sys.argv[2]
with tarfile.open(archive_path, "r:xz") as archive:
    archive.extractall(output_dir)
PY
}

mkdir -p "$CARGO_HOME_LOCAL" "$RUSTUP_HOME_LOCAL" "$ZIG_DIR"

export CARGO_HOME="$CARGO_HOME_LOCAL"
export RUSTUP_HOME="$RUSTUP_HOME_LOCAL"
export PATH="$CARGO_HOME_LOCAL/bin:$PATH"

if [ ! -x "$RUSTUP_INIT_BIN" ]; then
    download_rustup_init
fi

if [ ! -x "$CARGO_HOME_LOCAL/bin/rustup" ]; then
    echo "[bootstrap] installing local rustup toolchain"
    "$RUSTUP_INIT_BIN" -y --default-toolchain stable
else
    echo "[bootstrap] local rustup already present"
fi

if [ ! -x "$RUSTUP_HOME_LOCAL/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo" ]; then
    echo "[bootstrap] installing stable toolchain"
    "$CARGO_HOME_LOCAL/bin/rustup" toolchain install stable
    "$CARGO_HOME_LOCAL/bin/rustup" component add rustfmt clippy rust-analyzer --toolchain stable
    "$CARGO_HOME_LOCAL/bin/rustup" target add x86_64-unknown-linux-gnu --toolchain stable
else
    echo "[bootstrap] stable toolchain already present"
fi

if [ ! -x "$ZIG_EXTRACTED_DIR/zig" ]; then
    if [ ! -f "$ZIG_ARCHIVE" ]; then
        download_zig_archive
    fi

    echo "[bootstrap] extracting local zig archive"
    rm -rf "$ZIG_EXTRACTED_DIR"
    extract_zig_archive
else
    echo "[bootstrap] local zig already extracted"
fi

echo "[bootstrap] local toolchain ready"
echo "[bootstrap] export PATH=\"$CARGO_HOME_LOCAL/bin:$RUSTUP_HOME_LOCAL/toolchains/stable-x86_64-unknown-linux-gnu/bin:\$PATH\""
echo "[bootstrap] export RUSTC=\"$RUSTUP_HOME_LOCAL/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc\""
