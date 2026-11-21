# 0007: Build the wasm in CI, and check the core and the JavaScript on every PR

- Status: decided
- Date: 2026-11-20
- Issue: #39

## Context

The browser runs `wasm/jadereaper_core.wasm`, which was built by hand with `./build.sh` and committed. Nothing checked that it matched the Rust source, so a PR that changed `core/` without rebuilding it shipped the old core. CI ran only the font check and the smoke test: not the core tests, formatting, lints or anything on the JavaScript. On `main` the core had drifted far from the standard `cargo fmt` style and had clippy warnings nobody saw.

The plan from 2026-11-14 (the old roadmap's CI step) was to keep committing the wasm, add a CI check that a fresh build matched it byte for byte, and pin Rust to 1.99.1 so that check could pass. It listed building the wasm only in CI as the alternative.

## Decision

- The wasm is built in CI and is not committed. `wasm/` is ignored. The Pages deploy and the smoke test install stable Rust with the `wasm32-unknown-unknown` target and run `./build.sh` first.
- A Checks workflow runs on every PR and push to `main`: `cargo test`, `cargo fmt --check`, `cargo clippy` with warnings as errors, `node tools/content.mjs --check`, a wasm build, Biome (`biome check`: its formatter and linter) on `src/` and `tools/`, and the tool tests.
- The core uses the standard `cargo fmt` style, adopted in one formatting pass. There is no `rustfmt.toml`. The generated `core/src/content.rs` is skipped, because `tools/content.mjs` owns its layout.
- Biome is a pinned dev dependency, used as the JavaScript's linter and formatter with its default style (tab indents, double quotes, 80 columns), adopted in one formatting pass. The generated `src/content.js` is linted but not formatted.
- There is no `rust-toolchain.toml`. CI and contributors use the current stable Rust.

## Reasons

Building the wasm in CI removes the drift instead of checking for it. The site always runs the core that is in the source, nobody has to remember to rebuild and commit a binary, and PRs that touch the core no longer carry binary diffs or merge conflicts that cannot be resolved by hand.

Without a drift check nothing needs byte-exact builds, so there is no reason to pin Rust and keep bumping the pin. A `rustfmt.toml` that kept the old style was set aside for the standard style, which every Rust contributor and tool already expects.

For the JavaScript the same reasoning applies: Biome's default style over a configuration that kept the old one. Even set close to the old style its formatter rewrote about 2,000 lines, so the defaults cost little more and need no configuration to keep in sync.

## Consequences

- Rust with the wasm target (`rustup target add wasm32-unknown-unknown`) is now needed to run the game locally, not only to change the core. `npm start` and `npm run smoke` stop with a message naming `./build.sh` when the wasm is missing.
- After changing `core/` or `content/game.json`, run `./build.sh` before playing locally, but commit only the source and the generated tables.
- The deploy and the smoke test take longer, because they install Rust and build.
- A new stable Rust release can bring clippy warnings or formatting changes that fail CI with no code change. Fix them when that happens.
