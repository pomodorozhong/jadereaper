//! Puppet rigs: a character is a few rigid paper parts placed around its ground point, and it is animated by
//! posing the whole puppet (shift, tilt, squash) rather than by sprite frames. Each rig has its own wind-up, attack,
//! hurt and roll poses; walking flips between two tilts. The rigs are in `content/game.json`.
//!
//! Each part is also a sheet of paper in 3D: a whole puppet can turn, lean, lift and bend (`Sheet`), and its parts
//! are then drawn back to front by their depth after the turn, not by a depth buffer.

use crate::defs::{orient, Sheet};
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
    pub const REST: Pose = Pose {
        dx: 0.0,
        dy: 0.0,
        tilt: 0.0,
        sx: 1.0,
        sy: 1.0,
    };

    /// Blend from this pose to another by `k` in 0..1.
    pub fn mix(&self, o: &Pose, k: f32) -> Pose {
        let l = |a: f32, b: f32| a + (b - a) * k;
        Pose {
            dx: l(self.dx, o.dx),
            dy: l(self.dy, o.dy),
            tilt: l(self.tilt, o.tilt),
            sx: l(self.sx, o.sx),
            sy: l(self.sy, o.sy),
        }
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
    Walk {
        step: bool,
        lean: f32,
        hop: f32,
    },
    /// Winding up, `k` from 0 to 1.
    Windup {
        k: f32,
    },
    /// Striking, `k` from 0 to 1.
    Attack {
        k: f32,
    },
    Hurt,
    /// Knocked flat.
    Down,
    /// Rolling, `k` from 0 to 1 through the roll's movement: the puppet tucks and spins once.
    Roll {
        k: f32,
    },
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
    /// How the whole puppet is held in 3D, turning about its ground point.
    pub sheet: Sheet,
}

/// How far apart a puppet's parts stand, front to back, in world units, in their order in the rig. It only
/// decides the drawing order after a turn; it does not move the parts.
const LAYER: f32 = 1.0;
/// Most parts a rig can have, counting the held part.
const MAX_RIG_PARTS: usize = 16;

/// One part placed for drawing, before it is sorted by depth.
#[derive(Clone, Copy, Default)]
struct Placed {
    depth: f32,
    x: f32,
    y: f32,
    hw: f32,
    hh: f32,
    rot: f32,
    part: f32,
    seed: f32,
    color: [f32; 4],
}

impl Game {
    /// The pose for what a rig is doing.
    pub fn rig_pose(rig: &Rig, doing: Doing) -> (Pose, f32) {
        match doing {
            Doing::Walk { step, lean, hop } => {
                let t = if step { lean } else { -lean };
                (
                    Pose {
                        tilt: t,
                        dy: if step { 0.0 } else { hop },
                        ..Pose::REST
                    },
                    0.0,
                )
            }
            Doing::Windup { k } => (Pose::REST.mix(&rig.windup, (k * 3.0).min(1.0)), 0.0),
            Doing::Attack { k } => (rig.windup.mix(&rig.attack, (k * 4.0).min(1.0)), 0.0),
            Doing::Hurt => (rig.hurt, 0.0),
            Doing::Down => (
                Pose {
                    tilt: -1.35,
                    ..rig.hurt
                },
                0.0,
            ),
            Doing::Roll { k } => {
                let tuck = (k * core::f32::consts::PI).sin();
                (Pose::REST.mix(&rig.roll, tuck), k.min(1.0) * TAU)
            }
        }
    }

    /// How the ?debug toggle holds a puppet: turning round and round, leaning, lifting and bending, each puppet
    /// out of step with the others.
    pub(crate) fn debug_sheet(&self, seed: f32) -> Sheet {
        let t = self.anim + seed * 7.0;
        Sheet {
            turn: t * 1.6,
            lean: 0.6 * (t * 1.1).sin(),
            lift: 12.0 * (0.5 + 0.5 * (t * 2.3).sin()),
            bend: 0.5 * (t * 1.7).sin(),
        }
    }

    /// Draw a rig: each part placed by the pose (tilted about the ground point, squashed, mirrored by facing), then
    /// the whole puppet turned and leaned in 3D about its ground point, and its parts drawn back to front.
    pub(crate) fn draw_rig(&mut self, rig: &Rig, doing: Doing, look: Look) {
        let (pose, spin) = Game::rig_pose(rig, doing);
        let (f, s) = (look.face, look.scale);
        let tilt = (pose.tilt + spin) * f;
        let (c, sn) = (tilt.cos(), tilt.sin());
        // A spinning puppet turns about its middle, not its feet.
        let pivot = if spin != 0.0 {
            rig.height() * 0.5 * s
        } else {
            0.0
        };
        let sheet = if self.debug_spin {
            self.debug_sheet(look.seed)
        } else {
            look.sheet
        };
        let mut placed = [Placed::default(); MAX_RIG_PARTS];
        let mut n = 0;
        let mut held = None;
        for (k, p) in rig.parts.iter().enumerate().take(MAX_RIG_PARTS) {
            let tc = match look.tint {
                Some(t) => [
                    (p.color[0] + t[0]) * 0.5,
                    (p.color[1] + t[1]) * 0.5,
                    (p.color[2] + t[2]) * 0.5,
                ],
                None => p.color,
            };
            let color = [tc[0], tc[1], tc[2], look.alpha];
            let seed = (look.seed + k as f32 * 0.17).fract();
            if p.held {
                if let Some(a) = look.swing {
                    // Out in front and swinging, on top of the body.
                    let (ox, oy) = (f * 12.0 * s + pose.dx * f, -42.0 * s + pose.dy);
                    held = Some(Placed {
                        depth: 0.0,
                        x: ox,
                        y: oy,
                        hw: p.hw * s * f,
                        hh: p.hh * s,
                        rot: a * f,
                        part: p.part,
                        seed,
                        color,
                    });
                    continue;
                }
            }
            let (ox, oy) = (p.x * f * s * pose.sx, p.y * s * pose.sy + pivot);
            let (rx, ry) = (ox * c - oy * sn, ox * sn + oy * c);
            placed[n] = Placed {
                depth: k as f32,
                x: rx + pose.dx * f,
                y: ry - pivot + pose.dy,
                hw: p.hw * s * pose.sx * f,
                hh: p.hh * s * pose.sy,
                rot: p.rot * f + tilt,
                part: p.part,
                seed,
                color,
            };
            n += 1;
        }
        if let Some(h) = held {
            if n < MAX_RIG_PARTS {
                placed[n] = Placed {
                    depth: rig.parts.len() as f32,
                    ..h
                };
                n += 1;
            }
        }
        // Turn the puppet about its ground point. A part's layer only counts towards its depth, so a puppet at
        // rest is drawn exactly where and in the order it always was.
        for q in placed[..n].iter_mut() {
            let (x, y, _) = orient([q.x, q.y, 0.0], sheet.turn, sheet.lean);
            q.depth = orient([q.x, q.y, q.depth * LAYER], sheet.turn, sheet.lean).2;
            q.x = x;
            q.y = y;
        }
        // Back to front; a stable insertion sort, since rigs have a handful of parts.
        for i in 1..n {
            let mut j = i;
            while j > 0 && placed[j - 1].depth > placed[j].depth {
                placed.swap(j - 1, j);
                j -= 1;
            }
        }
        for q in &placed[..n] {
            self.put_sheet(
                look.x + q.x,
                look.y + q.y,
                q.hw,
                q.hh,
                q.rot,
                q.part,
                look.flash,
                q.seed,
                q.color,
                sheet,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bot::Bot;
    use crate::defs::*;

    fn look(sheet: Sheet) -> Look {
        Look {
            x: 0.0,
            y: 0.0,
            face: 1.0,
            scale: 1.0,
            flash: 0.0,
            alpha: 1.0,
            seed: 0.3,
            swing: None,
            tint: None,
            sheet,
        }
    }

    fn drawn(sheet: Sheet) -> Vec<Inst> {
        let mut game = Game::new(3);
        game.set_view(640.0, 360.0);
        game.cam = (0.0, 0.0);
        game.inst.clear();
        game.draw_rig(
            &RIGS[RG_PLAYER],
            Doing::Walk {
                step: true,
                lean: 0.0,
                hop: 0.0,
            },
            look(sheet),
        );
        game.inst.clone()
    }

    #[test]
    fn instances_are_sixteen_floats() {
        assert_eq!(core::mem::size_of::<Inst>(), INST_FLOATS * 4);
        assert_eq!(INST_FLOATS, 16);
    }

    #[test]
    fn a_puppet_at_rest_is_flat_and_in_rig_order() {
        let inst = drawn(Sheet::FLAT);
        let ids: Vec<f32> = inst.iter().map(|i| i.part % PART_SLOTS).collect();
        let rig: Vec<f32> = RIGS[RG_PLAYER].parts.iter().map(|p| p.part).collect();
        assert_eq!(ids, rig);
        for (i, p) in inst.iter().zip(RIGS[RG_PLAYER].parts) {
            assert_eq!((i.x, i.y), (p.x, p.y));
            assert_eq!((i.turn, i.lean, i.lift, i.bend), (0.0, 0.0, 0.0, 0.0));
        }
    }

    #[test]
    fn parts_reorder_when_a_puppet_turns_round() {
        let front = drawn(Sheet::FLAT);
        let back = drawn(Sheet {
            turn: core::f32::consts::PI,
            ..Sheet::FLAT
        });
        let ids = |v: &[Inst]| v.iter().map(|i| i.part % PART_SLOTS).collect::<Vec<_>>();
        let mut reversed = ids(&front);
        reversed.reverse();
        assert_eq!(ids(&back), reversed);
        // Seen from behind, a part on the right is now on the left.
        let head = |v: &[Inst]| {
            *v.iter()
                .find(|i| i.part % PART_SLOTS == P_PLAYER_HEAD)
                .unwrap()
        };
        assert!((head(&back).x + head(&front).x).abs() < 1e-3);
        assert!(back.iter().all(|i| i.turn == core::f32::consts::PI));
    }

    #[test]
    fn lying_back_shortens_a_puppet_and_lift_is_passed_on() {
        // The camera looks down, so a puppet lying on its back still shows at `PITCH` of its height.
        let upright = drawn(Sheet::FLAT);
        let leaned = drawn(Sheet {
            lean: core::f32::consts::FRAC_PI_2,
            lift: 9.0,
            ..Sheet::FLAT
        });
        let head = |v: &[Inst]| {
            *v.iter()
                .find(|i| i.part % PART_SLOTS == P_PLAYER_HEAD)
                .unwrap()
        };
        assert!((head(&leaned).y - head(&upright).y * PITCH).abs() < 1e-3);
        assert!(leaned.iter().all(|i| i.lift == 9.0));
    }

    #[test]
    fn the_debug_spin_changes_only_the_drawing() {
        let run = |spin: bool| {
            let mut game = Game::new(11);
            game.set_view(640.0, 360.0);
            game.set_debug_spin(spin);
            game.running = true;
            game.enter_room(0);
            let mut bot = Bot::default();
            let mut turned = false;
            for _ in 0..(60 * 20) {
                game.input = bot.input(&game);
                game.update(1.0 / 60.0);
                if game.choosing {
                    game.choose(0);
                }
                turned |= game.inst.iter().any(|i| i.turn != 0.0);
            }
            (game.state_hash(), turned)
        };
        let (plain, plain_turned) = run(false);
        let (spun, spun_turned) = run(true);
        assert_eq!(plain, spun);
        assert!(!plain_turned);
        assert!(spun_turned);
    }
}
