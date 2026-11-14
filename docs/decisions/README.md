# Decision records

Each file here records one decision about the game's design, the plan, or how the project is built and run: what was decided, why, and what it rules out. They are numbered in the order they were made, starting with [0001](0001-record-design-decisions.md).

## Rules

- **Decisions are made on the issue.** Anyone can raise a choice in a Proposal issue, and the maintainer decides it there by accepting it. The PR that adds the record already has the status `decided` and links that issue; merging it only files the record.
- **Only choices that shape later work need a record,** such as the toolchain, a design pillar or the language of the code generator. A routine accepted proposal does not.
- **Issues and PRs cite decisions from here.** An issue that depends on a design choice links the record that settles it. If no record exists yet, raise the choice on the issue and wait for the maintainer to decide it.
- **Records are not rewritten after they are decided.** To change a decision, add a new record that supersedes the old one, and set the old one's status to `superseded by NNNN`.
- **Chat is not a decision.** A conclusion from Discord or other chat counts only once the maintainer has agreed to it on an issue and it is recorded here.

## Writing a record

Copy [0000-template.md](0000-template.md) to the next free number, such as `0002-short-name.md`. Keep it to a page; longer reasoning belongs in the issue where it was decided.

`ROADMAP.md`, and any design document added later, should link the records they rely on instead of restating the decisions.

## Records

| Number | Decision | Status |
| --- | --- | --- |
| [0001](0001-record-design-decisions.md) | Record design decisions in this folder | decided |
| [0002](0002-node-for-tooling.md) | Run the repository's tools on Node, not Python | decided |
