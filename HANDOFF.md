# Handoff

## In progress

### feat/lightbox

- Task: add `tools/lightbox.html`, the tool for tracing paper drawings into character JSON described in `docs/character-art-pipeline.md` and decision 0003 (#32). No issue: the maintainer chose to skip one. Not linked to any issue.
- PR: #34, open. The maintainer wants to keep iterating on the Lightbox before it merges. Auto-fix is on for it in the desktop app. Its description does not yet mention the stroke joining, the burr removal or the new defaults (95a8f04, 7547ed0); P-18 asks to update it.
- Code checkpoint: 7547ed0 (`fix(tools): remove burrs from thick, wavering pencil lines in the lightbox`), on `main` at 16a530e.
- State: the Lightbox is served by `npm start` at `/tools/lightbox.html` (`LOCAL_ONLY` in `tools/serve.mjs`; decision 0004 says why, and the rules that keep it publishable on Pages later). Its eight steps trace the whole character once (Source to Strokes), then split it on the Parts step with lassos (a stroke crossing a lasso is cut, outside every lasso goes to the first part, the higher draw order wins on overlap) and a per-part paper fill brush for areas hidden behind other parts. Presets hold one settings object plus each part's `marks` (lassos and fill strokes) and pivot; older per-part presets open with the first part's settings. The character JSON format is unchanged.
- Decided by the maintainer: serve the Lightbox locally (option B) and keep the move to Pages (option C) cheap; split parts after tracing, with lasso regions and a per-part fill brush. On 2026-11-16, after tracing IMG_1617 by hand: one continuous line is preferred over several short strokes, and smooth, confident lines over a copy of the pencil's wobble. The defaults now come from that preset, a join pass links stroke pieces that meet or carry on in the same direction, a burr pass then removes short ticks, short doubled pieces and overshooting tails from ragged lines (lengths in line widths), and the boil is 4 fps, 6 poses, strength 0.9 (the pipeline doc and P-17 follow).
- Verification at 5e42bcd (still holds for the parts flow): `npm test` and `npm run smoke` ok (the smoke test also opens the Lightbox, and fails when it is not served). In the browser pane, on a synthetic drawing: lasso split, fill brush with undo and redo, boil preview, export, a saved-preset round trip (identical JSON) and opening an older per-part preset. Not run: the artist's photos; a real mouse or tablet (the lasso and brush were driven by dispatched pointer events); `cargo test` and `./build.sh` (no change to `core/`).
- Verification at 95a8f04: `npm test` and `npm run smoke` ok. In the browser pane, on the first-batch photos IMG_1616 to IMG_1618 (in the maintainer's Downloads, not the repository): the new background defaults (30 / 150) clean all three; on IMG_1617 the maintainer's preset traces 52 strokes instead of 135, and the new defaults with no brush edits 63 instead of 143; the preset's two parts still split and export. Not run: a real mouse or tablet; `cargo test` and `./build.sh` (no change to `core/`).
- Verification at 7547ed0: `npm test` and `npm run smoke` ok. In the browser pane, the burr pass takes the defaults on IMG_1617 from 63 strokes to 48 and on IMG_1616 from 123 to 97, removing the burrs the maintainer pointed out on the left hair and the right ear while keeping eyes, nose, mouth and small shapes. Scripted checks must merge `DEFAULTS` under an older preset's settings, as "Open preset" does: without it `joinGap` is undefined and the trace goes wrong.
- Learned: automatic background removal on phone photos is not reliable, which is why every mask step has a brush. Solid ink is decided before tracing, from the sharp ink mask, and must be clearly deeper than a line; otherwise thick joints and doubled pencil lines turn into blobs. Removing doubled strokes must leave short joint and crossing overlaps alone, or lines break into dashes. The artist's first batch (three photos) is not in the repository. `window.__lightbox` exposes the state and `loadFile` for scripted checks.
- Next action: the maintainer re-traces IMG_1617 with the new defaults and says what still needs work. Planned after that, as agreed: faster fixes for fill and guide lines, such as clicking a closed area to mark it empty or clicking a stroke to delete it, since the IMG_1617 preset needed 65 ink erasures and 52 fill edits. Then, once #34 merges, file P-17 and start the load-time bake, moving the jitter into `src/` so the Lightbox imports it (decision 0004).

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
- **P-18** Update the description of #34: add the stroke join and burr passes and the pencil defaults to "What changed" (Strokes bullet), and the IMG_1617 checks to "Verification". Why: editing a PR description publishes it. Blocks: nothing.
- **P-17** File a Proposal issue, labelled `enhancement` and `proposal`. Why: filing issues needs approval. Blocks: ranked item 3. Draft:
  - Title: Bake character art into the part atlas at load time
  - `### What to change`: Load each character JSON from `assets/characters/` at start-up and, in `src/renderer.js`, draw each part's boil poses (6 by default) with Canvas2D into the existing part atlas: paper fill, solid ink, then strokes of the part's single width, each stroke with its own seeded wobble per pose, using the same jitter as `tools/lightbox.html`. Add a rect per part per pose, and let the core pick the pose on a 4 fps cycle with `part_id = base + pose`, so the 12-float instance format does not change.
  - `### Why`: Decision 0003 (#32) chose this so hand-traced characters get a per-stroke line boil while each part stays one textured quad per frame. Nothing in the game reads character JSON yet.
  - `### Reference`: `docs/character-art-pipeline.md`, `docs/decisions/0003-character-art-pipeline.md`, #34 (the Lightbox). Puppet rigs and per-part pivots come with roadmap step 1.
