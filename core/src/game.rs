//! Game state and the simulation step.
//!
//! All simulation happens on the ground plane (x right, y down). Characters are drawn standing up from their
//! ground point, so draw code offsets parts upwards. Animation is done by posing rigid paper parts (tilt, hop,
//! flip) on a low, stepped frame rate instead of playing sprite frames.
//!
//! The player is in `player.rs`, enemies in `enemy.rs` and drawing in `draw.rs`; they all add to `Game`.

use crate::defs::*;
pub use crate::player::{Act, Input, Player};

pub(crate) const MAX_ENEMIES: usize = 500;
pub(crate) const MAX_GEMS: usize = 700;
pub(crate) const MAX_FX: usize = 500;
pub(crate) const MAX_SHOTS: usize = 200;
pub(crate) const CELL: f32 = 96.0;
pub(crate) const BUCKETS: usize = 4096;
/// The simulation always advances in steps of this length, so a seed and its inputs replay the same run.
pub const STEP: f32 = 1.0 / 60.0;
/// Most steps run in one update. A longer stall (a background tab, a debugger) is dropped, not caught up.
const MAX_STEPS: u32 = 8;

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
    pub id: u32,
    pub face: f32,
    /// Stagger built up by hits; past the kind's poise the enemy is interrupted.
    pub stagger: f32,
    /// Time left interrupted.
    pub stun: f32,
}

/// A projectile thrown by the player.
pub struct Shot {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub dmg: f32,
    pub stagger: f32,
    pub knockback: f32,
    pub r: f32,
    pub pierce: i32,
    pub last: u32,
}

pub struct Gem {
    pub x: f32,
    pub y: f32,
    pub value: f32,
    pub seed: f32,
    pub pull: bool,
    pub heart: bool,
}

pub struct Fx {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max: f32,
    pub part: f32,
    pub hw: f32,
    pub hh: f32,
    pub rot: f32,
    pub vr: f32,
    pub grow: f32,
    pub color: [f32; 4],
    pub seed: f32,
    /// Drawn above characters instead of on the ground.
    pub top: bool,
}

impl Fx {
    /// A still effect at a spot; set the other fields with struct update syntax.
    pub fn at(x: f32, y: f32, part: f32, life: f32, hw: f32, hh: f32, color: [f32; 4]) -> Fx {
        Fx { x, y, vx: 0.0, vy: 0.0, life, max: life, part, hw, hh, rot: 0.0, vr: 0.0, grow: 0.0, color, seed: 0.5, top: true }
    }
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
    /// Line boil of the traced characters: poses per second, and how many poses each part has.
    pub boil_fps: f32,
    pub boil_poses: u32,
    /// How big each character is drawn: the player, then each enemy kind. Hit areas do not change.
    pub draw_scale: [f32; 5],
    /// Each enemy kind's waddle: (tilt each way in radians, pose changes per second). Starts from `KINDS`.
    pub waddle: [(f32, f32); 4],
    /// Half extents of the visible area in world units.
    pub view_w: f32,
    pub view_h: f32,
    pub input: Input,
    pub(crate) prev_buttons: u32,
    pub player: Player,
    /// The skill on each of the three skill buttons.
    pub loadout: [usize; 3],
    pub enemies: Vec<Enemy>,
    pub shots: Vec<Shot>,
    pub gems: Vec<Gem>,
    pub fx: Vec<Fx>,
    pub up: [u8; N_UPGRADES],
    pub choices: [i32; 3],
    pub pending: u32,
    pub kills: u32,
    pub shake: f32,
    /// Time left frozen after a hit lands.
    pub hitstop: f32,
    /// Turns off camera shake and hit-stop.
    pub reduce_motion: bool,
    pub cam: (f32, f32),
    /// Time not yet simulated, below one `STEP`.
    acc: f32,
    /// Fixed steps run since the start.
    pub steps: u32,
    /// Spawn enemies on a timer. Tests turn it off and place enemies themselves.
    pub auto_spawn: bool,
    pub(crate) spawn_acc: f32,
    pub(crate) next_elite: f32,
    pub(crate) next_id: u32,
    pub inst: Vec<Inst>,
    pub events: Vec<[f32; 4]>,
    pub state: [f32; STATE_FLOATS],
    // Scratch buffers for the spatial grid and draw sorting.
    pub(crate) cell_start: Vec<u32>,
    pub(crate) cell_cursor: Vec<u32>,
    pub(crate) cell_items: Vec<u16>,
    pub(crate) push: Vec<(f32, f32)>,
    pub(crate) order: Vec<(f32, u16)>,
}

fn xp_for(level: u32) -> f32 {
    let l = level as f32;
    4.0 + l * 3.5 + l * l * 0.4
}

impl Game {
    pub fn new(seed: u32) -> Game {
        Game {
            rng: Rng(seed | 1),
            running: false,
            choosing: false,
            over: false,
            time: 0.0,
            anim: 0.0,
            boil_fps: 4.0,
            boil_poses: MAX_POSES,
            draw_scale: [1.0; 5],
            waddle: [
                (KINDS[0].tilt.to_radians(), KINDS[0].rate),
                (KINDS[1].tilt.to_radians(), KINDS[1].rate),
                (KINDS[2].tilt.to_radians(), KINDS[2].rate),
                (KINDS[3].tilt.to_radians(), KINDS[3].rate),
            ],
            view_w: 640.0,
            view_h: 360.0,
            input: Input::default(),
            prev_buttons: 0,
            player: Player::new(xp_for(1)),
            loadout: [crate::skills::SK_SCYTHE, crate::skills::SK_TALISMAN, crate::skills::SK_PULSE],
            enemies: Vec::with_capacity(MAX_ENEMIES),
            shots: Vec::with_capacity(MAX_SHOTS),
            gems: Vec::with_capacity(MAX_GEMS),
            fx: Vec::with_capacity(MAX_FX),
            up: [0; N_UPGRADES],
            choices: [-1; 3],
            pending: 0,
            kills: 0,
            shake: 0.0,
            hitstop: 0.0,
            reduce_motion: false,
            cam: (0.0, -30.0),
            acc: 0.0,
            steps: 0,
            auto_spawn: true,
            spawn_acc: 0.0,
            next_elite: 90.0,
            next_id: 1,
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

    /// Advance by a frame's time: the simulation runs in fixed `STEP`s and keeps the remainder for the next
    /// frame. Drawing, the camera and the HUD state follow every call.
    pub fn update(&mut self, dt: f32) {
        let dt = if dt.is_finite() { dt.clamp(0.0, 0.25) } else { 0.0 };
        self.events.clear();
        self.anim += dt;
        if self.running && !self.choosing && !self.over {
            self.acc += dt;
            let mut n = 0;
            // The small slack keeps 1/60 s frames from skipping a step through rounding.
            while self.acc >= STEP - 1e-5 && n < MAX_STEPS && !self.choosing && !self.over {
                self.step(STEP);
                self.acc -= STEP;
                n += 1;
            }
            if n == MAX_STEPS || self.choosing || self.over {
                self.acc = 0.0;
            }
            self.acc = self.acc.max(0.0);
        } else {
            self.acc = 0.0;
        }
        // Camera follows the player, with a decaying shake.
        self.shake = (self.shake - dt * 3.0).max(0.0);
        let s = if self.reduce_motion { 0.0 } else { self.shake * self.shake * 9.0 };
        let q = (self.anim * 30.0).floor();
        self.cam = (
            self.player.x + s * (q * 12.9898).sin(),
            self.player.y - 30.0 + s * (q * 78.233).cos(),
        );
        self.build();
        self.write_state();
    }

    pub(crate) fn event(&mut self, kind: f32, x: f32, y: f32, v: f32) {
        if self.events.len() < 256 {
            self.events.push([kind, x, y, v]);
        }
    }

    pub(crate) fn lvl(&self, id: usize) -> f32 {
        self.up[id] as f32
    }

    /// Freeze the action for a moment as a hit lands.
    pub(crate) fn hit_stop(&mut self, t: f32) {
        if !self.reduce_motion {
            self.hitstop = self.hitstop.max(t);
        }
    }

    // ---------------------------------------------------------------- step

    fn step(&mut self, dt: f32) {
        self.steps += 1;
        if self.hitstop > 0.0 {
            self.hitstop -= dt;
            return;
        }
        self.time += dt;
        self.move_player(dt);
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

    pub(crate) fn drop_gem(&mut self, x: f32, y: f32, value: f32, heart: bool) {
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

    pub(crate) fn add_fx(&mut self, f: Fx) {
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

    // --------------------------------------------------------------- state

    /// A hash of the simulation state, to check that a seed and its inputs replay the same run.
    pub fn state_hash(&self) -> u32 {
        let mut h: u32 = 0x811c_9dc5;
        let mut mix = |v: u32| {
            for b in v.to_le_bytes() {
                h ^= b as u32;
                h = h.wrapping_mul(0x0100_0193);
            }
        };
        mix(self.rng.0);
        mix(self.steps);
        let p = &self.player;
        for v in [p.x, p.y, p.hp, p.max_hp, p.xp] {
            mix(v.to_bits());
        }
        mix(self.enemies.len() as u32);
        for e in &self.enemies {
            for v in [e.x, e.y, e.hp] {
                mix(v.to_bits());
            }
        }
        mix(self.kills);
        h
    }

    /// HUD state read by the host. Layout:
    /// 0 hp, 1 max hp, 2 xp, 3 xp needed, 4 level, 5 time, 6 kills,
    /// 7 choosing, 8..10 offered upgrade ids (-1 for none), 11 game over,
    /// 12 camera x, 13 camera y, 14 enemy count, 15 running,
    /// 16..20 upgrade levels, 26 player x, 27 player y, 28..30 loadout skill ids.
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
        s[26] = self.player.x;
        s[27] = self.player.y;
        for c in 0..3 {
            s[28 + c] = self.loadout[c] as f32;
        }
    }
}
