//! The player: input, movement, the roll, skills and taking hits.

use crate::defs::*;
use crate::game::{Fx, Game, Shot};
use crate::geom::{dir_to, Area, Shape};
use crate::skills::*;

/// One step's input: movement, aim and buttons.
#[derive(Clone, Copy, Default)]
pub struct Input {
    /// Movement, each axis in -1..1.
    pub mx: f32,
    pub my: f32,
    /// Aim, from the player towards the aim point in world units. Zero keeps the last aim.
    pub ax: f32,
    pub ay: f32,
    /// Held buttons, `B_*` bits.
    pub buttons: u32,
}

impl Input {
    pub fn moving(mx: f32, my: f32) -> Input {
        Input { mx, my, ..Input::default() }
    }
}

/// What the player's body is doing.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Act {
    Free,
    /// Rolling in a direction; `t` counts up from 0 through the move and then the recovery.
    Roll { t: f32, dx: f32, dy: f32 },
    /// Using the skill on a button. `t` counts up from the start of the startup; `step` is the combo step.
    Skill { slot: usize, skill: usize, t: f32, dx: f32, dy: f32, step: u8, fired: bool },
    /// Knocked off balance by a hit.
    Hurt { t: f32 },
}

/// How long a hit knocks the player off balance.
pub const HURT_TIME: f32 = 0.3;
/// Invulnerable for this long after taking a hit.
pub const HURT_IFRAME: f32 = 0.5;
pub const WALK_SPEED: f32 = 190.0;

pub struct Player {
    pub x: f32,
    pub y: f32,
    /// Knockback velocity.
    pub kx: f32,
    pub ky: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub xp: f32,
    pub xp_next: f32,
    pub level: u32,
    /// Unit aim direction.
    pub aim: (f32, f32),
    /// Aim point offset from the player, as last sent.
    pub aim_at: (f32, f32),
    pub face: f32,
    pub walk: f32,
    pub moving: bool,
    pub iframe: f32,
    pub act: Act,
    /// Time left on each button's buffered press (skill 1, 2, 3, roll).
    pub buffer: [f32; N_BUTTONS],
    /// Enemy ids already hit by the current skill use, so each is hit once.
    pub hits: Vec<u32>,
}

impl Player {
    pub fn new(xp_next: f32) -> Player {
        Player {
            x: 0.0,
            y: 0.0,
            kx: 0.0,
            ky: 0.0,
            hp: 100.0,
            max_hp: 100.0,
            xp: 0.0,
            xp_next,
            level: 1,
            aim: (1.0, 0.0),
            aim_at: (100.0, 0.0),
            face: 1.0,
            walk: 0.0,
            moving: false,
            iframe: 0.0,
            act: Act::Free,
            buffer: [0.0; N_BUTTONS],
            hits: Vec::with_capacity(64),
        }
    }

    /// True while hits do no damage: after being hurt, and early in a roll.
    pub fn invulnerable(&self) -> bool {
        self.iframe > 0.0 || matches!(self.act, Act::Roll { t, .. } if t < ROLL_INVULN)
    }

    /// True while a hyper armor skill is in its active window.
    pub fn armored(&self) -> bool {
        match self.act {
            Act::Skill { skill, t, .. } => {
                let d = &SKILLS[skill];
                d.hyper && t >= d.startup && t < d.startup + d.active
            }
            _ => false,
        }
    }
}

impl Game {
    /// Read this step's input: buffer new presses, aim, then move, roll or use a skill.
    pub(crate) fn move_player(&mut self, dt: f32) {
        let inp = self.input;
        let pressed = inp.buttons & !self.prev_buttons;
        self.prev_buttons = inp.buttons;
        let p = &mut self.player;
        for (b, t) in p.buffer.iter_mut().enumerate() {
            *t = if pressed & (1 << b) != 0 { BUFFER } else { (*t - dt).max(0.0) };
        }
        let (ax, ay) = (inp.ax, inp.ay);
        let al = (ax * ax + ay * ay).sqrt();
        if al.is_finite() && al > 1.0 {
            p.aim = (ax / al, ay / al);
            p.aim_at = (ax, ay);
        }

        let (mut mx, mut my) = (inp.mx, inp.my);
        if !mx.is_finite() || !my.is_finite() {
            mx = 0.0;
            my = 0.0;
        }
        let mut len = (mx * mx + my * my).sqrt();
        if len > 1.0 {
            mx /= len;
            my /= len;
            len = 1.0;
        }
        let speed = WALK_SPEED * (1.0 + 0.08 * self.lvl(U_SWIFT));
        let regen = 0.4 * self.lvl(U_MENDING);

        self.start_actions(len, mx, my);

        let mut walk = 0.0;
        match self.player.act {
            Act::Free => walk = 1.0,
            Act::Roll { t, dx, dy } => {
                let p = &mut self.player;
                if t < ROLL_TIME {
                    // Fast at first, easing out, covering ROLL_DIST in all.
                    let v = |t: f32| 2.0 * t / ROLL_TIME - (t / ROLL_TIME) * (t / ROLL_TIME);
                    let d = ROLL_DIST * (v((t + dt).min(ROLL_TIME)) - v(t));
                    p.x += dx * d;
                    p.y += dy * d;
                }
                let t = t + dt;
                p.act = if t >= ROLL_TIME + ROLL_RECOVER - 1e-4 { Act::Free } else { Act::Roll { t, dx, dy } };
            }
            Act::Skill { .. } => walk = self.run_skill(dt),
            Act::Hurt { t } => {
                let t = t + dt;
                self.player.act = if t >= HURT_TIME { Act::Free } else { Act::Hurt { t } };
            }
        }

        let p = &mut self.player;
        p.moving = walk > 0.0 && len > 0.15;
        if p.moving {
            p.x += mx * speed * walk * dt;
            p.y += my * speed * walk * dt;
            p.walk += dt;
        }
        p.x += p.kx * dt;
        p.y += p.ky * dt;
        let decay = (-10.0 * dt).exp();
        p.kx *= decay;
        p.ky *= decay;
        let fx = match p.act {
            Act::Roll { dx, .. } | Act::Skill { dx, .. } => dx,
            _ => p.aim.0,
        };
        if fx.abs() > 0.1 {
            p.face = fx.signum();
        }
        p.iframe = (p.iframe - dt).max(0.0);
        p.hp = (p.hp + regen * dt).min(p.max_hp);
    }

    /// Start a buffered roll or skill when the body allows it.
    fn start_actions(&mut self, len: f32, mx: f32, my: f32) {
        let p = &self.player;
        let roll_dir = if len > 0.15 { (mx / len, my / len) } else { p.aim };
        match p.act {
            Act::Free => {
                if p.buffer[3] > 0.0 {
                    self.start_roll(roll_dir);
                } else if let Some(slot) = (0..3).find(|&s| p.buffer[s] > 0.0) {
                    self.start_skill(slot, 0);
                }
            }
            Act::Skill { slot, skill, t, step, .. } => {
                let d = &SKILLS[skill];
                if p.buffer[3] > 0.0 && t >= d.cancel_at() {
                    // The roll cuts the recovery short, only past the cancel point.
                    self.start_roll(roll_dir);
                } else if t >= d.startup + d.active && p.buffer[slot] > 0.0 && step + 1 < d.combo {
                    self.start_skill(slot, step + 1);
                }
            }
            _ => {}
        }
    }

    fn start_roll(&mut self, (dx, dy): (f32, f32)) {
        let p = &mut self.player;
        p.buffer[3] = 0.0;
        p.act = Act::Roll { t: 0.0, dx, dy };
    }

    fn start_skill(&mut self, slot: usize, step: u8) {
        let skill = self.loadout[slot];
        let p = &mut self.player;
        p.buffer[slot] = 0.0;
        p.hits.clear();
        p.act = Act::Skill { slot, skill, t: 0.0, dx: p.aim.0, dy: p.aim.1, step, fired: false };
    }

    /// The hit area of a skill use from where the player stands.
    pub fn skill_area(&self, skill: usize, step: u8, dir: (f32, f32)) -> Area {
        let d = &SKILLS[skill];
        let finish = d.combo > 1 && step + 1 == d.combo;
        let reach = d.reach * if finish { COMBO_FINISH_REACH } else { 1.0 };
        let shape = match d.area {
            AreaKind::Circle => Shape::Circle { r: reach },
            AreaKind::Arc => Shape::Arc { r: reach, half: d.width },
            AreaKind::Line => Shape::Line { len: reach, half_w: d.width },
            AreaKind::Ring => Shape::Ring { inner: (reach - d.width).max(0.0), outer: reach },
        };
        Area::new(self.player.x, self.player.y, dir, shape)
    }

    pub(crate) fn skill_damage(&self, skill: usize, step: u8) -> f32 {
        let d = &SKILLS[skill];
        let finish = d.combo > 1 && step + 1 == d.combo;
        d.damage * (1.0 + 0.12 * self.lvl(U_MIGHT)) * if finish { COMBO_FINISH_DAMAGE } else { 1.0 }
    }

    /// One step of a skill. Returns how fast the player may walk meanwhile, as a share of walking speed.
    fn run_skill(&mut self, dt: f32) -> f32 {
        let Act::Skill { slot, skill, t, dx, dy, step, mut fired } = self.player.act else { return 1.0 };
        let d = SKILLS[skill];
        let (a0, a1) = (d.startup, d.startup + d.active);
        if t + dt > a0 && t < a1 {
            if d.dash > 0.0 {
                let share = ((t + dt).min(a1) - t.max(a0)) / d.active.max(1e-4);
                self.player.x += dx * d.dash * share;
                self.player.y += dy * d.dash * share;
            }
            if !fired {
                fired = true;
                self.skill_fires(skill, step, (dx, dy));
            }
            if matches!(d.kind, SkillKind::Melee | SkillKind::Nova) {
                let area = self.skill_area(skill, step, (dx, dy));
                let dmg = self.skill_damage(skill, step);
                self.hit_area(area, dmg, d.stagger, d.knockback);
            }
        }
        let t = t + dt;
        self.player.act = if t >= d.total() - 1e-4 {
            Act::Free
        } else {
            Act::Skill { slot, skill, t, dx, dy, step, fired }
        };
        if t < a1 {
            d.walk
        } else {
            0.5
        }
    }

    /// The moment a skill comes out: its sound, its effect, and any projectile.
    fn skill_fires(&mut self, skill: usize, step: u8, dir: (f32, f32)) {
        let d = SKILLS[skill];
        let (px, py) = (self.player.x, self.player.y);
        let area = self.skill_area(skill, step, dir);
        let jade = [0.25, 0.62, 0.48, 0.9];
        match d.kind {
            SkillKind::Melee => {
                self.add_fx(Fx {
                    rot: dir.1.atan2(dir.0),
                    vr: 5.0 * self.player.face,
                    grow: 60.0,
                    seed: 0.3,
                    ..Fx::at(px, py - 24.0, P_SLASH, 0.22, area.reach(), area.reach(), jade)
                });
                self.event(EV_SLASH, px, py, 0.0);
            }
            SkillKind::Nova => {
                let life = 0.3;
                self.add_fx(Fx { grow: (area.reach() - 20.0) / life, seed: 0.7, ..Fx::at(px, py - 10.0, P_RING, life, 20.0, 20.0, jade) });
                self.event(EV_PULSE, px, py, 0.0);
            }
            SkillKind::Shot => {
                if self.shots.len() < crate::game::MAX_SHOTS {
                    let speed = 560.0;
                    let dmg = self.skill_damage(skill, step);
                    self.shots.push(Shot {
                        x: px,
                        y: py,
                        vx: dir.0 * speed,
                        vy: dir.1 * speed,
                        life: d.reach / speed,
                        dmg,
                        stagger: d.stagger,
                        knockback: d.knockback,
                        r: d.width,
                        pierce: 1,
                        last: 0,
                    });
                }
                self.event(EV_SHOOT, px, py, 0.0);
            }
        }
    }

    /// Hit every enemy in the area that this skill use has not hit yet.
    pub(crate) fn hit_area(&mut self, area: Area, dmg: f32, stagger: f32, knockback: f32) {
        for i in 0..self.enemies.len() {
            let e = &self.enemies[i];
            if e.hp <= 0.0 || self.player.hits.contains(&e.id) {
                continue;
            }
            if area.hits(e.x, e.y, KINDS[e.kind as usize].r) {
                let id = e.id;
                let (kx, ky) = dir_to(area.x, area.y, e.x, e.y);
                self.player.hits.push(id);
                self.hurt_enemy(i, dmg, stagger, kx * knockback, ky * knockback);
            }
        }
    }

    /// Hurt the player. A boss hit always interrupts; a normal hit does not interrupt a hyper armor skill in its
    /// active window, but its damage still lands. Returns true if the hit landed.
    pub fn hurt_player(&mut self, dmg: f32, boss: bool, from: (f32, f32)) -> bool {
        if self.player.invulnerable() || self.over {
            return false;
        }
        let (px, py) = (self.player.x, self.player.y);
        let (kx, ky) = dir_to(from.0, from.1, px, py);
        let armored = self.player.armored() && !boss;
        let p = &mut self.player;
        p.hp -= dmg;
        p.iframe = HURT_IFRAME;
        if !armored {
            p.act = Act::Hurt { t: 0.0 };
            p.kx = kx * 260.0;
            p.ky = ky * 260.0;
        }
        self.shake = 1.0;
        self.event(EV_HURT, px, py, dmg);
        for _ in 0..6 {
            let a = self.rng.range(0.0, core::f32::consts::TAU);
            let s = self.rng.range(80.0, 220.0);
            let seed = self.rng.f();
            self.add_fx(Fx {
                vx: a.cos() * s,
                vy: a.sin() * s - 60.0,
                rot: a,
                vr: 9.0,
                seed,
                ..Fx::at(px, py - 30.0, P_SCRAP, 0.5, 4.0, 4.0, [0.20, 0.42, 0.36, 1.0])
            });
        }
        if self.player.hp <= 0.0 {
            self.player.hp = 0.0;
            self.over = true;
            self.event(EV_DEATH, px, py, 0.0);
        }
        true
    }
}
