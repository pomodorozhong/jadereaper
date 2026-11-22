//! Puppet rigs: a character is a few rigid paper parts placed around its ground point, and it is animated by
//! posing the whole puppet (shift, tilt, squash) rather than by sprite frames. Each rig has its own wind-up, attack,
//! hurt and roll poses; walking flips between two tilts. The rigs are in `content/game.json`.
//!
//! Each part is also a sheet of paper in 3D: a whole puppet can turn, lean, lift and bend (`Sheet`), and its parts
//! are then drawn back to front by their depth after the turn, not by a depth buffer.

use crate::anim::*;
use crate::defs::{orient, Sheet, ROLL_TIME, SHEET_PAD};
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
    /// Grabbed: how far it is pinned back, and how hard it shakes.
    pub grab: f32,
    pub shake: f32,
    /// Roll: whole spins in its own plane, about its middle.
    pub roll: f32,
    /// Walking: how high it bobs each step, and how far it sways (turns) each way.
    pub bob: f32,
    pub sway: f32,
    /// How much of every lean bends the paper instead of tipping it, 0..1: at 1 the puppet bows from its planted
    /// feet with its top reaching the same height as the rigid lean would (lying down and being pinned stay rigid).
    pub flex: f32,
    /// Walking: how far it bows back and forth with each step, in radians at its top.
    pub walk_curl: f32,
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
            "grab" => &mut self.grab,
            "shake" => &mut self.shake,
            "roll" => &mut self.roll,
            "bob" => &mut self.bob,
            "sway" => &mut self.sway,
            "flex" => &mut self.flex,
            "walk_curl" => &mut self.walk_curl,
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
    /// A spin in the puppet's own plane about its middle, in radians (the roll), on top of the flat pose's tilt.
    pub spin: f32,
    /// How far the whole puppet bows as one sheet of paper: the angle its top reaches, in radians, with the
    /// curve growing evenly from the feet (positive bows back, away from the camera). On top of `sheet.lean`.
    pub curl: f32,
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

/// How long the pop upright at the cancel point takes, in seconds.
const POP_TIME: f32 = 0.12;
/// Where an enemy's recovery, which nothing cancels, starts to straighten up, as a share of it.
const ENEMY_RISE: f32 = 0.7;

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
    let mut spin = 0.0;
    let mut curl = 0.0;
    let off_step = (g.steps.floor() as i32) & 1 == 1;
    let walk = Pose {
        tilt: if off_step { -g.lean } else { g.lean },
        dy: if off_step { g.hop } else { 0.0 },
        ..Pose::REST
    };
    let flat = match a.phase {
        A_WALK => {
            let s = (g.steps * PI).sin();
            sheet.lift = n.bob * s.abs();
            sheet.turn = n.sway * s;
            curl = n.walk_curl * (g.steps * PI * 0.5).sin();
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
            // Tucks and spins in its own plane, like a cartwheel seen from the front.
            let m = (t / ROLL_TIME).min(1.0);
            spin = n.roll * TAU * m;
            Pose::REST.mix(&rig.roll, (m * PI).sin())
        }
        // Hit, knocked down or stunned: the hit's spin (`a.twist`, below) shows it.
        // Only the spin: no lean, shift or squash.
        A_DOWN | A_STUNNED | A_HURT => Pose::REST,
        A_GRABBED => {
            sheet.lean = n.grab * smooth(t / 0.1);
            sheet.turn = n.shake * jitter(g.clock, 30.0);
            sheet.bend = n.shake * jitter(g.clock + 0.5, 30.0);
            rig.hurt
        }
        _ => walk,
    };
    // A hit spins the sheet round about its vertical axis (`crate::twist`): a swing back and forth that dies away,
    // or whole turns for a heavy hit. The twist is in the world, so it is mirrored back for a puppet facing left.
    sheet.turn += a.twist * a.face;
    // Bend the paper: turn part of the lean into a bow from the feet. A bow whose top reaches angle C ends about
    // as high as a rigid lean of C / 2, so the lean's share is doubled. Lying down and being pinned stay rigid.
    if a.phase != A_DOWN && a.phase != A_GRABBED {
        let f = n.flex.clamp(0.0, 1.0);
        curl += 2.0 * f * sheet.lean;
        sheet.lean *= 1.0 - f;
    }
    Pose3 {
        flat,
        sheet,
        spin,
        curl,
    }
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
    /// Extra lean and bend from the puppet's bow at this part.
    lean: f32,
    bend: f32,
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
        let tilt = (pose.tilt + p3.spin) * f;
        // A spinning puppet turns about its middle, not its feet.
        let pivot = if p3.spin != 0.0 {
            rig.height() * 0.5 * s
        } else {
            0.0
        };
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
                        lean: 0.0,
                        bend: 0.0,
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
            let (ox, oy) = (p.x * f * s * pose.sx, p.y * s * pose.sy + pivot);
            let (rx, ry) = (ox * c - oy * sn, ox * sn + oy * c);
            placed[n] = Placed {
                depth: k as f32,
                lean: 0.0,
                bend: 0.0,
                z: 0.0,
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
        // Bow the puppet as one sheet: each part moves onto an arc rising from the feet, leans as far as the arc
        // has turned at its height, and bends with the arc's curvature. Then turn the puppet about its ground point.
        // A part's layer only counts towards its depth, so a puppet at rest is drawn exactly where and in the order
        // it always was.
        let height = (rig.height() * s).max(1.0);
        let curl = if self.debug_spin { 0.0 } else { p3.curl };
        let kappa = curl / height;
        for q in placed[..n].iter_mut() {
            let h = (-q.y).max(0.0);
            let (y0, z0) = if kappa.abs() > 1e-6 {
                (
                    -(kappa * h).sin() / kappa,
                    -(1.0 - (kappa * h).cos()) / kappa,
                )
            } else {
                (q.y, 0.0)
            };
            let (y0, z0) = if q.y > 0.0 { (q.y, 0.0) } else { (y0, z0) };
            q.lean = kappa * h;
            // The shader bends a sheet by the angle from its middle to its quad's edge (half height plus its pad);
            // bowing back curves both edges away from the camera.
            q.bend = -kappa * (q.hh.abs() + SHEET_PAD);
            let (x, y, z) = orient([q.x, y0, z0], sheet.turn, sheet.lean);
            q.z = z;
            q.depth = orient([q.x, y0, z0 + q.depth * LAYER], sheet.turn, sheet.lean).2;
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
                Sheet {
                    lean: sheet.lean + q.lean,
                    bend: sheet.bend + q.bend,
                    ..sheet
                },
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
        spin: 0.0,
        curl: 0.0,
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
            twist: 0.0,
        }
    }

    /// How far a pose tips the puppet's top back: its rigid lean, plus its bow, which reaches about as high as a
    /// rigid lean of half its angle.
    fn tip(p: Pose3) -> f32 {
        p.sheet.lean + 0.5 * p.curl
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
        let lean = |k| tip(posed(&rec(A_STARTUP, k, 0.4)));
        assert!(lean(0.0) < lean(0.4) && lean(0.4) < lean(0.79));
        assert!(lean(0.79) > 0.0);
        // And snaps forward on the strike.
        assert!(tip(posed(&rec(A_ACTIVE, 0.5, 0.1))) < 0.0);
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
        assert!((tip(before) - sag).abs() < 1e-3);
        assert!(before.sheet.bend > 0.0);
        assert_eq!(
            (after.sheet.lean, after.sheet.bend, after.curl),
            (0.0, 0.0, 0.0)
        );
        assert_eq!(after.flat, Pose::REST);
        // It pops: a small hop just after the cancel point.
        assert!(after.sheet.lift > 0.0);
        assert!(at(1.0).sheet.lift.abs() < 1e-3);
    }

    #[test]
    fn a_hit_turns_the_sheet_by_its_twist_and_nothing_else() {
        let twisted = |twist, face| {
            posed(&Anim {
                twist,
                face,
                hit_dmg: 30.0,
                hit_age: 0.05,
                ..rec(A_HURT, 0.0, 0.0)
            })
        };
        let rest = twisted(0.0, 1.0);
        // The twist turns the sheet about its vertical axis; it does not lean, bend or tilt it.
        let t = twisted(1.2, 1.0);
        assert!((t.sheet.turn - rest.sheet.turn - 1.2).abs() < 1e-5);
        assert_eq!(
            (t.sheet.lean, t.sheet.bend, t.flat),
            (rest.sheet.lean, rest.sheet.bend, rest.flat)
        );
        // The twist is in the world: facing left, the pose's turn is mirrored so it draws the same way round.
        assert!((twisted(1.2, -1.0).sheet.turn + 1.2).abs() < 1e-5);
        // Knocked down or stunned, the twist shows the same way.
        for phase in [A_DOWN, A_STUNNED] {
            let p = posed(&Anim {
                twist: 2.0,
                ..rec(phase, 0.0, 0.0)
            });
            assert!((p.sheet.turn - 2.0).abs() < 1e-5 && p.sheet.lean == 0.0);
        }
    }

    #[test]
    fn flex_bows_the_paper_instead_of_tipping_it() {
        // Charging, part of the lean becomes a bow; lying down stays a rigid lean.
        let charge = posed(&rec(A_STARTUP, 0.8, 0.4));
        let f = RIGS[RG_PLAYER].sheet.flex;
        assert!(charge.curl > 0.0 && f > 0.0);
        assert!((tip(charge) - RIGS[RG_PLAYER].sheet.charge).abs() < 1e-3);
        assert_eq!(posed(&rec(A_DOWN, 0.5, 0.9)).curl, 0.0);
        // Drawn, the bow leans the head further back than the body and curves both away from the camera.
        let bowed = drawn_pose(Pose3 { curl: 0.8, ..REST }, Sheet::FLAT);
        let body = *bowed
            .iter()
            .find(|i| i.part % PART_SLOTS == P_PLAYER_BODY)
            .unwrap();
        assert!(head(&bowed).lean > body.lean && body.lean > 0.0);
        assert!(head(&bowed).bend < 0.0 && body.bend < 0.0);
        // The head goes back, so it is further from the camera and lower on the screen than standing.
        assert!(head(&bowed).depth < 0.0);
        // Bowing forward does the opposite.
        let forward = drawn_pose(Pose3 { curl: -0.8, ..REST }, Sheet::FLAT);
        assert!(
            head(&forward).lean < 0.0 && head(&forward).bend > 0.0 && head(&forward).depth > 0.0
        );
    }

    #[test]
    fn walking_bows_back_and_forth() {
        let at = |steps| {
            let rig = &RIGS[RG_BLOT];
            pose(
                rig,
                &rig.sheet,
                &rec(A_WALK, 0.0, 0.0),
                Gait {
                    steps,
                    ..Gait::default()
                },
            )
            .curl
        };
        assert!(at(1.0) > 0.0 && at(3.0) < 0.0);
    }

    #[test]
    fn grabbed_is_pinned_flat() {
        assert!(posed(&rec(A_GRABBED, 0.5, 3.0)).sheet.lean > 1.3);
    }

    #[test]
    fn a_roll_spins_once_in_its_own_plane() {
        let len = ROLL_TIME + ROLL_RECOVER;
        let at = |k: f32| posed(&rec(A_ROLL, k, len));
        assert_eq!(at(0.0).spin, 0.0);
        let mut last = 0.0;
        for i in 1..=20 {
            let p = at(i as f32 / 20.0);
            assert!(p.spin >= last);
            // A cartwheel facing the camera: the sheet never turns about its vertical axis.
            assert_eq!(p.sheet.turn, 0.0);
            last = p.spin;
        }
        assert!((last - TAU).abs() < 1e-5);
        // Halfway, it is upside down about its middle: the head is below the body.
        let half = drawn_pose(at(0.5 * ROLL_TIME / len), Sheet::FLAT);
        let body = *half
            .iter()
            .find(|i| i.part % PART_SLOTS == P_PLAYER_BODY)
            .unwrap();
        assert!(head(&half).y > body.y);
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
