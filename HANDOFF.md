# Handoff

## In progress

### feat/lightbox

- Task: add `tools/lightbox.html`, the tool for tracing paper drawings into character JSON described in `docs/character-art-pipeline.md` and decision 0003 (#32). No issue: the maintainer chose to skip one. Not linked to any issue.
- PR: opened with the `open-pr` skill, awaiting review and merge.
- Code checkpoint: the Lightbox commit on `main` at 16a530e.
- Verification: `npm run smoke` ok; headless Chrome scripts (not committed) ran all 7 steps on the three first-batch drawings, brushes with undo and redo, a two-part export and a preset round trip, with no errors. Not run: `cargo test`, `./build.sh` (no change to `core/`); a pass with a real mouse or tablet.
- Next action: after this PR merges, the load-time bake in `src/renderer.js` from decision 0003 is the next piece of the pipeline.

## Ranked work

Roadmap steps are detailed in `ROADMAP.md`. They become issues once P-02 is approved.

1. Roadmap step 5, the rest of CI: the wasm drift check, cargo fmt and clippy, and a JavaScript check. The Pages deploy (#5) and the smoke test (#24) are done. Planned on the local branch `ci/core-checks`, which needs an `accepted` Proposal first (see that branch's own `HANDOFF.md`)
2. Decide which areas need the maintainer's sign-off before contributors build on them, made on an issue and recorded in `docs/decisions/`. Blocks #14 (`blocked`)
3. #3: pick English UI fonts to match the Chinese set (`accepted`; can run alongside item 1)
4. #16: run the Fonts workflow only when font tools change (`accepted`, small)
5. #18: smoke test connects to 127.0.0.1 instead of localhost (`accepted`, small)
6. #27: restructure the README and roadmap into focused pages (`accepted`)
7. Roadmap step 1: data-driven content and puppet rigs (unblocks most later steps)
8. Roadmap step 2: game feel pass (can run alongside step 3)
9. Roadmap step 3: fixed time step, replays and a headless balance tool
10. Roadmap step 4: run structure and meta progression (after step 1)
11. Roadmap step 6: touch controls and a WebGL2 fallback
12. Roadmap step 7: scale to thousands of enemies

## Pending

- **P-02** File one GitHub issue per roadmap step (seven issues, titles as in `ROADMAP.md`, body copied from each step's section, step 1 to 4 in a "Playable game" milestone). Under `CONTRIBUTING.md` they would go in as Proposals, labelled `enhancement` and `proposal`. Why: filing issues needs approval. Blocks: nothing else; `ROADMAP.md` covers the details until then.
- **P-10** Delete the local backup branches `backup/contribution-flow-v1` to `-v7`, earlier rebuilds of the merged #20. Why: they hold older drafts not in any surviving branch. Blocks: nothing.
- **P-16** Check the Auto-label workflow from #31 on `main`: file two test issues with `gh issue create` and no labels, one with a `### What to change` body (expect `enhancement` and `proposal`) and one with a plain body (expect `needs-triage`), then close both as not planned. Why: filing issues needs approval. Blocks: confirming #30 works on GitHub.
