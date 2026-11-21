//! Animation state for the renderer: for each character, what its body is doing, how far through it, which way
//! it points, and the last hit it took. The core keeps the timings; this buffer lets the drawing side pose a
//! character from them (decision 0006) without reading the simulation's own structs.
//!
//! One record per character, `ANIM_FLOATS` floats each, player first:
//!
//! | Index | Field |
//! | --- | --- |
//! | 0 | who: 0 the player, 1 + kind for an enemy |
//! | 1 | id: 0 for the player, the enemy id otherwise |
//! | 2, 3 | x, y: the ground point |
//! | 4 | face: 1 right, -1 left |
//! | 5 | phase, an `A_*` id |
//! | 6 | k: 0..1 through the phase |
//! | 7 | the phase's length in seconds (0 when it has none, such as idle) |
//! | 8 | the cancel point as a share of the recovery, or -1 when nothing can cancel it |
//! | 9, 10 | dir: the unit direction of the action (a skill or move), or the aim or facing |
//! | 11 | combo step, or the move id for an enemy |
//! | 12, 13 | the last hit's unit direction, from the attacker to this character |
//! | 14 | the last hit's damage |
//! | 15 | seconds since the last hit (large when never hit) |
//!
//! Hit-stop freezes the simulation but not this buffer; `Game::hitstop` is in the state array at 54.

use crate::defs::*;
use crate::game::{Act, Game, Phase};
use crate::player::{DOWN_TIME, GRAB_MAX, HURT_TIME};

pub const ANIM_FLOATS: usize = 16;

pub const A_IDLE: f32 = 0.0;
pub const A_WALK: f32 = 1.0;
/// Charging: a skill's startup, an enemy's wind-up.
pub const A_STARTUP: f32 = 2.0;
pub const A_ACTIVE: f32 = 3.0;
pub const A_RECOVERY: f32 = 4.0;
pub const A_ROLL: f32 = 5.0;
pub const A_HURT: f32 = 6.0;
pub const A_DOWN: f32 = 7.0;
pub const A_GRABBED: f32 = 8.0;
/// Interrupted: an enemy staggered or frozen.
pub const A_STUNNED: f32 = 9.0;

/// The last hit a character took.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    /// Unit direction from the attacker to the one hit.
    pub dx: f32,
    pub dy: f32,
    pub dmg: f32,
    /// Simulation time of the hit.
    pub at: f32,
}

impl Hit {
    pub const NONE: Hit = Hit {
        dx: 0.0,
        dy: 0.0,
        dmg: 0.0,
        at: -1e6,
    };
}

/// One character's record; see the module docs for the layout.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Anim {
    pub who: f32,
    pub id: f32,
    pub x: f32,
    pub y: f32,
    pub face: f32,
    pub phase: f32,
    pub k: f32,
    pub len: f32,
    pub cancel: f32,
    pub dx: f32,
    pub dy: f32,
    pub step: f32,
    pub hit_dx: f32,
    pub hit_dy: f32,
    pub hit_dmg: f32,
    pub hit_age: f32,
}

const _: () = assert!(core::mem::size_of::<Anim>() == ANIM_FLOATS * 4);

/// A phase, how far through it (0..1) and its length.
fn through(phase: f32, t: f32, len: f32) -> (f32, f32, f32) {
    (
        phase,
        if len > 0.0 {
            (t / len).clamp(0.0, 1.0)
        } else {
            0.0
        },
        len,
    )
}

impl Game {
    pub(crate) fn build_anims(&mut self) {
        self.anims.clear();
        let p = &self.player;
        let mut cancel = -1.0;
        let (mut dir, mut step) = (p.aim, 0.0);
        let (phase, k, len) = match p.act {
            Act::Free if p.moving => (A_WALK, 0.0, 0.0),
            Act::Free => (A_IDLE, 0.0, 0.0),
            Act::Roll { t, dx, dy } => {
                dir = (dx, dy);
                through(A_ROLL, t, ROLL_TIME + ROLL_RECOVER)
            }
            Act::Skill {
                slot,
                t,
                dx,
                dy,
                step: s,
                ..
            } => {
                let d = self.def(slot);
                dir = (dx, dy);
                step = s as f32;
                let (a0, a1) = (d.startup, d.startup + d.active);
                if t < a0 {
                    through(A_STARTUP, t, a0)
                } else if t < a1 {
                    through(A_ACTIVE, t - a0, d.active)
                } else {
                    cancel = if d.recovery > 0.0 {
                        (d.cancel / d.recovery).min(1.0)
                    } else {
                        0.0
                    };
                    through(A_RECOVERY, t - a1, d.recovery)
                }
            }
            Act::Hurt { t } => through(A_HURT, t, HURT_TIME),
            Act::Down { t } => through(A_DOWN, t, DOWN_TIME),
            Act::Grabbed { t, .. } => through(A_GRABBED, t, GRAB_MAX),
        };
        let rec = Anim {
            who: 0.0,
            id: 0.0,
            x: p.x,
            y: p.y,
            face: p.face,
            phase,
            k,
            len,
            cancel,
            dx: dir.0,
            dy: dir.1,
            step,
            hit_dx: p.hit.dx,
            hit_dy: p.hit.dy,
            hit_dmg: p.hit.dmg,
            hit_age: self.time - p.hit.at,
        };
        self.anims.push(rec);

        for i in 0..self.enemies.len() {
            let e = self.enemies[i];
            let m = &crate::moves::MOVES[e.mv as usize];
            let down = self.boss_of(i).map(|b| b.down);
            let (phase, k, len) = if let Some(t) = down.filter(|&t| t > 0.0) {
                through(A_DOWN, crate::boss::DOWN_TIME - t, crate::boss::DOWN_TIME)
            } else if e.status.frozen > 0.0 {
                through(
                    A_STUNNED,
                    crate::elements::FREEZE_TIME - e.status.frozen,
                    crate::elements::FREEZE_TIME,
                )
            } else if e.stun > 0.0 {
                through(
                    A_STUNNED,
                    crate::enemy::STUN_TIME - e.stun,
                    crate::enemy::STUN_TIME,
                )
            } else {
                match e.phase {
                    Phase::Windup => through(A_STARTUP, e.pt, self.windup_of(&e)),
                    Phase::Active => through(A_ACTIVE, e.pt, m.active),
                    Phase::Recover => through(A_RECOVERY, e.pt, m.recover),
                    Phase::Approach => (A_WALK, 0.0, 0.0),
                }
            };
            let dir = if e.phase == Phase::Approach {
                (e.face, 0.0)
            } else {
                (e.dx, e.dy)
            };
            self.anims.push(Anim {
                who: 1.0 + e.kind as f32,
                id: e.id as f32,
                x: e.x,
                y: e.y,
                face: e.face,
                phase,
                k,
                len,
                cancel: -1.0,
                dx: dir.0,
                dy: dir.1,
                step: e.mv as f32,
                hit_dx: e.hit.dx,
                hit_dy: e.hit.dy,
                hit_dmg: e.hit.dmg,
                hit_age: self.time - e.hit.at,
            });
        }
    }
}
