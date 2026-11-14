# 0002: Run the repository's tools on Node, not Python

- Status: decided
- Date: 2026-11-14
- Issue: #21, where it was decided

## Context

The font subset and check scripts were the only Python in the repository, so contributors and CI needed Python and uv just for them. The planned headless smoke test (roadmap step 5) also needs a browser driver and a local server, which could have been written in either language.

## Decision

Scripts in `tools/` and the CI jobs that run them use Node, pinned in `.node-version`, with exact dev dependencies in `package.json` and `package-lock.json`. The repository has no Python. The game itself stays plain ES modules and needs no install or build step; `npm ci` is only for the tools.

## Reasons

The game's front end is already JavaScript, so Node is one toolchain contributors are likely to have anyway, and one fewer to install, pin and keep up to date. Node has HarfBuzz (through `subset-font`) for subsetting fonts and Playwright for driving a browser, so nothing needed Python. Keeping Python for the fonts and adding Node for the smoke test was set aside because it would mean two toolchains. Rust was set aside for the tools because subsetting fonts and driving a browser are much easier with existing Node libraries.

## Consequences

- New tools go in `tools/` as Node scripts (`.mjs`), and any dependency they need is added to `package.json` at an exact version.
- CI jobs set up Node from `.node-version` and run `npm ci`.
- The Rust core is unaffected; it still builds with `./build.sh`.
