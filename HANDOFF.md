# Handoff

## In progress

### ci/pages-deploy

- Task: roadmap step 5, first part: a GitHub Pages deploy workflow (`.github/workflows/pages.yml`) that runs `tools/check-fonts.sh`, then publishes `index.html`, `style.css`, `src/`, `wasm/` and `assets/` as committed, on every push to `main` and by hand. The README gets a "Play it online" link. No issue; roadmap issues wait on P-02. Not linked to any issue.
- PR: none yet; title and body drafted and awaiting publish approval (P-05).
- Code checkpoint: b5b607f (`ci: deploy the game to github pages on push to main`), on `main` at b1b522d.
- Repository settings: Pages already deploys from GitHub Actions, and the `github-pages` environment only allows `main`, so nothing needs changing. The real deploy can only be tested after merge.
- Verification at b5b607f: `cargo test --release` passed; `tools/check-fonts.sh` passed; actionlint (via `uvx --from actionlint-py`) passed on both workflows; the workflow's copy step, served locally under `/jadereaper/`, loaded every file and started a run with no console errors (only `/favicon.ico` 404s, as on `main`).
- Still to do for step 5 (later PRs): wasm drift check, cargo fmt and clippy, JavaScript check, headless smoke test.
- Next action: on approval of P-05, push and open the PR, then check the deploy after it merges.
- Pending: **P-05** push this branch and open the PR "ci: deploy the game to github pages on push to main". Why: pushing and opening a PR need approval. Blocks: the PR, the first deploy, and resuming from another computer.

### docs/game-design

- Task: add `docs/design.md` (the game design, in Chinese, from the team's Discord discussion and four rounds of questions with the user) and rewrite `ROADMAP.md` steps 1 and 4 to follow it. Not linked to any issue.
- PR: none yet; not pushed.
- Code checkpoint: 9a8849a (`docs: add game design document and align roadmap`), on `main` at 80ca4a1.
- Verification: documentation only; no checks were run.
- Open design questions are listed in `docs/design.md` section 9; decisions that differ from the original discussion are in section 11 and need the team's confirmation.
- Next action: on approval of P-06, push and open the PR. Then split roadmap steps 1 and 4 into plan issues (they are now too large for one PR each), starting with the martial art and element axes.
- Pending: **P-06** push this branch and open the PR "docs: add game design document and align roadmap". Why: pushing and opening a PR need approval. Blocks: teammates reading the design on GitHub.
- Note: the `ci/pages-deploy` subsection above is stale; its PR merged as #5, so P-05 is obsolete.

## Ranked work

Steps are detailed in `ROADMAP.md`. They become issues once P-02 is approved.

1. Roadmap step 5: CI and one-command deploy, starting with a GitHub Pages deploy workflow that publishes the repository as is and may rerun `tools/check-fonts.sh` first (the user chose GitHub Pages and wants this as the PR after #4)
2. #3: pick English UI fonts to match the Chinese set (can run alongside item 1)
3. Roadmap step 1: data-driven content and puppet rigs (unblocks most later steps)
4. Roadmap step 2: game feel pass (can run alongside step 3)
5. Roadmap step 3: fixed time step, replays and a headless balance tool
6. Roadmap step 4: run structure and meta progression (after step 1)
7. Roadmap step 6: touch controls and a WebGL2 fallback
8. Roadmap step 7: scale to thousands of enemies

## Pending

- **P-02** File one GitHub issue per roadmap step (seven issues, titles as in `ROADMAP.md`, body copied from each step's section, label `enhancement`, step 1 to 4 in a "Playable game" milestone). Why: filing issues needs approval. Blocks: nothing else; `ROADMAP.md` covers the details until then.
