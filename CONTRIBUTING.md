# Contributing

Thanks for helping with Jade Reaper. The maintainer is @pomodorozhong, who decides what gets built and in what order. This page explains how to propose work so it lands smoothly. It applies the same way to people and to AI coding agents working for them.

## The flow

1. **Open an issue first.** Describe what you want or what went wrong, and how it can be checked. Search existing issues, closed ones included, before opening a new one.
2. **Wait for the maintainer to agree.** A proposal gets the `accepted` label, and a bug the `confirmed` label once it is reproduced. The maintainer may also ask for changes or decline. Discussion happens on the issue.
3. **Open one PR per agreed issue.** Link it with `Closes #N`. A PR without an `accepted` proposal or a `confirmed` bug behind it may be closed without review, however good it is.
4. **Keep PRs small.** One reason to change per PR. Work that needs several PRs gets one issue per PR, each saying which other issues it depends on. If the work turns out bigger than the issue said, stop and update the issue rather than growing the PR.

Typo fixes and other one-line corrections can skip the issue.

## Before you open a PR

Run the checks in [AGENTS.md](AGENTS.md) and list in the PR which ones you ran, what they showed, and which ones you skipped and why. Read [docs/README.md](docs/README.md) for the gotchas, for example that the built wasm and the bundled fonts are committed.

## AI coding agents

Agents are welcome. They follow this page like anyone else: no PR without an `accepted` proposal or a `confirmed` bug. The person running the agent is responsible for what it files.
