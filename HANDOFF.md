# Handoff

## In progress

### docs/contribution-flow

- Task: a written contribution process, prompted by a contributor filing six issues and opening PRs before the maintainer had agreed. Four commits: a contribution flow (`CONTRIBUTING.md` plus a one-line pointer in `AGENTS.md`); a full label set in `.github/labels.yml` applied with github-label-sync 3.0.0 through `tools/sync-labels.sh`, with `.github/labels.md` charting the bug and proposal flows; a proposal form, a bug form, a question form and a PR template in `.github/`; and decision records in `docs/decisions/`. The user rejected plan issues and a list of sign-off areas for this PR, and settled the label set in a side session. No issue; not linked to any issue.
- PR: #20, open, awaiting review and merge. The branch was rebuilt and force-pushed with the question label and form, the bug-fix wording and the labels.md fixes from a review session.
- Code checkpoint: 7c94100 (`docs: add a home for design decisions`), on `main` at 80ca4a1.
- Verification at 7c94100: every YAML file under `.github/` parses (PyYAML via uvx), and every label description is under GitHub's 100-character limit; `bash -n` passes on `tools/sync-labels.sh`. Not run: the sync dry run (needs the user's go-ahead, since it runs an npm package with their GitHub token); `cargo test`, `./build.sh` (no code change); shellcheck (not installed). Issue forms and the Mermaid charts can only be checked on GitHub.
- Next action: after #20 merges, P-07.
- Pending: **P-07** after merge, run `tools/sync-labels.sh --dry-run`, show the user its output, and run it for real only on their approval. The real run deletes `documentation`, `duplicate`, `invalid`, `help wanted`, `good first issue` and `accessibility`, and renames `wontfix` to `declined`. Then label the open issues #7 to #12, #14 and #19 `enhancement` and `proposal` (#14 and #19 only have `documentation`, which the sync deletes), and comment on #12 pointing to `CONTRIBUTING.md`. Why: changes the GitHub repository and posts a comment. Blocks: the labels being usable.

## Ranked work

Steps are detailed in `ROADMAP.md`. They become issues once P-02 is approved.

1. Roadmap step 5: CI and one-command deploy; the Pages deploy merged in #5, and the wasm drift check, cargo fmt and clippy, a JavaScript check and a headless smoke test remain
2. Decide which areas need the maintainer's sign-off before contributors build on them (the user wants this in its own PR and has not settled the structure)
3. #3: pick English UI fonts to match the Chinese set (can run alongside item 1)
4. Roadmap step 1: data-driven content and puppet rigs (unblocks most later steps)
5. Roadmap step 2: game feel pass (can run alongside step 3)
6. Roadmap step 3: fixed time step, replays and a headless balance tool
7. Roadmap step 4: run structure and meta progression (after step 1)
8. Roadmap step 6: touch controls and a WebGL2 fallback
9. Roadmap step 7: scale to thousands of enemies

## Pending

- **P-02** File one GitHub issue per roadmap step (seven issues, titles as in `ROADMAP.md`, body copied from each step's section, label `enhancement`, step 1 to 4 in a "Playable game" milestone). Why: filing issues needs approval. Blocks: nothing else; `ROADMAP.md` covers the details until then.
