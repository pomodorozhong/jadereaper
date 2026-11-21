//! Building the instance list the renderer draws: ground, characters, projectiles and effects.

use crate::anim::Anim;
use crate::defs::*;
use crate::elements::{element_color, STAR_RADIUS, TRAIL_TIME, WELL_RANGE, WELL_TIME};
use crate::game::{Act, Game, Phase};
use crate::geom::{Area, Shape};
use crate::moves::{After, Special, Stance, MOVES};
use crate::player::HURT_IFRAME;
use crate::rigs::{pose, Gait, Look};
use crate::skills::{AreaKind, SkillKind, SKILLS};
use core::f32::consts::TAU;

/// How long a puppet takes to turn round through edge-on, in seconds.
const FLIP_TIME: f32 = 0.12;
const WARN: [f32; 4] = [0.17, 0.15, 0.13, 0.55];
const WARN_BOSS: [f32; 4] = [0.62, 0.16, 0.12, 0.6];

impl Game {
    // ---------------------------------------------------------------- draw

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn put(
        &mut self,
        x: f32,
        y: f32,
        hw: f32,
        hh: f32,
        rot: f32,
        part: f32,
        flash: f32,
        seed: f32,
        color: [f32; 4],
    ) {
        self.put_sheet(x, y, hw, hh, rot, part, flash, seed, color, Sheet::FLAT);
    }

    /// `put` for a part held in 3D (see `Sheet`).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn put_sheet(
        &mut self,
        x: f32,
        y: f32,
        hw: f32,
        hh: f32,
        rot: f32,
        part: f32,
        flash: f32,
        seed: f32,
        color: [f32; 4],
        sheet: Sheet,
    ) {
        if self.inst.len() >= MAX_INST {
            return;
        }
        // Cull what the camera cannot see.
        let m = hw.abs().max(hh) + 24.0 + sheet.lift.abs();
        if (x - self.cam.0).abs() > self.view_w + m || (y - self.cam.1).abs() > self.view_h + m {
            return;
        }
        let part = part + PART_SLOTS * self.boil_pose(seed) as f32;
        self.inst.push(Inst {
            x,
            y,
            hw,
            hh,
            rot,
            part,
            flash,
            seed,
            color,
            turn: sheet.turn,
            lean: sheet.lean,
            lift: sheet.lift,
            bend: sheet.bend,
        });
    }

    /// Spin every puppet in 3D (the ?debug toggle), to show the sheets turn, lean, lift and bend.
    pub fn set_debug_spin(&mut self, on: bool) {
        self.debug_spin = on;
    }

    /// Set the line boil: poses per second, and poses per part (1 to `MAX_POSES`).
    pub fn set_boil(&mut self, fps: f32, poses: u32) {
        self.boil_fps = fps.max(0.0);
        self.boil_poses = poses.clamp(1, MAX_POSES);
    }

    /// Set how big a character is drawn: 0 is the player, 1 + kind an enemy kind. Hit areas do not change.
    pub fn set_draw_scale(&mut self, who: usize, scale: f32) {
        if let Some(s) = self.draw_scale.get_mut(who) {
            *s = scale.clamp(0.1, 4.0);
        }
    }

    /// Set an enemy kind's waddle: how far it tilts each way (radians) and how often it changes pose.
    pub fn set_waddle(&mut self, kind: usize, tilt: f32, rate: f32) {
        if let Some(w) = self.waddle.get_mut(kind) {
            *w = (tilt.clamp(0.0, 1.5), rate.clamp(0.0, 30.0));
        }
    }

    /// Set one of a rig's 3D pose numbers (the ?debug sliders): `field` indexes `SHEET_KEYS`. Drawing only.
    pub fn set_pose(&mut self, rig: usize, field: usize, value: f32) {
        if let (Some(r), Some(name)) = (self.sheet_pose.get_mut(rig), SHEET_KEYS.get(field)) {
            if let Some(v) = r.field_mut(name) {
                *v = if value.is_finite() { value } else { 0.0 };
            }
        }
    }

    /// Move each character's drawn facing towards its real one. Turning round is a quick turn through edge-on
    /// (`FLIP_TIME`), drawn from the old facing to edge-on and then from edge-on to the new one. It is drawing only
    /// and never delays an action; the facing the simulation uses changes at once.
    pub(crate) fn turn_round(&mut self) {
        let dt = (self.anim - self.flip_clock).clamp(0.0, 0.25);
        self.flip_clock = self.anim;
        let rate = 2.0 * dt / FLIP_TIME;
        let old = core::mem::take(&mut self.flips);
        let mut keep = Vec::with_capacity(self.anims.len());
        for a in &self.anims {
            let key = (a.who, a.id);
            let w = match old.iter().find(|f| f.0 == key) {
                Some(f) => {
                    let d = a.face - f.1;
                    f.1 + d.clamp(-rate, rate)
                }
                None => a.face,
            };
            keep.push((key, w));
        }
        self.flips = keep;
    }

    /// How a character's drawn facing stands mid-turn: the facing to draw and the extra turn about the vertical
    /// axis (edge-on at a quarter turn).
    pub(crate) fn facing(&self, a: &Anim) -> (f32, f32) {
        let w = self
            .flips
            .iter()
            .find(|f| f.0 == (a.who, a.id))
            .map_or(a.face, |f| f.1);
        let face = if w < 0.0 { -1.0 } else { 1.0 };
        (face, face * (1.0 - w.abs()) * core::f32::consts::FRAC_PI_2)
    }

    /// The line boil pose of one instance. Each instance starts at its own pose and steps at its own moment,
    /// both taken from its seed, so a crowd does not boil in lockstep.
    fn boil_pose(&self, seed: f32) -> u32 {
        if self.boil_poses <= 1 {
            return 0;
        }
        let phase = (seed * 7.31).fract();
        let start = (seed * 1000.0) as i32;
        let step = (self.anim * self.boil_fps + phase).floor() as i32;
        (start + step).rem_euclid(self.boil_poses as i32) as u32
    }

    pub(crate) fn build(&mut self) {
        self.inst.clear();
        self.turn_round();
        self.draw_ground();
        self.draw_walls();

        // Ground-level effects and pickups.
        for i in 0..self.fx.len() {
            if !self.fx[i].top {
                self.draw_fx(i);
            }
        }
        // Warnings for attacks that are winding up.
        for i in 0..self.enemies.len() {
            let e = self.enemies[i];
            if e.phase != Phase::Windup || e.stun > 0.0 {
                continue;
            }
            let m = MOVES[e.mv as usize];
            if m.reach <= 0.0 {
                continue;
            }
            let prog = (e.pt / self.windup_of(&e)).clamp(0.0, 1.0);
            let mut color = if e.kind == K_ELITE || KINDS[e.kind as usize].boss {
                WARN_BOSS
            } else {
                WARN
            };
            if e.kind == K_DRAGON {
                // The Dragon's warnings take its stance's colour.
                let c = self.stance_color();
                color = [c[0] * 0.8, c[1] * 0.8, c[2] * 0.8, 0.6];
            }
            if m.special == Special::Finisher {
                // The finisher fills the screen: a wash that darkens as it builds.
                let r = self.view_w.max(self.view_h) * 2.5;
                let (cx, cy) = self.cam;
                self.put(
                    cx,
                    cy,
                    r,
                    r,
                    0.0,
                    P_WARN_CIRCLE,
                    prog,
                    0.0,
                    [color[0], color[1], color[2], 0.25 + 0.35 * prog],
                );
                continue;
            }
            let (areas, n) = self.move_areas(&e);
            for a in &areas[..n] {
                self.draw_warning(*a, prog, color);
            }
            let area = areas[0];
            if let After::Wave { to, .. } = m.after {
                // The shock ring that will follow: its whole reach, faint.
                let ring = Area {
                    shape: Shape::Ring {
                        inner: m.reach,
                        outer: to,
                    },
                    ..area
                };
                self.draw_warning(ring, 0.0, [color[0], color[1], color[2], color[3] * 0.5]);
            }
        }
        self.draw_player_skill();
        for i in 0..self.patches.len() {
            let p = self.patches[i];
            let c = element_color(p.element);
            let a = (p.t / TRAIL_TIME).min(1.0) * 0.6;
            self.put(
                p.x,
                p.y,
                24.0,
                14.0,
                0.0,
                P_SPLAT,
                0.0,
                (i as f32 * 0.37).fract(),
                [c[0], c[1], c[2], a],
            );
        }
        for i in 0..self.wells.len() {
            let w = self.wells[i];
            let c = element_color(E_CELESTIAL + 1);
            let r = WELL_RANGE * (0.3 + 0.7 * w.t / WELL_TIME);
            self.put(
                w.x,
                w.y,
                r,
                r,
                0.0,
                P_WARN_RING,
                1.0,
                0.8,
                [c[0], c[1], c[2], 0.6],
            );
            self.put(
                w.x,
                w.y,
                STAR_RADIUS,
                STAR_RADIUS,
                0.0,
                P_WARN_CIRCLE,
                1.0 - w.t / WELL_TIME,
                0.0,
                [c[0], c[1], c[2], 0.5],
            );
        }
        for i in 0..self.hazards.len() {
            let h = self.hazards[i];
            if h.rehit > 0.0 {
                // A burning pool.
                let c = element_color(E_FIRE + 1);
                let flick = 0.5 + 0.1 * (self.anim * 12.0 + h.x).sin();
                self.put(
                    h.x,
                    h.y,
                    h.r,
                    h.r * 0.8,
                    0.0,
                    P_SPLAT,
                    0.0,
                    (h.x * 0.01).fract(),
                    [c[0], c[1], c[2], flick * (h.life / 4.0).min(1.0) + 0.2],
                );
                continue;
            }
            if h.band > 0.0 {
                let outer = h.r + h.band * 0.5;
                let inner = (h.r - h.band * 0.5).max(0.0);
                self.put(
                    h.x,
                    h.y,
                    outer,
                    outer,
                    0.0,
                    P_WARN_RING,
                    1.0,
                    inner / outer,
                    [INK[0], INK[1], INK[2], 0.75],
                );
            }
        }

        // Characters, sorted by depth.
        self.order.clear();
        for (i, e) in self.enemies.iter().enumerate() {
            self.order.push((e.y, i as u16));
        }
        self.order.push((self.player.y, u16::MAX));
        self.order
            .sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));
        for n in 0..self.order.len() {
            let idx = self.order[n].1;
            if idx == u16::MAX {
                self.draw_player();
            } else {
                self.draw_enemy(idx as usize);
            }
        }

        // Things in the air.
        for i in 0..self.hazards.len() {
            let h = self.hazards[i];
            if h.band == 0.0 {
                let wob = (self.anim * 9.0 + h.x * 0.05).sin() * 0.15;
                self.put(
                    h.x,
                    h.y - 22.0,
                    h.r * 1.3,
                    h.r * 1.1,
                    wob,
                    P_BLOT,
                    0.0,
                    0.77,
                    [0.20, 0.18, 0.22, 1.0],
                );
            }
        }
        for i in 0..self.shots.len() {
            let s = &self.shots[i];
            let (x, y, rot) = (s.x, s.y, s.vy.atan2(s.vx) + TAU * 0.25);
            self.put(
                x,
                y - 24.0,
                6.0,
                11.0,
                rot,
                P_TALISMAN,
                0.0,
                0.5,
                [0.89, 0.77, 0.38, 1.0],
            );
        }
        for k in 0..crate::skills::ORBIT_BEADS {
            if let Some((x, y)) = self.bead_pos(k) {
                self.put(
                    x,
                    y - 24.0,
                    9.0,
                    9.0,
                    0.0,
                    P_BEAD,
                    0.0,
                    k as f32 * 0.13,
                    [0.30, 0.66, 0.50, 1.0],
                );
            }
        }
        for i in 0..self.fx.len() {
            if self.fx[i].top {
                self.draw_fx(i);
            }
        }
    }

    /// The player's own skill, read like an enemy warning but in the skill's colour: through the startup a wash
    /// in the shape of the coming hit fills in, pointing where the skill will go; through the recovery a ring
    /// at the feet fills in ink until the cancel point, then turns to the skill's colour while a roll can cut it.
    fn draw_player_skill(&mut self) {
        let p = &self.player;
        let Act::Skill {
            slot,
            t,
            dx,
            dy,
            step,
            dist,
            ox,
            oy,
            ..
        } = p.act
        else {
            return;
        };
        let (px, py) = (p.x, p.y);
        let d = self.def(slot);
        let c = element_color(self.mods[slot].element);
        let (a0, a1) = (d.startup, d.startup + d.active);
        if t < a1 {
            let prog = (t / a0.max(1e-3)).min(1.0);
            let dir = (dx, dy);
            let area = match d.kind {
                SkillKind::Leap => Area::new(
                    ox + dx * dist,
                    oy + dy * dist,
                    dir,
                    Shape::Circle { r: d.reach },
                ),
                SkillKind::Shot => Area::new(
                    px,
                    py,
                    dir,
                    Shape::Line {
                        len: d.reach,
                        half_w: d.width,
                    },
                ),
                SkillKind::Orbit => Area::new(
                    px,
                    py,
                    dir,
                    Shape::Ring {
                        inner: d.reach - d.width,
                        outer: d.reach + d.width,
                    },
                ),
                SkillKind::Melee if d.area != AreaKind::Arc && d.dash > 0.0 => {
                    // A lunge or charge: the path still ahead of it.
                    let gone = ((t - a0) / d.active.max(1e-3)).clamp(0.0, 1.0);
                    let half_w = if d.area == AreaKind::Line {
                        d.width
                    } else {
                        d.reach
                    };
                    Area::new(
                        px,
                        py,
                        dir,
                        Shape::Line {
                            len: dist * (1.0 - gone) + d.reach,
                            half_w,
                        },
                    )
                }
                _ => self.skill_area(slot, step, dir),
            };
            self.draw_warning(area, prog, [c[0], c[1], c[2], 0.5]);
        } else {
            let into = t - a1;
            let (r, inner) = (36.0, 0.74);
            if into < d.cancel {
                self.draw_warning(
                    Area::new(
                        px,
                        py,
                        (1.0, 0.0),
                        Shape::Ring {
                            inner: r * inner,
                            outer: r,
                        },
                    ),
                    into / d.cancel.max(1e-3),
                    [INK[0], INK[1], INK[2], 0.55],
                );
            } else {
                let fade = 1.0 - (into - d.cancel) / (d.recovery - d.cancel).max(1e-3);
                self.draw_warning(
                    Area::new(
                        px,
                        py,
                        (1.0, 0.0),
                        Shape::Ring {
                            inner: r * inner,
                            outer: r,
                        },
                    ),
                    1.0,
                    [c[0], c[1], c[2], 0.35 + 0.5 * fade],
                );
            }
        }
    }

    /// An attack's warning: an ink wash in the shape of the coming hit, filling in as the wind-up runs.
    fn draw_warning(&mut self, a: Area, prog: f32, color: [f32; 4]) {
        let rot = a.dy.atan2(a.dx);
        match a.shape {
            Shape::Circle { r } => self.put(a.x, a.y, r, r, 0.0, P_WARN_CIRCLE, prog, 0.0, color),
            Shape::Arc { r, half } => self.put(
                a.x,
                a.y,
                r,
                r,
                rot,
                P_WARN_ARC,
                prog,
                half / core::f32::consts::PI,
                color,
            ),
            Shape::Line { len, half_w } => {
                let (cx, cy) = (a.x + a.dx * len * 0.5, a.y + a.dy * len * 0.5);
                self.put(
                    cx,
                    cy,
                    len * 0.5,
                    half_w,
                    rot,
                    P_WARN_LINE,
                    prog,
                    0.0,
                    color,
                )
            }
            Shape::Ring { inner, outer } => self.put(
                a.x,
                a.y,
                outer,
                outer,
                0.0,
                P_WARN_RING,
                prog,
                inner / outer,
                color,
            ),
        }
    }

    fn draw_fx(&mut self, i: usize) {
        let f = &self.fx[i];
        let t = (f.life / f.max).clamp(0.0, 1.0);
        let mut c = f.color;
        c[3] *= t.sqrt();
        let (x, y, hw, hh, rot, part, seed) = (f.x, f.y, f.hw, f.hh, f.rot, f.part, f.seed);
        self.put(x, y, hw, hh, rot, part, 0.0, seed, c);
    }

    /// Sparse ink strokes and stones so movement reads against the paper.
    fn draw_ground(&mut self) {
        const G: f32 = 170.0;
        let x0 = ((self.cam.0 - self.view_w) / G).floor() as i32;
        let x1 = ((self.cam.0 + self.view_w) / G).floor() as i32;
        let y0 = ((self.cam.1 - self.view_h) / G).floor() as i32;
        let y1 = ((self.cam.1 + self.view_h) / G).floor() as i32;
        for cy in y0..=y1 {
            for cx in x0..=x1 {
                let h = hash2(cx, cy);
                if h % 100 >= 55 {
                    continue;
                }
                let fx = ((h >> 8) & 255) as f32 / 255.0;
                let fy = ((h >> 16) & 255) as f32 / 255.0;
                let x = (cx as f32 + fx) * G;
                let y = (cy as f32 + fy) * G;
                let seed = (h & 255) as f32 / 255.0;
                if (h >> 24).is_multiple_of(4) {
                    self.put(
                        x,
                        y,
                        16.0 + seed * 12.0,
                        8.0 + seed * 5.0,
                        0.0,
                        P_STONE,
                        0.0,
                        seed,
                        [INK[0], INK[1], INK[2], 0.16],
                    );
                } else {
                    let flip = if h & 1 == 0 { 1.0 } else { -1.0 };
                    self.put(
                        x,
                        y,
                        (10.0 + seed * 6.0) * flip,
                        10.0 + seed * 8.0,
                        0.0,
                        P_GRASS,
                        0.0,
                        seed,
                        [0.22, 0.36, 0.30, 0.45],
                    );
                }
            }
        }
    }

    /// The colour of the Dragon's stance.
    fn stance_color(&self) -> [f32; 3] {
        match self.stance {
            Stance::Ice => element_color(E_ICE + 1),
            _ => element_color(E_FIRE + 1),
        }
    }

    /// The arena's edge: an ink band with stones along it.
    fn draw_walls(&mut self) {
        let Some([x0, y0, x1, y1]) = self.arena else {
            return;
        };
        let ink = [INK[0], INK[1], INK[2], 0.55];
        let (cx, cy, hw, hh) = (
            (x0 + x1) * 0.5,
            (y0 + y1) * 0.5,
            (x1 - x0) * 0.5,
            (y1 - y0) * 0.5,
        );
        for (x, y, w, h) in [
            (cx, y0, hw, 5.0),
            (cx, y1, hw, 5.0),
            (x0, cy, 5.0, hh),
            (x1, cy, 5.0, hh),
        ] {
            self.put(x, y, w + 5.0, h, 0.0, P_WARN_LINE, 1.0, 0.0, ink);
        }
        let step = 46.0;
        let (nx, ny) = (((x1 - x0) / step) as i32, ((y1 - y0) / step) as i32);
        for k in 0..=nx {
            let x = x0 + k as f32 * step;
            for (y, o) in [(y0 - 14.0, 0), (y1 + 14.0, 1)] {
                let seed = (hash2(k, o) & 255) as f32 / 255.0;
                self.put(
                    x,
                    y,
                    16.0 + seed * 8.0,
                    9.0 + seed * 4.0,
                    0.0,
                    P_STONE,
                    0.0,
                    seed,
                    [INK[0], INK[1], INK[2], 0.3],
                );
            }
        }
        for k in 0..=ny {
            let y = y0 + k as f32 * step;
            for (x, o) in [(x0 - 14.0, 2), (x1 + 14.0, 3)] {
                let seed = (hash2(k, o) & 255) as f32 / 255.0;
                self.put(
                    x,
                    y,
                    12.0 + seed * 6.0,
                    12.0 + seed * 6.0,
                    0.0,
                    P_STONE,
                    0.0,
                    seed,
                    [INK[0], INK[1], INK[2], 0.3],
                );
            }
        }
    }

    fn draw_enemy(&mut self, i: usize) {
        let e = self.enemies[i];
        let k = &KINDS[e.kind as usize];
        let rig = &RIGS[k.rig];
        // Sizes and waddles from the debug panel cover the four small kinds; bosses use their rig as it is.
        let s = if e.kind < 4 {
            self.draw_scale[1 + e.kind as usize]
        } else {
            1.0
        };
        // Two walk poses, flipped back and forth. That is the whole walk cycle.
        let (lean, rate) = if e.kind < 4 {
            self.waddle[e.kind as usize]
        } else {
            (
                KINDS[e.kind as usize].tilt.to_radians(),
                KINDS[e.kind as usize].rate,
            )
        };
        let mv = &MOVES[e.mv as usize];
        let mut x = e.x;
        let boss = self.boss_of(i);
        if e.phase == Phase::Windup && e.stun <= 0.0 && e.status.frozen <= 0.0 {
            // Tremble while winding up.
            x += ((self.anim * 40.0).floor() * 1.7).sin() * 1.5;
        }
        let mut a = self.anims[1 + i];
        let (face, flip) = self.facing(&a);
        a.face = face;
        let p3 = pose(
            rig,
            &self.sheet_pose[k.rig],
            &a,
            Gait {
                clock: self.anim + e.seed * 7.0,
                steps: self.anim * rate + e.seed * 7.0,
                lean,
                hop: -3.0,
            },
        );
        let mut y = e.y;
        if e.kind == K_DRAGON && e.phase == Phase::Windup && mv.special == Special::Dive {
            // Up into the air before the dive.
            y -= 160.0 * (e.pt / self.windup_of(&e)).min(1.0);
        }
        if e.kind == K_WISP {
            // Wisps hover.
            y += (self.anim * 5.0 + e.seed * 20.0).sin() * 3.0;
        }
        let shadow = [INK[0], INK[1], INK[2], 0.2];
        self.put(
            e.x,
            e.y,
            k.r * 0.95 * s,
            k.r * 0.36 * s,
            0.0,
            P_GROUND_SHADOW,
            0.0,
            0.0,
            shadow,
        );
        let tint = if e.kind == K_DRAGON {
            Some(self.stance_color())
        } else {
            None
        };
        self.draw_rig(
            rig,
            p3,
            Look {
                x,
                y,
                face,
                scale: s,
                flash: e.flash,
                alpha: 1.0,
                seed: e.seed,
                swing: None,
                tint,
                sheet: Sheet {
                    turn: flip,
                    ..Sheet::FLAT
                },
            },
        );
        if boss.is_some_and(|b| b.enraged()) {
            // Red ink veins once enraged.
            let h = rig.height() * s;
            for k in 0..4 {
                let a = (k as f32 * 1.3 + e.seed * 5.0).sin();
                let (vx, vy) = (x + a * h * 0.35, y - h * (0.3 + 0.12 * k as f32));
                self.put(
                    vx,
                    vy,
                    h * 0.18,
                    3.0,
                    a * 0.8 + 0.4,
                    P_SLASH,
                    0.0,
                    0.1 * k as f32,
                    [0.74, 0.12, 0.10, 0.85],
                );
            }
        }
        self.draw_status(&e, rig.height() * s);
    }

    /// Element marks on an enemy `top` units tall: flames, chill dots, an ice wash, star marks.
    fn draw_status(&mut self, e: &crate::game::Enemy, top: f32) {
        let st = e.status;
        let (x, y) = (e.x, e.y - top - 8.0);
        if st.frozen > 0.0 {
            let c = element_color(E_ICE + 1);
            self.put(
                e.x,
                e.y - top * 0.5,
                top * 0.6,
                top * 0.6,
                0.0,
                P_STONE,
                0.0,
                e.seed,
                [c[0], c[1], c[2], 0.55],
            );
        }
        if st.burn > 0.0 {
            let c = element_color(E_FIRE + 1);
            let q = (self.anim * 8.0 + e.seed * 9.0).floor();
            for k in 0..2 {
                let j = ((q + k as f32 * 3.1) * 1.7).sin();
                self.put(
                    e.x + (k as f32 - 0.5) * 14.0 + j * 3.0,
                    e.y - top * 0.4,
                    6.0,
                    9.0 + j * 2.0,
                    j * 0.3,
                    P_SPLAT,
                    0.0,
                    0.3 + k as f32 * 0.2,
                    [c[0], c[1], c[2], 0.85],
                );
            }
        }
        if st.tag > 0 {
            // The element tag waiting for a reaction: a coloured bead above the head.
            let c = element_color(st.tag);
            let a = (st.tag_t / crate::reactions::TAG_TIME).min(1.0) * 0.6 + 0.4;
            self.put(
                x,
                y - 14.0,
                6.0,
                6.0,
                0.0,
                P_BEAD,
                0.0,
                0.6,
                [c[0], c[1], c[2], a],
            );
        }
        let n = st.chill as usize + st.marks as usize;
        for k in 0..n {
            let ice = k < st.chill as usize;
            let c = if ice {
                element_color(E_ICE + 1)
            } else {
                [0.95, 0.82, 0.35]
            };
            let part = if ice { P_BEAD } else { P_GEM };
            self.put(
                x + (k as f32 - (n as f32 - 1.0) * 0.5) * 12.0,
                y,
                5.0,
                6.0,
                0.0,
                part,
                0.0,
                0.1,
                [c[0], c[1], c[2], 1.0],
            );
        }
    }

    fn draw_player(&mut self) {
        let p = &self.player;
        let (px, py) = (p.x, p.y);
        // Blink while invulnerable after a hit.
        let alpha = if p.iframe > 0.0 && ((p.iframe * 20.0) as i32) & 1 == 1 {
            0.4
        } else {
            1.0
        };
        let flash = if p.iframe > HURT_IFRAME - 0.15 {
            1.0
        } else {
            0.0
        };
        // The scythe: wound back during the startup, swept through the active window, held in the recovery.
        let swing = match p.act {
            Act::Skill { skill, t, .. } => {
                let d = &SKILLS[skill];
                Some(if t < d.startup {
                    -1.5 - 0.3 * t / d.startup.max(1e-3)
                } else {
                    let k = ((t - d.startup) / d.active.max(1e-3)).min(1.0);
                    -1.8 + 3.1 * k * k * (3.0 - 2.0 * k)
                })
            }
            _ => None,
        };
        let (steps, lean, hop) = if p.moving {
            (p.walk * 9.0, 0.10, -3.0)
        } else {
            (self.anim * 2.0, 0.0, -1.5)
        };
        let mut a = self.anims[0];
        let (face, flip) = self.facing(&a);
        a.face = face;
        let p3 = pose(
            &RIGS[RG_PLAYER],
            &self.sheet_pose[RG_PLAYER],
            &a,
            Gait {
                clock: self.anim,
                steps,
                lean,
                hop,
            },
        );
        let s = self.draw_scale[0];
        let shadow = [INK[0], INK[1], INK[2], 0.22];
        self.put(
            px,
            py,
            20.0 * s,
            7.0 * s,
            0.0,
            P_GROUND_SHADOW,
            0.0,
            0.0,
            shadow,
        );
        self.draw_rig(
            &RIGS[RG_PLAYER],
            p3,
            Look {
                x: px,
                y: py,
                face,
                scale: s,
                flash,
                alpha,
                seed: 0.37,
                swing,
                tint: None,
                sheet: Sheet {
                    turn: flip,
                    ..Sheet::FLAT
                },
            },
        );
    }
}
