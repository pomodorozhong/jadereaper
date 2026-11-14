# Contributing

Thanks for helping with Jade Reaper. The maintainer is @pomodorozhong, who decides what gets built and in what order. This page explains how to propose work so it lands smoothly. It applies the same way to people and to AI coding agents working for them.

## The flow

1. **Open an issue first,** using the **Proposal**, **Bug** or **Question** form. Describe what you want or what went wrong. A rough idea is enough; scope and how it will be checked get settled on the issue before it is accepted or confirmed. Search existing issues, closed ones included, before opening a new one.
2. **Wait for the maintainer to agree.** A proposal gets the `accepted` label, and a bug the `confirmed` label once it is reproduced. The maintainer may also ask for changes or decline. Discussion happens on the issue.
3. **Open one PR per agreed issue.** Fill in the PR template, which starts with the issue it implements, and link it with `Closes #N`. A PR without an `accepted` proposal or a `confirmed` bug behind it may be closed without review, however good it is.
4. **Keep PRs small.** One reason to change per PR. Work that needs several PRs gets one issue per PR, each saying which other issues it depends on. If the work turns out bigger than the issue said, stop and update the issue rather than growing the PR.

Typo fixes and other one-line corrections can skip the issue.

A bug fix that changes how the game plays counts as a proposal.

## Labels

Labels show what kind of issue it is and where it stands, such as `proposal`, `accepted` or `confirmed`. What each one means, and how an issue moves between them, is in [.github/labels.md](.github/labels.md).

When an issue has `waiting-for-author`, the maintainer is waiting on you. When you have answered, say so on the issue, and the maintainer removes the label.

## Decisions

Choices that shape later work, such as the toolchain or a design pillar, are decided by the maintainer on an issue and then recorded in [docs/decisions/](docs/decisions/README.md). An issue should cite the records it relies on. A choice without a record is still open, even if it was agreed in chat, so raise it on the issue instead of deciding it yourself.

## Before you open a PR

Run the checks in [AGENTS.md](AGENTS.md) and list in the PR which ones you ran, what they showed, and which ones you skipped and why. Read [docs/README.md](docs/README.md) for the gotchas, for example that the built wasm and the bundled fonts are committed.

## AI coding agents

Agents are welcome. They follow this page like anyone else: no PR without an `accepted` proposal or a `confirmed` bug. The person running the agent is responsible for what it files.
