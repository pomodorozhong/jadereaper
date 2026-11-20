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
    /// Part id plus `PART_SLOTS` times the line boil pose (see `Game::put`).
    pub part: f32,
    /// 0..1, flashes the part white when hit.
    pub flash: f32,
    /// Per-instance random value, used for shape variety and the line boil pose.
    pub seed: f32,
    pub color: [f32; 4],
}

pub const INST_FLOATS: usize = 12;
pub const MAX_INST: usize = 4096;

/// Part ids run below this. The instance's part is `id + PART_SLOTS * pose`, so the renderer can pick each
/// pose's image of a traced part while the shader still finds the shape from `part % PART_SLOTS`.
pub const PART_SLOTS: f32 = 32.0;
/// Most line boil poses a part can have; the rect table holds `PART_SLOTS * MAX_POSES` entries.
pub const MAX_POSES: u32 = 8;

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
/// The elite is a big brute, with its own part so a traced drawing keeps its line width at that size.
pub const P_ELITE: f32 = 19.0;
/// Ground warnings for enemy attacks, one per hit area shape. Drawn as a flat ink wash whose fill grows with the
/// instance's `flash` (0 at the start of the wind-up, 1 when it strikes). An arc's half angle over pi, or a ring's
/// inner radius over its outer one, travels in `seed`.
pub const P_WARN_CIRCLE: f32 = 20.0;
pub const P_WARN_ARC: f32 = 21.0;
pub const P_WARN_LINE: f32 = 22.0;
pub const P_WARN_RING: f32 = 23.0;

// Upgrade ids.
pub const U_VITALITY: usize = 0;
pub const U_SWIFT: usize = 1;
pub const U_LURE: usize = 2;
pub const U_MIGHT: usize = 3;
pub const U_MENDING: usize = 4;
pub const N_UPGRADES: usize = 5;
pub const UPGRADE_MAX: [u8; N_UPGRADES] = [5, 5, 5, 5, 5];

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
/// An enemy attack strikes; the value is the enemy kind.
pub const EV_STRIKE: f32 = 12.0;

pub const STATE_FLOATS: usize = 32;

// Button bits in `jr_input`.
pub const B_SKILL1: u32 = 1;
pub const B_SKILL2: u32 = 2;
pub const B_SKILL3: u32 = 4;
pub const B_ROLL: u32 = 8;
pub const N_BUTTONS: usize = 4;

/// How long a press is remembered, so one made just before an action can start still counts.
pub const BUFFER: f32 = 0.12;

// The roll: every character's one fixed move.
/// How long the roll moves.
pub const ROLL_TIME: f32 = 0.30;
/// Invulnerable from the start of the roll for this long.
pub const ROLL_INVULN: f32 = 0.25;
/// After the roll moves, the player recovers this long before acting or rolling again.
pub const ROLL_RECOVER: f32 = 0.14;
/// Distance covered by one roll.
pub const ROLL_DIST: f32 = 130.0;

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
    /// How far the puppet tilts each way as it waddles, in degrees.
    pub tilt: f32,
    /// Stagger it takes before a hit interrupts it.
    pub poise: f32,
}

pub const K_BLOT: u8 = 0;
pub const K_WISP: u8 = 1;
pub const K_BRUTE: u8 = 2;
pub const K_ELITE: u8 = 3;

pub const KINDS: [Kind; 4] = [
    Kind { hp: 14.0, speed: 62.0, r: 16.0, dmg: 5.0, xp: 1.0, part: P_BLOT, color: [0.33, 0.31, 0.30], hw: 24.0, hh: 24.0, rate: 2.0, tilt: 2.25, poise: 8.0 },
    Kind { hp: 7.0, speed: 118.0, r: 12.0, dmg: 4.0, xp: 1.0, part: P_WISP, color: [0.36, 0.40, 0.56], hw: 25.0, hh: 21.0, rate: 9.0, tilt: 3.75, poise: 5.0 },
    Kind { hp: 70.0, speed: 44.0, r: 26.0, dmg: 12.0, xp: 5.0, part: P_BRUTE, color: [0.58, 0.31, 0.24], hw: 30.0, hh: 30.0, rate: 3.0, tilt: 7.5, poise: 40.0 },
    Kind { hp: 600.0, speed: 56.0, r: 44.0, dmg: 20.0, xp: 40.0, part: P_ELITE, color: [0.74, 0.25, 0.18], hw: 52.0, hh: 52.0, rate: 2.5, tilt: 7.5, poise: 120.0 },
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
