//! Enemies: spawning, movement, taking hits and dying, and the player's projectiles.

use crate::defs::*;
use crate::game::*;
use core::f32::consts::TAU;

/// How long an enemy is interrupted when its stagger passes its poise.
pub const STUN_TIME: f32 = 0.45;

fn cell_key(cx: i32, cy: i32) -> usize {
    (((cy & 63) as usize) << 6) | ((cx & 63) as usize)
}

impl Game {
    /// Damage an enemy: knockback, stagger, a flash, a hit-stop and a damage number.
    pub(crate) fn hurt_enemy(&mut self, i: usize, dmg: f32, stagger: f32, kx: f32, ky: f32) {
        let e = &mut self.enemies[i];
        let k = &KINDS[e.kind as usize];
        e.hp -= dmg;
        e.flash = 1.0;
        let heavy = if e.kind >= K_BRUTE { 0.35 } else { 1.0 };
        e.kx += kx * heavy;
        e.ky += ky * heavy;
        e.stagger += stagger;
        if e.stagger >= k.poise {
            e.stagger = 0.0;
            e.stun = STUN_TIME;
        }
        let (x, y) = (e.x, e.y);
        self.event(EV_HIT, x, y, dmg);
        self.hit_stop((0.03 + dmg * 0.0015).min(0.09));
    }

    fn spawn_ring(&self) -> f32 {
        (self.view_w * self.view_w + self.view_h * self.view_h).sqrt() + 70.0
    }

    pub(crate) fn spawn_enemy(&mut self, kind: u8) {
        let a = self.rng.range(0.0, TAU);
        let ring = self.spawn_ring();
        let scale = 1.0 + self.time / 120.0;
        let (x, y) = (self.player.x + a.cos() * ring, self.player.y + a.sin() * ring);
        if let Some(i) = self.add_enemy(kind, x, y) {
            self.enemies[i].hp *= scale;
        }
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
        });
        Some(self.enemies.len() - 1)
    }

    pub(crate) fn spawn(&mut self, dt: f32) {
        if !self.auto_spawn {
            return;
        }
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

    pub(crate) fn move_enemies(&mut self, dt: f32) {
        let (px, py) = (self.player.x, self.player.y);
        let ring = self.spawn_ring();
        let far = ring * 1.8;
        let decay = (-8.0 * dt).exp();
        let mut hit: Option<(f32, f32, f32)> = None;
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
            e.flash = (e.flash - dt * 6.0).max(0.0);
            e.stagger = (e.stagger - k.poise * 0.3 * dt).max(0.0);
            let speed = if e.stun > 0.0 { 0.0 } else { k.speed };
            e.stun = (e.stun - dt).max(0.0);
            e.x += (dx / d * speed + e.kx) * dt;
            e.y += (dy / d * speed + e.ky) * dt;
            e.kx *= decay;
            e.ky *= decay;
            if dx.abs() > 4.0 && speed > 0.0 {
                e.face = dx.signum();
            }
            if d < k.r + 10.0 && hit.map_or(true, |h| k.dmg > h.0) {
                hit = Some((k.dmg, e.x, e.y));
            }
        }
        if let Some((dmg, x, y)) = hit {
            self.hurt_player(dmg, false, (x, y));
        }
    }

    /// Push overlapping enemies apart using a hashed uniform grid.
    pub(crate) fn separate(&mut self) {
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
                let (vx, vy, dmg, stagger, kb) = (sh.vx, sh.vy, sh.dmg, sh.stagger, sh.knockback);
                sh.last = self.enemies[i].id;
                sh.pierce -= 1;
                let v = (vx * vx + vy * vy).sqrt().max(1e-3);
                self.hurt_enemy(i, dmg, stagger, vx / v * kb, vy / v * kb);
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
