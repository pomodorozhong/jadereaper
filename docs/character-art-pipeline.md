# Character art pipeline

How a character goes from the artist's paper drawing to the screen. Why it works this way, and the options set aside, are in decision [0003](decisions/0003-character-art-pipeline.md).

## Flow

| Step | Where | When | Output |
| --- | --- | --- | --- |
| 1. Draw | On paper, outside the repository | Once per drawing | A photo or scan (PNG, JPG or WebP) |
| 2. Trace | The Lightbox, `tools/lightbox.html`, by hand | Once per character, again when the art changes | `assets/characters/<name>.json` and `<name>.preset.json`, committed |
| 3. Bake | `src/renderer.js`, at page load | Once per start-up | Atlas texture and rect table: one cell per part per boil pose |
| 4. Simulate | Rust core, `core/` | Every frame | Instance buffer, 12 floats per part, with the pose in the part id |
| 5. Draw | WebGPU, `src/shader.wgsl` | Every frame | The frame on screen: one quad per part, plus margin and shadow |

## Tracing in the Lightbox

Run `npm start` and open http://localhost:5173/tools/lightbox.html. Opening the file straight from disk is not supported; decision [0004](decisions/0004-serve-the-lightbox-locally.md) explains why. Open the photo, type the character's name, and work through the steps on the left: crop, paper, ink, paper fill, solid ink, strokes, parts, boil and export. Each step has its own sliders, and the ink, paper fill and solid ink steps have a paint and erase brush for what the automatic part gets wrong (⌘Z and ⌘⇧Z undo and redo).

The first six steps clean up and trace the whole character at once. The parts step then splits the clean drawing into parts that move on their own. Add a part, then drag a lasso around it; anything outside every lasso belongs to the first part, and a stroke that crosses a lasso is cut there. Where one part covers another, such as an arm over the body, use the fill brush to paint extra paper fill for the part underneath, so no hole shows when the top part moves. Each part has an id, a draw order (where lassos overlap, the higher one wins) and a pivot, set on the last step with "Set pivot" and a click on the view.

"Save character" keeps the work in this browser. "Download character JSON" and "Download preset" write the two files to commit. To change a character later, open the same photo, then "Open preset" with its `<name>.preset.json`.

## Source of truth

Each character has two committed files in `assets/characters/`:

- **`<name>.json`**, exported by the Lightbox. The game loads only this.
- **`<name>.preset.json`**, the Lightbox settings: crop, rotation, levels and brush edits for the whole character, and each part's id, draw order, pivot, lassos and fill brush strokes. It does not contain the photo, so keep the photo with the artist's originals.

The Lightbox can also export one part as SVG, but only to preview or share. The game never parses SVG.

## Character JSON

```json
{ "name": "reaper", "size": [509, 717],
  "parts": [ { "id": "body", "pivot": [254, 420], "z": 1,
    "paper": [[[x, y], ...]], "solid": [[[x, y], ...]],
    "strokes": [{ "pts": [[x, y], ...], "closed": false }], "width": 4.0 } ] }
```

- **`size`:** width and height of the character's coordinate space, in pixels of the original photo. Its top-left corner is the top-left corner of the crop. All parts and pivots use this space.
- **`paper`:** the white cutout fill, as closed loops filled with the even-odd rule.
- **`solid`:** filled ink areas, such as shoes and ear insides, also as closed loops.
- **`strokes`:** centerlines. Every stroke in a part has the same `width`.
- **`id`, `pivot`, `z`:** the part's name, the point it turns around, and its draw order (higher is drawn on top).

Anything hidden behind a moving part must still be drawn, or patched with the fill brush. Otherwise a hole shows when the part moves.

## Bake at load time

At start-up, `renderer.js` draws each part's boil poses (3 by default) with Canvas2D into the existing part atlas. It uses the same per-stroke jitter as the Lightbox: each stroke gets its own seeded wobble per pose, so neighbouring lines never move together. This takes a few milliseconds per character, once, and needs no build step or generated PNGs.

## Per-frame cost

The per-frame cost does not change. Each part of each character instance is still one textured quad. The Rust core picks the pose on an 8 fps cycle and encodes it in the part id (`part_id = base + pose`), so the 12-float instance format does not grow. Once the puppet rigs from roadmap step 1 exist, the core also sets each part's offset and tilt around its pivot. The shader still adds the cutout margin, shadow, hit flash and outline wobble.
