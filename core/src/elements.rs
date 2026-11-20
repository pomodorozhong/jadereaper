//! Element effects of skill hits:
//!
//! - Fire burns: damage over time.
//! - Ice chills, slowing; the third chill in a row freezes.
//! - Lightning chains to the nearest other enemy.
//! - Celestial leaves a star mark; the third mark opens a gravity well that pulls nearby enemies together, then a
//!   star falls on the spot. Bosses are never pulled; celestial hits add stagger to them instead.

use crate::defs::*;
use crate::game::{Fx, Game};
use crate::geom::{dir_to, dist};

pub const BURN_TIME: f32 = 3.0;
/// Burn damage per second, as a share of the hit that lit it (at least `BURN_MIN`).
pub const BURN_SHARE: f32 = 0.35;
pub const BURN_MIN: f32 = 3.0;
/// Each chill slows by this share; chills wear off after `CHILL_TIME`.
pub const CHILL_SLOW: f32 = 0.15;
pub const CHILL_TIME: f32 = 2.5;
pub const CHILLS_TO_FREEZE: u8 = 3;
pub const FREEZE_TIME: f32 = 1.2;
pub const CHAIN_RANGE: f32 = 170.0;
pub const CHAIN_SHARE: f32 = 0.5;
pub const MARKS_TO_WELL: u8 = 3;
pub const WELL_TIME: f32 = 0.8;
pub const WELL_RANGE: f32 = 170.0;
pub const WELL_PULL: f32 = 240.0;
pub const STAR_RADIUS: f32 = 80.0;
/// Extra stagger a celestial hit puts on a boss, as a share of the hit's stagger.
pub const CELESTIAL_BOSS_STAGGER: f32 = 0.6;
/// How long a fire trail patch burns.
pub const TRAIL_TIME: f32 = 1.5;

/// A celestial gravity well; when its time runs out a star falls on it.
#[derive(Clone, Copy, Debug)]
pub struct Well {
    pub x: f32,
    pub y: f32,
    pub t: f32,
    pub dmg: f32,
}

/// A patch of burning ground left by a fire trail roll.
#[derive(Clone, Copy, Debug)]
pub struct Patch {
    pub x: f32,
    pub y: f32,
    pub t: f32,
    pub element: u8,
}

/// Per-enemy element state.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Status {
    pub burn: f32,
    pub burn_dps: f32,
    pub burn_tick: f32,
    pub chill: u8,
    pub chill_t: f32,
    pub frozen: f32,
    pub marks: u8,
}

impl Status {
    /// How fast the enemy moves and acts, from 1 down.
    pub fn pace(&self) -> f32 {
        if self.frozen > 0.0 {
            0.0
        } else {
            1.0 - CHILL_SLOW * self.chill as f32
        }
    }
}

pub fn element_color(e: u8) -> [f32; 3] {
    if e == 0 {
        [0.25, 0.62, 0.48]
    } else {
        ELEMENTS[(e - 1) as usize].color
    }
}

impl Game {
    pub(crate) fn is_boss(&self, i: usize) -> bool {
        KINDS[self.enemies[i].kind as usize].boss
    }

    /// Extra stagger on a boss's meter.
    pub(crate) fn add_boss_stagger(&mut self, i: usize, stagger: f32) {
        let id = self.enemies[i].id;
        if let Some(b) = self.boss.as_mut().filter(|b| b.id == id) {
            if b.down <= 0.0 {
                b.meter += stagger;
            }
        }
    }

    /// Apply a hit's element (0 for none, or element id + 1) to enemy `i`, which the hit just damaged.
    pub(crate) fn apply_element(&mut self, i: usize, element: u8, dmg: f32, stagger: f32) {
        if element == 0 || i >= self.enemies.len() {
            return;
        }
        let boss = self.is_boss(i);
        let (x, y) = (self.enemies[i].x, self.enemies[i].y);
        match element - 1 {
            E_FIRE => {
                let s = &mut self.enemies[i].status;
                s.burn = BURN_TIME;
                s.burn_dps = s.burn_dps.max((dmg * BURN_SHARE).max(BURN_MIN));
            }
            E_ICE => {
                let s = &mut self.enemies[i].status;
                s.chill_t = CHILL_TIME;
                s.chill = (s.chill + 1).min(CHILLS_TO_FREEZE);
                if s.chill >= CHILLS_TO_FREEZE {
                    s.chill = 0;
                    if boss {
                        self.add_boss_stagger(i, stagger);
                    } else {
                        s.frozen = FREEZE_TIME;
                    }
                }
            }
            E_LIGHTNING => {
                let id = self.enemies[i].id;
                let next = (0..self.enemies.len())
                    .filter(|&j| self.enemies[j].id != id && self.enemies[j].hp > 0.0)
                    .map(|j| (j, dist(x, y, self.enemies[j].x, self.enemies[j].y)))
                    .filter(|&(_, d)| d < CHAIN_RANGE)
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(core::cmp::Ordering::Equal));
                if let Some((j, d)) = next {
                    let (tx, ty) = (self.enemies[j].x, self.enemies[j].y);
                    let rot = (ty - y).atan2(tx - x);
                    let c = element_color(element);
                    self.add_fx(Fx { rot, seed: 0.9, ..Fx::at((x + tx) * 0.5, (y + ty) * 0.5 - 24.0, P_SLASH, 0.15, d * 0.5, 6.0, [c[0], c[1], c[2], 0.95]) });
                    // The chain hit carries no element, so it does not chain on.
                    self.hurt_enemy(j, dmg * CHAIN_SHARE, stagger * CHAIN_SHARE, 0.0, 0.0);
                }
            }
            E_CELESTIAL => {
                if boss {
                    self.add_boss_stagger(i, stagger * CELESTIAL_BOSS_STAGGER);
                } else {
                    let s = &mut self.enemies[i].status;
                    s.marks += 1;
                    if s.marks >= MARKS_TO_WELL {
                        s.marks = 0;
                        self.wells.push(Well { x, y, t: WELL_TIME, dmg: dmg * 1.5 + 12.0 });
                    }
                }
            }
            _ => {}
        }
    }

    /// Burns, chills, freezes, gravity wells and fire trails, once a step.
    pub(crate) fn move_elements(&mut self, dt: f32) {
        for i in 0..self.enemies.len() {
            let s = &mut self.enemies[i].status;
            s.frozen = (s.frozen - dt).max(0.0);
            if s.chill > 0 {
                s.chill_t -= dt;
                if s.chill_t <= 0.0 {
                    s.chill = 0;
                }
            }
            if s.burn > 0.0 {
                s.burn -= dt;
                s.burn_tick += dt;
                if s.burn_tick >= 0.5 {
                    s.burn_tick -= 0.5;
                    let dmg = s.burn_dps * 0.5;
                    let e = &mut self.enemies[i];
                    e.hp -= dmg;
                    let (x, y) = (e.x, e.y);
                    self.event(EV_HIT, x, y, dmg);
                }
            } else {
                s.burn_dps = 0.0;
                s.burn_tick = 0.0;
            }
        }

        let mut w = 0;
        while w < self.wells.len() {
            let well = self.wells[w];
            for i in 0..self.enemies.len() {
                if self.is_boss(i) {
                    continue;
                }
                let e = &mut self.enemies[i];
                let d = dist(e.x, e.y, well.x, well.y);
                if d < WELL_RANGE && d > 6.0 {
                    let (dx, dy) = dir_to(e.x, e.y, well.x, well.y);
                    e.x += dx * WELL_PULL * dt;
                    e.y += dy * WELL_PULL * dt;
                }
            }
            self.wells[w].t -= dt;
            if self.wells[w].t <= 0.0 {
                // The star falls.
                for i in 0..self.enemies.len() {
                    let e = &self.enemies[i];
                    if dist(e.x, e.y, well.x, well.y) < STAR_RADIUS + KINDS[e.kind as usize].r {
                        self.hurt_enemy(i, well.dmg, 30.0, 0.0, 0.0);
                    }
                }
                let c = element_color(E_CELESTIAL + 1);
                self.add_fx(Fx { grow: 200.0, top: false, ..Fx::at(well.x, well.y, P_RING, 0.3, 20.0, 20.0, [c[0], c[1], c[2], 0.9]) });
                self.add_fx(Fx { vy: 0.0, ..Fx::at(well.x, well.y - 40.0, P_GEM, 0.3, 14.0, 18.0, [0.95, 0.85, 0.4, 1.0]) });
                self.shake = self.shake.max(0.5);
                self.event(EV_PULSE, well.x, well.y, 0.0);
                self.wells.swap_remove(w);
            } else {
                w += 1;
            }
        }

        // A fire trail: drop patches while rolling, and burn what stands in them.
        if self.stats.trail > 0 && matches!(self.player.act, crate::game::Act::Roll { .. }) && self.steps % 4 == 0 {
            let (x, y, element) = (self.player.x, self.player.y, self.stats.trail);
            if self.patches.len() < 40 {
                self.patches.push(Patch { x, y, t: TRAIL_TIME, element });
            }
        }
        let mut p = 0;
        while p < self.patches.len() {
            let patch = self.patches[p];
            if self.steps % 15 == 0 {
                for i in 0..self.enemies.len() {
                    let e = &self.enemies[i];
                    if dist(e.x, e.y, patch.x, patch.y) < 26.0 + KINDS[e.kind as usize].r {
                        self.apply_element(i, patch.element, 6.0, 0.0);
                    }
                }
            }
            self.patches[p].t -= dt;
            if self.patches[p].t <= 0.0 {
                self.patches.swap_remove(p);
            } else {
                p += 1;
            }
        }
    }
}
