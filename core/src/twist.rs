//! The spin a hit puts on a paper character, about its vertical axis. A hit kicks the sheet round: a light one
//! twists it, then a spring swings it back and forth and the swing dies away; a heavy one (a large share of the
//! character's max HP) sends it spinning whole turns, slowing down, and then it wobbles to rest facing front
//! again. While it spins or swings more than a little, the character cannot move or act: this replaces the old
//! fixed hurt, stun and knockdown times. It is part of the simulation, stepped at the fixed rate, so a run still
//! replays the same.

use core::f32::consts::TAU;

/// A hit worth this share of the character's max HP or more spins it; each further share as big adds a turn.
pub const SPIN_SHARE: f32 = 0.2;
/// The most whole turns a hit sends it round.
pub const MAX_TURNS: f32 = 3.0;
/// Turns for a knockdown (a boss's heavy hit on the player, or a boss's full stagger meter) and for a broken
/// poise or a shatter.
pub const DOWN_TURNS: f32 = 2.0;
pub const STUN_TURNS: f32 = 1.0;
/// The swing's spring: natural rate in radians per second and damping ratio, so a swing dies away by about
/// e^(-ζω) per second.
const RATE: f32 = 16.0;
const DAMPING: f32 = 0.2;
/// Spin a hit just short of `SPIN_SHARE` gives, in radians per second; lighter hits give less, in proportion.
/// About 1.4 rad (80°) of swing at the edge of a spin.
const KICK: f32 = 23.5;
/// How fast a spin slows, in radians per second squared, and how far past its last turn it carries before the
/// spring takes it back.
const SPIN_DECEL: f32 = 38.0;
const OVERSHOOT: f32 = 0.5;
/// Still enough to move again: within this angle of rest and slower than this.
const SETTLE_ANGLE: f32 = 0.15;
const SETTLE_SPEED: f32 = 1.5;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Twist {
    /// Angle about the vertical axis, radians; positive turns the front to the right (see `Sheet::turn`).
    pub angle: f32,
    /// Its speed, radians per second.
    pub speed: f32,
    /// The angle it comes to rest at: a whole number of turns, so it ends facing front.
    pub rest: f32,
    /// Spinning freely towards `rest` (plus the overshoot), before the spring takes over.
    pub spinning: bool,
}

impl Twist {
    /// Take a hit worth `share` of max HP, pushing the sheet's front round in `sign`'s direction.
    pub fn hit(&mut self, share: f32, sign: f32) {
        let sign = if sign < 0.0 { -1.0 } else { 1.0 };
        if share >= SPIN_SHARE {
            let turns = (1.0 + ((share - SPIN_SHARE) / SPIN_SHARE).floor()).min(MAX_TURNS);
            self.spin(turns, sign);
        } else if !self.spinning {
            self.speed += sign * KICK * share.max(0.0) / SPIN_SHARE;
        }
    }

    /// Send it spinning `turns` whole turns in `sign`'s direction, then let it wobble to rest.
    pub fn spin(&mut self, turns: f32, sign: f32) {
        let sign = if sign < 0.0 { -1.0 } else { 1.0 };
        // Rest at the next whole turn past `turns` from here, so it always ends facing front.
        self.rest = ((self.angle + sign * turns * TAU) / TAU).round() * TAU;
        let travel = (self.rest - self.angle).abs() + OVERSHOOT;
        self.speed = sign * (2.0 * SPIN_DECEL * travel).sqrt();
        self.spinning = true;
    }

    /// Step by `dt`.
    pub fn step(&mut self, dt: f32) {
        if self.spinning {
            let s = self.speed.signum();
            let next = self.speed - s * SPIN_DECEL * dt;
            if next.signum() != s {
                self.spinning = false;
                self.speed = 0.0;
            } else {
                self.speed = next;
                self.angle += self.speed * dt;
            }
            return;
        }
        let pull = -RATE * RATE * (self.angle - self.rest) - 2.0 * DAMPING * RATE * self.speed;
        self.speed += pull * dt;
        self.angle += self.speed * dt;
        if !self.locked() && (self.angle - self.rest).abs() < 0.02 && self.speed.abs() < 0.2 {
            // At rest: snap and bring the angle back near zero.
            self.angle = 0.0;
            self.rest = 0.0;
            self.speed = 0.0;
        }
    }

    /// True while it spins or swings too much for the character to move.
    pub fn locked(&self) -> bool {
        self.spinning
            || (self.angle - self.rest).abs() > SETTLE_ANGLE
            || self.speed.abs() > SETTLE_SPEED
    }

    /// Stop at once, facing front (a roll out of a knockdown).
    pub fn settle(&mut self) {
        *self = Twist::default();
    }

    /// The angle to draw: how far round from facing front.
    pub fn drawn(&self) -> f32 {
        self.angle - self.rest
    }
}

/// The angle to draw `k` (0..1) of the way through the motion `start` makes until it can move again, with a
/// moment of rest at the end: for the pose gallery, which scrubs rather than steps.
pub fn through(start: Twist, k: f32) -> f32 {
    const DT: f32 = 1.0 / 120.0;
    let mut t = start;
    let mut n = 0;
    while t.locked() && n < 1200 {
        t.step(DT);
        n += 1;
    }
    let steps = ((n + 12) as f32 * k.clamp(0.0, 1.0)) as usize;
    let mut t = start;
    for _ in 0..steps {
        t.step(DT);
    }
    t.drawn()
}

/// The share of a character's max HP a hit's damage is, for `Twist::hit`.
pub fn share(dmg: f32, max_hp: f32) -> f32 {
    if max_hp > 0.0 {
        dmg / max_hp
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    /// Step until it can move again; returns the seconds that took and the furthest it turned.
    fn run(t: &mut Twist) -> (f32, f32) {
        let (mut time, mut most) = (0.0, 0.0f32);
        while t.locked() && time < 10.0 {
            t.step(DT);
            time += DT;
            most = most.max(t.drawn().abs());
        }
        (time, most)
    }

    #[test]
    fn a_light_hit_swings_back_and_forth_and_dies_away() {
        let mut t = Twist::default();
        t.hit(0.1, 1.0);
        let mut signs = vec![];
        let mut time = 0.0;
        while t.locked() {
            t.step(DT);
            time += DT;
            let s = t.drawn().signum();
            if signs.last() != Some(&s) {
                signs.push(s);
            }
        }
        assert!(
            signs.len() >= 3,
            "it should swing back past rest, signs {signs:?}"
        );
        assert!(time > 0.2 && time < 1.0, "locked {time} s");
    }

    #[test]
    fn harder_hits_swing_further_and_lock_longer() {
        let (mut a, mut b) = (Twist::default(), Twist::default());
        a.hit(0.05, 1.0);
        b.hit(0.15, 1.0);
        let (ta, ma) = run(&mut a);
        let (tb, mb) = run(&mut b);
        assert!(mb > ma && tb > ta, "{ma} {ta} vs {mb} {tb}");
        assert!(mb < 1.6, "a light hit stays under a spin, {mb}");
    }

    #[test]
    fn a_tiny_hit_barely_turns_and_does_not_lock() {
        // A boss shrugs off a normal hit: 40 damage of 2400 HP.
        let mut t = Twist::default();
        t.hit(share(40.0, 2400.0), 1.0);
        let (time, _) = run(&mut t);
        assert!(time < 0.05, "locked {time} s");
    }

    #[test]
    fn a_heavy_hit_spins_whole_turns_and_ends_facing_front() {
        for (share, turns) in [(0.2, 1.0), (0.45, 2.0), (2.0, MAX_TURNS)] {
            let mut t = Twist::default();
            t.hit(share, -1.0);
            let start = t.angle;
            let mut far = 0.0f32;
            let mut time = 0.0;
            while t.locked() {
                t.step(DT);
                time += DT;
                far = far.max((t.angle - start).abs());
            }
            assert!(far >= turns * TAU, "{share}: turned {far}");
            assert!(far < (turns + 0.25) * TAU, "{share}: turned {far}");
            assert!(t.drawn().abs() <= SETTLE_ANGLE, "ends facing front");
            assert!(time < 2.5, "{share}: locked {time} s");
        }
    }

    #[test]
    fn it_comes_to_rest_at_zero() {
        let mut t = Twist::default();
        t.hit(0.3, 1.0);
        for _ in 0..600 {
            t.step(DT);
        }
        assert_eq!(t, Twist::default());
    }
}
