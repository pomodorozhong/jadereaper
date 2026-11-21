// Paper cutout renderer.
//
// Every instance is one paper part: a quad whose shape is either a signed
// distance function drawn here, or an image from the atlas when the part has
// an override in assets/parts.json. Cutout parts get an ink outline, a white
// paper margin and a drop shadow.
//
// Each part is a sheet of paper in 3D: it turns, leans, lifts and bends around
// its centre (the instance's last four floats), and is projected with a fixed
// parallel camera that leaves a flat sheet exactly where it always was. Its
// back is plain paper with the front's ink showing through, one fixed light at
// the upper left darkens it as it turns away, and its shadow stays on the
// ground. Plain WGSL with no compute, so a WebGL2 port can follow it.

struct Globals {
  view: vec2f,   // visible area in world units
  cam: vec2f,    // world position at the centre of the screen
  res: vec2f,    // canvas size in device pixels
  shadow: vec2f, // drop shadow offset, world units
  px: f32,       // world units per device pixel
  margin: f32,   // white cutout margin, world units
  shadow_a: f32, // drop shadow darkness
  thick: f32,     // paper thickness, world units: the edge drawn behind each sheet (0 for none)
  fade: f32,       // how far paper shifts towards the depth tone at fade_dist away from the camera
  fade_dist: f32,  // puppet heights away from the camera, from its ground point, for the full shift
  fade_tone: f32,  // what far paper shifts towards: 0 the ground, 1 shade
  fade_curve: f32, // the ramp's shape: the shift grows as (distance / fade_dist) to this power
};

// The instance's part is id + 32 * pose (core/src/defs.rs, PART_SLOTS), so traced parts show their own
// line boil pose. Parts without poses have the same rect in every pose slot.
const PART_SLOTS = 32;
const RECTS = 256;      // PART_SLOTS * MAX_POSES

struct PartRects {
  uv: array<vec4f, RECTS>,   // atlas rect per part id and pose (x, y, w, h); w = 0 means no image
};

@group(0) @binding(0) var<uniform> G: Globals;
@group(0) @binding(1) var<uniform> P: PartRects;
@group(0) @binding(2) var atlas: texture_2d<f32>;
@group(0) @binding(3) var samp: sampler;

const INK = vec3f(0.17, 0.15, 0.13);
const PAPER = vec3f(0.965, 0.94, 0.875);
const PAD = 10.0;        // extra quad size for margin and shadow, world units; SHEET_PAD in core/src/defs.rs matches
// The cutout margin and the shadow's offset and darkness come from G, so the ?debug panel can tune them.

// The camera looks down at the ground: a point one world unit further from it shows this much higher on
// screen, so a sheet lying on its back shows at this fraction of its height. PITCH in core/src/defs.rs matches.
const PITCH = 0.5;
// Each sheet is cut into this many strips across its height, so it can bend.
const STRIPS = 8u;
const SHEET_VERTS = 48u;   // STRIPS * 6
// Paper thickness is drawn as copies of the sheet's outline stacked behind it, this many, between the shadow and the
// sheet. Where the sheet is turned, leaned or bent, they show as its edge.
const EDGE_LAYERS = 2u;
const EDGE = vec3f(0.74, 0.69, 0.6);   // the colour of the paper's cut edge
// Towards the camera, and towards the light (upper left, in front). x right, y down, z towards the camera.
const VIEW_DIR = vec3f(0.0, -0.4472136, 0.8944272);   // normalize(0, -PITCH, 1)
const LIGHT_DIR = vec3f(-0.5773503, -0.5773503, 0.5773503);
const SHADE = 0.2;         // how much darker a sheet gets turned fully away from the light
const SHOW_THROUGH = 0.22; // how strongly the front's ink shows on the back
// What far paper shifts towards: the ground's base colour (fs_bg), like haze over the floor, or a warm shade,
// like the far end sinking into shadow. White paper is close to the ground colour, so shade reads far more.
const GROUND = vec3f(0.925, 0.885, 0.79);
const DEPTH_SHADE = vec3f(0.42, 0.39, 0.36);

fn hash21(p: vec2f) -> f32 {
  var q = fract(p * vec2f(123.34, 456.21));
  q += dot(q, q + 45.32);
  return fract(q.x * q.y);
}

fn vnoise(p: vec2f) -> f32 {
  let i = floor(p);
  let f = fract(p);
  let u = f * f * (3.0 - 2.0 * f);
  return mix(
    mix(hash21(i), hash21(i + vec2f(1.0, 0.0)), u.x),
    mix(hash21(i + vec2f(0.0, 1.0)), hash21(i + vec2f(1.0, 1.0)), u.x),
    u.y);
}

fn fbm(p: vec2f) -> f32 {
  var a = 0.5;
  var s = 0.0;
  var q = p;
  for (var i = 0; i < 4; i++) {
    s += a * vnoise(q);
    q = q * 2.03 + vec2f(17.1, 9.2);
    a *= 0.5;
  }
  return s;
}

fn sdSeg(p: vec2f, a: vec2f, b: vec2f) -> f32 {
  let pa = p - a;
  let ba = b - a;
  let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
  return length(pa - ba * h);
}

fn sdBox(p: vec2f, b: vec2f) -> f32 {
  let d = abs(p) - b;
  return length(max(d, vec2f(0.0))) + min(max(d.x, d.y), 0.0);
}

fn dot2(v: vec2f) -> f32 { return dot(v, v); }

fn sdHeart(p0: vec2f) -> f32 {
  let p = vec2f(abs(p0.x), p0.y);
  if (p.y + p.x > 1.0) {
    return sqrt(dot2(p - vec2f(0.25, 0.75))) - 0.35355;
  }
  return sqrt(min(dot2(p - vec2f(0.0, 1.0)), dot2(p - 0.5 * max(p.x + p.y, 0.0)))) * sign(p.x - p.y);
}

// d: signed distance in world units. ink: 0..1 drawn in ink. light: 0..1 drawn in paper white.
struct Shape { d: f32, ink: f32, light: f32 };

fn dab(d: f32) -> f32 { return smoothstep(0.03, -0.03, d); }

// p is in the part's own space (x right, y down), h is its half size.
fn shape(part: i32, p: vec2f, h: vec2f, seed: f32) -> Shape {
  let q = p / h;
  let m = min(h.x, h.y);
  var s = Shape(1e3, 0.0, 0.0);
  switch part {
    case P_PLAYER_BODY { // robe: bell shape with a zigzag hem and a sash
      let t = q.y * 0.5 + 0.5;
      let w = mix(0.42, 0.95, t * (0.4 + 0.6 * t));
      let hem = 1.0 - 0.14 * abs(fract(q.x * 2.0 + 0.5) * 2.0 - 1.0);
      s.d = max(abs(q.x) - w, max(q.y - hem, -q.y - 1.0)) * m;
      s.ink = 0.55 * dab(abs(q.y + 0.2) - 0.07);
    }
    case P_PLAYER_HEAD { // hood: dark face opening with two pale eyes
      s.d = (length(q) - 0.95) * m;
      let face = dab(length((q - vec2f(0.18, 0.12)) / vec2f(0.62, 0.6)) - 1.0);
      let eyes = dab(min(length(q - vec2f(0.02, 0.1)), length(q - vec2f(0.46, 0.1))) - 0.12);
      s.ink = 0.9 * face * (1.0 - eyes);
      s.light = eyes;
    }
    case P_SCYTHE { // scythe: handle and a crescent blade
      let handle = sdSeg(q, vec2f(-0.35, 1.0), vec2f(-0.35, -0.85)) - 0.08;
      let a = length(q - vec2f(0.0, -0.35)) - 0.62;
      let b = length(q - vec2f(0.05, -0.1)) - 0.68;
      let blade = max(max(a, -b), -0.45 - q.x);
      s.d = min(handle, blade) * m;
      s.light = 0.5 * dab(blade);
    }
    case P_BLOT { // ink blot spirit
      let ang = atan2(q.y, q.x);
      let r = 0.82 + 0.07 * sin(ang * 5.0 + seed * 40.0) + 0.05 * sin(ang * 3.0);
      s.d = (length(q) - r) * m;
      s.light = dab(min(length(q - vec2f(-0.2, -0.12)), length(q - vec2f(0.38, -0.12))) - 0.2);
      s.ink = dab(min(length(q - vec2f(-0.13, -0.1)), length(q - vec2f(0.45, -0.1))) - 0.09);
    }
    case P_WISP { // wisp: small body with two wings
      let body = (length(q / vec2f(0.42, 0.75)) - 1.0) * 0.42;
      let ax = abs(q.x);
      let wing = max(abs(q.y + 0.15) - (1.0 - ax) * 0.6, ax - 0.98);
      s.d = min(body, wing) * m;
      s.light = dab(min(length(q - vec2f(-0.14, -0.15)), length(q - vec2f(0.18, -0.15))) - 0.1);
    }
    case P_BRUTE, P_ELITE { // brute (19: elite, the same shape drawn bigger): horned block
      let body = sdBox(q - vec2f(0.0, 0.15), vec2f(0.5, 0.45)) - 0.3;
      let a = vec2f(abs(q.x), q.y);
      let horn = sdSeg(a, vec2f(0.4, -0.5), vec2f(0.72, -0.93)) - 0.1;
      s.d = min(body, horn) * m;
      s.light = dab(sdSeg(a, vec2f(0.14, -0.02), vec2f(0.44, -0.16)) - 0.075);
      s.ink = dab(sdSeg(q, vec2f(-0.3, 0.5), vec2f(0.3, 0.5)) - 0.045);
    }
    case P_GEM { // jade gem
      s.d = (abs(q.x) * 1.2 + abs(q.y) - 0.95) * 0.64 * m;
      s.light = 0.45 * smoothstep(0.05, -0.05, q.x + q.y + 0.1);
    }
    case P_SLASH { // slash arc, opening towards +x
      let ang = atan2(q.y, q.x);
      let t = clamp(1.0 - abs(ang) / 1.6, 0.0, 1.0);
      s.d = (abs(length(q) - 0.78) - 0.19 * t) * m;
      if (t <= 0.0) { s.d = 1e3; }
    }
    case P_TALISMAN { // talisman slip with ink marks
      s.d = sdBox(q, vec2f(0.88, 0.92)) * m;
      s.ink = 0.75 * step(abs(q.x), 0.5) * step(fract(q.y * 2.2 + 0.2), 0.32);
    }
    case P_BEAD { // bead
      s.d = (length(q) - 0.9) * m;
      s.light = 0.7 * smoothstep(0.32, 0.2, length(q - vec2f(-0.3, -0.3)));
      s.ink = dab(length(q) - 0.13);
    }
    case P_SPLAT { // ink splat
      let ang = atan2(q.y, q.x);
      let r = 0.5 + 0.45 * vnoise(vec2f(ang * 1.6 + 5.0, seed * 50.0));
      s.d = (length(q) - r) * m;
    }
    case P_GRASS { // three brush strokes of grass
      let a = sdSeg(q, vec2f(-0.45, 0.9), vec2f(-0.75, -0.2)) - 0.08;
      let b = sdSeg(q, vec2f(0.0, 0.9), vec2f(0.06, -0.8)) - 0.09;
      let c = sdSeg(q, vec2f(0.45, 0.9), vec2f(0.8, -0.35)) - 0.08;
      s.d = min(a, min(b, c)) * m;
    }
    case P_STONE { // flat stone wash
      s.d = (length(q / vec2f(0.95, 0.7)) - 1.0) * 0.7 * m;
    }
    case P_HEART { // heart
      s.d = sdHeart(vec2f(q.x * 0.62, (1.0 - q.y) * 0.52)) * m * 1.6;
      s.light = 0.4 * smoothstep(0.2, 0.1, length(q - vec2f(-0.4, -0.35)));
    }
    case P_RING { // ring
      s.d = (abs(length(q) - 0.9) - 0.05) * m;
    }
    case P_SCRAP { // paper scrap
      s.d = sdBox(q, vec2f(0.9, 0.9)) * m;
    }
    case P_GROUND_SHADOW { // soft ground shadow
      s.d = (length(q) - 0.6) * m;
    }
    default { // 0: ellipse
      s.d = (length(q) - 0.95) * m;
    }
  }
  return s;
}

// Ground warning for an enemy attack (parts P_WARN_CIRCLE to P_WARN_RING: circle, arc, line, ring), as an ink wash: a rough outline
// of the whole hit area, a faint tint inside, and a darker fill growing from the attacker as `prog` goes 0 to 1.
// `seed` holds an arc's half angle over pi, or a ring's inner radius over its outer one.
fn warnAlpha(part: i32, p: vec2f, h: vec2f, seed: f32, prog: f32) -> f32 {
  let r = length(p);
  var d = 1e3;      // signed distance to the area's edge, world units
  var grown = 0.0;  // 1 inside the part already filled
  if (part == P_WARN_CIRCLE) {
    d = r - h.x;
    grown = step(r, prog * h.x);
  } else if (part == P_WARN_ARC) {
    let half = seed * 3.14159265;
    let ang = abs(atan2(p.y, p.x));
    d = max(r - h.x, (ang - half) * max(r, 1.0));
    grown = step(r, prog * h.x);
  } else if (part == P_WARN_LINE) {
    d = sdBox(p, h);
    grown = step(p.x, -h.x + 2.0 * h.x * prog);
  } else {
    let inner = seed * h.x;
    d = max(r - h.x, inner - r);
    grown = step(r, inner + (h.x - inner) * prog);
  }
  let inside = smoothstep(1.0, -1.0, d);
  let edge = smoothstep(2.2, 0.6, abs(d + 1.2));
  return max(edge * 0.9, inside * (0.16 + 0.5 * grown));
}


struct VOut {
  @builtin(position) pos: vec4f,
  @location(0) p: vec2f,                         // part space (x right, y down), mirrored by facing
  @location(1) @interpolate(flat) half: vec2f,
  @location(2) @interpolate(flat) misc: vec4f,   // shadow (1) or sheet (0), part, flash, seed
  @location(3) @interpolate(flat) color: vec4f,
  @location(4) side: vec2f,                      // facing (> 0 the front shows), brightness from the light
  @location(5) under: vec2f,                     // for a shadow: the sheet's own part-space point over this pixel
  @location(6) depth: f32,                       // towards the camera from the puppet's ground point, in its heights
};

// Turn by `t` about the vertical axis, then lean by `l` about the horizontal one. core/src/defs.rs `orient`
// does the same for placing parts.
fn orient(v: vec3f, t: f32, l: f32) -> vec3f {
  let ct = cos(t);
  let st = sin(t);
  let a = vec3f(v.x * ct + v.z * st, v.y, -v.x * st + v.z * ct);
  let cl = cos(l);
  let sl = sin(l);
  return vec3f(a.x, a.y * cl - a.z * sl, a.y * sl + a.z * cl);
}

fn project(v: vec3f) -> vec2f { return vec2f(v.x, v.y + v.z * PITCH); }

// A point of the sheet, from its flat local position: bent along its height by `bend` (radians from the middle
// to the top and bottom edges of the quad), then turned in its plane by `rot`, then oriented in 3D.
fn sheetPoint(l: vec2f, top: f32, bend: f32, rot: f32, t: f32, lean: f32) -> vec3f {
  var b = vec3f(l, 0.0);
  if (abs(bend) > 1e-4) {
    let th = bend * l.y / top;
    b = vec3f(l.x, sin(th) / bend * top, (1.0 - cos(th)) / bend * top);
  }
  let cs = cos(rot);
  let sn = sin(rot);
  return orient(vec3f(b.x * cs - b.y * sn, b.x * sn + b.y * cs, b.z), t, lean);
}

fn sheetNormal(l: vec2f, top: f32, bend: f32, rot: f32, t: f32, lean: f32) -> vec3f {
  let th = bend * l.y / top;
  let n = vec3f(0.0, -sin(th), cos(th));
  let cs = cos(rot);
  let sn = sin(rot);
  return orient(vec3f(-n.y * sn, n.y * cs, n.z), t, lean);
}

// Each instance draws its ground shadow (the first SHEET_VERTS vertices), then EDGE_LAYERS copies of the sheet set
// back by the paper's thickness, then the sheet, each cut into STRIPS strips across its height. a: x, y, half width (negative mirrors), half height. b: rot, part, flash, seed.
// c: colour. d: turn, lean, lift, bend. e: the part's centre towards the camera from its puppet's ground point,
// and the puppet's height, both in world units.
@vertex
fn vs(@builtin(vertex_index) vi: u32,
      @location(0) a: vec4f,
      @location(1) b: vec4f,
      @location(2) c: vec4f,
      @location(3) d: vec4f,
      @location(4) e: vec2f) -> VOut {
  var corners = array<vec2f, 6>(
    vec2f(-1.0, 0.0), vec2f(1.0, 0.0), vec2f(-1.0, 1.0),
    vec2f(-1.0, 1.0), vec2f(1.0, 0.0), vec2f(1.0, 1.0));
  let layer = vi / SHEET_VERTS;
  let is_shadow = layer == 0u;
  let is_edge = layer >= 1u && layer <= EDGE_LAYERS;
  let k = vi % SHEET_VERTS;
  let corner = corners[k % 6u];
  let strip = f32(k / 6u);
  let half = abs(a.zw);
  let flip = select(1.0, -1.0, a.z < 0.0);
  let ext = half + vec2f(PAD);
  let l = vec2f(corner.x, -1.0 + 2.0 * (strip + corner.y) / f32(STRIPS)) * ext;
  let turn = d.x;
  let lean = d.y;
  let lift = d.z;
  let bend = d.w;
  var w3 = sheetPoint(l, ext.y, bend, b.x, turn, lean);
  let n = sheetNormal(l, ext.y, bend, b.x, turn, lean);
  let facing = dot(n, VIEW_DIR);
  if (is_edge) {
    // Set back from the camera along the sheet's normal, a share of the thickness per layer.
    w3 -= n * sign(facing + 1e-6) * G.thick * f32(layer) / f32(EDGE_LAYERS);
  }
  var w = a.xy + project(w3);
  // Brightness from the light, measured against a sheet standing flat to the camera, which keeps today's look.
  let lit = dot(n * sign(facing + 1e-6), LIGHT_DIR) / LIGHT_DIR.z;
  let shade = 1.0 - SHADE * clamp(1.0 - lit, 0.0, 1.0);

  var o: VOut;
  o.p = vec2f(l.x * flip, l.y);
  o.under = vec2f(1e4);
  if (is_shadow) {
    // The shadow is the sheet's outline moved to the ground and over by the shadow offset. It also carries the
    // part-space point of the sheet above each pixel, so a see-through sheet does not show its own shadow.
    let delta = G.shadow + vec2f(0.0, lift);
    w += G.shadow;
    // The strip is flat, so local to screen is linear across it: invert it to find the point under the sheet.
    let top = ext.y;
    let y0 = -1.0 + 2.0 * strip / f32(STRIPS);
    let y1 = y0 + 2.0 / f32(STRIPS);
    let e0 = project(sheetPoint(vec2f(0.0, y0 * top), top, bend, b.x, turn, lean));
    let ex = project(sheetPoint(vec2f(1.0, y0 * top), top, bend, b.x, turn, lean)) - e0;
    let ey = (project(sheetPoint(vec2f(0.0, y1 * top), top, bend, b.x, turn, lean)) - e0) / ((y1 - y0) * top);
    let det = ex.x * ey.y - ex.y * ey.x;
    if (abs(det) > 1e-4) {
      let dl = vec2f(delta.x * ey.y - delta.y * ey.x, ex.x * delta.y - ex.y * delta.x) / det;
      o.under = vec2f((l.x + dl.x) * flip, l.y + dl.y);
    }
    if (isFlat(i32(b.y + 0.5) % PART_SLOTS)) { w = vec2f(1e6); }   // flat parts cast no shadow
  } else {
    w.y -= lift;
    // Flat ink has no thickness, and a thickness of zero draws no edge.
    if (is_edge && (G.thick <= 0.0 || isFlat(i32(b.y + 0.5) % PART_SLOTS))) { w = vec2f(1e6); }
  }
  let ndc = (w - G.cam) / (G.view * 0.5);
  o.pos = vec4f(ndc.x, -ndc.y, 0.0, 1.0);
  o.half = half;
  o.misc = vec4f(select(select(0.0, 2.0, is_edge), 1.0, is_shadow), b.yzw);
  o.color = c;
  o.side = vec2f(facing, shade);
  // Depth in puppet heights; a part outside any puppet (reach 0) stays at zero and never fades.
  o.depth = select(0.0, (e.x + w3.z) / max(e.y, 1e-3), e.y > 0.0);
  return o;
}

fn img(rect: vec4f, q: vec2f) -> vec4f {
  let inside = step(0.0, q.x) * step(q.x, 1.0) * step(0.0, q.y) * step(q.y, 1.0);
  let t = textureSampleLevel(atlas, samp, rect.xy + clamp(q, vec2f(0.0), vec2f(1.0)) * rect.zw, 0.0);
  return t * inside;
}

// How much of a cutout part's paper, its shape and white margin, covers part-space point p.
fn paperCover(slot: i32, part: i32, p: vec2f, h: vec2f, seed: f32) -> f32 {
  let rect = P.uv[clamp(slot, 0, RECTS - 1)];
  if (rect.z > 0.0) {
    let q = p / h * 0.5 + 0.5;
    let r = G.margin / (2.0 * h);
    var mx = img(rect, q).a;
    for (var k = 0; k < 8; k++) {
      let ang = f32(k) * 0.7853982;
      mx = max(mx, img(rect, q + vec2f(cos(ang), sin(ang)) * r).a);
    }
    return mx;
  }
  return smoothstep(G.px, -G.px, shape(part, p, h, seed).d - G.margin);
}

fn luma(c: vec3f) -> f32 { return dot(c, vec3f(0.299, 0.587, 0.114)); }

@fragment
fn fs(i: VOut) -> @location(0) vec4f {
  let slot = i32(i.misc.y + 0.5);
  let part = slot % PART_SLOTS;
  let seed = i.misc.w;
  let flash = i.misc.z;
  let h = i.half;
  let aa = G.px;
  let rect = P.uv[clamp(slot, 0, RECTS - 1)];

  if (i.misc.x > 1.5) {
    // The paper's edge: the sheet's outline, in the cut edge's colour, shaded and faded like the sheet.
    let cover = paperCover(slot, part, i.p, h, seed) * i.color.a;
    if (cover < 0.004) { discard; }
    var e = EDGE * i.side.y;
    e = mix(e, mix(GROUND, DEPTH_SHADE, G.fade_tone), G.fade * pow(clamp(-i.depth / G.fade_dist, 0.0, 1.0), G.fade_curve));
    return vec4f(e, cover);
  }

  if (i.misc.x > 0.5) {
    // The ground shadow, drawn before the sheet. The sheet covers it where it is opaque; where the sheet is
    // see-through (a flickering player), take out the part the sheet's paper would have covered.
    var sh = 0.0;
    if (rect.z > 0.0) {
      sh = img(rect, i.p / h * 0.5 + 0.5).a;
    } else {
      sh = smoothstep(3.0, -3.0, shape(part, i.p, h, seed).d - G.margin);
    }
    sh *= G.shadow_a;
    if (i.color.a < 0.999 && sh > 0.004) {
      sh *= 1.0 - paperCover(slot, part, i.under, h, seed);
    }
    let a = sh * i.color.a;
    if (a < 0.004) { discard; }
    return vec4f(INK, a);
  }

  if (part >= P_WARN_CIRCLE && part <= P_WARN_RING) {
    let rough = vec2f(vnoise(i.p * 0.06 + seed * 4.0), vnoise(i.p * 0.06 + 17.0)) - 0.5;
    let a = warnAlpha(part, i.p + rough * 3.0, h, seed, flash) * i.color.a;
    if (a < 0.004) { discard; }
    let grain = vnoise(i.p * 0.4 + 3.0);
    return vec4f(i.color.rgb, a * (0.8 + 0.2 * grain));
  }
  let flat_part = isFlat(part);

  // Flat ink and washes get a fixed rough edge from their seed.
  var p = i.p;
  if (flat_part) {
    let bo = vec2f(seed * 17.0, seed * 5.0);
    p += vec2f(vnoise(i.p * 0.09 + bo), vnoise(i.p * 0.09 + bo + 31.7)) - 0.5;
  }
  let grain = vnoise(i.p * 1.3 + seed * 9.0) * 0.6 + vnoise(i.p * 0.35 + seed * 3.0) * 0.4;

  var fill = 0.0;     // coverage of the coloured shape
  var margin = 0.0;   // coverage including the white margin
  var ink = 0.0;      // how much of the front is ink, for the back to show through
  var col = i.color.rgb;

  if (rect.z > 0.0) {
    // Image override from the atlas.
    let t = img(rect, p / h * 0.5 + 0.5);
    fill = t.a;
    col = t.rgb;
    if (!flat_part) {
      margin = paperCover(slot, part, p, h, seed);
      ink = smoothstep(0.3, 0.12, luma(t.rgb)) * t.a;
    }
  } else {
    let s = shape(part, p, h, seed);
    if (part == P_GROUND_SHADOW) {
      fill = smoothstep(0.4 * min(h.x, h.y), -0.3 * min(h.x, h.y), s.d);
    } else {
      fill = smoothstep(aa, -aa, s.d);
    }
    if (!flat_part) {
      margin = smoothstep(aa, -aa, s.d - G.margin);
      let outline = smoothstep(-3.2, -1.2, s.d) * 0.7;
      col = col * (0.9 + 0.2 * grain);
      col = mix(col, INK, outline);   // ink outline
      col = mix(col, PAPER, s.light);
      col = mix(col, INK, s.ink);
      ink = max(outline, s.ink) * fill;
    } else {
      col = col * (0.92 + 0.16 * grain);
    }
  }

  if (flat_part) {
    col = mix(col, vec3f(1.0), flash * 0.85);
    let a = fill * i.color.a * (0.8 + 0.2 * grain);
    if (a < 0.004) { discard; }
    return vec4f(col, a);
  }

  let paper = PAPER * (0.95 + 0.07 * grain);
  var rgb = mix(paper, mix(col, vec3f(1.0), flash * 0.85), fill);
  if (i.side.x < 0.0) {
    // The back: plain paper, with the front's ink showing through. The sheet is seen from behind, so it shows
    // mirrored without flipping anything here.
    rgb = mix(paper, INK, SHOW_THROUGH * ink);
    rgb = mix(rgb, vec3f(1.0), flash * 0.85 * fill);
  }
  rgb *= i.side.y;
  // Paper further from the camera than its puppet's ground point shifts towards the depth tone, so a sheet lying back
  // or leaning away reads as going into the distance, not as squashed. At rest every part is at depth zero.
  let far = mix(GROUND, DEPTH_SHADE, G.fade_tone);
  rgb = mix(rgb, far, G.fade * pow(clamp(-i.depth / G.fade_dist, 0.0, 1.0), G.fade_curve));
  if (margin < 0.004) { discard; }
  return vec4f(rgb, margin * i.color.a);
}

// ---------------------------------------------------------------- background

@vertex
fn vs_bg(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4f {
  var tri = array<vec2f, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
  return vec4f(tri[vi], 0.0, 1.0);
}

@fragment
fn fs_bg(@builtin(position) fc: vec4f) -> @location(0) vec4f {
  let uv = fc.xy / G.res;
  let w = G.cam + (uv - 0.5) * G.view;
  var col = vec3f(0.925, 0.885, 0.79);
  // Age stains.
  col = mix(col, vec3f(0.86, 0.79, 0.66), smoothstep(0.35, 0.8, fbm(w * 0.0035)));
  // Faint jade ink wash, so the ground has landmarks.
  col = mix(col, vec3f(0.62, 0.71, 0.64), 0.35 * smoothstep(0.55, 0.75, fbm(w * 0.0016 + vec2f(7.3, 2.1))));
  // Paper fibres.
  col *= 0.97 + 0.04 * vnoise(w * vec2f(0.9, 0.12)) + 0.03 * vnoise(w * 1.7);
  // Vignette, like a lamp over the desk.
  let v = length(uv - 0.5) * 1.3;
  col *= 1.0 - 0.3 * v * v * v;
  return vec4f(col, 1.0);
}
