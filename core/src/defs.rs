//! Shared definitions: the instance format sent to the renderer, constants, and a tiny RNG.
//!
//! Part ids, event types, enemy kinds, skills and moves are generated into `content.rs` from `content/game.json`
//! (see `tools/content.mjs`) and re-exported here, as are elements and cards.

pub use crate::content::*;

/// One paper cutout drawn by the renderer. 16 floats, 64 bytes.
///
/// Each part is a sheet of paper oriented in 3D around its centre (`x`, `y`): `rot` turns it in its own plane,
/// then `turn`, `lean` and `bend` (see `Sheet`) set it in 3D, and the renderer projects it with the fixed camera.
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
    pub turn: f32,
    pub lean: f32,
    pub lift: f32,
    pub bend: f32,
}

pub const INST_FLOATS: usize = 16;

/// How a paper sheet is held in 3D, on top of its in-plane rotation. All zero is a sheet standing flat to the
/// camera, which is how every part was drawn before. The renderer's shader (`src/shader.wgsl`) applies them.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Sheet {
    /// Radians about the vertical axis; positive turns the sheet's front to the right. Edge-on at a quarter
    /// turn, its back shows past that.
    pub turn: f32,
    /// Radians about the horizontal axis; positive leans the top back, away from the camera.
    pub lean: f32,
    /// World units off the ground. The sheet rises on screen, its shadow stays on the ground.
    pub lift: f32,
    /// Curl along the sheet's height, in radians from its middle to its top and bottom edges; positive curls
    /// both edges towards the camera.
    pub bend: f32,
}

impl Sheet {
    pub const FLAT: Sheet = Sheet {
        turn: 0.0,
        lean: 0.0,
        lift: 0.0,
        bend: 0.0,
    };
}

/// How steeply the fixed camera looks down at the ground, as how far up the screen a point moves per world unit
/// it lies away from the camera. Standing sheets are drawn at full height; one lying on its back shows at this
/// fraction of its height. `PITCH` in `src/shader.wgsl` must match.
pub const PITCH: f32 = 0.5;

/// Place a point given relative to a sheet's anchor (x right, y down, z towards the camera) by turning it by
/// `turn` about the vertical axis and leaning it by `lean`, then project it onto the screen. Matches `orient` and
/// `project` in `src/shader.wgsl`. Returns the screen offset and the depth towards the camera.
pub fn orient(p: [f32; 3], turn: f32, lean: f32) -> (f32, f32, f32) {
    let (ct, st) = (turn.cos(), turn.sin());
    let (x, y, z) = (p[0] * ct + p[2] * st, p[1], -p[0] * st + p[2] * ct);
    let (cl, sl) = (lean.cos(), lean.sin());
    let (y, z) = (y * cl - z * sl, y * sl + z * cl);
    (x, y + z * PITCH, z)
}
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
    /// Its puppet (an id into `RIGS`).
    pub rig: usize,
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
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
    /// Uniform in 0..1.
    pub fn f(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / 16_777_216.0
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
