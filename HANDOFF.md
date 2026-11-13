# Handoff

## In progress

### feat/zh-ui-fonts

- Task: bundled fonts for the Traditional Chinese UI (Bakudai for titles, buttons and brush marks; LXGW WenKai TC for body text) and Caveat Brush for numbers in both languages. Fonts were chosen with the user in a throwaway gallery (`font-gallery.html`, untracked). The English font choice is issue #3. No issue for this branch.
- PR: drafted and awaiting the user's approval to push (P-04).
- Code checkpoint: a49f10a
- State: `assets/fonts/` holds the four fonts cut down to the game's characters (446 KB in total, from about 41 MB) with their OFL licenses. `tools/subset-fonts.sh` rebuilds them reproducibly from pinned, checksummed sources; `tools/check-fonts.sh` fails when a Chinese character in `src/*.js` or `index.html` is missing, and `.github/workflows/fonts.yml` runs it in CI. `style.css` adds a `--title` variable and `:lang(zh-Hant)` overrides; the English UI is unchanged apart from numbers.
- Verification: `cargo test --release` passed (2 tests) and `./build.sh` left the wasm unchanged. The check script passed, and failed as expected with an uncovered test character. Two subset runs gave identical bytes (0.5 s with a warm cache, about 9 s with downloads). In the browser pane: title, HUD, level-up cards, game over, settings and pause in Chinese, and level-up in English, showed the intended fonts with no console errors. Level-up, game over and settings were rendered by calling the UI module directly.
- Not checked: the CI workflow on GitHub (runs once pushed); other browsers and Windows or Android.
- Known: Bakudai's 繼 and 續 (pause screen "繼續") are thinner and plainer than its other characters, likely among the maintainer's later additions.
- Next action: after P-04 is approved, push and open the PR; then the GitHub Pages deploy PR, or roadmap step 1.
- Pending: **P-04** push this branch and open the PR "feat(web): bundle fonts for the chinese ui and numbers" into `main`. Why: pushing and opening a PR need approval. Blocks: shipping the Chinese fonts, and the GitHub Pages deploy PR.

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
