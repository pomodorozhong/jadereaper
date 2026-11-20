//! Building the instance list the renderer draws: ground, characters, projectiles and effects.

use crate::defs::*;
use crate::game::{Act, Game, Phase};
use crate::geom::{Area, Shape};
use crate::moves::{After, MOVES};
use crate::skills::SKILLS;
use core::f32::consts::TAU;

const WARN: [f32; 4] = [0.17, 0.15, 0.13, 0.55];
const WARN_BOSS: [f32; 4] = [0.62, 0.16, 0.12, 0.6];

impl Game {
    // ---------------------------------------------------------------- draw

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn put(&mut self, x: f32, y: f32, hw: f32, hh: f32, rot: f32, part: f32, flash: f32, seed: f32, color: [f32; 4]) {
        if self.inst.len() >= MAX_INST {
            return;
        }
        // Cull what the camera cannot see.
        let m = hw.abs().max(hh) + 24.0;
        if (x - self.cam.0).abs() > self.view_w + m || (y - self.cam.1).abs() > self.view_h + m {
            return;
        }
        let part = part + PART_SLOTS * self.boil_pose(seed) as f32;
        self.inst.push(Inst { x, y, hw, hh, rot, part, flash, seed, color });
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
        self.draw_ground();

        // Ground-level effects and pickups.
        for i in 0..self.fx.len() {
            if !self.fx[i].top {
                self.draw_fx(i);
            }
        }
        for i in 0..self.gems.len() {
            let g = &self.gems[i];
            let (x, y, seed, heart) = (g.x, g.y, g.seed, g.heart);
            // Two-pose rock, like a paper tag on a string.
            let pose = (((self.anim * 4.0 + seed * 9.0).floor() as i32) & 1) as f32;
            let rot = pose * 0.3 - 0.15;
            if heart {
                self.put(x, y - 12.0, 11.0, 11.0, rot, P_HEART, 0.0, seed, [0.78, 0.27, 0.18, 1.0]);
            } else {
                let big = self.gems[i].value >= 5.0;
                let s = if big { 10.0 } else { 7.0 };
                let c = if big { [0.85, 0.72, 0.30, 1.0] } else { [0.36, 0.72, 0.56, 1.0] };
                self.put(x, y - 8.0, s * 0.8, s, rot, P_GEM, 0.0, seed, c);
            }
        }

        // Warnings for attacks that are winding up.
        for i in 0..self.enemies.len() {
            let e = self.enemies[i];
            if e.phase != Phase::Windup || e.stun > 0.0 {
                continue;
            }
            let m = MOVES[e.mv as usize];
            let prog = (e.pt / m.windup).clamp(0.0, 1.0);
            let color = if e.kind == K_ELITE { WARN_BOSS } else { WARN };
            let area = self.move_area(&e);
            self.draw_warning(area, prog, color);
            if let After::Wave { to, .. } = m.after {
                // The shock ring that will follow: its whole reach, faint.
                let ring = Area { shape: Shape::Ring { inner: m.reach, outer: to }, ..area };
                self.draw_warning(ring, 0.0, [color[0], color[1], color[2], color[3] * 0.5]);
            }
        }
        for i in 0..self.hazards.len() {
            let h = self.hazards[i];
            if h.band > 0.0 {
                let outer = h.r + h.band * 0.5;
                let inner = (h.r - h.band * 0.5).max(0.0);
                self.put(h.x, h.y, outer, outer, 0.0, P_WARN_RING, 1.0, inner / outer, [INK[0], INK[1], INK[2], 0.75]);
            }
        }

        // Characters, sorted by depth.
        self.order.clear();
        for (i, e) in self.enemies.iter().enumerate() {
            self.order.push((e.y, i as u16));
        }
        self.order.push((self.player.y, u16::MAX));
        self.order.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));
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
                self.put(h.x, h.y - 22.0, h.r * 1.3, h.r * 1.1, wob, P_BLOT, 0.0, 0.77, [0.20, 0.18, 0.22, 1.0]);
            }
        }
        for i in 0..self.shots.len() {
            let s = &self.shots[i];
            let (x, y, rot) = (s.x, s.y, s.vy.atan2(s.vx) + TAU * 0.25);
            self.put(x, y - 24.0, 6.0, 11.0, rot, P_TALISMAN, 0.0, 0.5, [0.89, 0.77, 0.38, 1.0]);
        }
        for k in 0..crate::skills::ORBIT_BEADS {
            if let Some((x, y)) = self.bead_pos(k) {
                self.put(x, y - 24.0, 9.0, 9.0, 0.0, P_BEAD, 0.0, k as f32 * 0.13, [0.30, 0.66, 0.50, 1.0]);
            }
        }
        for i in 0..self.fx.len() {
            if self.fx[i].top {
                self.draw_fx(i);
            }
        }
    }

    /// An attack's warning: an ink wash in the shape of the coming hit, filling in as the wind-up runs.
    fn draw_warning(&mut self, a: Area, prog: f32, color: [f32; 4]) {
        let rot = a.dy.atan2(a.dx);
        match a.shape {
            Shape::Circle { r } => self.put(a.x, a.y, r, r, 0.0, P_WARN_CIRCLE, prog, 0.0, color),
            Shape::Arc { r, half } => self.put(a.x, a.y, r, r, rot, P_WARN_ARC, prog, half / core::f32::consts::PI, color),
            Shape::Line { len, half_w } => {
                let (cx, cy) = (a.x + a.dx * len * 0.5, a.y + a.dy * len * 0.5);
                self.put(cx, cy, len * 0.5, half_w, rot, P_WARN_LINE, prog, 0.0, color)
            }
            Shape::Ring { inner, outer } => self.put(a.x, a.y, outer, outer, 0.0, P_WARN_RING, prog, inner / outer, color),
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
                if (h >> 24) % 4 == 0 {
                    self.put(x, y, 16.0 + seed * 12.0, 8.0 + seed * 5.0, 0.0, P_STONE, 0.0, seed, [INK[0], INK[1], INK[2], 0.16]);
                } else {
                    let flip = if h & 1 == 0 { 1.0 } else { -1.0 };
                    self.put(x, y, (10.0 + seed * 6.0) * flip, 10.0 + seed * 8.0, 0.0, P_GRASS, 0.0, seed, [0.22, 0.36, 0.30, 0.45]);
                }
            }
        }
    }

    fn draw_enemy(&mut self, i: usize) {
        let e = self.enemies[i];
        let k = &KINDS[e.kind as usize];
        // Two poses, flipped back and forth. That is the whole walk cycle.
        let (lean, rate) = self.waddle[e.kind as usize];
        let pose = ((self.anim * rate + e.seed * 7.0).floor() as i32) & 1;
        let (mut tilt, mut hop) = if pose == 0 { (lean, 0.0) } else { (-lean, -3.0) };
        let mut ex = e.x;
        match e.phase {
            // Rear back and tremble while winding up, lean in while striking.
            Phase::Windup => {
                tilt = -0.25 * e.face;
                hop = -4.0;
                ex += ((self.anim * 40.0).floor() * 1.7).sin() * 1.5;
            }
            Phase::Active => tilt = 0.3 * e.face,
            _ => {}
        }
        let e = crate::game::Enemy { x: ex, ..e };
        let c = [k.color[0], k.color[1], k.color[2], 1.0];
        let shadow = [INK[0], INK[1], INK[2], 0.2];
        let s = self.draw_scale[1 + e.kind as usize];
        self.put(e.x, e.y, k.r * 0.95 * s, k.r * 0.36 * s, 0.0, P_GROUND_SHADOW, 0.0, 0.0, shadow);
        if e.kind == K_WISP {
            // Wisps hover and flap by squeezing their width.
            let hw = if pose == 0 { k.hw } else { k.hw * 0.7 };
            let bob = (self.anim * 5.0 + e.seed * 20.0).sin() * 3.0;
            self.put(e.x, e.y - 26.0 * s + bob, hw * s * e.face, k.hh * s, tilt, k.part, e.flash, e.seed, c);
        } else {
            self.put(e.x, e.y - k.hh * s + hop, k.hw * s * e.face, k.hh * s, tilt, k.part, e.flash, e.seed, c);
        }
    }

    fn draw_player(&mut self) {
        let p = &self.player;
        let (px, py, f) = (p.x, p.y, p.face);
        let (tilt, bob) = if let Act::Roll { t, .. } = p.act {
            // Tucked into a tumble while the roll moves.
            let k = (t / ROLL_TIME).min(1.0);
            (k * TAU * f, -6.0 * (k * TAU * 0.5).sin())
        } else if p.moving {
            if ((p.walk * 9.0).floor() as i32) & 1 == 0 { (0.10, 0.0) } else { (-0.10, -3.0) }
        } else if ((self.anim * 2.0).floor() as i32) & 1 == 0 {
            (0.0, 0.0)
        } else {
            (0.0, -1.5)
        };
        // Blink while invulnerable.
        let a = if p.iframe > 0.0 && ((p.iframe * 20.0) as i32) & 1 == 1 { 0.4 } else { 1.0 };
        let flash = if p.iframe > 0.65 { 1.0 } else { 0.0 };
        // The scythe swing: wound back during the startup, swept through the active window, held in the recovery.
        let swing = if let Act::Skill { skill, t, .. } = p.act {
            let d = &SKILLS[skill];
            if t < d.startup {
                Some(-1.5 - 0.3 * t / d.startup.max(1e-3))
            } else {
                let k = ((t - d.startup) / d.active.max(1e-3)).min(1.0);
                Some(-1.8 + 3.1 * k * k * (3.0 - 2.0 * k))
            }
        } else {
            None
        };
        // Drawn size; the hit area stays the same.
        let s = self.draw_scale[0];

        let shadow = [INK[0], INK[1], INK[2], 0.22];
        self.put(px, py, 20.0 * s, 7.0 * s, 0.0, P_GROUND_SHADOW, 0.0, 0.0, shadow);

        let scythe = [0.36, 0.66, 0.52, a];
        if swing.is_none() {
            // Resting over the shoulder, behind the body.
            self.put(px - f * 15.0 * s, py - 46.0 * s + bob, 20.0 * s * f, 34.0 * s, -f * 0.45 + tilt, P_SCYTHE, flash, 0.21, scythe);
        }
        let top_x = px + 26.0 * s * tilt.sin();
        // The traced player is one square drawing, so the robe's box is square and stands on the shadow.
        self.put(px, py - 40.0 * s + bob, 40.0 * s * f, 40.0 * s, tilt, P_PLAYER_BODY, flash, 0.37, [0.20, 0.42, 0.36, a]);
        self.put(top_x + f * 2.0 * s, py - 60.0 * s + bob * 1.5, 15.0 * s * f, 15.0 * s, -tilt * 0.6, P_PLAYER_HEAD, flash, 0.53, [0.15, 0.33, 0.29, a]);
        if let Some(a) = swing {
            let ang = a * f;
            self.put(px + f * 12.0 * s, py - 42.0 * s + bob, 20.0 * s * f, 34.0 * s, ang, P_SCYTHE, flash, 0.21, scythe);
        }
    }

}
