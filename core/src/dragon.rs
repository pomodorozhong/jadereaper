//! The Two-Element Dragon (Zhulong), the final boss. It fights in a fire stance or an ice stance, and the right
//! element keeps the player alive:
//!
//! - Each stance is weak to the other element. Enough damage of the weak element knocks the Dragon down.
//! - A stance ends after `STANCE_TIME`, or when its health passes two thirds or one third, with a finisher that fills
//!   the screen and builds for 3 s. The roll does not avoid it. Its damage drops with the weak-element damage dealt
//!   during that stance, down to `FINISHER_FLOOR`, and the first one never kills from full health.
//! - Then it changes stance.
//!
//! Fire moves: dive-bombs, lingering fire pools, a spreading flame line. Ice moves: lines of ice lances, a freezing
//! field that slows, falling shards. Flight and a third stance are left out for now.

use crate::boss::BossState;
use crate::defs::*;
use crate::game::{Enemy, Game, Hazard, Phase};
use crate::geom::{dir_to, Area, Shape};
use crate::moves::*;

/// Seconds before a stance ends with a finisher.
pub const STANCE_TIME: f32 = 45.0;
/// Health shares at which a stance also ends.
pub const STANCE_HP: [f32; 2] = [0.66, 0.33];
/// Weak-element damage in one stance that brings the finisher down to its floor.
pub const WEAK_FULL: f32 = 260.0;
/// Weak-element damage that knocks the Dragon down.
pub const WEAK_DOWN: f32 = 160.0;
/// The finisher's damage as a share of the player's maximum health: the first one, and later ones.
pub const FINISHER_FIRST: f32 = 0.8;
pub const FINISHER_LATER: f32 = 1.15;
/// The finisher never drops below this share of its full damage.
pub const FINISHER_FLOOR: f32 = 0.2;
/// How long a fire pool burns, and how often it hurts someone standing in it.
pub const POOL_TIME: f32 = 4.0;
pub const POOL_REHIT: f32 = 0.8;
/// How long the freezing field slows, and by how much.
pub const FIELD_SLOW_TIME: f32 = 3.0;
pub const FIELD_SLOW: f32 = 0.6;
/// The length of a line of lances that is erupting at once.
pub const LANCE_FRONT: f32 = 70.0;

/// The element the Dragon's current stance is weak to (element id + 1), or 0.
pub fn weakness(stance: Stance) -> u8 {
    match stance {
        Stance::Fire => E_ICE + 1,
        Stance::Ice => E_FIRE + 1,
        Stance::Any => 0,
    }
}

/// The Dragon's damage from the finisher: full when no weak-element damage was dealt in the stance.
pub fn finisher_damage(max_hp: f32, weak_dmg: f32, first: bool) -> f32 {
    let share = if first {
        FINISHER_FIRST
    } else {
        FINISHER_LATER
    };
    max_hp * share * (1.0 - weak_dmg / WEAK_FULL).max(FINISHER_FLOOR)
}

impl Game {
    pub(crate) fn is_dragon(&self, i: usize) -> bool {
        self.enemies[i].kind == K_DRAGON
    }

    /// Element damage on the Dragon: weak-element damage counts towards its knockdown and against the finisher.
    pub(crate) fn note_element_damage(&mut self, i: usize, element: u8, dmg: f32) {
        if element == 0 || !self.is_dragon(i) || element != weakness(self.stance) {
            return;
        }
        let id = self.enemies[i].id;
        let Some(b) = self.boss.as_mut().filter(|b| b.id == id) else {
            return;
        };
        b.weak_dmg += dmg;
        b.weak_meter += dmg;
        if b.weak_meter >= WEAK_DOWN && b.down <= 0.0 {
            b.weak_meter = 0.0;
            b.meter = KINDS[K_DRAGON as usize].poise;
        }
    }

    /// Stance upkeep, before the Dragon chooses a move: count the stance's time, and start the finisher when the
    /// stance is over.
    pub(crate) fn dragon_stance(&mut self, i: usize, b: &mut BossState, dt: f32) {
        if self.stance == Stance::Any {
            self.stance = Stance::Fire;
        }
        b.stance_t += dt;
        let max = KINDS[K_DRAGON as usize].hp;
        let hp = self.enemies[i].hp;
        let mut over = b.stance_t >= STANCE_TIME;
        for (k, &share) in STANCE_HP.iter().enumerate() {
            if hp < max * share && b.passed & (1 << k) == 0 {
                b.passed |= 1 << k;
                over = true;
            }
        }
        let e = &self.enemies[i];
        let busy =
            matches!(e.phase, Phase::Windup | Phase::Active) && e.mv as usize == M_DRAGON_FINISHER;
        if over && !busy {
            let e = &mut self.enemies[i];
            e.mv = M_DRAGON_FINISHER as u8;
            e.chain = 0;
            self.start_windup(i);
            let (x, y) = (self.enemies[i].x, self.enemies[i].y);
            self.event(EV_FINISHER, x, y, 0.0);
        }
    }

    /// Set up where a Dragon move lands as its wind-up starts: a dive's target, the pools and the shards.
    pub(crate) fn dragon_aim(&mut self, i: usize, b: &mut BossState) {
        let e = self.enemies[i];
        let (px, py) = (self.player.x, self.player.y);
        let (dx, dy) = dir_to(e.x, e.y, px, py);
        b.nspots = 0;
        match MOVES[e.mv as usize].special {
            Special::Dive => {
                let en = &mut self.enemies[i];
                en.ox = px;
                en.oy = py;
            }
            Special::Pools => {
                // One under the player and one to each side, across the line from the Dragon.
                for side in [0.0, 1.0, -1.0] {
                    b.spots[b.nspots as usize] = (px - dy * 120.0 * side, py + dx * 120.0 * side);
                    b.nspots += 1;
                }
            }
            Special::Shards => {
                b.spots[0] = (px, py);
                b.nspots = 1;
                for _ in 0..4 {
                    let a = self.rng.range(0.0, core::f32::consts::TAU);
                    let r = self.rng.range(70.0, 160.0);
                    b.spots[b.nspots as usize] = (px + a.cos() * r, py + a.sin() * r);
                    b.nspots += 1;
                }
            }
            _ => {}
        }
    }

    /// The hit areas of a boss move, which may be several (pools, shards). Returns them and how many there are.
    pub fn move_areas(&self, e: &Enemy) -> ([Area; 5], usize) {
        let m = &MOVES[e.mv as usize];
        let one = self.move_area(e);
        let mut out = [one; 5];
        match m.special {
            Special::Pools | Special::Shards => {
                let Some(b) = self.boss.filter(|b| b.id == e.id) else {
                    return (out, 1);
                };
                for (slot, spot) in out.iter_mut().zip(&b.spots).take(b.nspots as usize) {
                    *slot = Area::new(spot.0, spot.1, (1.0, 0.0), Shape::Circle { r: m.reach });
                }
                (out, b.nspots as usize)
            }
            Special::Dive | Special::Field | Special::Finisher => {
                out[0] = Area::new(e.ox, e.oy, (1.0, 0.0), Shape::Circle { r: m.reach });
                (out, 1)
            }
            _ => (out, 1),
        }
    }

    /// The hit areas at `k` (0..1) through the active window: a line of lances hurts only where it is erupting.
    pub fn strike_areas(&self, i: usize, k: f32) -> ([Area; 5], usize) {
        let e = &self.enemies[i];
        let m = &MOVES[e.mv as usize];
        if m.special == Special::Lances {
            let front = m.reach * k.clamp(0.0, 1.0);
            let start = (front - LANCE_FRONT).max(0.0);
            let a = Area::new(
                e.ox + e.dx * start,
                e.oy + e.dy * start,
                (e.dx, e.dy),
                Shape::Line {
                    len: front - start,
                    half_w: m.width,
                },
            );
            return ([a; 5], 1);
        }
        if matches!(
            m.special,
            Special::Pools | Special::Shards | Special::Dive | Special::Field | Special::Finisher
        ) {
            return self.move_areas(e);
        }
        ([self.boss_area(i, k); 5], 1)
    }

    /// The moment a Dragon move strikes: the dive lands, pools start burning.
    pub(crate) fn dragon_strikes(&mut self, i: usize, b: &mut BossState) {
        let e = self.enemies[i];
        let m = MOVES[e.mv as usize];
        match m.special {
            Special::Dive => {
                let en = &mut self.enemies[i];
                en.x = e.ox;
                en.y = e.oy;
                self.shake = self.shake.max(0.8);
                self.ground_burst(e.ox, e.oy, KINDS[e.kind as usize].r * 1.6, 1.3);
            }
            Special::Pools => {
                for &(x, y) in b.spots.iter().take(b.nspots as usize) {
                    self.add_hazard(Hazard {
                        x,
                        y,
                        vx: 0.0,
                        vy: 0.0,
                        r: m.reach,
                        band: 0.0,
                        to: 0.0,
                        grow: 0.0,
                        life: POOL_TIME,
                        dmg: m.damage * 0.5,
                        boss: true,
                        warned: m.windup,
                        rehit: POOL_REHIT,
                        cool: 0.0,
                    });
                }
            }
            Special::Finisher => self.finisher(b),
            _ => {}
        }
    }

    /// The finisher lands on the player wherever they are, invulnerable or not, then the stance changes.
    fn finisher(&mut self, b: &mut BossState) {
        let first = b.finishers == 0;
        let dmg = finisher_damage(self.player.max_hp, b.weak_dmg, first);
        let (px, py) = (self.player.x, self.player.y);
        self.min_warned = self.min_warned.min(MOVES[M_DRAGON_FINISHER].windup);
        self.player.hp -= dmg;
        self.shake = 1.4;
        self.event(EV_HURT, px, py, dmg);
        if self.player.hp <= 0.0 {
            self.player.hp = 0.0;
            self.over = true;
            self.event(EV_DEATH, px, py, 0.0);
        }
        b.finishers += 1;
        b.stance_t = 0.0;
        b.weak_dmg = 0.0;
        b.weak_meter = 0.0;
        self.stance = if self.stance == Stance::Ice {
            Stance::Fire
        } else {
            Stance::Ice
        };
        self.event(
            EV_STANCE,
            px,
            py,
            if self.stance == Stance::Fire {
                1.0
            } else {
                2.0
            },
        );
    }

    /// A hit from a Dragon move on the player: the freezing field slows.
    pub(crate) fn dragon_hit(&mut self, m: &Move) {
        if m.special == Special::Field {
            self.player.slow = FIELD_SLOW_TIME;
        }
    }
}
