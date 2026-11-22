//! The spin a hit puts on a paper character, about its vertical axis. A hit kicks the sheet round: a light one
//! twists it, then a spring swings it back and forth and the swing dies away; a heavy one (a large share of the
//! character's max HP) sends it spinning whole turns, slowing down, and then it wobbles to rest facing front
//! again. While it spins or swings more than a little, the character cannot move or act: this replaces the old
//! fixed hurt, stun and knockdown times. It is part of the simulation, stepped at the fixed rate, so a run still
//! replays the same.

use core::f32::consts::TAU;

/// The most whole turns a hit sends it round.
pub const MAX_TURNS: f32 = 3.0;
/// Turns for a knockdown (a boss's heavy hit on the player, or a boss's full stagger meter) and for a broken
/// poise or a shatter.
pub const DOWN_TURNS: f32 = 2.0;
pub const STUN_TURNS: f32 = 1.0;
/// Still enough to move again: slower than this, in radians per second (and within `Tune::settle` of rest).
const SETTLE_SPEED: f32 = 1.5;

/// The spin's numbers. The game holds one set (`Game::spin_tune`), which the ?debug panel's Hit spin sliders
/// change live (`jr_set_spin`); `src/main.js` `SPIN` mirrors the defaults, in `TUNE_KEYS` order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tune {
    /// A hit worth this share of the character's max HP or more spins it; each further share as big adds a turn.
    pub spin_share: f32,
    /// How far, in radians, a hit just short of a spin swings it; lighter hits swing less, in proportion.
    pub kick: f32,
    /// The swing's spring: its natural rate in radians per second, and its damping ratio, so a swing dies away
    /// by about e^(-damping · rate) per second.
    pub rate: f32,
    pub damping: f32,
    /// How fast a spin slows, in radians per second squared, and how far past its last turn it carries before
    /// the spring takes it back.
    pub decel: f32,
    pub overshoot: f32,
    /// Within this angle of rest (and slow enough) the character can move again: smaller locks it longer.
    pub settle: f32,
}

impl Tune {
    pub const DEFAULT: Tune = Tune {
        spin_share: 0.2,
        kick: 1.45,
        rate: 16.0,
        damping: 0.2,
        decel: 38.0,
        overshoot: 0.5,
        settle: 0.15,
    };

    /// A number by its index in `TUNE_KEYS`, for the ?debug setter.
    pub fn field_mut(&mut self, i: usize) -> Option<&mut f32> {
        Some(match i {
            0 => &mut self.spin_share,
            1 => &mut self.kick,
            2 => &mut self.rate,
            3 => &mut self.damping,
            4 => &mut self.decel,
            5 => &mut self.overshoot,
            6 => &mut self.settle,
            _ => return None,
        })
    }
}

impl Default for Tune {
    fn default() -> Tune {
        Tune::DEFAULT
    }
}

/// `Tune`'s fields in setter order, as `SPIN` in `src/main.js` names them.
pub const TUNE_KEYS: [&str; 7] = [
    "spin_share",
    "kick",
    "rate",
    "damping",
    "decel",
    "overshoot",
    "settle",
];

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
    pub fn hit(&mut self, share: f32, sign: f32, tune: &Tune) {
        let sign = if sign < 0.0 { -1.0 } else { 1.0 };
        let at = tune.spin_share.max(1e-3);
        if share >= at {
            let turns = (1.0 + ((share - at) / at).floor()).min(MAX_TURNS);
            self.spin(turns, sign, tune);
        } else if !self.spinning {
            // An undamped swing of `kick` radians needs this speed; the damping takes a little off it.
            self.speed += sign * tune.kick * tune.rate * share.max(0.0) / at;
        }
    }

    /// Send it spinning `turns` whole turns in `sign`'s direction, then let it wobble to rest.
    pub fn spin(&mut self, turns: f32, sign: f32, tune: &Tune) {
        let sign = if sign < 0.0 { -1.0 } else { 1.0 };
        // Rest at the next whole turn past `turns` from here, so it always ends facing front.
        self.rest = ((self.angle + sign * turns * TAU) / TAU).round() * TAU;
        let travel = (self.rest - self.angle).abs() + tune.overshoot.max(0.0);
        self.speed = sign * (2.0 * tune.decel.max(1e-3) * travel).sqrt();
        self.spinning = true;
    }

    /// Step by `dt`.
    pub fn step(&mut self, dt: f32, tune: &Tune) {
        if self.spinning {
            let s = self.speed.signum();
            let next = self.speed - s * tune.decel.max(1e-3) * dt;
            if next.signum() != s {
                self.spinning = false;
                self.speed = 0.0;
            } else {
                self.speed = next;
                self.angle += self.speed * dt;
            }
            return;
        }
        let pull = -tune.rate * tune.rate * (self.angle - self.rest)
            - 2.0 * tune.damping * tune.rate * self.speed;
        self.speed += pull * dt;
        self.angle += self.speed * dt;
        if !self.locked(tune) && (self.angle - self.rest).abs() < 0.02 && self.speed.abs() < 0.2 {
            // At rest: snap and bring the angle back near zero.
            self.angle = 0.0;
            self.rest = 0.0;
            self.speed = 0.0;
        }
    }

    /// True while it spins or swings too much for the character to move.
    pub fn locked(&self, tune: &Tune) -> bool {
        self.spinning
            || (self.angle - self.rest).abs() > tune.settle
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

/// How long a dead character spins before it is gone, in seconds.
pub const DEATH_TIME: f32 = 1.0;

/// A death spin: the same motion as a hit's spin (fast at first, slowing evenly, ending facing front), but always
/// `DEATH_TIME` long, so the overkill sets how fast it spins rather than for how long. Looks only.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeathSpin {
    /// The drawn angle when it died, and the signed angle it turns through to end facing front.
    pub from: f32,
    pub turn: f32,
}

impl DeathSpin {
    /// For a character drawn at `from` when it died, `overkill` damage past its last HP of `max_hp`, pushed in
    /// `sign`'s direction: one turn, and one more for each further `spin_share` of max HP, the hit's rule.
    pub fn new(from: f32, overkill: f32, max_hp: f32, sign: f32, tune: &Tune) -> DeathSpin {
        let sign = if sign < 0.0 { -1.0 } else { 1.0 };
        let at = tune.spin_share.max(1e-3);
        let turns = 1.0 + (share(overkill.max(0.0), max_hp) / at).floor().min(8.0);
        let end = ((from + sign * turns * TAU) / TAU).round() * TAU;
        DeathSpin {
            from,
            turn: end - from,
        }
    }

    /// The angle to draw `t` seconds after it died.
    pub fn at(&self, t: f32) -> f32 {
        let k = (t / DEATH_TIME).clamp(0.0, 1.0);
        self.from + self.turn * (1.0 - (1.0 - k) * (1.0 - k))
    }
}

/// The angle to draw `k` (0..1) of the way through the motion `start` makes until it can move again, with a
/// moment of rest at the end: for the pose gallery, which scrubs rather than steps.
pub fn through(start: Twist, k: f32, tune: &Tune) -> f32 {
    const DT: f32 = 1.0 / 120.0;
    let mut t = start;
    let mut n = 0;
    while t.locked(tune) && n < 1200 {
        t.step(DT, tune);
        n += 1;
    }
    let steps = ((n + 12) as f32 * k.clamp(0.0, 1.0)) as usize;
    let mut t = start;
    for _ in 0..steps {
        t.step(DT, tune);
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
    const T: Tune = Tune::DEFAULT;

    /// Step until it can move again; returns the seconds that took and the furthest it turned.
    fn run(t: &mut Twist) -> (f32, f32) {
        let (mut time, mut most) = (0.0, 0.0f32);
        while t.locked(&T) && time < 10.0 {
            t.step(DT, &T);
            time += DT;
            most = most.max(t.drawn().abs());
        }
        (time, most)
    }

    #[test]
    fn a_light_hit_swings_back_and_forth_and_dies_away() {
        let mut t = Twist::default();
        t.hit(0.1, 1.0, &T);
        let mut signs = vec![];
        let mut time = 0.0;
        while t.locked(&T) {
            t.step(DT, &T);
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
        a.hit(0.05, 1.0, &T);
        b.hit(0.15, 1.0, &T);
        let (ta, ma) = run(&mut a);
        let (tb, mb) = run(&mut b);
        assert!(mb > ma && tb > ta, "{ma} {ta} vs {mb} {tb}");
        assert!(mb < 1.6, "a light hit stays under a spin, {mb}");
    }

    #[test]
    fn a_tiny_hit_barely_turns_and_does_not_lock() {
        // A boss shrugs off a normal hit: 40 damage of 2400 HP.
        let mut t = Twist::default();
        t.hit(share(40.0, 2400.0), 1.0, &T);
        let (time, _) = run(&mut t);
        assert!(time < 0.05, "locked {time} s");
    }

    #[test]
    fn a_heavy_hit_spins_whole_turns_and_ends_facing_front() {
        for (share, turns) in [(0.2, 1.0), (0.45, 2.0), (2.0, MAX_TURNS)] {
            let mut t = Twist::default();
            t.hit(share, -1.0, &T);
            let start = t.angle;
            let mut far = 0.0f32;
            let mut time = 0.0;
            while t.locked(&T) {
                t.step(DT, &T);
                time += DT;
                far = far.max((t.angle - start).abs());
            }
            assert!(far >= turns * TAU, "{share}: turned {far}");
            assert!(far < (turns + 0.25) * TAU, "{share}: turned {far}");
            assert!(t.drawn().abs() <= T.settle, "ends facing front");
            assert!(time < 2.5, "{share}: locked {time} s");
        }
    }

    #[test]
    fn the_tuning_changes_the_spin() {
        // A lower spin share turns a light hit into a spin; less damping keeps a swing going longer.
        let mut light = Twist::default();
        light.hit(
            0.1,
            1.0,
            &Tune {
                spin_share: 0.08,
                ..T
            },
        );
        assert!(light.spinning);
        let (mut a, mut b) = (Twist::default(), Twist::default());
        let loose = Tune { damping: 0.08, ..T };
        a.hit(0.1, 1.0, &T);
        b.hit(0.1, 1.0, &loose);
        let (ta, mut tb) = (run(&mut a).0, 0.0);
        while b.locked(&loose) && tb < 10.0 {
            b.step(DT, &loose);
            tb += DT;
        }
        assert!(tb > ta * 1.5, "{tb} vs {ta}");
        // Every key has a field, in order.
        let mut t = T;
        for i in 0..TUNE_KEYS.len() {
            *t.field_mut(i).unwrap() = i as f32 + 100.0;
        }
        assert_eq!(t.spin_share, 100.0);
        assert_eq!(t.settle, 106.0);
        assert!(t.field_mut(TUNE_KEYS.len()).is_none());
    }

    #[test]
    fn a_death_spin_takes_a_second_and_spins_faster_for_more_overkill() {
        let slow = DeathSpin::new(0.0, 0.0, 30.0, 1.0, &T);
        let fast = DeathSpin::new(0.0, 30.0, 30.0, 1.0, &T);
        // One turn for a bare kill; a whole max HP of overkill adds five more at the default spin share.
        assert!((slow.turn - TAU).abs() < 1e-4);
        assert!((fast.turn - 6.0 * TAU).abs() < 1e-4);
        // Both start where they were, end facing front a second later, and slow down evenly.
        for d in [slow, fast] {
            assert_eq!(d.at(0.0), 0.0);
            assert!((d.at(DEATH_TIME) / TAU).fract().abs() < 1e-4);
            assert!(d.at(0.25) - d.at(0.0) > d.at(1.0) - d.at(0.75));
        }
        assert!(fast.at(0.1) > slow.at(0.1));
        // Mid-swing from a hit, it carries on from that angle and still ends facing front.
        let mid = DeathSpin::new(0.7, 5.0, 30.0, -1.0, &T);
        assert_eq!(mid.at(0.0), 0.7);
        assert!(mid.turn < 0.0 && ((mid.from + mid.turn) / TAU).fract().abs() < 1e-4);
    }

    #[test]
    fn it_comes_to_rest_at_zero() {
        let mut t = Twist::default();
        t.hit(0.3, 1.0, &T);
        for _ in 0..600 {
            t.step(DT, &T);
        }
        assert_eq!(t, Twist::default());
    }
}
