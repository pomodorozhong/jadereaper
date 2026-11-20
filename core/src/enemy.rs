//! Enemies: spawning, movement, taking hits and dying, and the player's projectiles.

use crate::defs::*;
use crate::game::*;
use crate::geom::{dir_to, dist, Area, Shape};
use crate::moves::*;
use crate::player::PLAYER_R;
use crate::elements::Status;
use crate::skills::AreaKind;
use core::f32::consts::TAU;

/// How long an enemy is interrupted when its stagger passes its poise.
pub const STUN_TIME: f32 = 0.45;

fn cell_key(cx: i32, cy: i32) -> usize {
    (((cy & 63) as usize) << 6) | ((cx & 63) as usize)
}

impl Game {
    /// Damage an enemy: knockback, stagger, a flash, a hit-stop and a damage number.
    pub(crate) fn hurt_enemy(&mut self, i: usize, dmg: f32, stagger: f32, kx: f32, ky: f32) {
        let dmg = self.boss_hit(i, dmg, stagger);
        let boss = KINDS[self.enemies[i].kind as usize].boss;
        let e = &mut self.enemies[i];
        let k = &KINDS[e.kind as usize];
        e.hp -= dmg;
        e.flash = 1.0;
        let heavy = if e.kind >= K_BRUTE { 0.35 } else { 1.0 };
        e.kx += kx * heavy;
        e.ky += ky * heavy;
        e.stagger += if boss { 0.0 } else { stagger };
        if !boss && e.stagger >= k.poise {
            e.stagger = 0.0;
            e.stun = STUN_TIME;
        }
        let (x, y) = (e.x, e.y);
        self.event(EV_HIT, x, y, dmg);
        self.hit_stop((0.03 + dmg * 0.0015).min(0.09));
    }

    /// Put an enemy at a spot. Returns its index, or None when there is no room.
    pub fn add_enemy(&mut self, kind: u8, x: f32, y: f32) -> Option<usize> {
        if self.enemies.len() >= MAX_ENEMIES {
            return None;
        }
        let k = &KINDS[kind as usize];
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        let seed = self.rng.f();
        let think = self.rng.range(0.6, 1.4);
        self.enemies.push(Enemy {
            x,
            y,
            kx: 0.0,
            ky: 0.0,
            hp: k.hp,
            kind,
            flash: 0.0,
            seed,
            id,
            face: 1.0,
            stagger: 0.0,
            stun: 0.0,
            phase: Phase::Approach,
            pt: 0.0,
            mv: 0,
            dx: 1.0,
            dy: 0.0,
            ox: x,
            oy: y,
            chain: 0,
            struck: false,
            think,
            age: 0.0,
            status: Status::default(),
        });
        if k.boss {
            self.boss = Some(crate::boss::BossState::new(id));
        }
        Some(self.enemies.len() - 1)
    }

    /// The hit area of an enemy's current move.
    pub fn move_area(&self, e: &Enemy) -> Area {
        let m = &MOVES[e.mv as usize];
        let (x, y) = match m.area {
            // A lunge's line covers the path; other dashing moves hit where the dash ends.
            AreaKind::Line => (e.ox, e.oy),
            _ => (e.ox + e.dx * m.dash, e.oy + e.dy * m.dash),
        };
        if m.special == Special::Sweep {
            // Warned as the whole arc the line will sweep.
            return Area::new(e.ox, e.oy, (e.dx, e.dy), Shape::Arc { r: m.reach, half: SWEEP_HALF });
        }
        let shape = match m.area {
            AreaKind::Circle => Shape::Circle { r: m.reach },
            AreaKind::Arc => Shape::Arc { r: m.reach, half: m.width },
            AreaKind::Line => Shape::Line { len: m.reach, half_w: m.width },
            AreaKind::Ring => Shape::Ring { inner: (m.reach - m.width).max(0.0), outer: m.reach },
        };
        Area::new(x, y, (e.dx, e.dy), shape)
    }

    /// Where a striking move hits right now. A lunge or charge along a line hits with the attacker's body as it
    /// passes, so the warning shows the whole path but a roll timed to its arrival avoids it.
    pub fn strike_area(&self, e: &Enemy) -> Area {
        let m = &MOVES[e.mv as usize];
        if m.area == AreaKind::Line && m.dash > 0.0 && m.special != Special::Sweep {
            let r = KINDS[e.kind as usize].r + m.width * 0.5;
            return Area::new(e.x, e.y, (e.dx, e.dy), Shape::Circle { r });
        }
        self.move_area(e)
    }

    pub(crate) fn start_windup(&mut self, i: usize) {
        let (px, py) = (self.player.x, self.player.y);
        let e = &mut self.enemies[i];
        let (dx, dy) = dir_to(e.x, e.y, px, py);
        e.phase = Phase::Windup;
        e.pt = 0.0;
        e.dx = dx;
        e.dy = dy;
        e.ox = e.x;
        e.oy = e.y;
        e.struck = false;
        if dx.abs() > 0.1 {
            e.face = dx.signum();
        }
    }

    pub(crate) fn move_enemies(&mut self, dt: f32) {
        let (px, py) = (self.player.x, self.player.y);
        let decay = (-8.0 * dt).exp();
        for i in 0..self.enemies.len() {
            if KINDS[self.enemies[i].kind as usize].boss {
                self.move_boss(i, dt);
                continue;
            }
            let e = &mut self.enemies[i];
            let k = &KINDS[e.kind as usize];
            e.age += dt;
            e.flash = (e.flash - dt * 6.0).max(0.0);
            e.stagger = (e.stagger - k.poise * 0.3 * dt).max(0.0);
            e.x += e.kx * dt;
            e.y += e.ky * dt;
            e.kx *= decay;
            e.ky *= decay;
            let pace = e.status.pace();
            if e.stun > 0.0 || pace <= 0.0 {
                // Interrupted or frozen: a wind-up or strike in progress is lost.
                e.stun = (e.stun - dt).max(0.0);
                if matches!(e.phase, Phase::Windup | Phase::Active) {
                    e.phase = Phase::Recover;
                    e.pt = 0.0;
                }
                continue;
            }
            let d = dist(e.x, e.y, px, py);
            let (tx, ty) = dir_to(e.x, e.y, px, py);
            let m = MOVES[e.mv as usize];
            // Chills slow everything it does, wind-ups included, so warnings only get longer.
            let dt = dt * pace;
            e.pt += dt;
            match e.phase {
                Phase::Approach => {
                    e.think -= dt;
                    let moves = KINDS[e.kind as usize].moves;
                    // Wisps keep their distance; the rest close in to striking range.
                    let (near, far) = if e.kind == K_WISP { (200.0, 300.0) } else { (0.0, MOVES[moves[0]].range.1 * 0.7) };
                    let dir = if d > far {
                        1.0
                    } else if d < near {
                        -0.8
                    } else {
                        0.0
                    };
                    e.x += tx * k.speed * dir * dt;
                    e.y += ty * k.speed * dir * dt;
                    if tx.abs() > 0.1 && dir != 0.0 {
                        e.face = (tx * dir).signum();
                    }
                    if e.think <= 0.0 && e.age > 0.8 {
                        let pick = moves.iter().copied().filter(|&mv| d >= MOVES[mv].range.0 && d <= MOVES[mv].range.1);
                        let n = pick.clone().count();
                        if n > 0 {
                            let r = (self.rng.next() as usize) % n;
                            let mv = pick.clone().nth(r).unwrap();
                            let e = &mut self.enemies[i];
                            e.mv = mv as u8;
                            e.chain = MOVES[mv].chain;
                            self.start_windup(i);
                        }
                    }
                }
                Phase::Windup => {
                    if e.pt >= m.windup {
                        e.phase = Phase::Active;
                        e.pt = 0.0;
                        let (ex, ey, dx, dy, boss) = (e.x, e.y, e.dx, e.dy, KINDS[e.kind as usize].boss);
                        let warned = m.windup;
                        match m.after {
                            After::None => {}
                            After::Shot { speed, r } => self.add_hazard(Hazard {
                                x: ex,
                                y: ey,
                                vx: dx * speed,
                                vy: dy * speed,
                                r,
                                band: 0.0,
                                to: 0.0,
                                grow: 0.0,
                                life: m.reach / speed,
                                dmg: m.damage,
                                boss,
                                warned,
                                rehit: 0.0,
                                cool: 0.0,
                            }),
                            After::Wave { to, speed, band } => {
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
                                    boss,
                                    warned,
                                    rehit: 0.0,
                                    cool: 0.0,
                                });
                            }
                        }
                        self.event(EV_STRIKE, ex, ey, self.enemies[i].kind as f32);
                    }
                }
                Phase::Active => {
                    if m.dash > 0.0 {
                        e.x += e.dx * m.dash * dt / m.active;
                        e.y += e.dy * m.dash * dt / m.active;
                    }
                    let done = e.pt >= m.active;
                    let (struck, ex, ey, warned) = (e.struck, e.x, e.y, m.windup + e.pt);
                    if !struck && !matches!(m.after, After::Shot { .. }) {
                        let a = self.strike_area(&self.enemies[i]);
                        if a.hits(px, py, PLAYER_R) {
                            self.enemies[i].struck = true;
                            let boss = KINDS[self.enemies[i].kind as usize].boss;
                            self.hurt_player(m.damage, boss, (ex, ey), warned);
                        }
                    }
                    if done {
                        let e = &mut self.enemies[i];
                        if e.chain > 0 {
                            e.chain -= 1;
                            self.start_windup(i);
                        } else {
                            e.phase = Phase::Recover;
                            e.pt = 0.0;
                        }
                    }
                }
                Phase::Recover => {
                    if e.pt >= m.recover {
                        e.phase = Phase::Approach;
                        e.pt = 0.0;
                        e.think = self.rng.range(0.3, 1.1);
                    }
                }
            }
        }
    }

    /// True when an attack will hit where the player stands within `within` seconds: a wind-up about to strike,
    /// a strike under way, or a shot or ring about to arrive. The scripted bot rolls on it.
    pub fn threat(&self, within: f32) -> bool {
        let (px, py) = (self.player.x, self.player.y);
        for (i, e) in self.enemies.iter().enumerate() {
            if e.stun > 0.0 || e.status.frozen > 0.0 || self.boss_of(i).map_or(false, |b| b.down > 0.0) {
                continue;
            }
            let m = &MOVES[e.mv as usize];
            if matches!(m.special, Special::Summon | Special::Eat | Special::Finisher) {
                continue;
            }
            let soon = match e.phase {
                Phase::Windup => self.windup_of(e) - e.pt <= within && !matches!(m.after, After::Shot { .. }),
                Phase::Active => !e.struck && !matches!(m.after, After::Shot { .. }),
                _ => false,
            };
            if self.boss_of(i).is_some() {
                let (areas, n) = if e.phase == Phase::Active {
                    self.strike_areas(i, (e.pt + within) / m.active)
                } else if m.special == Special::Sweep {
                    ([self.boss_area(i, 0.0); 5], 1)
                } else {
                    self.move_areas(e)
                };
                if soon && areas[..n].iter().any(|a| a.hits(px, py, PLAYER_R + 6.0)) {
                    return true;
                }
                continue;
            }
            let area = if m.special == Special::Sweep {
                // Where the swept line will be.
                let k = if e.phase == Phase::Active { (e.pt + within) / m.active } else { 0.0 };
                self.boss_area(i, k)
            } else if e.phase == Phase::Active {
                // Where the strike will be, for a body still dashing in.
                let mut a = self.strike_area(e);
                if m.dash > 0.0 && e.pt < m.active {
                    let ahead = (m.dash / m.active) * within.min(m.active - e.pt);
                    a.x += e.dx * ahead;
                    a.y += e.dy * ahead;
                }
                a
            } else {
                self.move_area(e)
            };
            if soon && area.hits(px, py, PLAYER_R + 6.0) {
                return true;
            }
        }
        for h in &self.hazards {
            if h.rehit > 0.0 {
                // Lingering ground is walked out of, not rolled through.
                continue;
            }
            let d = dist(h.x, h.y, px, py);
            let gap = if h.band > 0.0 {
                (d - h.r).abs() - h.band * 0.5 - PLAYER_R
            } else {
                // How far the shot is from passing the player, along its path.
                let (rx, ry) = (px - h.x, py - h.y);
                let v = (h.vx * h.vx + h.vy * h.vy).sqrt().max(1e-3);
                let along = (rx * h.vx + ry * h.vy) / v;
                let side = (-rx * h.vy + ry * h.vx).abs() / v;
                if side > h.r + PLAYER_R || along < -h.r { f32::MAX } else { along - h.r - PLAYER_R }
            };
            let speed = if h.band > 0.0 { h.grow } else { (h.vx * h.vx + h.vy * h.vy).sqrt() };
            // A shot or ring takes most of a roll's invulnerability to pass, so only count it as it arrives.
            if gap <= speed * within.min(0.03) {
                return true;
            }
        }
        false
    }

    /// The way out of the nearest lingering pool the player stands in, if any.
    pub fn pool_escape(&self) -> Option<(f32, f32)> {
        let (px, py) = (self.player.x, self.player.y);
        self.hazards
            .iter()
            .filter(|h| h.rehit > 0.0 && dist(h.x, h.y, px, py) < h.r + PLAYER_R + 8.0)
            .map(|h| dir_to(h.x, h.y, px, py))
            .next()
    }

    pub(crate) fn add_hazard(&mut self, h: Hazard) {
        if self.hazards.len() < 64 {
            self.hazards.push(h);
        }
    }

    /// Ink shots fly and shock rings spread; each hits the player at most once.
    pub(crate) fn move_hazards(&mut self, dt: f32) {
        let (px, py) = (self.player.x, self.player.y);
        let mut i = 0;
        while i < self.hazards.len() {
            let h = &mut self.hazards[i];
            h.x += h.vx * dt;
            h.y += h.vy * dt;
            h.r = (h.r + h.grow * dt).min(h.to.max(h.r));
            h.life -= dt;
            h.warned += dt;
            let d = dist(h.x, h.y, px, py);
            let hit = if h.band > 0.0 { (d - h.r).abs() < h.band * 0.5 + PLAYER_R } else { d < h.r + PLAYER_R };
            h.cool = (h.cool - dt).max(0.0);
            let h = *h;
            let mut gone = h.life <= 0.0;
            if h.rehit > 0.0 {
                // A lingering pool: hurts whoever stands in it, again every `rehit` seconds.
                if hit && h.cool <= 0.0 && self.hurt_player(h.dmg, h.boss, (h.x, h.y), h.warned) {
                    self.hazards[i].cool = h.rehit;
                }
            } else {
                if hit && self.hurt_player(h.dmg, h.boss, (h.x, h.y), h.warned) {
                    gone = true;
                }
                if hit && h.band == 0.0 && !self.player.invulnerable() {
                    gone = true;
                }
            }
            if gone {
                self.hazards.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// Push overlapping enemies apart using a hashed uniform grid.
    pub(crate) fn separate(&mut self) {
        let n = self.enemies.len();
        self.push_player();
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

    /// Enemies are solid to the player, except while rolling through them.
    pub(crate) fn push_player(&mut self) {
        if matches!(self.player.act, Act::Roll { .. }) {
            return;
        }
        for e in &self.enemies {
            let rr = KINDS[e.kind as usize].r + PLAYER_R;
            let (dx, dy) = (self.player.x - e.x, self.player.y - e.y);
            let d2 = dx * dx + dy * dy;
            if d2 < rr * rr && d2 > 1e-4 {
                let d = d2.sqrt();
                self.player.x += dx / d * (rr - d);
                self.player.y += dy / d * (rr - d);
            }
        }
    }

    pub(crate) fn move_shots(&mut self, dt: f32) {
        let mut s = 0;
        while s < self.shots.len() {
            let (x, y, r, last) = {
                let sh = &mut self.shots[s];
                sh.x += sh.vx * dt;
                sh.y += sh.vy * dt;
                sh.life -= dt;
                (sh.x, sh.y, sh.r, sh.last)
            };
            let mut hit = None;
            for (i, e) in self.enemies.iter().enumerate() {
                if e.id == last || e.hp <= 0.0 {
                    continue;
                }
                let rr = KINDS[e.kind as usize].r + r;
                let (dx, dy) = (e.x - x, e.y - y);
                if dx * dx + dy * dy < rr * rr {
                    hit = Some(i);
                    break;
                }
            }
            if let Some(i) = hit {
                let sh = &mut self.shots[s];
                let (vx, vy, dmg, stagger, kb, element) = (sh.vx, sh.vy, sh.dmg, sh.stagger, sh.knockback, sh.element);
                sh.last = self.enemies[i].id;
                sh.pierce -= 1;
                let v = (vx * vx + vy * vy).sqrt().max(1e-3);
                self.hurt_enemy(i, dmg, stagger, vx / v * kb, vy / v * kb);
                self.apply_element(i, element, dmg, stagger);
            }
            if self.shots[s].life <= 0.0 || self.shots[s].pierce <= 0 {
                self.shots.swap_remove(s);
            } else {
                s += 1;
            }
        }
    }

    /// Remove dead enemies and drop their rewards.
    pub(crate) fn reap(&mut self) {
        let mut i = 0;
        while i < self.enemies.len() {
            if self.enemies[i].hp > 0.0 {
                i += 1;
                continue;
            }
            let e = self.enemies.swap_remove(i);
            if e.hp == f32::NEG_INFINITY {
                // Eaten by a boss.
                continue;
            }
            let k = &KINDS[e.kind as usize];
            if self.boss.map_or(false, |b| b.id == e.id) {
                self.release_grab();
                self.boss = None;
                self.focus = None;
                self.shake = 1.2;
            }
            self.kills += 1;
            self.event(EV_KILL, e.x, e.y, e.kind as f32);
            let n = if e.kind >= K_BRUTE { 9 } else { 4 };
            for _ in 0..n {
                let a = self.rng.range(0.0, TAU);
                let s = self.rng.range(20.0, 150.0);
                let size = self.rng.range(0.25, 0.6) * k.r;
                let seed = self.rng.f();
                self.add_fx(Fx {
                    vx: a.cos() * s,
                    vy: a.sin() * s * 0.6,
                    rot: a,
                    grow: 8.0,
                    seed,
                    top: false,
                    ..Fx::at(e.x, e.y - k.r * 0.5, P_SPLAT, 0.55, size, size * 0.8, [INK[0], INK[1], INK[2], 0.7])
                });
            }
        }
    }
}
