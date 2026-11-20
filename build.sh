#!/bin/sh
# Generate the content tables from content/game.json, then build the Rust simulation core to
# wasm/jadereaper_core.wasm.
# Needs: Node (see .node-version) and rustup target add wasm32-unknown-unknown
set -e
cd "$(dirname "$0")"
node tools/content.mjs
cargo build --release --target wasm32-unknown-unknown --manifest-path core/Cargo.toml
cp core/target/wasm32-unknown-unknown/release/jadereaper_core.wasm wasm/
ls -l wasm/jadereaper_core.wasm
