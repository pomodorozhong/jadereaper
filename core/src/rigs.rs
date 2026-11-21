//! Puppet rigs: a character is a few rigid paper parts placed around its ground point, and it is animated by
//! posing the whole puppet (shift, tilt, squash) rather than by sprite frames. Each rig has its own wind-up, attack,
//! hurt and roll poses; walking flips between two tilts. The rigs are in `content/game.json`.

use crate::game::Game;
use core::f32::consts::TAU;

/// One paper part of a rig, placed relative to the ground point with the character facing right.
#[derive(Clone, Copy, Debug)]
pub struct RigPart {
    pub part: f32,
    pub x: f32,
    pub y: f32,
    pub hw: f32,
    pub hh: f32,
    pub rot: f32,
    pub color: [f32; 3],
    /// Held in the hand (the scythe): it swings with an attack instead of following the pose.
    pub held: bool,
}

/// A whole-puppet pose: a shift, a tilt about the ground point, and a squash.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub dx: f32,
    pub dy: f32,
    pub tilt: f32,
    pub sx: f32,
    pub sy: f32,
}

impl Pose {
    pub const REST: Pose = Pose { dx: 0.0, dy: 0.0, tilt: 0.0, sx: 1.0, sy: 1.0 };

    /// Blend from this pose to another by `k` in 0..1.
    pub fn mix(&self, o: &Pose, k: f32) -> Pose {
        let l = |a: f32, b: f32| a + (b - a) * k;
        Pose { dx: l(self.dx, o.dx), dy: l(self.dy, o.dy), tilt: l(self.tilt, o.tilt), sx: l(self.sx, o.sx), sy: l(self.sy, o.sy) }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rig {
    pub parts: &'static [RigPart],
    pub windup: Pose,
    pub attack: Pose,
    pub hurt: Pose,
    pub roll: Pose,
}

impl Rig {
    /// The rig's height from the ground at rest.
    pub fn height(&self) -> f32 {
        self.parts.iter().map(|p| -p.y + p.hh).fold(0.0, f32::max)
    }
}

/// What a puppet is doing, for its pose.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Doing {
    /// Standing or walking; `step` flips between the two walk tilts, `lean` is how far each tilts.
    Walk { step: bool, lean: f32, hop: f32 },
    /// Winding up, `k` from 0 to 1.
    Windup { k: f32 },
    /// Striking, `k` from 0 to 1.
    Attack { k: f32 },
    Hurt,
    /// Knocked flat.
    Down,
    /// Rolling, `k` from 0 to 1 through the roll's movement: the puppet tucks and spins once.
    Roll { k: f32 },
}

/// How a puppet is drawn this frame.
#[derive(Clone, Copy, Debug)]
pub struct Look {
    pub x: f32,
    pub y: f32,
    /// 1 facing right, -1 facing left.
    pub face: f32,
    /// Drawn size; hit areas do not change.
    pub scale: f32,
    pub flash: f32,
    pub alpha: f32,
    pub seed: f32,
    /// The held part's angle, while swinging.
    pub swing: Option<f32>,
    /// Blends every part's colour halfway to this one (the Dragon's stance).
    pub tint: Option<[f32; 3]>,
}

impl Game {
    /// The pose for what a rig is doing.
    pub fn rig_pose(rig: &Rig, doing: Doing) -> (Pose, f32) {
        match doing {
            Doing::Walk { step, lean, hop } => {
                let t = if step { lean } else { -lean };
                (Pose { tilt: t, dy: if step { 0.0 } else { hop }, ..Pose::REST }, 0.0)
            }
            Doing::Windup { k } => (Pose::REST.mix(&rig.windup, (k * 3.0).min(1.0)), 0.0),
            Doing::Attack { k } => (rig.windup.mix(&rig.attack, (k * 4.0).min(1.0)), 0.0),
            Doing::Hurt => (rig.hurt, 0.0),
            Doing::Down => (Pose { tilt: -1.35, ..rig.hurt }, 0.0),
            Doing::Roll { k } => {
                let tuck = (k * core::f32::consts::PI).sin();
                (Pose::REST.mix(&rig.roll, tuck), k.min(1.0) * TAU)
            }
        }
    }

    /// Draw a rig: each part placed by the pose (tilted about the ground point, squashed, mirrored by facing).
    pub(crate) fn draw_rig(&mut self, rig: &Rig, doing: Doing, look: Look) {
        let (pose, spin) = Game::rig_pose(rig, doing);
        let (f, s) = (look.face, look.scale);
        let tilt = (pose.tilt + spin) * f;
        let (c, sn) = (tilt.cos(), tilt.sin());
        // A spinning puppet turns about its middle, not its feet.
        let pivot = if spin != 0.0 { rig.height() * 0.5 * s } else { 0.0 };
        let mut held = None;
        for (k, p) in rig.parts.iter().enumerate() {
            let tc = match look.tint {
                Some(t) => [(p.color[0] + t[0]) * 0.5, (p.color[1] + t[1]) * 0.5, (p.color[2] + t[2]) * 0.5],
                None => p.color,
            };
            let col = [tc[0], tc[1], tc[2], look.alpha];
            let seed = (look.seed + k as f32 * 0.17).fract();
            if p.held {
                if let Some(a) = look.swing {
                    // Out in front and swinging, on top of the body.
                    let (hx, hy) = (look.x + f * 12.0 * s + pose.dx * f, look.y - 42.0 * s + pose.dy);
                    held = Some((hx, hy, p.hw * s * f, p.hh * s, a * f, p.part, look.flash, seed, col));
                    continue;
                }
            }
            let (ox, oy) = (p.x * f * s * pose.sx, p.y * s * pose.sy + pivot);
            let (rx, ry) = (ox * c - oy * sn, ox * sn + oy * c);
            let x = look.x + rx + pose.dx * f;
            let y = look.y + ry - pivot + pose.dy;
            self.put(x, y, p.hw * s * pose.sx * f, p.hh * s * pose.sy, p.rot * f + tilt, p.part, look.flash, seed, col);
        }
        if let Some((x, y, hw, hh, rot, part, flash, seed, col)) = held {
            self.put(x, y, hw, hh, rot, part, flash, seed, col);
        }
    }
}
