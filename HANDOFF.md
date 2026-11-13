# Handoff

## In progress

### ci/pages-deploy

- Task: roadmap step 5, starting with a GitHub Pages deploy workflow that publishes the repository as is (the game is static files plus the committed wasm and fonts). The user chose GitHub Pages and no CDN for fonts. No issue yet; roadmap issues wait on P-02. Local branch only, created from `main` at b1b522d to carry this handoff.
- PR: none yet.
- Code checkpoint: b1b522d (`main`, with #4 merged: bundled Chinese UI fonts and the `Fonts` CI check).
- State: not started.
- Next action: plan the workflow with the user per the `open-pr` skill. Open questions: Pages source (GitHub Actions deploy vs. branch), whether the deploy reruns `tools/check-fonts.sh` first, which files to leave out of the site (`tools/`, `core/`, docs), and the repository's Pages settings, which need the user's approval to change.
- Pending: **P-05** push this branch. Why: pushing needs approval. Blocks: resuming from another computer.

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
