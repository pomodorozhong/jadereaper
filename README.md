# Jade Reaper

A survivor game backbone that runs natively in the browser. The simulation is Rust compiled to WebAssembly, the picture is drawn with WebGPU, and everything on screen is styled as cut paper and ink.

## Run it

Serve the folder with any static file server and open it in a browser with WebGPU (current Chrome, Edge or Safari). WebGPU needs `http://localhost` or `https`, so opening `index.html` from disk will not work.

```bash
python3 -m http.server 5173
```

Then open http://localhost:5173.

The built `wasm/jadereaper_core.wasm` is checked in, so Rust is only needed if you change the core.

## Controls

| Action | Keyboard | Gamepad |
| --- | --- | --- |
| Move | WASD or arrow keys | Left stick or d-pad |
| Pause | Esc or P | Start |
| Menu move | Arrow keys or WASD | Stick or d-pad |
| Menu confirm | Enter or Space | A |
| Back | Esc | B |
| Pick upgrade | 1, 2, 3 | Stick, then A |
| Mute | M | |

Attacks are automatic. The game pauses itself when the window loses focus.

## How the paper style works

Characters are puppets made of rigid paper parts, not sprite sheets. The reaper is a robe, a hood and a scythe. A walk cycle is two poses: tilt one way, then tilt the other with a small hop. Enemies do the same, each on its own offset. This needs one still image per part and no animation frames.

The renderer adds the rest to every part: an ink outline, a white cutout margin, a drop shadow, paper grain, and an outline that "boils" at 8 frames per second like stop-motion. Parts are drawn in code as signed distance shapes, and any part can be swapped for your own PNG. See [assets/parts/README.md](assets/parts/README.md).

The UI is HTML and CSS over the canvas: torn-edge paper sheets, tags and cards, a hand-written font and a vermilion seal, using two SVG filters defined in `index.html`.

## Layout

| Path | What it is |
| --- | --- |
| `core/` | Rust simulation. No dependencies, no bindgen. |
| `core/src/defs.rs` | Instance format, part ids, upgrade ids, enemy table |
| `core/src/game.rs` | State, simulation step, puppet posing, instance building |
| `core/src/lib.rs` | The `jr_*` functions exported to JavaScript, and tests |
| `wasm/` | Built core |
| `src/main.js` | Loads the core, frame loop, game modes |
| `src/renderer.js` | WebGPU setup, part image atlas, drawing |
| `src/shader.wgsl` | Paper shapes, cutout shading, paper background |
| `src/input.js` | Keyboard and gamepad |
| `src/audio.js` | Synthesized sound effects (no audio files) |
| `src/ui.js` | Screens, HUD, menu focus |
| `src/save.js` | Records and settings in localStorage |
| `src/data.js` | Part names, upgrade text, state and event ids |
| `assets/parts.json` | Optional image overrides for parts |
| `assets/fonts/` | Bundled UI fonts, cut down to the game's characters, with their licenses |
| `tools/` | Font subset and check scripts |

## Fonts

The Traditional Chinese UI uses bundled fonts: [Bakudai](https://github.com/max32002/bakudaifont) (莫大毛筆) for titles, buttons and brush marks, and [LXGW WenKai TC](https://github.com/lxgw/LxgwWenkaiTC) for body text. [Caveat Brush](https://fonts.google.com/specimen/Caveat+Brush) draws numbers in every language. The English UI still uses system fonts (issue #3). All three fonts are under the SIL Open Font License 1.1; the license texts are in `assets/fonts/`.

The full fonts are 12 to 16 MB each, so `assets/fonts/` holds copies cut down to only the characters the game shows, about 440 KB in total. After changing any Chinese text, rebuild them and commit `assets/fonts/`:

```bash
tools/subset-fonts.sh
```

The script needs [uv](https://docs.astral.sh/uv/). It collects every Chinese character in `src/*.js` and `index.html`, downloads the pinned source fonts into `.font-cache/`, and writes the woff2 files. `tools/check-fonts.sh` checks the committed fonts against the text without downloading anything, and CI runs it on every pull request.

## How core and host talk

Each frame the host calls `jr_input`, then `jr_update(dt)`, and reads three buffers out of wasm memory:

- Instances: 12 floats per paper part (`x, y, half width, half height, rotation, part id, flash, seed, r, g, b, a`). These bytes go to the GPU unchanged.
- State: 32 floats for the HUD (health, experience, time, offered upgrades and so on). Indices are in `src/data.js` under `STATE`.
- Events: 4 floats each (`type, x, y, value`), used for sound.

When the core sets the "choosing" flag the host shows the upgrade cards and answers with `jr_choose(slot)`.

## Build the core

```bash
rustup target add wasm32-unknown-unknown
```

```bash
./build.sh
```

Run the simulation tests natively:

```bash
cargo test --release --manifest-path core/Cargo.toml
```

## Extending

- New enemy: add a row to `KINDS` in `core/src/defs.rs` and a spawn rule in `Game::spawn`.
- New upgrade: add an id and max level in `defs.rs`, its effect in `Game::weapons` or where the stat is read, and its glyph in `src/data.js`, and its card text as `upgrade.<id>.name` and `upgrade.<id>.desc` in every language in `src/i18n.js`.
- New text: add a key to every language in `src/i18n.js`, then use `t(key)` in code or a `data-i18n` attribute in `index.html`. Missing keys fall back to English.
- New language: add its code to `LANGS` and a table to `STRINGS` in `src/i18n.js`, and to start new players in it, set `DEFAULT_LANG`.
- New part shape: add a part id in `defs.rs`, a `case` in `shape()` in `src/shader.wgsl`, and a name in `PART_IDS` in `src/data.js`.

## What is in the backbone

- Four enemy kinds (blot, wisp, brute, elite every 90 seconds) with time-scaled health and spawn rate, capped at 500 on screen
- Four weapons (scythe, talisman, jade beads, spirit pulse) and six passive upgrades, with level-up choices
- Experience gems, health drops, pickup magnet, knockback, invulnerability frames, camera shake
- Title, settings, level-up, pause and game-over screens, playable by keyboard or gamepad
- Saved best time, kills, level, sound and language settings
- Traditional Chinese (the default) and English text, switchable in settings

Not included yet: touch controls, damage numbers, music, meta progression between runs, and a WebGL fallback for browsers without WebGPU.

The ranked plan for what comes next is in [ROADMAP.md](ROADMAP.md).
