#!/bin/sh
# Build the Rust simulation core to wasm/jadereaper_core.wasm.
# Needs: rustup target add wasm32-unknown-unknown
set -e
cd "$(dirname "$0")"
cargo build --release --target wasm32-unknown-unknown --manifest-path core/Cargo.toml
cp core/target/wasm32-unknown-unknown/release/jadereaper_core.wasm wasm/
ls -l wasm/jadereaper_core.wasm
