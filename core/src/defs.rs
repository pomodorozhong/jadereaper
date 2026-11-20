//! Shared definitions: the instance format sent to the renderer, constants, and a tiny RNG.
//!
//! Part ids, event types, enemy kinds, skills and moves are generated into `content.rs` from `content/game.json`
//! (see `tools/content.mjs`) and re-exported here, as are elements and cards.

pub use crate::content::*;

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

pub const STATE_FLOATS: usize = 64;

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
    /// A boss: never pulled by gravity wells, never frozen, and its hits always interrupt.
    pub boss: bool,
    /// Its attacks (ids into `MOVES`).
    pub moves: &'static [usize],
}

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
