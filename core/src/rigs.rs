//! Puppet rigs: a character is a few rigid paper parts placed around its ground point, and it is animated by
//! posing the whole puppet (shift, tilt, squash) rather than by sprite frames. Each rig has its own wind-up, attack,
//! hurt and roll poses; walking flips between two tilts. The rigs are in `content/game.json`.
//!
//! Each part is also a sheet of paper in 3D: a whole puppet can turn, lean, lift and bend (`Sheet`), and its parts
//! are then drawn back to front by their depth after the turn, not by a depth buffer.

use crate::anim::*;
use crate::defs::{orient, Sheet, ROLL_TIME};
use crate::game::Game;

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

/// A rig's 3D pose numbers (`sheet` in `content/game.json`): how its sheet turns, leans, lifts and bends in each
/// animation phase. Angles are in radians, lifts in world units, and turns are for the puppet facing right. All
/// zero (the bosses, for now) keeps the puppet flat.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SheetPose {
    /// Charging and striking: the most it turns towards the skill's direction on the ground.
    pub aim: f32,
    /// Charging: how far it leans back at full charge.
    pub charge: f32,
    /// Charging: how hard it trembles over the last fifth.
    pub tremble: f32,
    /// Striking: the lean it snaps to (negative is forward, towards the camera).
    pub strike: f32,
    /// Recovery: the lean and bend it sags to before the cancel point.
    pub sag: f32,
    pub sag_bend: f32,
    /// Recovery: how high it hops as it pops upright at the cancel point.
    pub pop: f32,
    /// Hit: lean and bend per point of damage, and the most of either.
    pub hit: f32,
    pub hit_bend: f32,
    pub hit_max: f32,
    /// Down: how far it falls back (a quarter turn is flat on the ground).
    pub down: f32,
    /// Grabbed: how far it is pinned back, and how hard it shakes.
    pub grab: f32,
    pub shake: f32,
    /// Roll: whole turns about its vertical axis.
    pub roll: f32,
    /// Stunned: how far it wobbles each way towards edge-on, and wobbles per second.
    pub wobble: f32,
    pub wobble_rate: f32,
    /// Walking: how high it bobs each step, and how far it sways (turns) each way.
    pub bob: f32,
    pub sway: f32,
}

impl SheetPose {
    /// A number by its name in `SHEET_KEYS`, for the ?debug setter.
    pub fn field_mut(&mut self, name: &str) -> Option<&mut f32> {
        Some(match name {
            "aim" => &mut self.aim,
            "charge" => &mut self.charge,
            "tremble" => &mut self.tremble,
            "strike" => &mut self.strike,
            "sag" => &mut self.sag,
            "sag_bend" => &mut self.sag_bend,
            "pop" => &mut self.pop,
            "hit" => &mut self.hit,
            "hit_bend" => &mut self.hit_bend,
            "hit_max" => &mut self.hit_max,
            "down" => &mut self.down,
            "grab" => &mut self.grab,
            "shake" => &mut self.shake,
            "roll" => &mut self.roll,
            "wobble" => &mut self.wobble,
            "wobble_rate" => &mut self.wobble_rate,
            "bob" => &mut self.bob,
            "sway" => &mut self.sway,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rig {
    pub parts: &'static [RigPart],
    pub windup: Pose,
    pub attack: Pose,
    pub hurt: Pose,
    pub roll: Pose,
    pub sheet: SheetPose,
}

impl Rig {
    /// The rig's height from the ground at rest.
    pub fn height(&self) -> f32 {
        self.parts.iter().map(|p| -p.y + p.hh).fold(0.0, f32::max)
    }
}

/// A puppet's whole pose: the flat pose (shift, tilt, squash) and how the puppet's sheet is held in 3D, both for
/// the puppet facing right. `draw_rig` mirrors them by the facing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose3 {
    pub flat: Pose,
    pub sheet: Sheet,
}

/// The walk cycle and a clock, which the animation record does not carry: they come from the drawing side.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Gait {
    /// Drawing time in seconds, offset per puppet, for trembles, shakes and wobbles.
    pub clock: f32,
    /// Walk steps taken; the puppet flips between its two walk tilts each whole step.
    pub steps: f32,
    /// How far each walk tilt leans, and how high the off step hops (negative is up).
    pub lean: f32,
    pub hop: f32,
}

/// How long a hit flinch takes to snap out, and until it has settled back, in seconds.
const FLINCH_IN: f32 = 0.04;
const FLINCH_OUT: f32 = 0.35;
/// How long the pop upright at the cancel point takes, in seconds.
const POP_TIME: f32 = 0.12;
/// Where an enemy's recovery, which nothing cancels, starts to straighten up, as a share of it.
const ENEMY_RISE: f32 = 0.7;
/// Down: the share of the time it takes to fall, and the share at the end it takes to get up.
const FALL: f32 = 0.25;
const RISE: f32 = 0.15;

fn smooth(k: f32) -> f32 {
    let k = k.clamp(0.0, 1.0);
    k * k * (3.0 - 2.0 * k)
}

/// A stop-motion shake in -1..1 that changes `rate` times a second.
fn jitter(clock: f32, rate: f32) -> f32 {
    ((clock * rate).floor() * 1.7).sin()
}

/// The turn, for the puppet facing right, that points its front along the ground direction (`dx`, `dy`), at most
/// `most` either way. A negative turn swings the front towards the camera (down the screen).
fn aim_turn(face: f32, dx: f32, dy: f32, most: f32) -> f32 {
    if dx == 0.0 && dy == 0.0 {
        return 0.0;
    }
    (-dy).atan2(face * dx).clamp(-most, most)
}

/// The pose of a rig from its character's animation record (`core/src/anim.rs`), the rig's 3D numbers and the
/// gait. It reads nothing else, so a test can feed it a made-up record.
pub fn pose(rig: &Rig, n: &SheetPose, a: &Anim, g: Gait) -> Pose3 {
    use core::f32::consts::{PI, TAU};
    let k = a.k;
    let t = k * a.len;
    let aim = aim_turn(a.face, a.dx, a.dy, n.aim);
    let mut sheet = Sheet::FLAT;
    let off_step = (g.steps.floor() as i32) & 1 == 1;
    let walk = Pose {
        tilt: if off_step { -g.lean } else { g.lean },
        dy: if off_step { g.hop } else { 0.0 },
        ..Pose::REST
    };
    let mut flat = match a.phase {
        A_WALK => {
            let s = (g.steps * PI).sin();
            sheet.lift = n.bob * s.abs();
            sheet.turn = n.sway * s;
            walk
        }
        A_STARTUP => {
            // Turn to the skill's direction early, lean back as it charges, hold and tremble over the last fifth.
            sheet.turn = aim * smooth(k * 3.0);
            sheet.lean = n.charge * smooth(k / 0.8);
            if k > 0.8 {
                let r = (k - 0.8) / 0.2;
                sheet.turn += n.tremble * r * jitter(g.clock, 40.0);
                sheet.lean += n.tremble * r * jitter(g.clock + 0.37, 40.0);
            }
            Pose::REST.mix(&rig.windup, (k * 3.0).min(1.0))
        }
        A_ACTIVE => {
            let s = (k * 4.0).min(1.0);
            sheet.turn = aim;
            sheet.lean = n.charge + (n.strike - n.charge) * s;
            rig.windup.mix(&rig.attack, s)
        }
        A_RECOVERY if a.cancel >= 0.0 => {
            if k < a.cancel {
                // Sag until the cancel point.
                let s = smooth(k / a.cancel * 2.0);
                sheet.turn = aim;
                sheet.lean = n.strike + (n.sag - n.strike) * s;
                sheet.bend = n.sag_bend * s;
                rig.attack.mix(&Pose::REST, 0.5 * s)
            } else {
                // Upright at once at the cancel point, with a small hop.
                let u = if a.len > 0.0 {
                    ((k - a.cancel) * a.len / POP_TIME).min(1.0)
                } else {
                    1.0
                };
                sheet.lift = n.pop * (u * PI).sin();
                Pose::REST
            }
        }
        A_RECOVERY => {
            // An enemy sags, then straightens up over the end of its recovery.
            let s = smooth(k / ENEMY_RISE * 2.0);
            let up = 1.0 - smooth((k - ENEMY_RISE) / (1.0 - ENEMY_RISE));
            sheet.turn = aim * up;
            sheet.lean = (n.strike + (n.sag - n.strike) * s) * up;
            sheet.bend = n.sag_bend * s * up;
            Pose::REST.mix(&rig.attack.mix(&Pose::REST, 0.5 * s), up)
        }
        A_ROLL => {
            let m = (t / ROLL_TIME).min(1.0);
            sheet.turn = n.roll * TAU * m;
            Pose::REST.mix(&rig.roll, (m * PI).sin())
        }
        A_DOWN => {
            sheet.lean = n.down * smooth(k / FALL).min(smooth((1.0 - k) / RISE));
            rig.hurt
        }
        A_GRABBED => {
            sheet.lean = n.grab * smooth(t / 0.1);
            sheet.turn = n.shake * jitter(g.clock, 30.0);
            sheet.bend = n.shake * jitter(g.clock + 0.5, 30.0);
            rig.hurt
        }
        A_STUNNED => {
            sheet.turn = n.wobble * (g.clock * n.wobble_rate * TAU).sin();
            rig.hurt
        }
        A_HURT => rig.hurt,
        _ => walk,
    };
    // A hit snaps the puppet away from it and lets it settle back, on top of whatever it is doing. Lying down or
    // pinned, it already shows the hit.
    if a.phase != A_DOWN && a.phase != A_GRABBED && (0.0..FLINCH_OUT).contains(&a.hit_age) {
        let e = if a.hit_age < FLINCH_IN {
            a.hit_age / FLINCH_IN
        } else {
            1.0 - smooth((a.hit_age - FLINCH_IN) / (FLINCH_OUT - FLINCH_IN))
        };
        let lean = (a.hit_dmg * n.hit).min(n.hit_max) * e;
        let bend = (a.hit_dmg * n.hit_bend).min(n.hit_max) * e;
        // Pushed up the screen (away from the camera) the top goes back; pushed sideways it tilts with the push.
        sheet.lean -= a.hit_dy * lean;
        flat.tilt += a.hit_dx * a.face * lean;
        sheet.bend += bend;
    }
    Pose3 { flat, sheet }
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
    /// Added to the pose's sheet, already mirrored: the quick turn through edge-on when the puppet turns round.
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
    /// Towards the camera from the ground point, without the layer: what the shader fades by.
    z: f32,
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

    /// Draw a rig in a pose: each part placed by the flat pose (tilted about the ground point, squashed, mirrored
    /// by facing), then the whole puppet turned, leaned and lifted in 3D about its ground point by the pose's sheet
    /// and the look's, and its parts drawn back to front.
    pub(crate) fn draw_rig(&mut self, rig: &Rig, p3: Pose3, look: Look) {
        let pose = p3.flat;
        let (f, s) = (look.face, look.scale);
        let tilt = pose.tilt * f;
        let (c, sn) = (tilt.cos(), tilt.sin());
        let sheet = if self.debug_spin {
            self.debug_sheet(look.seed)
        } else {
            Sheet {
                turn: p3.sheet.turn * f + look.sheet.turn,
                lean: p3.sheet.lean + look.sheet.lean,
                lift: p3.sheet.lift * s + look.sheet.lift,
                bend: p3.sheet.bend + look.sheet.bend,
            }
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
                        z: 0.0,
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
            let (ox, oy) = (p.x * f * s * pose.sx, p.y * s * pose.sy);
            let (rx, ry) = (ox * c - oy * sn, ox * sn + oy * c);
            placed[n] = Placed {
                depth: k as f32,
                z: 0.0,
                x: rx + pose.dx * f,
                y: ry + pose.dy,
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
            let (x, y, z) = orient([q.x, q.y, 0.0], sheet.turn, sheet.lean);
            q.z = z;
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
        let reach = rig.height() * s;
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
                (q.z, reach),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bot::Bot;
    use crate::defs::*;
    use core::f32::consts::{FRAC_PI_2, PI, TAU};

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

    const REST: Pose3 = Pose3 {
        flat: Pose::REST,
        sheet: Sheet::FLAT,
    };

    fn drawn_pose(p3: Pose3, sheet: Sheet) -> Vec<Inst> {
        let mut game = Game::new(3);
        game.set_view(640.0, 360.0);
        game.cam = (0.0, 0.0);
        game.inst.clear();
        game.draw_rig(&RIGS[RG_PLAYER], p3, look(sheet));
        game.inst.clone()
    }

    fn drawn(sheet: Sheet) -> Vec<Inst> {
        drawn_pose(REST, sheet)
    }

    fn ids(v: &[Inst]) -> Vec<f32> {
        v.iter().map(|i| i.part % PART_SLOTS).collect()
    }

    fn head(v: &[Inst]) -> Inst {
        *v.iter()
            .find(|i| i.part % PART_SLOTS == P_PLAYER_HEAD)
            .unwrap()
    }

    /// A made-up animation record for the player: facing right, acting to the right, never hit.
    fn rec(phase: f32, k: f32, len: f32) -> Anim {
        Anim {
            who: 0.0,
            id: 0.0,
            x: 0.0,
            y: 0.0,
            face: 1.0,
            phase,
            k,
            len,
            cancel: -1.0,
            dx: 1.0,
            dy: 0.0,
            step: 0.0,
            hit_dx: 0.0,
            hit_dy: 0.0,
            hit_dmg: 0.0,
            hit_age: 1e6,
        }
    }

    fn posed(a: &Anim) -> Pose3 {
        let rig = &RIGS[RG_PLAYER];
        pose(rig, &rig.sheet, a, Gait::default())
    }

    #[test]
    fn instances_are_eighteen_floats() {
        assert_eq!(core::mem::size_of::<Inst>(), INST_FLOATS * 4);
        assert_eq!(INST_FLOATS, 18);
    }

    #[test]
    fn far_parts_carry_their_depth() {
        // At rest every part is at depth zero, so the look does not change.
        assert!(drawn(Sheet::FLAT).iter().all(|i| i.depth == 0.0));
        // Lying back, the head is further from the camera than the body.
        let flat = drawn(Sheet {
            lean: core::f32::consts::FRAC_PI_2,
            ..Sheet::FLAT
        });
        let body = *flat
            .iter()
            .find(|i| i.part % PART_SLOTS == P_PLAYER_BODY)
            .unwrap();
        assert!(head(&flat).depth < body.depth && body.depth < 0.0);
        assert!(flat.iter().all(|i| i.reach == RIGS[RG_PLAYER].height()));
    }

    #[test]
    fn every_sheet_key_names_a_number() {
        let mut n = SheetPose::default();
        for k in SHEET_KEYS {
            assert!(n.field_mut(k).is_some(), "{k}");
        }
        let mut game = Game::new(1);
        game.set_pose(RG_BLOT, 1, 0.9);
        assert_eq!(game.sheet_pose[RG_BLOT].charge, 0.9);
    }

    #[test]
    fn a_puppet_at_rest_is_flat_and_in_rig_order() {
        let inst = drawn(Sheet::FLAT);
        let rig: Vec<f32> = RIGS[RG_PLAYER].parts.iter().map(|p| p.part).collect();
        assert_eq!(ids(&inst), rig);
        for (i, p) in inst.iter().zip(RIGS[RG_PLAYER].parts) {
            assert_eq!((i.x, i.y), (p.x, p.y));
            assert_eq!((i.turn, i.lean, i.lift, i.bend), (0.0, 0.0, 0.0, 0.0));
        }
        // Standing still is the rest pose.
        assert_eq!(posed(&rec(A_IDLE, 0.0, 0.0)), REST);
    }

    #[test]
    fn parts_reorder_when_a_puppet_turns_round() {
        let front = drawn(Sheet::FLAT);
        let back = drawn(Sheet {
            turn: PI,
            ..Sheet::FLAT
        });
        let mut reversed = ids(&front);
        reversed.reverse();
        assert_eq!(ids(&back), reversed);
        // Seen from behind, a part on the right is now on the left.
        assert!((head(&back).x + head(&front).x).abs() < 1e-3);
        assert!(back.iter().all(|i| i.turn == PI));
    }

    #[test]
    fn lying_back_shortens_a_puppet_and_lift_is_passed_on() {
        // The camera looks down, so a puppet lying on its back still shows at `PITCH` of its height.
        let upright = drawn(Sheet::FLAT);
        let leaned = drawn(Sheet {
            lean: FRAC_PI_2,
            lift: 9.0,
            ..Sheet::FLAT
        });
        assert!((head(&leaned).y - head(&upright).y * PITCH).abs() < 1e-3);
        assert!(leaned.iter().all(|i| i.lift == 9.0));
    }

    #[test]
    fn the_charging_pose_faces_the_skills_direction() {
        // Within the rig's `aim`, the puppet's front points exactly along the skill's direction on the ground.
        let most = RIGS[RG_PLAYER].sheet.aim;
        for (face, dx, dy) in [
            (1.0, 0.8, 0.6),
            (1.0, 0.8, -0.6),
            (-1.0, -0.8, 0.6),
            (-1.0, -0.8, -0.6),
            (1.0, 1.0, 0.0),
        ] {
            let a = Anim {
                face,
                dx,
                dy,
                ..rec(A_STARTUP, 0.75, 0.4)
            };
            let turn = posed(&a).sheet.turn * face;
            // The front is +x before the mirror; z, towards the camera, is down the screen on the ground.
            let (x, _, z) = orient([face, 0.0, 0.0], turn, 0.0);
            assert!(
                (x - dx).abs() < 1e-4 && (z - dy).abs() < 1e-4,
                "{face} {dx} {dy}: {x} {z}"
            );
        }
        // Straight down the screen, it turns only as far as `aim`, so it never goes edge-on.
        let down = Anim {
            dx: 0.0,
            dy: 1.0,
            ..rec(A_STARTUP, 0.75, 0.4)
        };
        assert!((posed(&down).sheet.turn + most).abs() < 1e-5);
        // It leans back as it charges.
        let lean = |k| posed(&rec(A_STARTUP, k, 0.4)).sheet.lean;
        assert!(lean(0.0) < lean(0.4) && lean(0.4) < lean(0.79));
        assert!(lean(0.79) > 0.0);
        // And snaps forward on the strike.
        assert!(posed(&rec(A_ACTIVE, 0.5, 0.1)).sheet.lean < 0.0);
    }

    #[test]
    fn the_recovery_pose_changes_at_the_cancel_point() {
        let at = |k| {
            posed(&Anim {
                cancel: 0.5,
                ..rec(A_RECOVERY, k, 0.4)
            })
        };
        let (before, after) = (at(0.49), at(0.51));
        let sag = RIGS[RG_PLAYER].sheet.sag;
        assert!((before.sheet.lean - sag).abs() < 1e-3);
        assert!(before.sheet.bend > 0.0);
        assert_eq!((after.sheet.lean, after.sheet.bend), (0.0, 0.0));
        assert_eq!(after.flat, Pose::REST);
        // It pops: a small hop just after the cancel point.
        assert!(after.sheet.lift > 0.0);
        assert!(at(1.0).sheet.lift.abs() < 1e-3);
    }

    #[test]
    fn the_hit_pose_leans_away_from_the_hit_and_more_for_more_damage() {
        let hit = |hdx, hdy, dmg| {
            posed(&Anim {
                hit_dx: hdx,
                hit_dy: hdy,
                hit_dmg: dmg,
                hit_age: 0.05,
                ..rec(A_HURT, 0.2, 0.3)
            })
        };
        let rest = posed(&rec(A_HURT, 0.2, 0.3));
        // Pushed up the screen (hit from the camera's side) it leans back; pushed down the screen, forward.
        assert!(hit(0.0, -1.0, 20.0).sheet.lean > 0.0);
        assert!(hit(0.0, 1.0, 20.0).sheet.lean < 0.0);
        // Pushed right, its top goes right, facing either way.
        assert!(hit(1.0, 0.0, 20.0).flat.tilt > rest.flat.tilt);
        let left = posed(&Anim {
            face: -1.0,
            hit_dx: 1.0,
            hit_dmg: 20.0,
            hit_age: 0.05,
            ..rec(A_HURT, 0.2, 0.3)
        });
        // Facing left, the drawn tilt is the pose's tilt mirrored.
        assert!(-left.flat.tilt > -rest.flat.tilt);
        // More damage leans and bends it more.
        let (small, big) = (hit(0.0, -1.0, 10.0), hit(0.0, -1.0, 30.0));
        assert!(big.sheet.lean > small.sheet.lean);
        assert!(big.sheet.bend > small.sheet.bend);
        // And it settles back.
        let late = posed(&Anim {
            hit_dy: -1.0,
            hit_dmg: 30.0,
            hit_age: 0.5,
            ..rec(A_HURT, 0.2, 0.3)
        });
        assert_eq!(late.sheet.lean, 0.0);
    }

    #[test]
    fn the_down_pose_lies_flat() {
        let down = posed(&rec(A_DOWN, 0.5, 0.9));
        assert!((down.sheet.lean - FRAC_PI_2).abs() < 0.01);
        // Drawn lying on its back, it shows at `PITCH` of its height.
        let upright = drawn(Sheet::FLAT);
        let flat = drawn_pose(
            Pose3 {
                flat: Pose::REST,
                ..down
            },
            Sheet::FLAT,
        );
        assert!((head(&flat).y - head(&upright).y * PITCH).abs() < 0.1);
        // It gets up at the end.
        assert_eq!(posed(&rec(A_DOWN, 1.0, 0.9)).sheet.lean, 0.0);
        // Pinned flat when grabbed.
        assert!(posed(&rec(A_GRABBED, 0.5, 3.0)).sheet.lean > 1.3);
    }

    #[test]
    fn a_roll_turns_once() {
        let len = ROLL_TIME + ROLL_RECOVER;
        let turn = |k: f32| posed(&rec(A_ROLL, k, len)).sheet.turn;
        assert_eq!(turn(0.0), 0.0);
        let mut last = 0.0;
        for i in 1..=20 {
            let t = turn(i as f32 / 20.0);
            assert!(t >= last);
            last = t;
        }
        assert!((last - TAU).abs() < 1e-5);
        // Halfway round its back is to the camera, so its parts are drawn in the other order.
        let half = drawn_pose(posed(&rec(A_ROLL, 0.5 * ROLL_TIME / len, len)), Sheet::FLAT);
        let mut reversed = ids(&drawn(Sheet::FLAT));
        reversed.reverse();
        assert_eq!(ids(&half), reversed);
    }

    #[test]
    fn turning_round_passes_through_edge_on() {
        let mut game = Game::new(1);
        let mut a = rec(A_IDLE, 0.0, 0.0);
        game.anims = vec![a];
        game.turn_round();
        assert_eq!(game.facing(&a), (1.0, 0.0));
        a.face = -1.0;
        game.anims = vec![a];
        let mut seen = vec![];
        for _ in 0..10 {
            game.anim += 0.02;
            game.turn_round();
            seen.push(game.facing(&a));
        }
        // Old facing turning towards edge-on, then the new facing turning back from it, then done.
        assert!(seen[0].0 == 1.0 && seen[0].1 > 0.0);
        assert!(seen.iter().any(|&(f, t)| f == -1.0 && t < 0.0));
        assert!(seen.iter().all(|&(_, t)| t.abs() <= FRAC_PI_2));
        assert_eq!(*seen.last().unwrap(), (-1.0, 0.0));
    }

    #[test]
    fn drawing_changes_only_the_drawing() {
        let run = |spin: bool, flat: bool| {
            let mut game = Game::new(11);
            game.set_view(640.0, 360.0);
            game.set_debug_spin(spin);
            if flat {
                game.sheet_pose = [SheetPose::default(); N_RIGS];
            }
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
                turned |= game.inst.iter().any(|i| i.lean != 0.0 || i.lift != 0.0);
            }
            (game.state_hash(), turned)
        };
        let (posed, posed_moved) = run(false, false);
        let (spun, spun_moved) = run(true, false);
        let (flat, _) = run(false, true);
        assert_eq!(posed, spun);
        assert_eq!(posed, flat);
        assert!(posed_moved && spun_moved);
    }
}
