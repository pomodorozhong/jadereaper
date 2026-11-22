//! Bosses. A boss is an enemy whose kind is marked as a boss, run by its own state machine: stalk, choose a move,
//! wind up with a warning, strike, recover, and fall when its stagger meter fills. Its moves are data (in
//! `content/game.json`) picked by distance and weight among those its phase allows and that are off cooldown.
//!
//! The Glutton (Taotie) is relentless and gets worse when angry: below 60% health, after heavy damage in a short
//! time, or after eating enough of its summoned blots, it enrages for good, winds up 20% faster and chains two or
//! three moves in a row, adding an ink breath and a grab.

use crate::defs::*;
use crate::game::{Act, Enemy, Game, Hazard, Phase};
use crate::geom::{dir_to, dist, Area, Shape};
use crate::moves::*;
use crate::player::PLAYER_R;
use core::f32::consts::TAU;

/// Health share below which the Glutton enrages.
pub const ENRAGE_HP: f32 = 0.6;
/// Damage taken within about `RECENT_TIME` seconds, as a share of maximum health, that enrages it at once.
pub const ENRAGE_BURST: f32 = 0.12;
pub const RECENT_TIME: f32 = 4.0;
/// Wind-ups take this share of their time once enraged, but never less than the minimum warning.
pub const ENRAGED_WINDUP: f32 = 0.8;
/// A full stagger meter knocks the boss down for this long, taking extra damage.
pub const DOWN_TIME: f32 = 2.0;
pub const DOWN_DAMAGE: f32 = 1.5;
/// Rage from eating one blot; at 1 it enrages.
pub const EAT_RAGE: f32 = 0.5;
pub const EAT_HEAL: f32 = 0.04;
/// How fast a stalking boss turns, radians per second.
pub const TURN: f32 = 2.6;
/// Most enemies on the field before a summon is skipped.
pub const SUMMON_CAP: usize = 5;

/// What only a boss keeps.
#[derive(Clone, Copy, Debug)]
pub struct BossState {
    /// The boss enemy's id.
    pub id: u32,
    /// 1, or 2 once enraged (or past its stance change).
    pub phase: u8,
    pub meter: f32,
    /// Time left knocked down.
    pub down: f32,
    pub cooldowns: [f32; N_MOVES],
    /// Moves still to chain straight on (enraged).
    pub chain: u8,
    pub recent: f32,
    pub rage: f32,
    /// The way it faces, radians.
    pub dir: f32,
    /// The player is held by its grab.
    pub holding: bool,
    /// The Dragon: time in the current stance, weak-element damage dealt in it (all of it, and towards the next
    /// knockdown), health thresholds passed (bits), finishers so far, and where its current move lands.
    pub stance_t: f32,
    pub weak_dmg: f32,
    pub weak_meter: f32,
    pub passed: u8,
    pub finishers: u8,
    pub spots: [(f32, f32); 5],
    pub nspots: u8,
}

impl BossState {
    pub fn new(id: u32) -> BossState {
        BossState {
            id,
            phase: 1,
            meter: 0.0,
            down: 0.0,
            cooldowns: [0.0; N_MOVES],
            chain: 0,
            recent: 0.0,
            rage: 0.0,
            dir: 0.0,
            holding: false,
            stance_t: 0.0,
            weak_dmg: 0.0,
            weak_meter: 0.0,
            passed: 0,
            finishers: 0,
            spots: [(0.0, 0.0); 5],
            nspots: 0,
        }
    }

    pub fn enraged(&self) -> bool {
        self.phase >= 2
    }
}

/// The wind-up a move really has: shorter for an enraged boss, but never below the minimum warning.
pub fn windup_for(m: &Move, enraged: bool) -> f32 {
    if !enraged {
        return m.windup;
    }
    let floor = if m.big { MIN_BIG_WARNING } else { MIN_WARNING };
    (m.windup * ENRAGED_WINDUP).max(floor).min(m.windup)
}

fn angle_diff(a: f32, b: f32) -> f32 {
    let mut d = (b - a) % TAU;
    if d > TAU * 0.5 {
        d -= TAU;
    } else if d < -TAU * 0.5 {
        d += TAU;
    }
    d
}

impl Game {
    /// The boss state, if `i` is the boss.
    pub(crate) fn boss_of(&self, i: usize) -> Option<BossState> {
        self.boss.filter(|b| b.id == self.enemies[i].id)
    }

    /// The wind-up time of enemy `i`'s current move.
    pub fn windup_of(&self, e: &Enemy) -> f32 {
        let m = &MOVES[e.mv as usize];
        let enraged = self.boss.is_some_and(|b| b.id == e.id && b.enraged());
        windup_for(m, enraged)
    }

    /// A hit on the boss: it fills the stagger meter instead of interrupting, counts towards enraging, and an eat
    /// in progress is stopped. Returns the damage after the knocked-down bonus.
    pub(crate) fn boss_hit(&mut self, i: usize, dmg: f32, stagger: f32) -> f32 {
        let Some(mut b) = self.boss_of(i) else {
            return dmg;
        };
        let dmg = if b.down > 0.0 { dmg * DOWN_DAMAGE } else { dmg };
        if b.down <= 0.0 {
            b.meter += stagger;
        }
        b.recent += dmg;
        let e = &mut self.enemies[i];
        if e.phase == Phase::Windup && MOVES[e.mv as usize].special == Special::Eat {
            e.phase = Phase::Recover;
            e.pt = 0.0;
        }
        self.boss = Some(b);
        dmg
    }

    /// One step of the boss's state machine.
    pub(crate) fn move_boss(&mut self, i: usize, dt: f32) {
        let Some(mut b) = self.boss_of(i) else { return };
        let (px, py) = (self.player.x, self.player.y);
        let max_hp = KINDS[self.enemies[i].kind as usize].hp;
        let poise = KINDS[self.enemies[i].kind as usize].poise;
        for c in b.cooldowns.iter_mut() {
            *c = (*c - dt).max(0.0);
        }
        b.recent *= (-dt / RECENT_TIME).exp();
        {
            let e = &mut self.enemies[i];
            e.age += dt;
            e.flash = (e.flash - dt * 6.0).max(0.0);
            let decay = (-8.0 * dt).exp();
            e.x += e.kx * dt * 0.3;
            e.y += e.ky * dt * 0.3;
            e.kx *= decay;
            e.ky *= decay;
            e.stun = 0.0;
            e.twist.step(dt);
        }
        self.focus = Some((self.enemies[i].x, self.enemies[i].y));

        // Enrage, for good.
        let hp = self.enemies[i].hp;
        if self.enemies[i].kind == K_GLUTTON
            && b.phase == 1
            && (hp < max_hp * ENRAGE_HP || b.recent > max_hp * ENRAGE_BURST || b.rage >= 1.0)
        {
            b.phase = 2;
            let (x, y) = (self.enemies[i].x, self.enemies[i].y);
            self.shake = self.shake.max(1.0);
            self.event(EV_ENRAGE, x, y, 0.0);
            // A burst of red paper flung up off its body.
            let h = KINDS[self.enemies[i].kind as usize].r;
            self.paper_burst(
                x,
                y,
                crate::paper::Burst {
                    size: (6.0, 11.0),
                    out: (120.0, 320.0),
                    up: (300.0, 520.0),
                    life: (1.2, 1.7),
                    from: h * 0.5,
                    z: h,
                    ..crate::paper::Burst::of(P_SHRED, [0.74, 0.18, 0.14, 1.0], 18)
                },
            );
        }

        // Knocked down by a full meter.
        if b.down > 0.0 {
            b.down -= dt;
            self.boss = Some(b);
            return;
        }
        if b.meter >= poise {
            b.meter = 0.0;
            b.down = DOWN_TIME;
            self.enemies[i].twist.spin(crate::twist::DOWN_TURNS, 1.0);
            b.holding = false;
            self.release_grab();
            let e = &mut self.enemies[i];
            e.phase = Phase::Recover;
            e.pt = 0.0;
            let (x, y) = (e.x, e.y);
            self.event(EV_BOSS_DOWN, x, y, 0.0);
            self.boss = Some(b);
            return;
        }
        b.meter = (b.meter - poise * 0.04 * dt).max(0.0);
        if self.is_dragon(i) {
            self.dragon_stance(i, &mut b, dt);
        }
        // Spinning from a hit (after the knockdowns, which win): it cannot act, and a wind-up or strike in progress is lost. Bosses have so much HP
        // that this takes a very heavy hit.
        if self.enemies[i].twist.locked() {
            let e = &mut self.enemies[i];
            if matches!(e.phase, Phase::Windup | Phase::Active) {
                e.phase = Phase::Recover;
                e.pt = 0.0;
            }
            self.boss = Some(b);
            return;
        }

        let pace = self.enemies[i].status.pace().max(0.5);
        let dt = dt * pace;
        let e = self.enemies[i];
        let m = MOVES[e.mv as usize];
        let d = dist(e.x, e.y, px, py);
        let to_player = (py - e.y).atan2(px - e.x);
        let speed = KINDS[e.kind as usize].speed * if b.enraged() { 1.25 } else { 1.0 };
        let enraged = b.enraged();
        self.enemies[i].pt += dt;
        let pt = self.enemies[i].pt;

        match e.phase {
            Phase::Approach => {
                // Stalk: turn towards the player and close in.
                let turn = angle_diff(b.dir, to_player).clamp(-TURN * dt, TURN * dt);
                b.dir += turn;
                if d > 150.0 {
                    let en = &mut self.enemies[i];
                    en.x += b.dir.cos() * speed * dt;
                    en.y += b.dir.sin() * speed * dt;
                }
                self.enemies[i].think -= dt;
                if self.enemies[i].think <= 0.0 && e.age > 1.0 {
                    if let Some(mv) = self.pick_boss_move(i, &b, d, to_player) {
                        let en = &mut self.enemies[i];
                        en.mv = mv as u8;
                        // An enraged combo runs one hit longer.
                        en.chain =
                            MOVES[mv].chain + if enraged && MOVES[mv].chain > 0 { 1 } else { 0 };
                        b.cooldowns[mv] = MOVES[mv].cooldown;
                        // Turn to face the player as the wind-up starts.
                        if MOVES[mv].special != Special::Behind {
                            b.dir = to_player;
                        }
                        self.start_windup(i);
                        self.dragon_aim(i, &mut b);
                    } else {
                        self.enemies[i].think = 0.2;
                    }
                }
            }
            Phase::Windup => {
                if pt >= windup_for(&m, enraged) {
                    let en = &mut self.enemies[i];
                    en.phase = Phase::Active;
                    en.pt = 0.0;
                    let (x, y) = (en.x, en.y);
                    match m.special {
                        Special::Summon => {
                            for k in 0..2 {
                                if self.enemies.len() < SUMMON_CAP + 1 {
                                    let a = b.dir + if k == 0 { 1.8 } else { -1.8 };
                                    self.add_enemy(
                                        K_BLOT,
                                        x + a.cos() * 120.0,
                                        y + a.sin() * 120.0,
                                    );
                                }
                            }
                        }
                        Special::Eat => {
                            if let Some(j) = self.nearest_blot(i, 260.0) {
                                // Eaten: removed with the dead, without counting as a kill.
                                self.enemies[j].hp = f32::NEG_INFINITY;
                                self.enemies[i].hp =
                                    (self.enemies[i].hp + max_hp * EAT_HEAL).min(max_hp);
                                b.rage += EAT_RAGE;
                                self.event(EV_HEAL, x, y, 0.0);
                            }
                        }
                        _ => {
                            self.dragon_strikes(i, &mut b);
                            if let After::Wave { to, speed, band } = m.after {
                                let a = self.move_area(&self.enemies[i]);
                                self.add_hazard(Hazard {
                                    x: a.x,
                                    y: a.y,
                                    vx: 0.0,
                                    vy: 0.0,
                                    r: m.reach,
                                    band,
                                    to,
                                    grow: speed,
                                    life: (to - m.reach) / speed,
                                    dmg: m.damage * 0.6,
                                    boss: true,
                                    warned: windup_for(&m, enraged),
                                    rehit: 0.0,
                                    cool: 0.0,
                                });
                            }
                            self.event(EV_STRIKE, x, y, e.kind as f32);
                        }
                    }
                }
            }
            Phase::Active => {
                if m.dash > 0.0 {
                    let en = &mut self.enemies[i];
                    en.x += en.dx * m.dash * dt / m.active;
                    en.y += en.dy * m.dash * dt / m.active;
                }
                let attack = !matches!(
                    m.special,
                    Special::Summon | Special::Eat | Special::Finisher
                );
                if attack && !self.enemies[i].struck {
                    let (areas, n) = self.strike_areas(i, pt / m.active);
                    let (ex, ey) = (self.enemies[i].x, self.enemies[i].y);
                    if areas[..n].iter().any(|a| a.hits(px, py, PLAYER_R)) {
                        let warned = windup_for(&m, enraged) + pt;
                        if self.hurt_player(m.damage, true, (ex, ey), warned) {
                            self.enemies[i].struck = true;
                            self.dragon_hit(&m);
                            if m.special == Special::Grab {
                                b.holding = true;
                                self.player.act = Act::Grabbed { t: 0.0, mash: 0 };
                                self.event(EV_GRAB, px, py, 0.0);
                            } else if m.knockdown {
                                let sign = if px >= ex { 1.0 } else { -1.0 };
                                self.player.twist.spin(crate::twist::DOWN_TURNS, sign);
                                self.player.act = Act::Down { t: 0.0 };
                            }
                        }
                    }
                }
                if self.enemies[i].pt >= m.active {
                    let en = &mut self.enemies[i];
                    if en.chain > 0 {
                        en.chain -= 1;
                        b.dir = to_player;
                        self.start_windup(i);
                        self.dragon_aim(i, &mut b);
                    } else if enraged && b.chain > 0 && !b.holding {
                        // Enraged: straight into the next move.
                        b.chain -= 1;
                        en.phase = Phase::Approach;
                        en.pt = 0.0;
                        en.think = 0.0;
                    } else {
                        en.phase = Phase::Recover;
                        en.pt = 0.0;
                    }
                }
            }
            Phase::Recover => {
                if b.holding {
                    // Holding the player in its jaws until they break free.
                    if !matches!(self.player.act, Act::Grabbed { .. }) {
                        b.holding = false;
                    } else {
                        let (x, y) = (e.x + b.dir.cos() * 70.0, e.y + b.dir.sin() * 70.0);
                        self.player.x = x;
                        self.player.y = y;
                        self.enemies[i].pt = 0.0;
                    }
                } else if pt >= m.recover * if enraged { 0.7 } else { 1.0 } {
                    let en = &mut self.enemies[i];
                    en.phase = Phase::Approach;
                    en.pt = 0.0;
                    en.think = if enraged { 0.1 } else { 0.4 };
                    if enraged {
                        b.chain = 1 + (self.rng.next_u32() % 2) as u8;
                    }
                }
            }
        }
        let en = &mut self.enemies[i];
        en.dx = if en.phase == Phase::Approach {
            b.dir.cos()
        } else {
            en.dx
        };
        en.dy = if en.phase == Phase::Approach {
            b.dir.sin()
        } else {
            en.dy
        };
        if b.dir.cos().abs() > 0.15 {
            en.face = b.dir.cos().signum();
        }
        self.boss = Some(b);
    }

    /// Choose a move by weight among those that fit: the phase allows it, it is off cooldown, the player is in its
    /// range, and its special condition holds.
    fn pick_boss_move(&mut self, i: usize, b: &BossState, d: f32, to_player: f32) -> Option<usize> {
        let kind = self.enemies[i].kind as usize;
        let behind = angle_diff(b.dir, to_player).abs() > 2.0;
        let mut pool = [(0usize, 0.0f32); 16];
        let mut n = 0;
        for &mv in KINDS[kind].moves {
            let m = &MOVES[mv];
            let fits = m.phases & (1 << (b.phase - 1)) != 0
                && b.cooldowns[mv] <= 0.0
                && d >= m.range.0
                && d <= m.range.1
                && match m.special {
                    Special::Behind => behind,
                    Special::Summon => self.enemies.len() < 3,
                    Special::Eat => self.nearest_blot(i, 260.0).is_some(),
                    _ => !behind || m.area == crate::skills::AreaKind::Circle,
                }
                && self.stance_allows(m);
            if fits && n < pool.len() {
                pool[n] = (mv, m.weight);
                n += 1;
            }
        }
        let total: f32 = pool[..n].iter().map(|p| p.1).sum();
        if total <= 0.0 {
            return None;
        }
        let mut r = self.rng.f() * total;
        for p in &pool[..n] {
            if r < p.1 {
                return Some(p.0);
            }
            r -= p.1;
        }
        Some(pool[n - 1].0)
    }

    /// True when the move suits the boss's current stance (only the Dragon has stances).
    pub(crate) fn stance_allows(&self, m: &Move) -> bool {
        match m.stance {
            Stance::Any => true,
            Stance::Fire => self.stance == Stance::Fire,
            Stance::Ice => self.stance == Stance::Ice,
        }
    }

    fn nearest_blot(&self, boss: usize, range: f32) -> Option<usize> {
        let (bx, by) = (self.enemies[boss].x, self.enemies[boss].y);
        (0..self.enemies.len())
            .filter(|&j| {
                self.enemies[j].kind == K_BLOT
                    && self.enemies[j].hp > 0.0
                    && dist(bx, by, self.enemies[j].x, self.enemies[j].y) < range
            })
            .min_by(|&a, &c| {
                let da = dist(bx, by, self.enemies[a].x, self.enemies[a].y);
                let dc = dist(bx, by, self.enemies[c].x, self.enemies[c].y);
                da.partial_cmp(&dc).unwrap_or(core::cmp::Ordering::Equal)
            })
    }

    /// The boss move's hit area at `k` (0..1) through its active window. A swept line turns across its arc.
    pub fn boss_area(&self, i: usize, k: f32) -> Area {
        let e = &self.enemies[i];
        let m = &MOVES[e.mv as usize];
        if m.special == Special::Sweep {
            let base = e.dy.atan2(e.dx);
            let a = base - SWEEP_HALF + 2.0 * SWEEP_HALF * k.clamp(0.0, 1.0);
            return Area::new(
                e.x,
                e.y,
                (a.cos(), a.sin()),
                Shape::Line {
                    len: m.reach,
                    half_w: m.width,
                },
            );
        }
        self.strike_area(e)
    }

    /// Hit the boss with an element-free hit of the first skill button's element, for tests.
    pub fn hit_boss_for_test(&mut self, dmg: f32) {
        if let Some(i) = self
            .boss
            .and_then(|b| self.enemies.iter().position(|e| e.id == b.id))
        {
            let element = self.mods[0].element;
            self.hurt_enemy(i, dmg, 10.0, 0.0, 0.0);
            self.apply_element(i, element, dmg, 10.0);
        }
    }

    /// Let go of a grabbed player.
    pub(crate) fn release_grab(&mut self) {
        if let Act::Grabbed { .. } = self.player.act {
            self.player.act = Act::Free;
            self.player.iframe = 0.6;
            if let Some(i) = self
                .boss
                .and_then(|b| self.enemies.iter().position(|e| e.id == b.id))
            {
                let (kx, ky) = dir_to(
                    self.enemies[i].x,
                    self.enemies[i].y,
                    self.player.x,
                    self.player.y,
                );
                self.player.kx = kx * 300.0;
                self.player.ky = ky * 300.0;
            }
        }
    }
}
