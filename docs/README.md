# Docs

Project overview, controls, layout and how to extend the game are in the root [README.md](../README.md). The plan is in [ROADMAP.md](../ROADMAP.md).

## Gotchas

- **WebGPU needs a secure origin.** Open the game through `http://localhost` or `https`. Opening `index.html` from disk fails, and the game shows its "Cannot start" screen.
- **No `strip = true` in `core/Cargo.toml`.** On this machine's Rust toolchain, `rust-objcopy` cannot find `libLLVM.dylib`, so stripping fails during the build. The release profile leaves it out; the wasm is 69 KB without it.
- **The built wasm is committed.** After changing anything in `core/`, run `./build.sh` and commit `wasm/jadereaper_core.wasm` with the source change, or the browser keeps running the old core.
- **Ids are mirrored by hand** in `core/src/defs.rs`, `src/data.js` and `src/shader.wgsl` until roadmap step 1 generates them. Change all three together.
- **Wasm memory views go stale.** `src/main.js` rebuilds its `Float32Array` views of wasm memory on every read, because memory growth detaches old views. Keep that pattern when reading new buffers.
- **Balance numbers** in `core/src/defs.rs` (`KINDS`) and `core/src/game.rs` (`xp_for`, spawn rate, weapon stats) were tuned only against a bot that walks in circles. Treat them as placeholders.
