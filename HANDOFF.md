# Handoff

## In progress

### ci/smoke-comment

- Task: correct the comments at the top of `.github/workflows/smoke.yml` and `tools/smoke.mjs`, which said CI usually has no WebGPU and checks the "Cannot start" screen. CI runs showed the opposite: Chromium finds a software adapter on the Ubuntu runner and plays a few seconds of a run (answered on #17, now closed). A comment-only correction, which `CONTRIBUTING.md` lets skip an issue. Local branch only, not linked to any issue. It also carries this handoff refresh after #20, #23 and #24 merged.
- PR: none yet.
- Code checkpoint: 9c01200 (`ci: say which path the smoke test takes on the runner`), on `main` at f739727.
- Verification at 9c01200: `node --check tools/smoke.mjs` passes and `smoke.yml` parses (PyYAML via uvx). Not run: `npm run smoke`, `cargo test`, `./build.sh` (comments only, no behaviour change); the `Smoke` job runs on the PR.
- Next action: P-12, then open the PR with the `open-pr` skill.
- Pending: **P-12** push this branch and open a PR for it. Why: pushing and opening a PR need approval. Blocks: the corrected comments and this handoff reaching `main`.

## Ranked work

Roadmap steps are detailed in `ROADMAP.md`. They become issues once P-02 is approved.

1. Roadmap step 5, the rest of CI: the wasm drift check, cargo fmt and clippy, and a JavaScript check. The Pages deploy (#5) and the smoke test (#24) are done. Planned on the local branch `ci/core-checks`, which needs an `accepted` Proposal first (see that branch's own `HANDOFF.md`)
2. Decide which areas need the maintainer's sign-off before contributors build on them, made on an issue and recorded in `docs/decisions/`. Blocks #14 (`blocked`)
3. #3: pick English UI fonts to match the Chinese set (`accepted`; can run alongside item 1)
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
