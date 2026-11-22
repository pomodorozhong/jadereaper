//! The pose gallery (`?gallery`): every rig in every pose in one grid, for tuning the 3D pose numbers. One row per
//! rig, one column per pose (`GALLERY_POSES`), all scrubbed together by one progress value. While it is on, the
//! run is frozen and only the gallery is drawn. Drawing only; nothing here is in `state_hash`.

use crate::anim::*;
use crate::defs::*;
use crate::game::Game;
use crate::rigs::{pose, Gait, Look};
use crate::twist::{self, Twist};
use core::f32::consts::FRAC_PI_2;

/// The gallery's columns, in order. `GALLERY_POSES` in `src/main.js` names them the same way.
pub const GALLERY_POSES: usize = 11;
/// Share of the view kept for the column labels at the top and the row labels at the left.
pub const GALLERY_TOP: f32 = 0.08;
pub const GALLERY_LEFT: f32 = 0.08;
/// Share of the view left empty at the bottom, under the last row.
pub const GALLERY_BOTTOM: f32 = 0.04;

/// What the gallery shows: how far through each pose (0..1), the hit's damage, and the direction (radians, 0 is
/// right, positive is down the screen) that skills aim and hits push.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GalleryView {
    pub progress: f32,
    pub damage: f32,
    pub angle: f32,
}

impl Default for GalleryView {
    fn default() -> Self {
        GalleryView {
            progress: 0.5,
            damage: 20.0,
            angle: 0.5,
        }
    }
}

fn smooth(k: f32) -> f32 {
    let k = k.clamp(0.0, 1.0);
    k * k * (3.0 - 2.0 * k)
}

impl Game {
    /// Turn the gallery on or off.
    pub fn set_gallery(&mut self, on: bool) {
        self.gallery = on;
    }

    /// The record, gait, held part's swing and extra turn for one rig in one gallery column.
    fn gallery_cell(&self, rig: usize, col: usize) -> (Anim, Gait, Option<f32>, f32) {
        let v = self.gallery_view;
        let k = v.progress.clamp(0.0, 1.0);
        let (dx, dy) = (v.angle.cos(), v.angle.sin());
        let mut a = Anim {
            who: rig as f32,
            id: col as f32,
            x: 0.0,
            y: 0.0,
            face: 1.0,
            phase: A_IDLE,
            k,
            len: 0.0,
            cancel: -1.0,
            dx,
            dy,
            step: 0.0,
            hit_dx: 0.0,
            hit_dy: 0.0,
            hit_dmg: 0.0,
            hit_age: 1e6,
            twist: 0.0,
        };
        let player = rig == RG_PLAYER;
        // The walk tilt each rig uses in play: the player's, or its enemy kind's waddle.
        let lean = if player {
            0.10
        } else {
            KINDS
                .iter()
                .position(|kd| kd.rig == rig)
                .map_or(0.0, |kind| {
                    self.waddle
                        .get(kind)
                        .map_or(KINDS[kind].tilt.to_radians(), |w| w.0)
                })
        };
        // Scrubbing moves the clock too, so trembles, shakes and wobbles can be stepped through.
        let mut gait = Gait {
            clock: k * 2.0,
            steps: 0.0,
            lean: 0.0,
            hop: -1.5,
        };
        let mut swing = None;
        let mut turn = 0.0;
        match col {
            0 => {}
            1 => {
                a.phase = A_WALK;
                gait.steps = k * 4.0;
                gait.lean = lean;
                gait.hop = -3.0;
            }
            2 => {
                (a.phase, a.len) = (A_STARTUP, 0.4);
                swing = Some(-1.5 - 0.3 * k);
            }
            3 => {
                (a.phase, a.len) = (A_ACTIVE, 0.1);
                swing = Some(-1.8 + 3.1 * smooth(k));
            }
            4 => {
                (a.phase, a.len) = (A_RECOVERY, 0.4);
                if player {
                    a.cancel = 0.5;
                }
                swing = Some(1.3);
            }
            5 => {
                // A hit pushing along the gallery's direction, as a share of 100 HP (the player's): it swings
                // back and forth, or spins whole turns, as it plays.
                a.phase = A_HURT;
                (a.hit_dx, a.hit_dy, a.hit_dmg) = (dx, dy, v.damage);
                let mut t = Twist::default();
                t.hit(v.damage / 100.0, dx);
                a.twist = twist::through(t, k);
            }
            6 => {
                a.phase = A_DOWN;
                let mut t = Twist::default();
                t.spin(twist::DOWN_TURNS, 1.0);
                a.twist = twist::through(t, k);
            }
            7 => (a.phase, a.len) = (A_GRABBED, 3.0),
            8 => (a.phase, a.len) = (A_ROLL, ROLL_TIME + ROLL_RECOVER),
            9 => {
                a.phase = A_STUNNED;
                let mut t = Twist::default();
                t.spin(twist::STUN_TURNS, 1.0);
                a.twist = twist::through(t, k);
            }
            _ => {
                // Turning round from facing right to facing left, through edge-on.
                let w = 1.0 - 2.0 * k;
                a.face = if w < 0.0 { -1.0 } else { 1.0 };
                turn = a.face * (1.0 - w.abs()) * FRAC_PI_2;
            }
        }
        if !player {
            swing = None;
        }
        (a, gait, swing, turn)
    }

    /// Draw the gallery: each rig scaled to fit its row, standing on a shadow in each pose's cell.
    pub(crate) fn build_gallery(&mut self) {
        self.inst.clear();
        self.cam = (0.0, 0.0);
        self.zoom = 1.0;
        let (vw, vh) = (self.view_w, self.view_h);
        let rows = RIGS.len();
        let (x0, y0) = (-vw + 2.0 * vw * GALLERY_LEFT, -vh + 2.0 * vh * GALLERY_TOP);
        let cw = 2.0 * vw * (1.0 - GALLERY_LEFT) / GALLERY_POSES as f32;
        let ch = 2.0 * vh * (1.0 - GALLERY_TOP - GALLERY_BOTTOM) / rows as f32;
        for (r, rig) in RIGS.iter().enumerate() {
            let width = rig
                .parts
                .iter()
                .map(|p| p.x.abs() + p.hw.abs())
                .fold(1.0, f32::max);
            let s = (0.62 * ch / rig.height().max(1.0)).min(0.4 * cw / width);
            for c in 0..GALLERY_POSES {
                let (a, gait, swing, turn) = self.gallery_cell(r, c);
                let (x, y) = (x0 + (c as f32 + 0.5) * cw, y0 + (r as f32 + 0.85) * ch);
                self.put(
                    x,
                    y,
                    0.35 * cw,
                    0.08 * ch,
                    0.0,
                    P_GROUND_SHADOW,
                    0.0,
                    0.0,
                    [INK[0], INK[1], INK[2], 0.15],
                );
                let p3 = pose(rig, &self.sheet_pose[r], &a, gait);
                self.draw_rig(
                    rig,
                    p3,
                    Look {
                        x,
                        y,
                        face: a.face,
                        scale: s,
                        flash: 0.0,
                        alpha: 1.0,
                        seed: (r * GALLERY_POSES + c) as f32 * 0.137 % 1.0,
                        swing,
                        tint: None,
                        sheet: Sheet {
                            turn,
                            ..Sheet::FLAT
                        },
                    },
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_gallery_draws_every_rig_in_every_pose_and_freezes_the_run() {
        let mut game = Game::new(5);
        game.set_view(640.0, 360.0);
        game.running = true;
        game.enter_room(0);
        for _ in 0..30 {
            game.update(1.0 / 60.0);
        }
        let before = game.state_hash();
        game.set_gallery(true);
        for k in 0..=10 {
            game.gallery_view.progress = k as f32 / 10.0;
            game.update(1.0 / 60.0);
            // A shadow and every part for each rig and pose.
            let parts: usize = RIGS.iter().map(|r| r.parts.len()).sum();
            assert_eq!(game.inst.len(), (parts + RIGS.len()) * GALLERY_POSES);
            assert!(game
                .inst
                .iter()
                .all(|i| [i.x, i.y, i.turn, i.lean, i.lift, i.bend]
                    .iter()
                    .all(|v| v.is_finite())));
        }
        assert_eq!(game.state_hash(), before);
        game.set_gallery(false);
        game.update(1.0 / 60.0);
        assert_ne!(game.state_hash(), before);
    }
}
