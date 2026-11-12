//! Game state, simulation step and instance building.
//!
//! All simulation happens on the ground plane (x right, y down). Characters
//! are drawn standing up from their ground point, so draw code offsets parts
//! upwards. Animation is done by posing rigid paper parts (tilt, hop, flip)
//! on a low, stepped frame rate instead of playing sprite frames.

use crate::defs::*;
use core::f32::consts::TAU;

const MAX_ENEMIES: usize = 500;
const MAX_GEMS: usize = 700;
const MAX_FX: usize = 500;
const MAX_SHOTS: usize = 200;
const CELL: f32 = 96.0;
const BUCKETS: usize = 4096;

#[derive(Clone, Copy)]
pub struct Enemy {
    pub x: f32,
    pub y: f32,
    /// Knockback velocity.
    pub kx: f32,
    pub ky: f32,
    pub hp: f32,
    pub kind: u8,
    pub flash: f32,
    pub seed: f32,
    pub bead_cd: f32,
    pub id: u32,
    pub face: f32,
}

pub struct Shot {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    dmg: f32,
    pierce: i32,
    last: u32,
}

pub struct Gem {
    x: f32,
    y: f32,
    value: f32,
    seed: f32,
    pull: bool,
    heart: bool,
}

pub struct Fx {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max: f32,
    part: f32,
    hw: f32,
    hh: f32,
    rot: f32,
    vr: f32,
    grow: f32,
    color: [f32; 4],
    seed: f32,
    /// Drawn above characters instead of on the ground.
    top: bool,
}

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub xp: f32,
    pub xp_next: f32,
    pub level: u32,
    pub aim: (f32, f32),
    pub face: f32,
    pub walk: f32,
    pub moving: bool,
    pub iframe: f32,
    pub attack: f32,
}

pub struct Game {
    pub rng: Rng,
    pub running: bool,
    pub choosing: bool,
    pub over: bool,
    /// Run time in seconds. Stops while choosing an upgrade.
    pub time: f32,
    /// Animation clock. Keeps going on the title and upgrade screens.
    pub anim: f32,
    /// Half extents of the visible area in world units.
    pub view_w: f32,
    pub view_h: f32,
    pub input: (f32, f32),
    pub player: Player,
    pub enemies: Vec<Enemy>,
    pub shots: Vec<Shot>,
    pub gems: Vec<Gem>,
    pub fx: Vec<Fx>,
    pub up: [u8; N_UPGRADES],
    pub choices: [i32; 3],
    pub pending: u32,
    pub kills: u32,
    pub shake: f32,
    pub cam: (f32, f32),
    spawn_acc: f32,
    next_elite: f32,
    next_id: u32,
    t_scythe: f32,
    t_talisman: f32,
    t_pulse: f32,
    pub inst: Vec<Inst>,
    pub events: Vec<[f32; 4]>,
    pub state: [f32; STATE_FLOATS],
    // Scratch buffers for the spatial grid and draw sorting.
    cell_start: Vec<u32>,
    cell_cursor: Vec<u32>,
    cell_items: Vec<u16>,
    push: Vec<(f32, f32)>,
    order: Vec<(f32, u16)>,
}

fn xp_for(level: u32) -> f32 {
    let l = level as f32;
    4.0 + l * 3.5 + l * l * 0.4
}

fn cell_key(cx: i32, cy: i32) -> usize {
    (((cy & 63) as usize) << 6) | ((cx & 63) as usize)
}

impl Game {
    pub fn new(seed: u32) -> Game {
        let mut up = [0u8; N_UPGRADES];
        up[U_SCYTHE] = 1;
        Game {
            rng: Rng(seed | 1),
            running: false,
            choosing: false,
            over: false,
            time: 0.0,
            anim: 0.0,
            view_w: 640.0,
            view_h: 360.0,
            input: (0.0, 0.0),
            player: Player {
                x: 0.0,
                y: 0.0,
                hp: 100.0,
                max_hp: 100.0,
                xp: 0.0,
                xp_next: xp_for(1),
                level: 1,
                aim: (1.0, 0.0),
                face: 1.0,
                walk: 0.0,
                moving: false,
                iframe: 0.0,
                attack: 0.0,
            },
            enemies: Vec::with_capacity(MAX_ENEMIES),
            shots: Vec::with_capacity(MAX_SHOTS),
            gems: Vec::with_capacity(MAX_GEMS),
            fx: Vec::with_capacity(MAX_FX),
            up,
            choices: [-1; 3],
            pending: 0,
            kills: 0,
            shake: 0.0,
            cam: (0.0, -30.0),
            spawn_acc: 0.0,
            next_elite: 90.0,
            next_id: 1,
            t_scythe: 0.6,
            t_talisman: 0.5,
            t_pulse: 2.0,
            inst: Vec::with_capacity(MAX_INST),
            events: Vec::with_capacity(256),
            state: [0.0; STATE_FLOATS],
            cell_start: vec![0; BUCKETS + 1],
            cell_cursor: vec![0; BUCKETS + 1],
            cell_items: Vec::with_capacity(MAX_ENEMIES),
            push: Vec::with_capacity(MAX_ENEMIES),
            order: Vec::with_capacity(MAX_ENEMIES + 1),
        }
    }

    pub fn set_view(&mut self, half_w: f32, half_h: f32) {
        self.view_w = half_w.max(100.0);
        self.view_h = half_h.max(100.0);
    }

    pub fn update(&mut self, dt: f32) {
        let dt = if dt.is_finite() { dt.clamp(0.0, 0.05) } else { 0.0 };
        self.events.clear();
        self.anim += dt;
        if self.running && !self.choosing && !self.over {
            self.step(dt);
        }
        // Camera follows the player, with a decaying shake.
        self.shake = (self.shake - dt * 3.0).max(0.0);
        let s = self.shake * self.shake * 9.0;
        let q = (self.anim * 30.0).floor();
        self.cam = (
            self.player.x + s * (q * 12.9898).sin(),
            self.player.y - 30.0 + s * (q * 78.233).cos(),
        );
        self.build();
        self.write_state();
    }

    fn event(&mut self, kind: f32, x: f32, y: f32, v: f32) {
        if self.events.len() < 256 {
            self.events.push([kind, x, y, v]);
        }
    }

    fn lvl(&self, id: usize) -> f32 {
        self.up[id] as f32
    }

    // ---------------------------------------------------------------- step

    fn step(&mut self, dt: f32) {
        self.time += dt;

        // Player movement.
        let (mut mx, mut my) = self.input;
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
        let speed = 190.0 * (1.0 + 0.08 * self.lvl(U_SWIFT));
        let regen = 0.4 * self.lvl(U_MENDING);
        let p = &mut self.player;
        p.moving = len > 0.15;
        if p.moving {
            p.x += mx * speed * dt;
            p.y += my * speed * dt;
            p.aim = (mx / len, my / len);
            p.walk += dt;
            if mx.abs() > 0.1 {
                p.face = mx.signum();
            }
        }
        p.iframe = (p.iframe - dt).max(0.0);
        p.attack = (p.attack - dt).max(0.0);
        p.hp = (p.hp + regen * dt).min(p.max_hp);

        self.weapons(dt);
        self.spawn(dt);
        self.move_enemies(dt);
        self.separate();
        self.move_shots(dt);
        self.reap();
        self.move_gems(dt);
        self.move_fx(dt);

        // Level ups.
        while self.player.xp >= self.player.xp_next {
            self.player.xp -= self.player.xp_next;
            self.player.level += 1;
            self.player.xp_next = xp_for(self.player.level);
            self.pending += 1;
        }
        if self.pending > 0 && !self.choosing && !self.over {
            self.offer();
        }
    }

    fn hurt_enemy(&mut self, i: usize, dmg: f32, kx: f32, ky: f32) {
        let e = &mut self.enemies[i];
        e.hp -= dmg;
        e.flash = 1.0;
        let heavy = if e.kind >= K_BRUTE { 0.35 } else { 1.0 };
        e.kx += kx * heavy;
        e.ky += ky * heavy;
        let (x, y) = (e.x, e.y);
        self.event(EV_HIT, x, y, dmg);
    }

    fn weapons(&mut self, dt: f32) {
        let might = 1.0 + 0.12 * self.lvl(U_MIGHT);
        let haste = 0.92f32.powf(self.lvl(U_HASTE));
        let (px, py) = (self.player.x, self.player.y);

        // Scythe: a sweeping arc in the aim direction.
        self.t_scythe -= dt;
        if self.t_scythe <= 0.0 {
            let lvl = self.lvl(U_SCYTHE);
            self.t_scythe = 0.9 * haste;
            let (ax, ay) = self.player.aim;
            let reach = 105.0 + 14.0 * lvl;
            let dmg = (14.0 + 6.0 * lvl) * might;
            let cosw = if lvl >= 5.0 { -2.0 } else { -0.15 };
            for i in 0..self.enemies.len() {
                let e = &self.enemies[i];
                let (dx, dy) = (e.x - px, e.y - py);
                let d = (dx * dx + dy * dy).sqrt().max(0.001);
                let r = KINDS[e.kind as usize].r;
                if d < reach + r && (d < 30.0 || (dx * ax + dy * ay) / d > cosw) {
                    self.hurt_enemy(i, dmg, dx / d * 240.0, dy / d * 240.0);
                }
            }
            self.player.attack = 0.25;
            let rot = ay.atan2(ax);
            let arcs = if lvl >= 5.0 { 2 } else { 1 };
            for k in 0..arcs {
                self.add_fx(Fx {
                    x: px,
                    y: py - 24.0,
                    vx: 0.0,
                    vy: 0.0,
                    life: 0.22,
                    max: 0.22,
                    part: P_SLASH,
                    hw: reach,
                    hh: reach,
                    rot: rot + k as f32 * TAU * 0.5,
                    vr: 5.0,
                    grow: 60.0,
                    color: [0.25, 0.62, 0.48, 0.9],
                    seed: 0.3,
                    top: true,
                });
            }
            self.event(EV_SLASH, px, py, 0.0);
        }

        // Talisman: flies at the nearest enemies.
        if self.up[U_TALISMAN] > 0 {
            self.t_talisman -= dt;
            if self.t_talisman <= 0.0 {
                let lvl = self.lvl(U_TALISMAN);
                self.t_talisman = 0.95 * haste;
                let count = 1 + (lvl as usize) / 2;
                let mut best = [(f32::MAX, 0usize); 4];
                for (i, e) in self.enemies.iter().enumerate() {
                    let d = (e.x - px).powi(2) + (e.y - py).powi(2);
                    for s in 0..count {
                        if d < best[s].0 {
                            for t in (s + 1..count).rev() {
                                best[t] = best[t - 1];
                            }
                            best[s] = (d, i);
                            break;
                        }
                    }
                }
                let mut fired = false;
                for s in 0..count {
                    if best[s].0 > 520.0 * 520.0 || self.shots.len() >= MAX_SHOTS {
                        continue;
                    }
                    let e = &self.enemies[best[s].1];
                    let (dx, dy) = (e.x - px, e.y - py);
                    let d = (dx * dx + dy * dy).sqrt().max(0.001);
                    self.shots.push(Shot {
                        x: px,
                        y: py,
                        vx: dx / d * 540.0,
                        vy: dy / d * 540.0,
                        life: 1.3,
                        dmg: (8.0 + 4.0 * lvl) * might,
                        pierce: 1 + (lvl as i32) / 3,
                        last: 0,
                    });
                    fired = true;
                }
                if fired {
                    self.event(EV_SHOOT, px, py, 0.0);
                } else {
                    self.t_talisman = 0.15;
                }
            }
        }

        // Jade beads: orbit the player. Positions come from the clock, so
        // there is nothing to store.
        if self.up[U_BEADS] > 0 {
            let lvl = self.lvl(U_BEADS);
            let dmg = (6.0 + 3.0 * lvl) * might;
            let n = self.bead_count();
            for k in 0..n {
                let (bx, by) = self.bead_pos(k, n);
                for i in 0..self.enemies.len() {
                    let e = &self.enemies[i];
                    if e.bead_cd > 0.0 {
                        continue;
                    }
                    let r = KINDS[e.kind as usize].r + 12.0;
                    let (dx, dy) = (e.x - bx, e.y - by);
                    if dx * dx + dy * dy < r * r {
                        let (ox, oy) = (e.x - px, e.y - py);
                        let d = (ox * ox + oy * oy).sqrt().max(0.001);
                        self.enemies[i].bead_cd = 0.45;
                        self.hurt_enemy(i, dmg, ox / d * 160.0, oy / d * 160.0);
                    }
                }
            }
        }

        // Spirit pulse: a ring that damages and shoves everything nearby.
        if self.up[U_PULSE] > 0 {
            self.t_pulse -= dt;
            if self.t_pulse <= 0.0 {
                let lvl = self.lvl(U_PULSE);
                self.t_pulse = 4.2 * haste;
                let radius = 150.0 + 30.0 * lvl;
                let dmg = (10.0 + 6.0 * lvl) * might;
                for i in 0..self.enemies.len() {
                    let e = &self.enemies[i];
                    let (dx, dy) = (e.x - px, e.y - py);
                    let d = (dx * dx + dy * dy).sqrt().max(0.001);
                    if d < radius {
                        self.hurt_enemy(i, dmg, dx / d * 520.0, dy / d * 520.0);
                    }
                }
                self.add_fx(Fx {
                    x: px,
                    y: py - 10.0,
                    vx: 0.0,
                    vy: 0.0,
                    life: 0.35,
                    max: 0.35,
                    part: P_RING,
                    hw: 20.0,
                    hh: 20.0,
                    rot: 0.0,
                    vr: 0.0,
                    grow: (radius - 20.0) / 0.35,
                    color: [0.25, 0.62, 0.48, 0.8],
                    seed: 0.7,
                    top: true,
                });
                self.event(EV_PULSE, px, py, 0.0);
            }
        }
    }

    fn bead_count(&self) -> usize {
        if self.up[U_BEADS] == 0 {
            0
        } else {
            1 + self.up[U_BEADS] as usize
        }
    }

    fn bead_pos(&self, k: usize, n: usize) -> (f32, f32) {
        let radius = 78.0 + 6.0 * self.lvl(U_BEADS);
        let a = self.time * 2.8 + k as f32 * TAU / n as f32;
        (self.player.x + a.cos() * radius, self.player.y + a.sin() * radius * 0.8)
    }

    fn spawn_ring(&self) -> f32 {
        (self.view_w * self.view_w + self.view_h * self.view_h).sqrt() + 70.0
    }

    fn spawn_enemy(&mut self, kind: u8) {
        if self.enemies.len() >= MAX_ENEMIES {
            return;
        }
        let a = self.rng.range(0.0, TAU);
        let ring = self.spawn_ring();
        let k = &KINDS[kind as usize];
        let scale = 1.0 + self.time / 120.0;
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        let seed = self.rng.f();
        self.enemies.push(Enemy {
            x: self.player.x + a.cos() * ring,
            y: self.player.y + a.sin() * ring,
            kx: 0.0,
            ky: 0.0,
            hp: k.hp * scale,
            kind,
            flash: 0.0,
            seed,
            bead_cd: 0.0,
            id,
            face: 1.0,
        });
    }

    fn spawn(&mut self, dt: f32) {
        let t = self.time;
        let rate = 1.0 + t / 24.0;
        self.spawn_acc += rate * dt;
        while self.spawn_acc >= 1.0 {
            self.spawn_acc -= 1.0;
            let roll = self.rng.f();
            let brute = if t > 60.0 { ((t - 60.0) / 400.0).min(0.25) } else { 0.0 };
            let wisp = if t > 25.0 { 0.3 } else { 0.0 };
            let kind = if roll < brute {
                K_BRUTE
            } else if roll < brute + wisp {
                K_WISP
            } else {
                K_BLOT
            };
            self.spawn_enemy(kind);
        }
        if t >= self.next_elite {
            self.next_elite += 90.0;
            self.spawn_enemy(K_ELITE);
            self.event(EV_ELITE, self.player.x, self.player.y, 0.0);
        }
    }

    fn move_enemies(&mut self, dt: f32) {
        let (px, py) = (self.player.x, self.player.y);
        let ring = self.spawn_ring();
        let far = ring * 1.8;
        let decay = (-8.0 * dt).exp();
        let mut hit: Option<f32> = None;
        for e in self.enemies.iter_mut() {
            let k = &KINDS[e.kind as usize];
            let (dx, dy) = (px - e.x, py - e.y);
            let d = (dx * dx + dy * dy).sqrt().max(0.001);
            if d > far {
                // Left far behind: bring it back to the spawn ring on the
                // other side so the crowd keeps closing in.
                e.x = px + dx / d * ring;
                e.y = py + dy / d * ring;
                continue;
            }
            e.x += (dx / d * k.speed + e.kx) * dt;
            e.y += (dy / d * k.speed + e.ky) * dt;
            e.kx *= decay;
            e.ky *= decay;
            e.flash = (e.flash - dt * 6.0).max(0.0);
            e.bead_cd = (e.bead_cd - dt).max(0.0);
            if dx.abs() > 4.0 {
                e.face = dx.signum();
            }
            if d < k.r + 10.0 {
                hit = Some(hit.map_or(k.dmg, |h: f32| h.max(k.dmg)));
            }
        }
        if let Some(dmg) = hit {
            if self.player.iframe <= 0.0 {
                self.player.hp -= dmg;
                self.player.iframe = 0.8;
                self.shake = 1.0;
                self.event(EV_HURT, px, py, dmg);
                for _ in 0..6 {
                    let a = self.rng.range(0.0, TAU);
                    let s = self.rng.range(80.0, 220.0);
                    let seed = self.rng.f();
                    self.add_fx(Fx {
                        x: px,
                        y: py - 30.0,
                        vx: a.cos() * s,
                        vy: a.sin() * s - 60.0,
                        life: 0.5,
                        max: 0.5,
                        part: P_SCRAP,
                        hw: 4.0,
                        hh: 4.0,
                        rot: a,
                        vr: 9.0,
                        grow: 0.0,
                        color: [0.20, 0.42, 0.36, 1.0],
                        seed,
                        top: true,
                    });
                }
                if self.player.hp <= 0.0 {
                    self.player.hp = 0.0;
                    self.over = true;
                    self.event(EV_DEATH, px, py, 0.0);
                }
            }
        }
    }

    /// Push overlapping enemies apart using a hashed uniform grid.
    fn separate(&mut self) {
        let n = self.enemies.len();
        if n < 2 {
            return;
        }
        for c in self.cell_start.iter_mut() {
            *c = 0;
        }
        for e in &self.enemies {
            let k = cell_key((e.x / CELL).floor() as i32, (e.y / CELL).floor() as i32);
            self.cell_start[k + 1] += 1;
        }
        for i in 0..BUCKETS {
            self.cell_start[i + 1] += self.cell_start[i];
        }
        self.cell_cursor.copy_from_slice(&self.cell_start);
        self.cell_items.clear();
        self.cell_items.resize(n, 0);
        for (i, e) in self.enemies.iter().enumerate() {
            let k = cell_key((e.x / CELL).floor() as i32, (e.y / CELL).floor() as i32);
            self.cell_items[self.cell_cursor[k] as usize] = i as u16;
            self.cell_cursor[k] += 1;
        }
        self.push.clear();
        self.push.resize(n, (0.0, 0.0));
        for i in 0..n {
            let e = self.enemies[i];
            let er = KINDS[e.kind as usize].r;
            let cx = (e.x / CELL).floor() as i32;
            let cy = (e.y / CELL).floor() as i32;
            for oy in -1..=1 {
                for ox in -1..=1 {
                    let k = cell_key(cx + ox, cy + oy);
                    for s in self.cell_start[k]..self.cell_start[k + 1] {
                        let j = self.cell_items[s as usize] as usize;
                        if j <= i {
                            continue;
                        }
                        let o = &self.enemies[j];
                        let rr = er + KINDS[o.kind as usize].r;
                        let (dx, dy) = (o.x - e.x, o.y - e.y);
                        let d2 = dx * dx + dy * dy;
                        if d2 < rr * rr && d2 > 0.0001 {
                            let d = d2.sqrt();
                            let f = (rr - d) * 0.5 / d;
                            self.push[i].0 -= dx * f;
                            self.push[i].1 -= dy * f;
                            self.push[j].0 += dx * f;
                            self.push[j].1 += dy * f;
                        }
                    }
                }
            }
        }
        for (e, p) in self.enemies.iter_mut().zip(self.push.iter()) {
            e.x += p.0 * 0.6;
            e.y += p.1 * 0.6;
        }
    }

    fn move_shots(&mut self, dt: f32) {
        let mut s = 0;
        while s < self.shots.len() {
            let (x, y, dmg, last) = {
                let sh = &mut self.shots[s];
                sh.x += sh.vx * dt;
                sh.y += sh.vy * dt;
                sh.life -= dt;
                (sh.x, sh.y, sh.dmg, sh.last)
            };
            let mut hit = None;
            for (i, e) in self.enemies.iter().enumerate() {
                if e.id == last || e.hp <= 0.0 {
                    continue;
                }
                let r = KINDS[e.kind as usize].r + 8.0;
                let (dx, dy) = (e.x - x, e.y - y);
                if dx * dx + dy * dy < r * r {
                    hit = Some(i);
                    break;
                }
            }
            if let Some(i) = hit {
                let (vx, vy) = (self.shots[s].vx, self.shots[s].vy);
                self.shots[s].last = self.enemies[i].id;
                self.shots[s].pierce -= 1;
                self.hurt_enemy(i, dmg, vx * 0.3, vy * 0.3);
            }
            if self.shots[s].life <= 0.0 || self.shots[s].pierce <= 0 {
                self.shots.swap_remove(s);
            } else {
                s += 1;
            }
        }
    }

    /// Remove dead enemies and drop their rewards.
    fn reap(&mut self) {
        let mut i = 0;
        while i < self.enemies.len() {
            if self.enemies[i].hp > 0.0 {
                i += 1;
                continue;
            }
            let e = self.enemies.swap_remove(i);
            let k = &KINDS[e.kind as usize];
            self.kills += 1;
            self.event(EV_KILL, e.x, e.y, e.kind as f32);
            let heart = self.rng.f() < if e.kind == K_ELITE { 1.0 } else { 0.012 };
            self.drop_gem(e.x, e.y, k.xp, false);
            if heart {
                self.drop_gem(e.x + 14.0, e.y, 0.0, true);
            }
            let n = if e.kind >= K_BRUTE { 9 } else { 4 };
            for _ in 0..n {
                let a = self.rng.range(0.0, TAU);
                let s = self.rng.range(20.0, 150.0);
                let size = self.rng.range(0.25, 0.6) * k.r;
                let seed = self.rng.f();
                self.add_fx(Fx {
                    x: e.x,
                    y: e.y - k.r * 0.5,
                    vx: a.cos() * s,
                    vy: a.sin() * s * 0.6,
                    life: 0.55,
                    max: 0.55,
                    part: P_SPLAT,
                    hw: size,
                    hh: size * 0.8,
                    rot: a,
                    vr: 0.0,
                    grow: 8.0,
                    color: [INK[0], INK[1], INK[2], 0.7],
                    seed,
                    top: false,
                });
            }
        }
    }

    fn drop_gem(&mut self, x: f32, y: f32, value: f32, heart: bool) {
        if !heart && self.gems.len() >= MAX_GEMS {
            // Too many on the ground: fold the value into an existing gem.
            let i = (self.rng.next() as usize) % self.gems.len();
            self.gems[i].value += value;
            return;
        }
        let seed = self.rng.f();
        let jx = self.rng.range(-6.0, 6.0);
        let jy = self.rng.range(-6.0, 6.0);
        self.gems.push(Gem { x: x + jx, y: y + jy, value, seed, pull: false, heart });
    }

    fn move_gems(&mut self, dt: f32) {
        let (px, py) = (self.player.x, self.player.y);
        let lure = 95.0 * (1.0 + 0.3 * self.lvl(U_LURE));
        let mut i = 0;
        while i < self.gems.len() {
            let g = &mut self.gems[i];
            let (dx, dy) = (px - g.x, py - g.y);
            let d = (dx * dx + dy * dy).sqrt().max(0.001);
            if d < lure {
                g.pull = true;
            }
            if g.pull {
                let v = 460.0 * dt;
                g.x += dx / d * v;
                g.y += dy / d * v;
            }
            if d < 20.0 {
                let g = self.gems.swap_remove(i);
                if g.heart {
                    self.player.hp = (self.player.hp + 25.0).min(self.player.max_hp);
                    self.event(EV_HEAL, px, py, 25.0);
                } else {
                    self.player.xp += g.value;
                    self.event(EV_PICKUP, px, py, g.value);
                }
            } else {
                i += 1;
            }
        }
    }

    fn add_fx(&mut self, f: Fx) {
        if self.fx.len() < MAX_FX {
            self.fx.push(f);
        }
    }

    fn move_fx(&mut self, dt: f32) {
        let mut i = 0;
        while i < self.fx.len() {
            let f = &mut self.fx[i];
            f.life -= dt;
            if f.life <= 0.0 {
                self.fx.swap_remove(i);
                continue;
            }
            f.x += f.vx * dt;
            f.y += f.vy * dt;
            let drag = (-5.0 * dt).exp();
            f.vx *= drag;
            f.vy *= drag;
            f.rot += f.vr * dt;
            f.hw += f.grow * dt;
            f.hh += f.grow * dt;
            i += 1;
        }
    }

    // ------------------------------------------------------------ upgrades

    fn offer(&mut self) {
        let mut pool = [0usize; N_UPGRADES];
        let mut n = 0;
        for id in 0..N_UPGRADES {
            if self.up[id] < UPGRADE_MAX[id] {
                pool[n] = id;
                n += 1;
            }
        }
        if n == 0 {
            // Everything maxed: heal instead.
            self.player.hp = (self.player.hp + 30.0 * self.pending as f32).min(self.player.max_hp);
            self.pending = 0;
            return;
        }
        for c in 0..3 {
            if c < n {
                let j = c + (self.rng.next() as usize) % (n - c);
                pool.swap(c, j);
                self.choices[c] = pool[c] as i32;
            } else {
                self.choices[c] = -1;
            }
        }
        self.choosing = true;
        self.event(EV_LEVELUP, self.player.x, self.player.y, self.player.level as f32);
    }

    pub fn choose(&mut self, i: usize) {
        if !self.choosing || i >= 3 || self.choices[i] < 0 {
            return;
        }
        let id = self.choices[i] as usize;
        if self.up[id] < UPGRADE_MAX[id] {
            self.up[id] += 1;
            if id == U_VITALITY {
                self.player.max_hp += 20.0;
                self.player.hp += 20.0;
            }
        }
        self.choices = [-1; 3];
        self.choosing = false;
        self.pending = self.pending.saturating_sub(1);
        if self.pending > 0 {
            self.offer();
        }
    }

    // ---------------------------------------------------------------- draw

    #[allow(clippy::too_many_arguments)]
    fn put(&mut self, x: f32, y: f32, hw: f32, hh: f32, rot: f32, part: f32, flash: f32, seed: f32, color: [f32; 4]) {
        if self.inst.len() >= MAX_INST {
            return;
        }
        // Cull what the camera cannot see.
        let m = hw.abs().max(hh) + 24.0;
        if (x - self.cam.0).abs() > self.view_w + m || (y - self.cam.1).abs() > self.view_h + m {
            return;
        }
        self.inst.push(Inst { x, y, hw, hh, rot, part, flash, seed, color });
    }

    fn build(&mut self) {
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
        for i in 0..self.shots.len() {
            let s = &self.shots[i];
            let (x, y, rot) = (s.x, s.y, s.vy.atan2(s.vx) + TAU * 0.25);
            self.put(x, y - 24.0, 6.0, 11.0, rot, P_TALISMAN, 0.0, 0.5, [0.89, 0.77, 0.38, 1.0]);
        }
        let n = self.bead_count();
        for k in 0..n {
            let (x, y) = self.bead_pos(k, n);
            self.put(x, y - 24.0, 9.0, 9.0, 0.0, P_BEAD, 0.0, k as f32 * 0.13, [0.30, 0.66, 0.50, 1.0]);
        }
        for i in 0..self.fx.len() {
            if self.fx[i].top {
                self.draw_fx(i);
            }
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
        let pose = ((self.anim * k.rate + e.seed * 7.0).floor() as i32) & 1;
        let (tilt, hop) = if pose == 0 { (0.13, 0.0) } else { (-0.13, -3.0) };
        let c = [k.color[0], k.color[1], k.color[2], 1.0];
        let shadow = [INK[0], INK[1], INK[2], 0.2];
        self.put(e.x, e.y, k.r * 0.95, k.r * 0.36, 0.0, P_SHADOW, 0.0, 0.0, shadow);
        if e.kind == K_WISP {
            // Wisps hover and flap by squeezing their width.
            let hw = if pose == 0 { k.hw } else { k.hw * 0.7 };
            let bob = (self.anim * 5.0 + e.seed * 20.0).sin() * 3.0;
            self.put(e.x, e.y - 26.0 + bob, hw * e.face, k.hh, tilt * 0.5, k.part, e.flash, e.seed, c);
        } else {
            self.put(e.x, e.y - k.hh + hop, k.hw * e.face, k.hh, tilt, k.part, e.flash, e.seed, c);
        }
    }

    fn draw_player(&mut self) {
        let p = &self.player;
        let (px, py, f) = (p.x, p.y, p.face);
        let (tilt, bob) = if p.moving {
            if ((p.walk * 9.0).floor() as i32) & 1 == 0 { (0.10, 0.0) } else { (-0.10, -3.0) }
        } else if ((self.anim * 2.0).floor() as i32) & 1 == 0 {
            (0.0, 0.0)
        } else {
            (0.0, -1.5)
        };
        // Blink while invulnerable.
        let a = if p.iframe > 0.0 && ((p.iframe * 20.0) as i32) & 1 == 1 { 0.4 } else { 1.0 };
        let flash = if p.iframe > 0.65 { 1.0 } else { 0.0 };
        let attack = p.attack;

        let shadow = [INK[0], INK[1], INK[2], 0.22];
        self.put(px, py, 20.0, 7.0, 0.0, P_SHADOW, 0.0, 0.0, shadow);

        let scythe = [0.36, 0.66, 0.52, a];
        let swinging = attack > 0.0;
        if !swinging {
            // Resting over the shoulder, behind the body.
            self.put(px - f * 15.0, py - 46.0 + bob, 20.0 * f, 34.0, -f * 0.45 + tilt, P_SCYTHE, flash, 0.21, scythe);
        }
        let top_x = px + 26.0 * tilt.sin();
        self.put(px, py - 26.0 + bob, 20.0 * f, 26.0, tilt, P_ROBE, flash, 0.37, [0.20, 0.42, 0.36, a]);
        self.put(top_x + f * 2.0, py - 60.0 + bob * 1.5, 15.0 * f, 15.0, -tilt * 0.6, P_HOOD, flash, 0.53, [0.15, 0.33, 0.29, a]);
        if swinging {
            let t = 1.0 - attack / 0.25;
            let ease = t * t * (3.0 - 2.0 * t);
            let ang = (-1.3 + 2.8 * ease) * f;
            self.put(px + f * 12.0, py - 42.0 + bob, 20.0 * f, 34.0, ang, P_SCYTHE, flash, 0.21, scythe);
        }
    }

    // --------------------------------------------------------------- state

    /// HUD state read by the host. Layout:
    /// 0 hp, 1 max hp, 2 xp, 3 xp needed, 4 level, 5 time, 6 kills,
    /// 7 choosing, 8..10 offered upgrade ids (-1 for none), 11 game over,
    /// 12 camera x, 13 camera y, 14 enemy count, 15 running,
    /// 16..25 upgrade levels.
    fn write_state(&mut self) {
        let p = &self.player;
        let s = &mut self.state;
        s[0] = p.hp;
        s[1] = p.max_hp;
        s[2] = p.xp;
        s[3] = p.xp_next;
        s[4] = p.level as f32;
        s[5] = self.time;
        s[6] = self.kills as f32;
        s[7] = self.choosing as u8 as f32;
        for c in 0..3 {
            s[8 + c] = self.choices[c] as f32;
        }
        s[11] = self.over as u8 as f32;
        s[12] = self.cam.0;
        s[13] = self.cam.1;
        s[14] = self.enemies.len() as f32;
        s[15] = self.running as u8 as f32;
        for id in 0..N_UPGRADES {
            s[16 + id] = self.up[id] as f32;
        }
    }
}
