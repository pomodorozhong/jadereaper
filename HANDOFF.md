# Handoff

## In progress

### ci/auto-label

- Task: #30 (`accepted`), label issues filed without the forms. Branch linked to #30; the PR fully resolves it.
- PR: none yet.
- Code checkpoint: 4c32111 (`ci: label issues filed without the forms`), on `main` at a0d53d1.
- Answers to #30's open questions, taken as the recommended defaults because the maintainer accepted the issue without answering them: an issue matching no form gets `needs-triage`, and the workflow uses only GitHub's own actions with `GITHUB_TOKEN`.
- Verification at 4c32111: `npm test` passes (6 tests against the real forms), and the workflow's script ran against a mocked GitHub client for a proposal body, an unmatched body and an already-labelled issue. Not run: `cargo test`, `./build.sh`, `npm run smoke` (no change to the game); actionlint (not installed). The `label` job only runs on a real new issue on `main`, so it is untested on GitHub until it merges.
- Next action: P-15.
- Pending: **P-15** open a PR for this branch with the `open-pr` skill. Why: opening a PR needs approval. Blocks: #30.

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
