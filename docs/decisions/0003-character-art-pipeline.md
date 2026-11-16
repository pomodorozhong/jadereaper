# 0003: Trace character art to stroke JSON and bake the boil at load time

- Status: decided
- Date: 2026-11-16
- Issue: none. The maintainer decided this directly, without a Proposal issue.

## Context

The artist delivers characters as pencil drawings on paper, photographed or scanned, and we do not want to ask her to change that. The game needs the art as movable parts with a stop-motion line boil, where each line wobbles on its own. Hordes of enemies must stay cheap to draw every frame, and redoing a trace when the art changes must be easy. So we had to choose what format the art is stored in, what the game loads, and where the boil is produced.

## Decision

Each character is traced by hand in the Lightbox (`tools/lightbox.html`) and stored as two committed files in `assets/characters/`: `<name>.json`, the traced parts (centerline strokes of one width, solid ink areas and the paper fill, plus each part's id, pivot and draw order), and `<name>.preset.json`, the Lightbox settings needed to redo the trace. At page load, `src/renderer.js` draws each part's boil poses (3 by default) into the existing part atlas with Canvas2D, giving every stroke its own seeded wobble per pose. Every frame, the Rust core picks the pose and encodes it in the part id, and each part stays one textured quad. The game never parses SVG; SVG is only a preview or share export from the Lightbox.

## Reasons

Run-time performance and developer ergonomics were the priorities. Baking poses into the atlas keeps the per-frame cost exactly where it is today: one quad per part per instance, with no new instance floats. Baking at load time takes a few milliseconds per character and needs no build step or committed PNGs, and the same code can move to build time later without changing the game side. Storing strokes rather than images is what makes a per-stroke boil possible at all.

These were set aside:

- **SVG at run time:** needs a parser in the game, and hand-edited SVG breaks easily.
- **Triangle meshes per stroke (for example `lyon`):** cost grows with the number of strokes, too much for hordes.
- **Strokes as distance shapes in the shader:** too slow per pixel with this many strokes.
- **Drawing on the CPU and uploading every frame:** the slowest option.
- **Layered PNGs plus a rig JSON:** the usual industry approach, but it cannot produce a per-stroke boil.

## Consequences

- Tracing is a manual step in the Lightbox, once per character and again when the art changes. The preset file means it never starts from scratch.
- All parts of a character share one coordinate space, so anything hidden behind a moving part must still be drawn, or patched with the Lightbox's fill brush.
- The renderer gains a load-time bake from character JSON into the atlas, and the core gains pose selection; both follow the part ids and rigs from roadmap step 1.
- The format is described in [character-art-pipeline.md](../character-art-pipeline.md). Changing what the game loads, or moving the boil to run time, needs a new record that supersedes this one.
