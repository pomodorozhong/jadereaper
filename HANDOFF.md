# Handoff

## In progress

### feat/prototype-v1

- Task: prototype v1, the survivor game backbone (Rust/WASM simulation, WebGPU paper renderer, paper UI, gamepad, sound, saves) and plan the next steps. Branched from `feat/game-backbone` (same commits) so the PR carries the prototype v1 name. No issue: the repository has no GitHub remote yet.
- PR: none yet; drafted and awaiting approval (P-01).
- Code checkpoint: 9c6fe52
- State: backbone complete and playable at http://localhost:5173 after `python3 -m http.server 5173`. `ROADMAP.md` ranks seven next steps with scope, definition of done and risks. All work is committed on this branch; `main` holds only the `.gitignore` commit.
- Verification: on 2026-11-13 at 9c6fe52, `cargo test --release` passed (2 tests) and `./build.sh` rebuilt the wasm with no changes. Earlier, `cargo test --release --manifest-path core/Cargo.toml` passed (2 tests, including a simulated 10-minute run). In the browser pane: title, combat, level-up, pause, settings, game over, saved records and a part image override all worked with no console errors; frame time about 8.3 ms median at roughly 250 enemies.
- Not checked: gamepad on real hardware, how the sound effects actually sound, balance with a human player (only tuned against a bot that circles).
- Next action: once P-01 is approved, create the repository, push, and open the PR; then start roadmap step 1 on a new branch.
- Pending: **P-01** create a GitHub repository (visibility to be chosen), push `main` and this branch, and open the PR "feat: prototype v1 of the survivor game" from this branch into `main`. Why: creating a remote, pushing and opening a PR need approval. Blocks: issues for the roadmap steps, roadmap step 5 (CI and deploy), resuming from another computer.

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
