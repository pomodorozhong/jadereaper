# Jade Reaper

A browser action game: Rust simulation compiled to WebAssembly, WebGPU paper-cutout renderer, paper-style HTML UI. See [README.md](README.md) for how to run and extend it, [ROADMAP.md](ROADMAP.md) for the plan and [docs/README.md](docs/README.md) for gotchas.

Checks (CI runs them in `.github/workflows/checks.yml`): `cargo test --release --manifest-path core/Cargo.toml`, `cargo fmt --check` and `cargo clippy --release --all-targets -- -D warnings` in `core/`, then `./build.sh` (the wasm is not committed, so always build before playing or the smoke test), then `npm run lint` (Biome: formatting and lint; `npm run format` fixes formatting), `npm test` and `npm run smoke` (after `npm ci`).

## Contributing

Before opening an issue or a PR, read [CONTRIBUTING.md](CONTRIBUTING.md) and follow it.
