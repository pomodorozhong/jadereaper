// Bake hand-traced characters (assets/characters/<name>.json, exported by the Lightbox) into part images.
//
// assets/characters.json maps a part name from PART_IDS to a part of a character, or to null to hide that
// part. Each mapped part is drawn with Canvas2D in BOIL_POSES poses, each stroke with its own seeded wobble per
// pose, as the Lightbox's boil step shows it. The renderer packs the poses into the atlas and steps through them.
//
// The drawing is fitted into the box the core draws the part in, keeping its proportions and standing on the
// box's bottom edge, because the shader stretches an image to fill that box.

export const BOIL_POSES = 6;
export const BOIL_FPS = 4;
const BOIL_AMP = 0.9;      // the Lightbox default, in photo pixels
const MAX_SIDE = 320;      // longest side of one baked pose, in atlas pixels
const INK = '#2b2622';
const PAPER = '#fffdf6';

/** Width / height of the box the core draws each part in (core/src/game.rs and defs.rs). */
const BOX_ASPECT = {
  player_body: 1, scythe: 20 / 34, blot: 1, wisp: 20 / 17, brute: 1,
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

function bounds(part) {
  let x0 = Infinity, y0 = Infinity, x1 = -Infinity, y1 = -Infinity;
  for (const l of [...part.paper, ...part.solid, ...part.strokes.map((s) => s.pts)]) {
    for (const [x, y] of l) { x0 = Math.min(x0, x); y0 = Math.min(y0, y); x1 = Math.max(x1, x); y1 = Math.max(y1, y); }
  }
  const m = part.width + BOIL_AMP * 2;
  return [x0 - m, y0 - m, x1 + m, y1 + m];
}

/** Draw one pose of a part into a new canvas shaped like its box. */
function bakePose(part, pi, pose, aspect) {
  const [x0, y0, x1, y1] = bounds(part);
  const w = x1 - x0, h = y1 - y0;
  // The box in drawing units: wide enough for the drawing, with the drawing on its bottom edge, centred.
  const bw = Math.max(w, h * aspect), bh = bw / aspect;
  const s = MAX_SIDE / Math.max(bw, bh);
  const canvas = document.createElement('canvas');
  canvas.width = Math.round(bw * s);
  canvas.height = Math.round(bh * s);
  const ctx = canvas.getContext('2d');
  ctx.setTransform(s, 0, 0, s, (bw - w) / 2 * s - x0 * s, (bh - h) * s - y0 * s);
  const seed = (kind, i) => (kind * 104729 + i * 31 + pose * 15485863 + pi * 7919) | 0;

  ctx.beginPath();
  part.paper.forEach((l, i) => curve(ctx, jitter(l, true, seed(1, i), BOIL_AMP * 0.6), true));
  ctx.fillStyle = PAPER;
  ctx.fill('evenodd');
  ctx.beginPath();
  part.solid.forEach((l, i) => curve(ctx, jitter(l, true, seed(2, i), BOIL_AMP * 0.6), true));
  ctx.fillStyle = INK;
  ctx.fill('evenodd');
  ctx.strokeStyle = INK;
  ctx.lineWidth = part.width;
  ctx.lineCap = ctx.lineJoin = 'round';
  part.strokes.forEach((st, i) => {
    ctx.beginPath();
    curve(ctx, jitter(st.pts, st.closed, seed(3, i), BOIL_AMP), st.closed);
    ctx.stroke();
  });
  return canvas;
}

/**
 * Load assets/characters.json and bake every mapped part.
 * @param {Record<string, number>} partIds part name to part id
 * @returns {Promise<{id: number, poses: (HTMLCanvasElement|null)[]}[]>} null poses hide the part
 */
export async function bakeCharacters(partIds) {
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
      for (let k = 0; k < BOIL_POSES; k++) poses.push(bakePose(ch.parts[pi], pi, k, BOX_ASPECT[name] ?? 1));
      out.push({ id: partIds[name], poses });
    } catch (err) {
      console.warn(`Could not bake "${name}" from character "${ref.character}"`, err);
    }
  }
  return out;
}
