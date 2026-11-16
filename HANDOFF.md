# Handoff

## In progress

### feat/lightbox

- Task: add `tools/lightbox.html`, the tool for tracing paper drawings into character JSON described in `docs/character-art-pipeline.md` and decision 0003 (#32). No issue: the maintainer chose to skip one, so the branch is not linked to any issue and the session history is on #34.
- PR: #34, open, awaiting the maintainer's further iteration before review and merge. Auto-fix is on for it in the desktop app. Its description is out of date (P-18).
- Code checkpoint: af393fd (`feat(tools): carry on the maintainer's line edits as lightbox defaults`), on `main` at 16a530e.
- State: `npm start` serves the Lightbox at `/tools/lightbox.html` (`LOCAL_ONLY` in `tools/serve.mjs`, decision 0004). It traces the whole character once, then splits it into parts with lassos. Tracing joins line pieces (meeting, continuing, or overlapping side by side), removes burrs and short struts, extends line ends to meet a nearby line, and keeps ink under erased paper fill. On the Strokes step a cubic bezier editor reshapes or deletes any whole line (`S.curves`, applied on top of the trace, so edits survive re-tracing), with end anchors snapping onto lines. The view zooms and pans. Defaults come from the maintainer's two IMG_1617 presets; the boil is 4 fps, 6 poses, strength 0.9. The character JSON format is unchanged.
- Maintainer preferences (also in the user's auto-memory): one continuous line rather than several short strokes; smooth, confident lines rather than the pencil's wobble; line ends that meet the line they head for; no short restarted strokes beside a line. Their second preset (`IMG_1617.preset (1).json`, in their Downloads with the photos, not in the repository) is the current reference: defaults alone give 29 strokes against their 33, within about 3.5% at 5 px.
- Verification at af393fd: `npm test` and `npm run smoke` ok. In the browser pane on IMG_1617 and IMG_1616: the trace comparison above, real mouse checks of selecting, bending and zoomed editing, end snapping, a preset round trip with identical JSON, and export. Not tried: a tablet or a real trackpad pinch. `cargo test` and `./build.sh` not run (no change to `core/`). CI's smoke job failed once on c6618af waiting for the game HUD and passed on re-run: flaky.
- Avoid: twin-line removal and removing short marks beside a line (both removed lines the maintainer kept); rerouting a line that meets another at a shallow Y (the two halves swap back and forth); cutting long tails past a crossing automatically (it would cut the sword's cross-guard).
- Next action: the maintainer re-traces IMG_1617 (and a new drawing) with the current defaults and says what still needs work. Planned after that: faster fill fixes, such as clicking a closed area to mark it empty. Once #34 merges, file P-17 and start the load-time bake, moving the jitter into `src/` so the Lightbox imports it (decision 0004).
- **P-18** Update the description of #34: add to "What changed" the stroke join, burr, strut and extend passes, the bezier line editor with end snapping, zoom and pan, keeping ink under erased fill, and the pencil defaults; add the IMG_1617 and IMG_1616 checks to "Verification". Why: editing a PR description publishes it. Blocks: an accurate review of #34.

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
  - `### What to change`: Load each character JSON from `assets/characters/` at start-up and, in `src/renderer.js`, draw each part's boil poses (6 by default) with Canvas2D into the existing part atlas: paper fill, solid ink, then strokes of the part's single width, each stroke with its own seeded wobble per pose, using the same jitter as `tools/lightbox.html`. Add a rect per part per pose, and let the core pick the pose on a 4 fps cycle with `part_id = base + pose`, so the 12-float instance format does not change.
  - `### Why`: Decision 0003 (#32) chose this so hand-traced characters get a per-stroke line boil while each part stays one textured quad per frame. Nothing in the game reads character JSON yet.
  - `### Reference`: `docs/character-art-pipeline.md`, `docs/decisions/0003-character-art-pipeline.md`, #34 (the Lightbox). Puppet rigs and per-part pivots come with roadmap step 1.
