// Bake hand-traced characters (assets/characters/<name>.json, exported by the Lightbox) into part images.
//
// assets/characters.json maps a part name from PART_IDS to a part of a character, or to null to hide that
// part. Each mapped part is drawn with Canvas2D in BAKE.poses poses, each stroke with its own seeded wobble per
// pose, as the Lightbox's boil step shows it. The renderer packs the poses into the atlas and steps through them.
//
// The drawing is fitted into the box the core draws the part in, keeping its proportions and standing on the
// box's bottom edge, because the shader stretches an image to fill that box. Strokes are drawn BAKE.lineWidth world
// units wide, not at the width traced from the photo, so every character has the same line on screen.

/** Bake settings. The ?debug panel changes them at run time and re-bakes. */
export const BAKE = {
  // Tuned by the maintainer in the ?debug panel.
  lineWidth: 0.8,   // every traced stroke, in world units, whatever the drawing's own scale
  boilAmp: 0.25,    // per-stroke wobble between poses, in world units
  poses: 8,         // boil poses per part
  texels: 6,        // atlas pixels per world unit, so every character is equally sharp
};
const INK = '#2b2622';
const PAPER = '#fffdf6';

/** Width and height, in world units, of the box the core draws each part in (core/src/game.rs and defs.rs). */
const BOX = {
  player_body: [80, 80], scythe: [40, 68], blot: [48, 48], wisp: [50, 42], brute: [60, 60], elite: [104, 104],
};

// Same jitter as tools/lightbox.html, so the game boils like the Lightbox preview.
function rng(seed) {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}
function jitter(pts, closed, seed, amp) {
  if (!amp) return pts;
  const r = rng(seed), len = [0];
  for (let i = 1; i < pts.length; i++) len.push(len[i - 1] + Math.hypot(pts[i][0] - pts[i - 1][0], pts[i][1] - pts[i - 1][1]));
  const T = len[len.length - 1] || 1, ox = (r() - 0.5) * amp, oy = (r() - 0.5) * amp, waves = [];
  for (let k = 0; k < 2; k++) {
    const lam = 20 + r() * 50;
    waves.push({ fr: closed ? (2 * Math.PI * Math.max(1, Math.round(T / lam))) / T : (2 * Math.PI) / lam, px: r() * 6.283, py: r() * 6.283, w: k ? 0.35 : 0.65 });
  }
  return pts.map(([x, y], i) => {
    let dx = 0, dy = 0;
    for (const v of waves) { dx += v.w * Math.sin(len[i] * v.fr + v.px); dy += v.w * Math.sin(len[i] * v.fr + v.py); }
    return [x + ox + dx * amp * 0.8, y + oy + dy * amp * 0.8];
  });
}
/** Trace a smooth curve through the points (uniform Catmull-Rom as cubic beziers), like the Lightbox's bez(). */
function curve(ctx, p, closed) {
  if (p.length === 2) { ctx.moveTo(...p[0]); ctx.lineTo(...p[1]); return; }
  let e;
  if (closed) { p = p.slice(0, -1); e = [p[p.length - 1], ...p, p[0], p[1]]; } else e = [p[0], ...p, p[p.length - 1]];
  ctx.moveTo(...e[1]);
  for (let i = 1; i < e.length - 2; i++) {
    const [a, b, c, q] = [e[i - 1], e[i], e[i + 1], e[i + 2]];
    ctx.bezierCurveTo(b[0] + (c[0] - a[0]) / 6, b[1] + (c[1] - a[1]) / 6, c[0] - (q[0] - b[0]) / 6, c[1] - (q[1] - b[1]) / 6, c[0], c[1]);
  }
  if (closed) ctx.closePath();
}

function bounds(part, margin) {
  let x0 = Infinity, y0 = Infinity, x1 = -Infinity, y1 = -Infinity;
  for (const l of [...part.paper, ...part.solid, ...part.strokes.map((s) => s.pts)]) {
    for (const [x, y] of l) { x0 = Math.min(x0, x); y0 = Math.min(y0, y); x1 = Math.max(x1, x); y1 = Math.max(y1, y); }
  }
  return [x0 - margin, y0 - margin, x1 + margin, y1 + margin];
}

/** Draw one pose of a part into a new canvas shaped like its box. */
function bakePose(part, pi, pose, [boxW, boxH], bake) {
  // Fit in two passes: the margin for the line and the boil depends on the scale, which depends on the margin.
  let world = 1, lineW = part.width, amp = 0.9, box;
  for (let k = 0; k < 2; k++) {
    const [x0, y0, x1, y1] = bounds(part, lineW / 2 + amp * 2);
    const w = x1 - x0, h = y1 - y0;
    // The box in drawing units: big enough for the drawing, with the drawing on its bottom edge, centred.
    const bw = Math.max(w, h * boxW / boxH);
    world = boxW / bw;
    lineW = bake.lineWidth / world;
    amp = bake.boilAmp / world;
    box = { x0, y0, w, h, bw, bh: bw * boxH / boxW };
  }
  const s = world * bake.texels;
  const canvas = document.createElement('canvas');
  canvas.width = Math.round(boxW * bake.texels);
  canvas.height = Math.round(boxH * bake.texels);
  const ctx = canvas.getContext('2d');
  ctx.setTransform(s, 0, 0, s, ((box.bw - box.w) / 2 - box.x0) * s, (box.bh - box.h - box.y0) * s);
  const seed = (kind, i) => (kind * 104729 + i * 31 + pose * 15485863 + pi * 7919) | 0;

  ctx.beginPath();
  part.paper.forEach((l, i) => curve(ctx, jitter(l, true, seed(1, i), amp * 0.6), true));
  ctx.fillStyle = PAPER;
  ctx.fill('evenodd');
  ctx.beginPath();
  part.solid.forEach((l, i) => curve(ctx, jitter(l, true, seed(2, i), amp * 0.6), true));
  ctx.fillStyle = INK;
  ctx.fill('evenodd');
  ctx.strokeStyle = INK;
  ctx.lineWidth = lineW;
  ctx.lineCap = ctx.lineJoin = 'round';
  part.strokes.forEach((st, i) => {
    ctx.beginPath();
    curve(ctx, jitter(st.pts, st.closed, seed(3, i), amp), st.closed);
    ctx.stroke();
  });
  return canvas;
}

/**
 * Load assets/characters.json and bake every mapped part.
 * @param {Record<string, number>} partIds part name to part id
 * @param {typeof BAKE} bake bake settings
 * @returns {Promise<{id: number, poses: (HTMLCanvasElement|null)[]}[]>} null poses hide the part
 */
export async function bakeCharacters(partIds, bake = BAKE) {
  let manifest = {};
  try {
    const res = await fetch('assets/characters.json', { cache: 'no-cache' });
    if (res.ok) manifest = await res.json();
  } catch {
    return [];
  }
  const chars = {};
  const out = [];
  for (const [name, ref] of Object.entries(manifest)) {
    if (!(name in partIds)) continue;
    if (!ref) { out.push({ id: partIds[name], poses: null }); continue; }
    try {
      chars[ref.character] ??= fetch(`assets/characters/${ref.character}.json`, { cache: 'no-cache' }).then((r) => r.json());
      const ch = await chars[ref.character];
      const pi = ch.parts.findIndex((p) => p.id === ref.part);
      if (pi < 0) throw new Error(`no part "${ref.part}"`);
      const poses = [];
      for (let k = 0; k < bake.poses; k++) poses.push(bakePose(ch.parts[pi], pi, k, BOX[name] ?? [64, 64], bake));
      out.push({ id: partIds[name], poses });
    } catch (err) {
      console.warn(`Could not bake "${name}" from character "${ref.character}"`, err);
    }
  }
  return out;
}
