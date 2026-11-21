# Character art pipeline

How a character goes from the artist's paper drawing to the screen. Why it works this way, and the options set aside, are in decision [0003](decisions/0003-character-art-pipeline.md).

## Flow

| Step | Where | When | Output |
| --- | --- | --- | --- |
| 1. Draw | On paper, outside the repository | Once per drawing | A photo or scan (PNG, JPG or WebP) |
| 2. Trace | The Lightbox, `tools/lightbox.html`, by hand | Once per character, again when the art changes | `assets/characters/<name>.json` and `<name>.preset.json`, committed |
| 3. Bake | `src/characters.js` and `src/renderer.js`, at page load | Once per start-up | Atlas texture and rect table: one cell per part per boil pose |
| 4. Simulate | Rust core, `core/` | Every frame | Instance buffer, 16 floats per part, with the pose in the part id and the part's turn, lean, lift and bend in 3D |
| 5. Draw | WebGPU, `src/shader.wgsl` | Every frame | The frame on screen: one quad per part, plus margin and shadow |

## Tracing in the Lightbox

Run `npm start` and open http://localhost:5173/tools/lightbox.html. Opening the file straight from disk is not supported; decision [0004](decisions/0004-serve-the-lightbox-locally.md) explains why. Open the photo, type the character's name, and work through the steps on the left: crop, paper, ink, paper fill, solid ink, strokes, parts, boil and export. Each step has its own sliders; the ↺ beside a slider you have moved puts it back to its default. The ink, paper fill and solid ink steps have a paint and erase brush for what the automatic part gets wrong (⌘Z and ⌘⇧Z undo and redo). To zoom, pinch, Ctrl/⌘-scroll or turn the mouse wheel over the view, or use the + and − buttons (keys `=`, `-`, and `0` to fit); scroll with two fingers, or hold Space or the middle button and drag, to pan. "Hide panel" (or `\`) hides the left panel to give the view the whole window.

The first six steps clean up and trace the whole character at once. The defaults are tuned on the artist's pencil photos: they favour smooth, confident lines over a faithful copy of every pencil wobble, join line pieces that meet, carry on in the same direction, or overlap where the pencil started a new stroke beside the last one, so one drawn line becomes one stroke, and carry a line that stops just short of another on until it meets it. Smoothing and Simplify are set for a 4 px pencil line and scale with the drawing's own line width, so a drawing with thinner lines keeps its smaller details. Smoothing stops at sharp corners, so a V or the corner of a box stays sharp, and a short rung or strut drawn between two lines (two line widths or longer) is kept. Every traced line and solid ink outline is a cubic bezier curve, fitted to within the Simplify distance of the pencil. On the strokes step, click one to reshape it: drag its anchors and handles (an end anchor snaps onto a nearby line; hold Shift to place it freely), double-click it to add an anchor, or click an anchor to select it and press Delete to remove that anchor. A square anchor is smooth (its handles stay in line), a diamond is a corner (its handles move on their own); double-click an anchor to switch it. With no anchor selected, Delete removes the whole line or shape. A solid ink shape is a closed outline filled with ink. The tool buttons do the same with one click (Add anchor, Remove anchor), and Draw line and Draw solid add new ones: click to place anchors, drag while placing one to bend the curve through it, click the first anchor to close it, and double-click or press Enter to finish. Split cuts a line in two wherever you click it (or opens a closed line there), and Join connects two lines end to end: click near an end of each, or both ends of one line to close it. On any step, tap Tab (or tick Show ink) to switch to the pencil ink, with the traced strokes on top once there are any, and back; hold Tab to look only while it is held. After a split the editor goes back to Edit, so you can pick the next line straight away. Edits sit on top of the trace, so they stay when you change the sliders. The parts step then splits the clean drawing into parts that move on their own. Add a part, then drag a lasso around it; anything outside every lasso belongs to the first part, and a stroke that crosses a lasso is cut there. Where one part covers another, such as an arm over the body, use the fill brush to paint extra paper fill for the part underneath, so no hole shows when the top part moves. Each part has an id, a draw order (where lassos overlap, the higher one wins) and a pivot, set on the last step with "Set pivot" and a click on the view; "Hide cross" hides its red marker while you look at the boil.

"Save character" keeps the work in this browser. "Download character JSON" and "Download preset" write the two files to commit. To change a character later, open the same photo, then "Open preset" with its `<name>.preset.json`. When the line editor does something unexpected, click "Save debug log" right away: it downloads a report of the recent editor activity, the settings and the edit history, so the problem can be replayed.

## Source of truth

Each character has two committed files in `assets/characters/`:

- **`<name>.json`**, exported by the Lightbox. The game loads only this.
- **`<name>.preset.json`**, the Lightbox settings: crop, rotation, levels, brush edits and line edits for the whole character, and each part's id, draw order, pivot, lassos and fill brush strokes. It does not contain the photo, so keep the photo with the artist's originals.

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

At start-up, `src/characters.js` draws each part mapped in `assets/characters.json` in its boil poses (8 by default, tuned in the game; decision 0003 first planned 3) with Canvas2D, fitted to the box the core draws the part in, and `src/renderer.js` packs them into the existing part atlas. Strokes are drawn the same width on screen for every character. The jitter lives in `src/boil.js`, which the Lightbox imports too, so its boil preview matches the game: each stroke gets its own seeded wobble per pose, so neighbouring lines never move together. This takes a few milliseconds per character, once, and needs no build step or generated PNGs.

Open the game with `?debug` for a panel of sliders that tune the bake (line width, line boil, poses, sharpness), the look (boil rate, paper margin, shadow), each character's drawn size and each monster's waddle (how far it tilts and how often it steps), while the game runs. "Spin puppets" turns, leans, lifts and bends every puppet, to check how the parts look as sheets in 3D. "Copy values" copies them as JSON, to make tuned values the defaults in `src/characters.js` and `src/renderer.js`.

## Per-frame cost

The boil adds nothing per frame. Each part of each character instance is one textured sheet. The Rust core picks each instance's pose on a 4 fps cycle and encodes it in the part id (`part_id = base + 32 * pose`), so the boil needs no room in the 16-float instance format. Each instance starts on its own pose and steps at its own moment, both taken from its seed, so a crowd does not boil in lockstep. Parts without traced art have the same image in every pose row of the rect table. The core also sets each part's offset and tilt from its puppet rig, and its turn, lean, lift and bend as a sheet of paper in 3D (roadmap step 2). The shader orients the sheet, draws its back as plain paper with the ink showing through, darkens it as it turns from the light, and still adds the cutout margin, shadow and hit flash. The line boil replaced the shader's old outline wobble, which is gone.
