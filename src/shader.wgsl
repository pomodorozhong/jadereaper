// Paper cutout renderer.
//
// Every instance is one paper part: a quad whose shape is either a signed
// distance function drawn here, or an image from the atlas when the part has
// an override in assets/parts.json. Cutout parts get an ink outline, a white
// paper margin and a drop shadow. The outline "boils" on a stepped clock so
// it looks hand cut and re-shot frame by frame.

struct Globals {
  view: vec2f,   // visible area in world units
  cam: vec2f,    // world position at the centre of the screen
  res: vec2f,    // canvas size in device pixels
  time: f32,
  qtime: f32,    // time stepped to the stop-motion frame rate
  px: f32,       // world units per device pixel
  pad: f32,
};

struct PartRects {
  uv: array<vec4f, 32>,   // atlas rect per part id (x, y, w, h); w = 0 means no image
};

@group(0) @binding(0) var<uniform> G: Globals;
@group(0) @binding(1) var<uniform> P: PartRects;
@group(0) @binding(2) var atlas: texture_2d<f32>;
@group(0) @binding(3) var samp: sampler;

const INK = vec3f(0.17, 0.15, 0.13);
const PAPER = vec3f(0.965, 0.94, 0.875);
const PAD = 10.0;        // extra quad size for margin and shadow, world units
const MARGIN = 3.5;      // white cutout margin
const SHADOW_OFF = vec2f(4.0, 6.0);

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
    case 1 { // robe: bell shape with a zigzag hem and a sash
      let t = q.y * 0.5 + 0.5;
      let w = mix(0.42, 0.95, t * (0.4 + 0.6 * t));
      let hem = 1.0 - 0.14 * abs(fract(q.x * 2.0 + 0.5) * 2.0 - 1.0);
      s.d = max(abs(q.x) - w, max(q.y - hem, -q.y - 1.0)) * m;
      s.ink = 0.55 * dab(abs(q.y + 0.2) - 0.07);
    }
    case 2 { // hood: dark face opening with two pale eyes
      s.d = (length(q) - 0.95) * m;
      let face = dab(length((q - vec2f(0.18, 0.12)) / vec2f(0.62, 0.6)) - 1.0);
      let eyes = dab(min(length(q - vec2f(0.02, 0.1)), length(q - vec2f(0.46, 0.1))) - 0.12);
      s.ink = 0.9 * face * (1.0 - eyes);
      s.light = eyes;
    }
    case 3 { // scythe: handle and a crescent blade
      let handle = sdSeg(q, vec2f(-0.35, 1.0), vec2f(-0.35, -0.85)) - 0.08;
      let a = length(q - vec2f(0.0, -0.35)) - 0.62;
      let b = length(q - vec2f(0.05, -0.1)) - 0.68;
      let blade = max(max(a, -b), -0.45 - q.x);
      s.d = min(handle, blade) * m;
      s.light = 0.5 * dab(blade);
    }
    case 4 { // ink blot spirit
      let ang = atan2(q.y, q.x);
      let r = 0.82 + 0.07 * sin(ang * 5.0 + seed * 40.0) + 0.05 * sin(ang * 3.0 - G.qtime * 2.0);
      s.d = (length(q) - r) * m;
      s.light = dab(min(length(q - vec2f(-0.2, -0.12)), length(q - vec2f(0.38, -0.12))) - 0.2);
      s.ink = dab(min(length(q - vec2f(-0.13, -0.1)), length(q - vec2f(0.45, -0.1))) - 0.09);
    }
    case 5 { // wisp: small body with two wings
      let body = (length(q / vec2f(0.42, 0.75)) - 1.0) * 0.42;
      let ax = abs(q.x);
      let wing = max(abs(q.y + 0.15) - (1.0 - ax) * 0.6, ax - 0.98);
      s.d = min(body, wing) * m;
      s.light = dab(min(length(q - vec2f(-0.14, -0.15)), length(q - vec2f(0.18, -0.15))) - 0.1);
    }
    case 6 { // brute: horned block
      let body = sdBox(q - vec2f(0.0, 0.15), vec2f(0.5, 0.45)) - 0.3;
      let a = vec2f(abs(q.x), q.y);
      let horn = sdSeg(a, vec2f(0.4, -0.5), vec2f(0.72, -0.93)) - 0.1;
      s.d = min(body, horn) * m;
      s.light = dab(sdSeg(a, vec2f(0.14, -0.02), vec2f(0.44, -0.16)) - 0.075);
      s.ink = dab(sdSeg(q, vec2f(-0.3, 0.5), vec2f(0.3, 0.5)) - 0.045);
    }
    case 7 { // jade gem
      s.d = (abs(q.x) * 1.2 + abs(q.y) - 0.95) * 0.64 * m;
      s.light = 0.45 * smoothstep(0.05, -0.05, q.x + q.y + 0.1);
    }
    case 8 { // slash arc, opening towards +x
      let ang = atan2(q.y, q.x);
      let t = clamp(1.0 - abs(ang) / 1.6, 0.0, 1.0);
      s.d = (abs(length(q) - 0.78) - 0.19 * t) * m;
      if (t <= 0.0) { s.d = 1e3; }
    }
    case 9 { // talisman slip with ink marks
      s.d = sdBox(q, vec2f(0.88, 0.92)) * m;
      s.ink = 0.75 * step(abs(q.x), 0.5) * step(fract(q.y * 2.2 + 0.2), 0.32);
    }
    case 10 { // bead
      s.d = (length(q) - 0.9) * m;
      s.light = 0.7 * smoothstep(0.32, 0.2, length(q - vec2f(-0.3, -0.3)));
      s.ink = dab(length(q) - 0.13);
    }
    case 11 { // ink splat
      let ang = atan2(q.y, q.x);
      let r = 0.5 + 0.45 * vnoise(vec2f(ang * 1.6 + 5.0, seed * 50.0));
      s.d = (length(q) - r) * m;
    }
    case 12 { // three brush strokes of grass
      let a = sdSeg(q, vec2f(-0.45, 0.9), vec2f(-0.75, -0.2)) - 0.08;
      let b = sdSeg(q, vec2f(0.0, 0.9), vec2f(0.06, -0.8)) - 0.09;
      let c = sdSeg(q, vec2f(0.45, 0.9), vec2f(0.8, -0.35)) - 0.08;
      s.d = min(a, min(b, c)) * m;
    }
    case 13 { // flat stone wash
      s.d = (length(q / vec2f(0.95, 0.7)) - 1.0) * 0.7 * m;
    }
    case 14 { // heart
      s.d = sdHeart(vec2f(q.x * 0.62, (1.0 - q.y) * 0.52)) * m * 1.6;
      s.light = 0.4 * smoothstep(0.2, 0.1, length(q - vec2f(-0.4, -0.35)));
    }
    case 15 { // ring
      s.d = (abs(length(q) - 0.9) - 0.05) * m;
    }
    case 16 { // paper scrap
      s.d = sdBox(q, vec2f(0.9, 0.9)) * m;
    }
    case 18 { // soft ground shadow
      s.d = (length(q) - 0.6) * m;
    }
    default { // 0: ellipse
      s.d = (length(q) - 0.95) * m;
    }
  }
  return s;
}

// Parts drawn as flat ink or wash: no outline, margin or shadow.
fn isFlat(part: i32) -> bool {
  return part == 8 || part == 11 || part == 12 || part == 13 || part == 15 || part == 16 || part == 18;
}

struct VOut {
  @builtin(position) pos: vec4f,
  @location(0) p: vec2f,
  @location(1) @interpolate(flat) half: vec2f,
  @location(2) @interpolate(flat) misc: vec4f,   // rot, part, flash, seed
  @location(3) @interpolate(flat) color: vec4f,
  @location(4) @interpolate(flat) soff: vec2f,   // shadow offset in part space
};

@vertex
fn vs(@builtin(vertex_index) vi: u32,
      @location(0) a: vec4f,
      @location(1) b: vec4f,
      @location(2) c: vec4f) -> VOut {
  var corners = array<vec2f, 6>(
    vec2f(-1.0, -1.0), vec2f(1.0, -1.0), vec2f(-1.0, 1.0),
    vec2f(-1.0, 1.0), vec2f(1.0, -1.0), vec2f(1.0, 1.0));
  let half = abs(a.zw);
  let flip = select(1.0, -1.0, a.z < 0.0);
  let l = corners[vi] * (half + vec2f(PAD));
  let cs = cos(b.x);
  let sn = sin(b.x);
  let w = a.xy + vec2f(l.x * cs - l.y * sn, l.x * sn + l.y * cs);
  let ndc = (w - G.cam) / (G.view * 0.5);
  var o: VOut;
  o.pos = vec4f(ndc.x, -ndc.y, 0.0, 1.0);
  o.p = vec2f(l.x * flip, l.y);
  o.half = half;
  o.misc = b;
  o.color = c;
  let so = vec2f(SHADOW_OFF.x * cs + SHADOW_OFF.y * sn, -SHADOW_OFF.x * sn + SHADOW_OFF.y * cs);
  o.soff = vec2f(so.x * flip, so.y);
  return o;
}

fn img(rect: vec4f, q: vec2f) -> vec4f {
  let inside = step(0.0, q.x) * step(q.x, 1.0) * step(0.0, q.y) * step(q.y, 1.0);
  let t = textureSampleLevel(atlas, samp, rect.xy + clamp(q, vec2f(0.0), vec2f(1.0)) * rect.zw, 0.0);
  return t * inside;
}

@fragment
fn fs(i: VOut) -> @location(0) vec4f {
  let part = i32(i.misc.y + 0.5);
  let seed = i.misc.w;
  let flash = i.misc.z;
  let h = i.half;
  let aa = G.px;
  let flat_part = isFlat(part);

  // Line boil: nudge the shape a little, changing only on stepped frames.
  let bo = vec2f(seed * 17.0 + G.qtime * 3.1, seed * 5.0 - G.qtime * 1.7);
  let boil = vec2f(vnoise(i.p * 0.09 + bo), vnoise(i.p * 0.09 + bo + 31.7)) - 0.5;
  let amp = select(2.0, 1.0, flat_part);
  let p = i.p + boil * amp;
  let grain = vnoise(i.p * 1.3 + seed * 9.0) * 0.6 + vnoise(i.p * 0.35 + seed * 3.0) * 0.4;

  var fill = 0.0;     // coverage of the coloured shape
  var margin = 0.0;   // coverage including the white margin
  var shadow = 0.0;
  var col = i.color.rgb;

  let rect = P.uv[clamp(part, 0, 31)];
  if (rect.z > 0.0) {
    // Image override from the atlas.
    let q = p / h * 0.5 + 0.5;
    let t = img(rect, q);
    fill = t.a;
    col = t.rgb;
    if (!flat_part) {
      let r = MARGIN / (2.0 * h);
      var mx = t.a;
      for (var k = 0; k < 8; k++) {
        let ang = f32(k) * 0.7853982;
        mx = max(mx, img(rect, q + vec2f(cos(ang), sin(ang)) * r).a);
      }
      margin = mx;
      shadow = img(rect, q - i.soff / (2.0 * h)).a * 0.3;
    }
  } else {
    let s = shape(part, p, h, seed);
    if (part == 18) {
      fill = smoothstep(0.4 * min(h.x, h.y), -0.3 * min(h.x, h.y), s.d);
    } else {
      fill = smoothstep(aa, -aa, s.d);
    }
    if (!flat_part) {
      margin = smoothstep(aa, -aa, s.d - MARGIN);
      let sd = shape(part, p - i.soff, h, seed).d;
      shadow = smoothstep(3.0, -3.0, sd - MARGIN) * 0.3;
      col = col * (0.9 + 0.2 * grain);
      col = mix(col, INK, smoothstep(-3.2, -1.2, s.d) * 0.7);   // ink outline
      col = mix(col, PAPER, s.light);
      col = mix(col, INK, s.ink);
    } else {
      col = col * (0.92 + 0.16 * grain);
    }
  }

  col = mix(col, vec3f(1.0), flash * 0.85);

  if (flat_part) {
    let a = fill * i.color.a * (0.8 + 0.2 * grain);
    if (a < 0.004) { discard; }
    return vec4f(col, a);
  }

  let paper = PAPER * (0.95 + 0.07 * grain);
  let rgb = mix(paper, col, fill);
  let under = shadow * (1.0 - margin);
  let a = margin + under;
  if (a < 0.004) { discard; }
  let outc = (rgb * margin + INK * under) / a;
  return vec4f(outc, a * i.color.a);
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
