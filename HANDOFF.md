# Handoff

## In progress

### feat/lightbox

- Task: add `tools/lightbox.html`, the tool for tracing paper drawings into character JSON described in `docs/character-art-pipeline.md` and decision 0003 (#32). The maintainer wants to iterate on it more before #34 merges. Decision 0004 (on this branch) serves it from the local server at `/tools/lightbox.html` via `LOCAL_ONLY` in `tools/serve.mjs`, and keeps it static so it can be published on Pages later. No issue: the maintainer chose to skip one. Not linked to any issue.
- PR: #34, open and awaiting review and merge. Auto-fix is on for it in the desktop app.
- Code checkpoint: 0f65b00 (`feat(tools): serve the lightbox from the local server`), on `main` at 16a530e.
- Verification at 0f65b00: `npm test` and `npm run smoke` ok; the smoke test now also opens the Lightbox, and fails when it is not served (checked). Earlier, at 3f50c70: `npm run smoke` ok; headless Chrome scripts (Playwright, not committed) ran all 7 steps on the three first-batch drawings, brushes with undo and redo, a two-part export and a preset round trip (identical JSON), with no errors. Not run: `cargo test`, `./build.sh` (no change to `core/`); a pass with a real mouse or tablet.
- Learned: automatic background removal on phone photos is not reliable, which is why every mask step has a brush. Solid ink is decided before tracing, from the sharp ink mask, and must be clearly deeper than a line; otherwise thick joints and doubled pencil lines turn into blobs. Removing doubled strokes must leave short joint and crossing overlaps alone, or lines break into dashes. The artist's first batch (three photos) is not in the repository.
- Next action: keep iterating on the Lightbox with the maintainer; once #34 merges, file P-17 and start the load-time bake. P-17 should move the jitter into `src/` and have the Lightbox import it (decision 0004).

## Ranked work

Roadmap steps are detailed in `ROADMAP.md`. They become issues once P-02 is approved.

1. Roadmap step 5, the rest of CI: the wasm drift check, cargo fmt and clippy, and a JavaScript check. The Pages deploy (#5) and the smoke test (#24) are done. Planned on the local branch `ci/core-checks`, which needs an `accepted` Proposal first (see that branch's own `HANDOFF.md`)
2. Decide which areas need the maintainer's sign-off before contributors build on them, made on an issue and recorded in `docs/decisions/`. Blocks #14 (`blocked`)
3. Character art: bake character JSON into the part atlas at load time, per decision 0003, after #34 merges. Its issue is drafted as P-17
4. #27: restructure the README and roadmap into focused pages (`accepted`; its three open questions are unanswered on the issue)
5. #3: pick English UI fonts to match the Chinese set (`accepted`; can run alongside item 1)
6. #16: run the Fonts workflow only when font tools change (`accepted`, small)
7. #18: smoke test connects to 127.0.0.1 instead of localhost (`accepted`, small)
8. Roadmap step 1: data-driven content and puppet rigs (unblocks most later steps)
9. Roadmap step 2: game feel pass (can run alongside step 3)
10. Roadmap step 3: fixed time step, replays and a headless balance tool
11. Roadmap step 4: run structure and meta progression (after step 1)
12. Roadmap step 6: touch controls and a WebGL2 fallback
13. Roadmap step 7: scale to thousands of enemies

## Pending

- **P-02** File one GitHub issue per roadmap step (seven issues, titles as in `ROADMAP.md`, body copied from each step's section, step 1 to 4 in a "Playable game" milestone). Under `CONTRIBUTING.md` they would go in as Proposals, labelled `enhancement` and `proposal`. Why: filing issues needs approval. Blocks: nothing else; `ROADMAP.md` covers the details until then.
- **P-10** Delete the local backup branches `backup/contribution-flow-v1` to `-v7`, earlier rebuilds of the merged #20. Why: they hold older drafts not in any surviving branch. Blocks: nothing.
- **P-16** Check the Auto-label workflow from #31 on `main`: file two test issues with `gh issue create` and no labels, one with a `### What to change` body (expect `enhancement` and `proposal`) and one with a plain body (expect `needs-triage`), then close both as not planned. Why: filing issues needs approval. Blocks: confirming #30 works on GitHub.
- **P-17** File a Proposal issue, labelled `enhancement` and `proposal`. Why: filing issues needs approval. Blocks: ranked item 3. Draft:
  - Title: Bake character art into the part atlas at load time
  - `### What to change`: Load each character JSON from `assets/characters/` at start-up and, in `src/renderer.js`, draw each part's boil poses (3 by default) with Canvas2D into the existing part atlas: paper fill, solid ink, then strokes of the part's single width, each stroke with its own seeded wobble per pose, using the same jitter as `tools/lightbox.html`. Add a rect per part per pose, and let the core pick the pose on an 8 fps cycle with `part_id = base + pose`, so the 12-float instance format does not change.
  - `### Why`: Decision 0003 (#32) chose this so hand-traced characters get a per-stroke line boil while each part stays one textured quad per frame. Nothing in the game reads character JSON yet.
  - `### Reference`: `docs/character-art-pipeline.md`, `docs/decisions/0003-character-art-pipeline.md`, #34 (the Lightbox). Puppet rigs and per-part pivots come with roadmap step 1.
