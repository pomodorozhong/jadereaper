# Jade Reaper

A room-based action game running natively in the browser: pick three skills, fight through ink-spirit rooms with a roll and well-timed attacks, collect element cards, and bring down two bosses. [ROADMAP.md](ROADMAP.md) has the plan. The simulation is Rust compiled to WebAssembly, the picture is drawn with WebGPU, and everything on screen is styled as cut paper and ink.

## Play it online

https://pomodorozhong.github.io/jadereaper/ (needs a browser with WebGPU). Every push to `main` republishes it through the `Pages` workflow in `.github/workflows/pages.yml`, which checks the fonts and then deploys `index.html`, `style.css`, `src/`, `wasm/` and `assets/` as they are committed.

## Run it

Serve the folder with any static file server and open it in a browser with WebGPU (current Chrome, Edge or Safari). WebGPU needs `http://localhost` or `https`, so opening `index.html` from disk will not work. The repository has a small one with no dependencies (Node 26):

```bash
npm start
```

Then open http://localhost:5173. The same server has the Lightbox, the tool for tracing character art, at http://localhost:5173/tools/lightbox.html (see [docs/character-art-pipeline.md](docs/character-art-pipeline.md)).

The built `wasm/jadereaper_core.wasm` is checked in, so Rust is only needed if you change the core.

To check that the page loads cleanly, run the smoke test. It loads the game in headless Chromium and fails on any console error or failed request; without WebGPU it checks the "Cannot start" screen instead. CI runs it on every pull request.

```bash
npm ci
npx playwright install chromium
npm run smoke
```

## Controls

| Action | Keyboard | Gamepad |
| --- | --- | --- |
| Move | WASD or arrow keys | Left stick or d-pad |
| Aim | Mouse | Right stick |
| Skills | Left button, right button, Q | X, Y, RB |
| Roll | Space | A or B |
| Pause | Esc or P | Start |
| Menu move | Arrow keys or WASD | Stick or d-pad |
| Menu confirm | Enter or Space | A |
| Back | Esc | B |
| Pick a card | 1, 2, 3 | Stick, then A |
| Mute | M | |

Skills have no cooldowns: each has a startup, an active window and a recovery, and a roll can cut the recovery short after its cancel point. The roll is a short dodge with a moment of invulnerability; mash it to break a grab. Enemies warn every attack with an ink wash on the ground. The game pauses itself when the window loses focus.

## How the paper style works

Characters are puppets made of rigid paper parts, not sprite sheets, placed by rigs in `content/game.json`. The reaper is a robe, a hood and a scythe. A walk cycle is two poses: tilt one way, then tilt the other with a small hop. Each rig also has wind-up, attack, hurt and roll poses; the roll tucks and spins the cutout. This needs one still image per part and no animation frames.

The renderer adds the rest to every part: an ink outline, a white cutout margin, a drop shadow and paper grain. Hand-traced characters get a per-stroke line boil like stop-motion, 4 poses a second; see [docs/character-art-pipeline.md](docs/character-art-pipeline.md). Parts are drawn in code as signed distance shapes, and any part can be swapped for your own PNG. See [assets/parts/README.md](assets/parts/README.md).

The UI is HTML and CSS over the canvas: torn-edge paper sheets, tags and cards, a hand-written font and a vermilion seal, using two SVG filters defined in `index.html`.

## Layout

| Path | What it is |
| --- | --- |
| `core/` | Rust simulation. No dependencies, no bindgen. Its modules are described in [docs/architecture.md](docs/architecture.md). |
| `core/src/content.rs` | Generated from `content/game.json`: part ids, events, rigs, enemy kinds, skills, cards, rooms, moves |
| `core/src/lib.rs` | The `jr_*` functions exported to JavaScript, and tests |
| `core/tests/` | Simulation tests, including whole runs played by the scripted bot in `core/src/bot.rs` |
| `content/game.json` | All game content, in one file |
| `wasm/` | Built core |
| `src/main.js` | Loads the core, frame loop, game modes |
| `src/renderer.js` | WebGPU setup, part image atlas, drawing |
| `src/characters.js` | Bakes the hand-traced characters into part images |
| `src/boil.js` | The line boil shared with the Lightbox |
| `src/shader.wgsl` | Paper shapes, cutout shading, paper background |
| `src/input.js` | Keyboard and gamepad |
| `src/audio.js` | Synthesized sound effects (no audio files) |
| `src/ui.js` | Screens, HUD, menu focus |
| `src/save.js` | Records and settings in localStorage |
| `src/data.js` | State indices and button bits, and the generated tables from `src/content.js` |
| `src/content.js`, `src/content.wgsl` | Generated from `content/game.json` |
| `assets/parts.json` | Optional image overrides for parts |
| `assets/fonts/` | Bundled UI fonts, cut down to the game's characters, with their licenses |
| `tools/` | The content generator, font subset and check scripts, the local server, the smoke test and the Lightbox |

## Fonts

The Traditional Chinese UI uses bundled fonts: [Bakudai](https://github.com/max32002/bakudaifont) (莫大毛筆) for titles, buttons and brush marks, and [LXGW WenKai TC](https://github.com/lxgw/LxgwWenkaiTC) for body text. [Caveat Brush](https://fonts.google.com/specimen/Caveat+Brush) draws numbers in every language. The English UI still uses system fonts (issue #3). All three fonts are under the SIL Open Font License 1.1; the license texts are in `assets/fonts/`.

The full fonts are 12 to 16 MB each, so `assets/fonts/` holds copies cut down to only the characters the game shows, about 270 KB in total. After changing any Chinese text, rebuild them and commit `assets/fonts/`:

```bash
npm ci
tools/subset-fonts.sh
```

The scripts need Node 26 (see `.node-version`) and the dev dependencies that `npm ci` installs into `node_modules/`; the game itself needs neither. `tools/subset-fonts.sh` collects every Chinese character in `src/*.js` and `index.html`, downloads the pinned source fonts into `.font-cache/`, and writes the woff2 files. `tools/check-fonts.sh` checks the committed fonts against the text without downloading anything, and CI runs it on every pull request.

## How core and host talk

Each frame the host calls `jr_input(mx, my, aim_x, aim_y, buttons)`, then `jr_update(dt)`, and reads three buffers out of wasm memory. The core simulates in fixed 60 Hz steps and keeps the rest of `dt` for the next frame, so a seed and its inputs replay the same run (`jr_state_hash` checks it).

- Instances: 12 floats per paper part (`x, y, half width, half height, rotation, part id, flash, seed, r, g, b, a`). These bytes go to the GPU unchanged.
- State: 64 floats for the HUD (health, room and wave, offered cards, the boss's health and stance, and so on). Indices are in `src/data.js` under `STATE`, and the layout is documented on `Game::write_state` in `core/src/game.rs`.
- Events: 4 floats each (`type, x, y, value`), used for sound, damage numbers and reaction names.
- Animation state: 16 floats per character, player first (`jr_anims_ptr`, `jr_anim_count`): what its body is doing (startup, active, recovery, roll, hurt and so on), how far through it, the cancel point, the action's direction, and the last hit's direction, damage and age. The layout is documented in `core/src/anim.rs` and mirrored in `src/data.js` under `ANIM`. Nothing draws from it yet; it is where the drawing side will pose characters in 3D ([0006](docs/decisions/0006-paper-puppets-in-3d.md)). `jadereaper.anims()` reads it in the console.

A run starts with `jr_start(seed)`, `jr_set_loadout(a, b, c)` and `jr_offer_passive()`. When the core sets the "choosing" flag the host shows the offered cards and answers with `jr_choose(slot)`.

## Build the core

```bash
rustup target add wasm32-unknown-unknown
```

```bash
./build.sh
```

`./build.sh` first runs `node tools/content.mjs`, which checks `content/game.json` and writes the generated tables for Rust, JavaScript and WGSL. Commit them with the content change; `node tools/content.mjs --check` (and `npm test`) fails when they are stale.

Run the simulation tests natively:

```bash
cargo test --release --manifest-path core/Cargo.toml
```

## Extending

Most content is data in `content/game.json`; run `./build.sh` after changing it.

- New skill: add it to `skills` (timing, damage, stagger, hit area, hyper armor, movement), and its text as `skill.<name>.name` and `.desc` in every language in `src/i18n.js`. A new kind of delivery needs a `SkillKind` in `core/src/skills.rs`.
- New card: add it to `cards` (what it attaches to, an element, an effect), and its text as `card.<name>.name` and `.desc`.
- New enemy: add a rig to `rigs`, its moves to `moves` (wind-up at least 0.4 s), a kind to `kinds`, and put it in a room's waves.
- New boss move: add it to `moves` with its weight, cooldown, phases and special, and list it on the boss's kind. A new special needs code in `core/src/boss.rs` or `core/src/dragon.rs`.
- New room: add it to `rooms`, before the boss rooms.
- New text: add a key to every language in `src/i18n.js`, then use `t(key)` in code or a `data-i18n` attribute in `index.html`. Missing keys fall back to English.
- New language: add its code to `LANGS` and a table to `STRINGS` in `src/i18n.js`, and to start new players in it, set `DEFAULT_LANG`.
- New part shape: add a part to `parts` in `content/game.json` and a `case P_<NAME>` in `shape()` in `src/shader.wgsl`; a part without a case is drawn as an ellipse.

## What is in the game

- Aim with the mouse or the right stick, three skills on three buttons, and a roll, with a 120 ms input buffer
- Eight skills, light to heavy, balanced by timing instead of cooldowns; heavy skills have hyper armor
- Hit areas with shapes, hit-stop, knockback, stagger and damage numbers, and a setting that turns off shake and hit-stop
- Four enemy kinds that wind up behind a ground warning, at most 10 at once
- One card pool: a starting passive, a card after each room, four elements (fire, ice, lightning, celestial) and six reactions between them
- A run of four rooms of waves, then two bosses: the Glutton, which enrages, and the Two-Element Dragon, which changes stance and has an element check
- Puppet rigs posed from data, saves with a version and a migration, Traditional Chinese (the default) and English text

Not included yet: music, meta progression between runs, more characters, touch controls, and a WebGL fallback for browsers without WebGPU. The bosses are flat placeholder shapes until their art is traced.

The ranked plan for what comes next is in [ROADMAP.md](ROADMAP.md).
