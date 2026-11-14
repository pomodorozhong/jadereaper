# 0001: Record design decisions in this folder

- Status: decided
- Date: 2026-11-14
- Issue: none; decided with the PR that added this folder

## Context

Design conclusions came from chat and from documents still under review, and plans for several PRs were built on them before the maintainer had agreed. There was no single place that said which choices were settled and which were still open.

## Decision

Choices that shape later work, such as the toolchain or a design pillar, are recorded one per file in `docs/decisions/`. Only the maintainer decides, on the issue, and a record files the decision. Issues and PRs cite these records, and a choice with no record counts as open.

## Reasons

A short numbered file per decision is easy to link from issues and PRs, keeps its history in git, and can be reviewed in a PR like any other change. A single large design document mixes settled and open items and is harder to cite precisely.

## Consequences

- Issues name the records they rely on, or raise the missing decisions on the issue.
- `ROADMAP.md`, and any design document added later, should link records instead of restating decisions.
- Changing a decision means adding a new record that supersedes the old one.
