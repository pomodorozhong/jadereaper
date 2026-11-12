# Part images

Every character and object is built from paper parts. By default each part is drawn in code (see `src/shader.wgsl`). To use your own art for a part, put a PNG with a transparent background in this folder and list it in `assets/parts.json`:

```json
{
  "player_body": "parts/player_body.png",
  "blot": "parts/blot.png"
}
```

Reload the page and the image replaces the code-drawn shape. The renderer still adds the white cutout margin, the drop shadow, the hit flash and the outline wobble, and the game still poses the part (tilt, hop, flip), so one still image per part is enough.

## Part names

| Name | Used for | Drawn size in world units (width x height) |
| --- | --- | --- |
| `player_body` | Reaper's robe | 40 x 52 |
| `player_head` | Reaper's hood | 30 x 30 |
| `scythe` | Scythe, handle down and blade at the top curving right | 40 x 68 |
| `blot` | Basic ink spirit | 36 x 36 |
| `wisp` | Fast flying spirit | 44 x 26 |
| `brute` | Tough horned spirit, also the elite at a larger size | 60 x 60 |
| `gem` | Experience gem | 11 x 14 |
| `heart` | Health pickup | 22 x 22 |
| `talisman` | Thrown paper slip, pointing up | 12 x 22 |
| `bead` | Orbiting jade bead | 18 x 18 |
| `slash` | Scythe sweep, arc opening to the right | square, sized to reach |
| `ring` | Spirit pulse ring | square, sized to radius |
| `splat` | Ink splash when a spirit dies | varies |
| `scrap` | Paper scrap when the reaper is hurt | 8 x 8 |
| `grass` | Ground brush strokes | varies |
| `stone` | Ground stone wash | varies |
| `ground_shadow` | Soft shadow under characters | varies |
| `ellipse` | Spare generic shape | any |

## Guidelines

- Draw characters facing right. The game mirrors them when they face left.
- The image is stretched to the part's size, so match the aspect ratio in the table. Any resolution works; images larger than 512 pixels on a side are scaled down when packed.
- Leave a few pixels of transparent border so the white margin and shadow have room.
- `slash`, `ring`, `splat`, `scrap`, `grass`, `stone` and `ground_shadow` are flat marks. They are drawn without margin or shadow, and their image is faded by the game's own opacity.
