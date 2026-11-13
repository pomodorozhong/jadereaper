//! Shared definitions: the instance format sent to the renderer, part ids,
//! tuning tables and a tiny RNG.
//!
//! Part ids and upgrade ids are mirrored in `src/data.js` and `src/shader.wgsl`.
//! If you add one here, add it there too.

/// One paper cutout drawn by the renderer. 12 floats, 48 bytes.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Inst {
    pub x: f32,
    pub y: f32,
    /// Half width. A negative value mirrors the part horizontally.
    pub hw: f32,
    pub hh: f32,
    pub rot: f32,
    pub part: f32,
    /// 0..1, flashes the part white when hit.
    pub flash: f32,
    /// Per-instance random value, used for outline wobble and shape variety.
    pub seed: f32,
    pub color: [f32; 4],
}

pub const INST_FLOATS: usize = 12;
pub const MAX_INST: usize = 4096;

// Part ids.
pub const P_ELLIPSE: f32 = 0.0;
pub const P_ROBE: f32 = 1.0;
pub const P_HOOD: f32 = 2.0;
pub const P_SCYTHE: f32 = 3.0;
pub const P_BLOT: f32 = 4.0;
pub const P_WISP: f32 = 5.0;
pub const P_BRUTE: f32 = 6.0;
pub const P_GEM: f32 = 7.0;
pub const P_SLASH: f32 = 8.0;
pub const P_TALISMAN: f32 = 9.0;
pub const P_BEAD: f32 = 10.0;
pub const P_SPLAT: f32 = 11.0;
pub const P_GRASS: f32 = 12.0;
pub const P_STONE: f32 = 13.0;
pub const P_HEART: f32 = 14.0;
pub const P_RING: f32 = 15.0;
pub const P_SCRAP: f32 = 16.0;
pub const P_SHADOW: f32 = 18.0;

// Upgrade ids.
pub const U_SCYTHE: usize = 0;
pub const U_TALISMAN: usize = 1;
pub const U_BEADS: usize = 2;
pub const U_PULSE: usize = 3;
pub const U_VITALITY: usize = 4;
pub const U_SWIFT: usize = 5;
pub const U_LURE: usize = 6;
pub const U_MIGHT: usize = 7;
pub const U_HASTE: usize = 8;
pub const U_MENDING: usize = 9;
pub const N_UPGRADES: usize = 10;
pub const UPGRADE_MAX: [u8; N_UPGRADES] = [6, 6, 6, 6, 5, 5, 5, 5, 5, 5];

// Event types, read by the host for sound.
pub const EV_HIT: f32 = 1.0;
pub const EV_KILL: f32 = 2.0;
pub const EV_PICKUP: f32 = 3.0;
pub const EV_LEVELUP: f32 = 4.0;
pub const EV_HURT: f32 = 5.0;
pub const EV_SLASH: f32 = 6.0;
pub const EV_SHOOT: f32 = 7.0;
pub const EV_PULSE: f32 = 8.0;
pub const EV_DEATH: f32 = 9.0;
pub const EV_HEAL: f32 = 10.0;
pub const EV_ELITE: f32 = 11.0;

pub const STATE_FLOATS: usize = 32;

pub const INK: [f32; 3] = [0.17, 0.15, 0.13];

/// Enemy archetype.
pub struct Kind {
    pub hp: f32,
    pub speed: f32,
    /// Collision radius.
    pub r: f32,
    pub dmg: f32,
    pub xp: f32,
    pub part: f32,
    pub color: [f32; 3],
    pub hw: f32,
    pub hh: f32,
    /// Puppet pose changes per second.
    pub rate: f32,
}

pub const K_BLOT: u8 = 0;
pub const K_WISP: u8 = 1;
pub const K_BRUTE: u8 = 2;
pub const K_ELITE: u8 = 3;

pub const KINDS: [Kind; 4] = [
    Kind { hp: 14.0, speed: 62.0, r: 16.0, dmg: 5.0, xp: 1.0, part: P_BLOT, color: [0.33, 0.31, 0.30], hw: 18.0, hh: 18.0, rate: 5.0 },
    Kind { hp: 7.0, speed: 118.0, r: 12.0, dmg: 4.0, xp: 1.0, part: P_WISP, color: [0.36, 0.40, 0.56], hw: 22.0, hh: 13.0, rate: 9.0 },
    Kind { hp: 70.0, speed: 44.0, r: 26.0, dmg: 12.0, xp: 5.0, part: P_BRUTE, color: [0.58, 0.31, 0.24], hw: 30.0, hh: 30.0, rate: 3.0 },
    Kind { hp: 600.0, speed: 56.0, r: 44.0, dmg: 20.0, xp: 40.0, part: P_BRUTE, color: [0.74, 0.25, 0.18], hw: 52.0, hh: 52.0, rate: 2.5 },
];

pub struct Rng(pub u32);

impl Rng {
    pub fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
    /// Uniform in 0..1.
    pub fn f(&mut self) -> f32 {
        (self.next() >> 8) as f32 / 16_777_216.0
    }
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f()
    }
}

pub fn hash2(x: i32, y: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(374_761_393) ^ (y as u32).wrapping_mul(668_265_263);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^ (h >> 16)
}
