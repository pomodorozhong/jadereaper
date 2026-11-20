//! Element reactions. An element hit leaves a tag on the enemy for `TAG_TIME`. A hit of a different element on a
//! tagged enemy sets off a reaction between the two, and the tag is used up:
//!
//! | Pair | Reaction |
//! | --- | --- |
//! | Fire and ice | Steam burst: a scalding blast around the target |
//! | Fire and lightning | Overload: an explosion that throws enemies back |
//! | Fire and celestial | Collapsing star: nearby enemies are pulled in, then a big blast |
//! | Ice and lightning | Shatter: heavy damage to the target, which is stunned |
//! | Ice and celestial | Stasis: enemies around are frozen in place |
//! | Lightning and celestial | Starfall chain: lightning leaps to the three nearest enemies |
//!
//! Bosses are never pulled or frozen by a reaction; they take extra stagger instead.

use crate::defs::*;
use crate::game::{Fx, Game};
use crate::geom::{dir_to, dist};

/// How long an element tag lasts.
pub const TAG_TIME: f32 = 4.0;

pub const R_STEAM: u8 = 1;
pub const R_OVERLOAD: u8 = 2;
pub const R_COLLAPSE: u8 = 3;
pub const R_SHATTER: u8 = 4;
pub const R_STASIS: u8 = 5;
pub const R_STARFALL: u8 = 6;

/// The reaction between two different elements (element ids, not + 1), or 0 for none.
pub fn reaction(a: u8, b: u8) -> u8 {
    let (lo, hi) = (a.min(b), a.max(b));
    match (lo, hi) {
        (E_FIRE, E_ICE) => R_STEAM,
        (E_FIRE, E_LIGHTNING) => R_OVERLOAD,
        (E_FIRE, E_CELESTIAL) => R_COLLAPSE,
        (E_ICE, E_LIGHTNING) => R_SHATTER,
        (E_ICE, E_CELESTIAL) => R_STASIS,
        (E_LIGHTNING, E_CELESTIAL) => R_STARFALL,
        _ => 0,
    }
}

impl Game {
    /// Tag an enemy with a hit's element (element id + 1), or react with the tag it already has. Returns the
    /// reaction set off, or 0.
    pub(crate) fn tag_or_react(&mut self, i: usize, element: u8, dmg: f32) -> u8 {
        let s = &mut self.enemies[i].status;
        let old = s.tag;
        if old == 0 || old == element || s.tag_t <= 0.0 {
            s.tag = element;
            s.tag_t = TAG_TIME;
            return 0;
        }
        s.tag = 0;
        s.tag_t = 0.0;
        let r = reaction(old - 1, element - 1);
        if r > 0 {
            self.react(i, r, dmg);
        }
        r
    }

    /// Hit enemy `i` with an element (element id + 1) for 10 damage and return the reaction, for tests.
    pub fn react_for_test(&mut self, i: usize, element: u8) -> u8 {
        self.tag_or_react(i, element, 10.0)
    }

    /// Damage every enemy within `radius` of a spot except `skip`, and stagger bosses.
    fn blast(&mut self, x: f32, y: f32, radius: f32, dmg: f32, stagger: f32, knock: f32) {
        for j in 0..self.enemies.len() {
            let e = &self.enemies[j];
            if e.hp <= 0.0 || dist(e.x, e.y, x, y) > radius + KINDS[e.kind as usize].r {
                continue;
            }
            let (kx, ky) = dir_to(x, y, e.x, e.y);
            self.hurt_enemy(j, dmg, stagger, kx * knock, ky * knock);
        }
    }

    fn react(&mut self, i: usize, r: u8, dmg: f32) {
        let (x, y) = (self.enemies[i].x, self.enemies[i].y);
        let boss = self.is_boss(i);
        let color = |e: u8| crate::elements::element_color(e + 1);
        let ring = |g: &mut Game, radius: f32, c: [f32; 3]| {
            g.add_fx(Fx { grow: radius / 0.25, top: false, ..Fx::at(x, y, P_RING, 0.25, 10.0, 10.0, [c[0], c[1], c[2], 0.9]) });
        };
        match r {
            R_STEAM => {
                ring(self, 90.0, [0.92, 0.92, 0.9]);
                self.blast(x, y, 90.0, dmg * 1.2 + 14.0, 20.0, 200.0);
                let s = &mut self.enemies[i].status;
                s.burn = 0.0;
                s.chill = 0;
            }
            R_OVERLOAD => {
                ring(self, 110.0, color(E_FIRE));
                self.blast(x, y, 110.0, dmg + 18.0, 30.0, 520.0);
                self.shake = self.shake.max(0.6);
            }
            R_COLLAPSE => {
                // Gravity pulls them together, then the star collapses.
                for j in 0..self.enemies.len() {
                    if self.is_boss(j) {
                        continue;
                    }
                    let e = &mut self.enemies[j];
                    let d = dist(e.x, e.y, x, y);
                    if d < 180.0 && d > 1.0 {
                        let (dx, dy) = dir_to(e.x, e.y, x, y);
                        let pull = (d - 24.0).max(0.0) * 0.8;
                        e.x += dx * pull;
                        e.y += dy * pull;
                    }
                }
                ring(self, 130.0, color(E_CELESTIAL));
                self.blast(x, y, 130.0, dmg * 1.5 + 30.0, 40.0, 120.0);
                self.shake = self.shake.max(0.8);
            }
            R_SHATTER => {
                ring(self, 40.0, color(E_ICE));
                if i < self.enemies.len() {
                    self.hurt_enemy(i, dmg * 2.0 + 10.0, 50.0, 0.0, 0.0);
                    if !boss {
                        self.enemies[i].stun = 0.6;
                    }
                }
            }
            R_STASIS => {
                ring(self, 120.0, color(E_ICE));
                for j in 0..self.enemies.len() {
                    let e = &self.enemies[j];
                    if dist(e.x, e.y, x, y) < 120.0 + KINDS[e.kind as usize].r {
                        if self.is_boss(j) {
                            self.add_boss_stagger(j, 40.0);
                        } else {
                            self.enemies[j].status.frozen = 1.5;
                        }
                    }
                }
            }
            R_STARFALL => {
                let mut near: Vec<(usize, f32)> = (0..self.enemies.len())
                    .filter(|&j| j != i && self.enemies[j].hp > 0.0)
                    .map(|j| (j, dist(x, y, self.enemies[j].x, self.enemies[j].y)))
                    .filter(|p| p.1 < 260.0)
                    .collect();
                near.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(core::cmp::Ordering::Equal));
                let c = color(E_LIGHTNING);
                for &(j, d) in near.iter().take(3) {
                    let (tx, ty) = (self.enemies[j].x, self.enemies[j].y);
                    let rot = (ty - y).atan2(tx - x);
                    self.add_fx(Fx { rot, seed: 0.9, ..Fx::at((x + tx) * 0.5, (y + ty) * 0.5 - 24.0, P_SLASH, 0.2, d * 0.5, 6.0, [c[0], c[1], c[2], 0.95]) });
                    self.hurt_enemy(j, dmg * 0.6 + 12.0, 20.0, 0.0, 0.0);
                }
            }
            _ => {}
        }
        self.event(EV_REACTION, x, y, r as f32);
    }
}
