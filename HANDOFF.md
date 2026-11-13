# Handoff

## In progress

### feat/i18n-zh

- Task: Traditional Chinese (zh-Hant) language support for all player-facing text, with a language button in settings. No issue.
- PR: drafted and awaiting the user's approval to push (P-03).
- Code checkpoint: 912daa5
- State: all UI text, upgrade names and descriptions, and WebGPU error messages come from `src/i18n.js`. New players start in Traditional Chinese (`DEFAULT_LANG`), whatever the browser prefers; the settings button cycles 繁體中文 and English and saves the choice as `lang` in the save.
- Verification: `cargo test --release` passed (2 tests) and `./build.sh` left the wasm unchanged. In the browser pane: a fresh save starting in Chinese on an English-first browser, a saved English choice still loading in English, title, settings, language switch, the choice surviving a reload, HUD, level-up cards, game over, records and pause all showed correct text in both languages, with no console errors. Level-up and game over were rendered by calling the UI module directly, because the hidden pane stalls the game loop.
- Not checked: a full real run in Chinese, and the translation by a native speaker.
- Next action: after P-03 is approved, push and open the PR; then start roadmap step 1 on a new branch from `main`.
- Pending: **P-03** push this branch and open the PR "feat(web): add traditional chinese language support" into `main`. Why: pushing and opening a PR need approval. Blocks: merging the translation.

## Ranked work

Steps are detailed in `ROADMAP.md`. They become issues once P-02 is approved.

1. Roadmap step 1: data-driven content and puppet rigs (unblocks most later steps)
2. Roadmap step 2: game feel pass (can run alongside step 3)
3. Roadmap step 3: fixed time step, replays and a headless balance tool
4. Roadmap step 4: run structure and meta progression (after step 1)
5. Roadmap step 5: CI and one-command deploy
6. Roadmap step 6: touch controls and a WebGL2 fallback
7. Roadmap step 7: scale to thousands of enemies

## Pending

- **P-02** File one GitHub issue per roadmap step (seven issues, titles as in `ROADMAP.md`, body copied from each step's section, label `enhancement`, step 1 to 4 in a "Playable game" milestone). Why: filing issues needs approval. Blocks: nothing else; `ROADMAP.md` covers the details until then.
