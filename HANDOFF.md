# Handoff

## In progress

### docs/handoff-refresh

- Task: carry this handoff to `main` after #26 merged. Docs only (`HANDOFF.md`); no issue, not linked to any issue. No code work is in progress on any pushed branch; the next task is picked from `Ranked work`.
- PR: none yet.
- Code checkpoint: a0d53d1 (`main`, with #26 merged).
- Verification: none needed for a docs-only change; `main` at a0d53d1 passed CI.
- Next action: P-14.
- Pending: **P-14** open a PR for this branch with the `open-pr` skill. Why: opening a PR needs approval. Blocks: this handoff reaching `main`.

## Ranked work

Roadmap steps are detailed in `ROADMAP.md`. They become issues once P-02 is approved.

1. Roadmap step 5, the rest of CI: the wasm drift check, cargo fmt and clippy, and a JavaScript check. The Pages deploy (#5) and the smoke test (#24) are done. Planned on the local branch `ci/core-checks`, which needs an `accepted` Proposal first (see that branch's own `HANDOFF.md`)
2. Decide which areas need the maintainer's sign-off before contributors build on them, made on an issue and recorded in `docs/decisions/`. Blocks #14 (`blocked`)
3. #3: pick English UI fonts to match the Chinese set (`accepted`; can run alongside item 1). An untracked `font-gallery.html` at the repository root of the main checkout is a throwaway font comparison page; keep it out of commits
4. #16: run the Fonts workflow only when font tools change (`accepted`, small)
5. #18: smoke test connects to 127.0.0.1 instead of localhost (`accepted`, small)
6. Roadmap step 1: data-driven content and puppet rigs (unblocks most later steps)
7. Roadmap step 2: game feel pass (can run alongside step 3)
8. Roadmap step 3: fixed time step, replays and a headless balance tool
9. Roadmap step 4: run structure and meta progression (after step 1)
10. Roadmap step 6: touch controls and a WebGL2 fallback
11. Roadmap step 7: scale to thousands of enemies

## Pending

- **P-02** File one GitHub issue per roadmap step (seven issues, titles as in `ROADMAP.md`, body copied from each step's section, step 1 to 4 in a "Playable game" milestone). Under `CONTRIBUTING.md` they would go in as Proposals, labelled `enhancement` and `proposal`. Why: filing issues needs approval. Blocks: nothing else; `ROADMAP.md` covers the details until then.
- **P-10** Delete the local backup branches `backup/contribution-flow-v1` to `-v7`, earlier rebuilds of the merged #20. Why: they hold older drafts not in any surviving branch. Blocks: nothing.
