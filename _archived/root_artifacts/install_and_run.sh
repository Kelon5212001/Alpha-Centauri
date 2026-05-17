#!/bin/bash
set -e

if ! command -v cargo &> /dev/null; then
    echo "Installing Rust toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

cargo install cargo-edit || true
cargo add bevy serde serde_json rand anyhow || true

cargo build
cargo run
