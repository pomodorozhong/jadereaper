# 0004: Serve the Lightbox from the local server, and keep it publishable

- Status: decided
- Date: 2026-11-16
- Issue: none. The maintainer decided this directly, without a Proposal issue.

## Context

The Lightbox (`tools/lightbox.html`, from decision [0003](0003-character-art-pipeline.md)) is a single HTML page for tracing paper drawings into character JSON. Its first version was meant to be opened straight from disk (`file://`), but nobody had decided that. The question came up before its PR merged: should it open from disk, be served by the local dev server, or be published on the Pages site? The maintainer will do most of the tracing, so the tool can rely on the repository and `npm start`.

## Decision

The Lightbox is opened through the local server: run `npm start`, then open `http://localhost:5173/tools/lightbox.html`. `tools/serve.mjs` lists it in `LOCAL_ONLY`, which it serves but the Pages site does not publish. It is not on the Pages site for now, but it must stay publishable. These rules keep that a config change:

- **Static only.** It reads files and never calls an endpoint on the dev server. Saving straight into the repository, if wanted, uses the browser's File System Access API, not a server route.
- **Relative paths only.** The site is served from `/jadereaper/`, so imports and fetches use paths such as `../src/x.js`, never `/src/x.js`.
- **Shared code lives in `src/`.** Code the game and the Lightbox both need, such as the boil jitter, goes in `src/`, which is already published.
- **No `node_modules` at run time.** If it needs a library, the library is vendored into the repository.
- **Narrow listing.** `LOCAL_ONLY` names the Lightbox's own files, not all of `tools/`.

## Reasons

Opening from disk needs no install, but browsers block ES module imports and `fetch` from `file://`. So the Lightbox could not share the boil jitter with `src/renderer.js` for the load-time bake (decision 0003). The two copies would have to be kept in sync by hand, and the preview would drift from the game. It also could not load committed character files, and `localStorage` and the clipboard behave inconsistently across browsers on `file://`. Serving it locally fixes all of these, and its cost (the repository and Node) does not matter while the maintainer does the tracing.

Publishing it on the Pages site was set aside for now: it would put a tracing tool on the public site before anyone else needs it. The rules above keep that option open.

## Consequences

- Tracing needs a checkout and `npm start`. The smoke test opens the Lightbox, so a broken page fails CI.
- The game must never load a `LOCAL_ONLY` file, because it would 404 on the Pages site. The smoke test fails if it does.
- Publishing the Lightbox later means moving its entry from `LOCAL_ONLY` to `PUBLISHED` in `tools/serve.mjs` and adding it to the copy step in `.github/workflows/pages.yml`. That change needs a new record that supersedes this one.
