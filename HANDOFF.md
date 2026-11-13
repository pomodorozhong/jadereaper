# Handoff

## In progress

### feat/game-backbone

- Task: build the survivor game backbone (Rust/WASM simulation, WebGPU paper renderer, paper UI, gamepad, sound, saves) and plan the next steps. No issue: the repository has no GitHub remote yet.
- PR: none. There is no remote to push to.
- Code checkpoint: c73ab1a
- State: backbone complete and playable at http://localhost:5173 after `python3 -m http.server 5173`. `ROADMAP.md` ranks seven next steps with scope, definition of done and risks. All work is committed on this branch; `main` holds only the `.gitignore` commit.
- Verification: `cargo test --release --manifest-path core/Cargo.toml` passed (2 tests, including a simulated 10-minute run). In the browser pane: title, combat, level-up, pause, settings, game over, saved records and a part image override all worked with no console errors; frame time about 8.3 ms median at roughly 250 enemies.
- Not checked: gamepad on real hardware, how the sound effects actually sound, balance with a human player (only tuned against a bot that circles).
- Next action: start roadmap step 1 (data-driven content and puppet rigs) on a new branch from this one, or merge this branch into `main` first if the user prefers.
- Pending: **P-01** create a GitHub repository, push `main` and this branch, and open a PR from this branch into `main`. Why: creating a remote, pushing and opening a PR need approval. Blocks: issues for the roadmap steps, roadmap step 5 (CI and deploy), resuming from another computer.

## Ranked work

Steps are detailed in `ROADMAP.md`. They become issues once P-01 gives the repository a remote.

1. Roadmap step 1: data-driven content and puppet rigs (unblocks most later steps)
2. Roadmap step 2: game feel pass (can run alongside step 3)
3. Roadmap step 3: fixed time step, replays and a headless balance tool
4. Roadmap step 4: run structure and meta progression (after step 1)
5. Roadmap step 5: CI and one-command deploy (blocked by P-01)
6. Roadmap step 6: touch controls and a WebGL2 fallback
7. Roadmap step 7: scale to thousands of enemies

## Pending

- **P-02** File one GitHub issue per roadmap step (seven issues, titles as in `ROADMAP.md`, body copied from each step's section, label `enhancement`, step 1 to 4 in a "Playable game" milestone). Why: filing issues needs approval and a remote. Blocks: nothing else; `ROADMAP.md` covers the details until then.
