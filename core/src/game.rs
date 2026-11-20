//! Game state and the simulation step.
//!
//! All simulation happens on the ground plane (x right, y down). Characters are drawn standing up from their
//! ground point, so draw code offsets parts upwards. Animation is done by posing rigid paper parts (tilt, hop,
//! flip) on a low, stepped frame rate instead of playing sprite frames.
//!
//! The player is in `player.rs`, enemies in `enemy.rs` and drawing in `draw.rs`; they all add to `Game`.

use crate::cards::{Pick, SkillMods, Stats, NO_SLOT};
use crate::defs::*;
use crate::elements::{Patch, Status, Well};
use crate::rooms::Stage;
pub use crate::player::{Act, Input, Player};

/// Enemies at once. Fights are about reading a few attackers, not hordes.
pub const MAX_ENEMIES: usize = 10;
pub(crate) const MAX_FX: usize = 500;
pub(crate) const MAX_SHOTS: usize = 200;
pub(crate) const CELL: f32 = 96.0;
pub(crate) const BUCKETS: usize = 4096;
/// The simulation always advances in steps of this length, so a seed and its inputs replay the same run.
pub const STEP: f32 = 1.0 / 60.0;
/// How far the camera leads towards the aim, as a share of the view's short side.
const CAM_LEAD: f32 = 0.15;
/// The widest the camera goes to keep a boss in view.
const CAM_MAX_ZOOM: f32 = 1.45;
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
    pub phase: Phase,
    /// Time in the current phase.
    pub pt: f32,
    /// The move being used (an index into `MOVES`).
    pub mv: u8,
    /// The move's direction, set when its wind-up starts.
    pub dx: f32,
    pub dy: f32,
    /// Where the move's hit area starts.
    pub ox: f32,
    pub oy: f32,
    /// Combo repeats left.
    pub chain: u8,
    /// The current strike has already hit the player.
    pub struck: bool,
    /// Time before it may start another move.
    pub think: f32,
    /// Time since it appeared.
    pub age: f32,
    /// Burn, chill, freeze and star marks.
    pub status: Status,
}

/// An enemy's attack cycle.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Phase {
    Approach,
    Windup,
    Active,
    Recover,
}

/// Something an enemy attack leaves behind that can still hit: an ink shot or a spreading shock ring.
#[derive(Clone, Copy, Debug)]
pub struct Hazard {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    /// Shot radius, or the ring's current radius.
    pub r: f32,
    /// Ring: band width (0 for a shot).
    pub band: f32,
    /// Ring: final radius and growth speed.
    pub to: f32,
    pub grow: f32,
    pub life: f32,
    pub dmg: f32,
    pub boss: bool,
    /// Time since the warning for the attack that made it started.
    pub warned: f32,
}

/// Beads flung into an orbit around the player by an orbit skill.
pub struct Orbit {
    /// The skill button that flung it.
    pub slot: usize,
    /// Time left.
    pub t: f32,
    pub spin: f32,
    /// Targets hit recently, with the time before each can be hit again.
    pub hits: Vec<(u32, f32)>,
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
    /// The element of the skill that threw it (0 for none, or element id + 1).
    pub element: u8,
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
    pub orbit: Option<Orbit>,
    pub hazards: Vec<Hazard>,
    /// The shortest warning any hit on the player has had, for tests.
    pub min_warned: f32,
    pub fx: Vec<Fx>,
    pub choices: [i32; 3],
    pub kills: u32,
    /// The room being fought (an index into `ROOMS`), its current wave, and where it stands.
    pub room: usize,
    pub wave: usize,
    pub stage: Stage,
    /// Time before the next wave comes in.
    pub wave_t: f32,
    pub rooms_cleared: u32,
    /// The last boss is beaten.
    pub won: bool,
    pub shake: f32,
    /// Time left frozen after a hit lands.
    pub hitstop: f32,
    /// Cards on each skill button.
    pub mods: [SkillMods; 3],
    /// Cards on the character and the roll.
    pub stats: Stats,
    /// How many of each card the player has.
    pub owned: [u8; N_CARDS],
    /// The skill button each offered card goes on, or `NO_SLOT`.
    pub choice_slots: [i32; 3],
    /// What the offered cards are for.
    pub pick: Pick,
    /// Elements the next offers lean hard towards (element id + 1, 0 for none), such as before the final boss.
    pub want_element: [u8; 2],
    pub wells: Vec<Well>,
    pub patches: Vec<Patch>,
    /// Turns off camera shake and hit-stop.
    pub reduce_motion: bool,
    pub cam: (f32, f32),
    /// The camera's resting point before shake.
    cam_at: (f32, f32),
    /// Jump straight to the camera's target on the next frame (entering a room).
    cam_snap: bool,
    /// How much wider than normal the view is drawn (1 or more); the host scales its view by it.
    pub zoom: f32,
    /// A point the camera keeps in view along with the player, such as a boss.
    pub focus: Option<(f32, f32)>,
    /// The arena's edges (x0, y0, x1, y1), when the fight is in one; the camera stays inside.
    pub arena: Option<[f32; 4]>,
    /// Time not yet simulated, below one `STEP`.
    acc: f32,
    /// Fixed steps run since the start.
    pub steps: u32,
    /// Send in the rooms' waves. Tests turn it off and place enemies themselves.
    pub auto_spawn: bool,
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
            player: Player::new(),
            loadout: [crate::skills::SK_SCYTHE, crate::skills::SK_TALISMAN, crate::skills::SK_PULSE],
            enemies: Vec::with_capacity(MAX_ENEMIES),
            shots: Vec::with_capacity(MAX_SHOTS),
            orbit: None,
            hazards: Vec::with_capacity(64),
            min_warned: f32::INFINITY,
            fx: Vec::with_capacity(MAX_FX),
            mods: [SkillMods::default(); 3],
            stats: Stats::default(),
            owned: [0; N_CARDS],
            choice_slots: [NO_SLOT; 3],
            pick: Pick::None,
            want_element: [0; 2],
            wells: Vec::with_capacity(8),
            patches: Vec::with_capacity(40),
            choices: [-1; 3],
            kills: 0,
            room: 0,
            wave: 0,
            stage: Stage::Waiting,
            wave_t: crate::rooms::WAVE_DELAY,
            rooms_cleared: 0,
            won: false,
            shake: 0.0,
            hitstop: 0.0,
            reduce_motion: false,
            cam: (0.0, -30.0),
            cam_at: (0.0, 0.0),
            cam_snap: false,
            zoom: 1.0,
            focus: None,
            arena: None,
            acc: 0.0,
            steps: 0,
            auto_spawn: true,
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
        self.move_camera(dt);
        self.build();
        self.write_state();
    }

    /// The camera leads towards the aim, keeps a focus point (a boss) in view by widening, stays inside the
    /// arena, and shakes after big hits unless motion is reduced.
    fn move_camera(&mut self, dt: f32) {
        let p = &self.player;
        let short = 2.0 * self.view_w.min(self.view_h);
        let lead = CAM_LEAD * short;
        let (mut tx, mut ty) = (p.x + p.aim.0 * lead, p.y - 30.0 + p.aim.1 * lead);
        let mut zoom = 1.0;
        if let Some((fx, fy)) = self.focus {
            // Centre between the player and the focus, and widen until both fit with a margin.
            tx = (p.x + fx) * 0.5;
            ty = (p.y + fy) * 0.5 - 30.0;
            let (hw, hh) = (self.view_w / self.zoom, self.view_h / self.zoom);
            let need = ((p.x - fx).abs() * 0.5 + 120.0) / hw;
            let need = need.max(((p.y - fy).abs() * 0.5 + 140.0) / hh);
            zoom = need.clamp(1.0, CAM_MAX_ZOOM);
        }
        let k = 1.0 - (-dt * 8.0).exp();
        self.zoom += (zoom - self.zoom) * (1.0 - (-dt * 2.0).exp());
        if self.steps == 0 || self.cam_snap {
            self.cam_snap = false;
            self.cam_at = (tx, ty);
        } else {
            self.cam_at.0 += (tx - self.cam_at.0) * k;
            self.cam_at.1 += (ty - self.cam_at.1) * k;
        }
        if let Some([x0, y0, x1, y1]) = self.arena {
            // Keep the view inside the arena, or centred on it when the arena is smaller than the view.
            let (hw, hh) = (self.view_w, self.view_h);
            let m = 60.0;
            let clamp = |v: f32, lo: f32, hi: f32| if lo > hi { (lo + hi) * 0.5 } else { v.clamp(lo, hi) };
            self.cam_at.0 = clamp(self.cam_at.0, x0 - m + hw, x1 + m - hw);
            self.cam_at.1 = clamp(self.cam_at.1, y0 - m - 40.0 + hh, y1 + m - hh);
        }
        self.shake = (self.shake - dt * 3.0).max(0.0);
        let s = if self.reduce_motion { 0.0 } else { self.shake * self.shake * 9.0 };
        let q = (self.anim * 30.0).floor();
        self.cam = (self.cam_at.0 + s * (q * 12.9898).sin(), self.cam_at.1 + s * (q * 78.233).cos());
    }

    pub(crate) fn snap_camera(&mut self) {
        self.cam_snap = true;
    }

    pub(crate) fn event(&mut self, kind: f32, x: f32, y: f32, v: f32) {
        if self.events.len() < 256 {
            self.events.push([kind, x, y, v]);
        }
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
        self.move_orbit(dt);
        self.run_room(dt);
        self.move_enemies(dt);
        self.separate();
        self.keep_inside();
        self.move_hazards(dt);
        self.move_shots(dt);
        self.move_elements(dt);
        self.reap();
        self.move_fx(dt);
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

    /// After a card is taken: a reward leads on to the next room.
    pub(crate) fn after_pick(&mut self, pick: Pick) {
        if pick == Pick::Reward {
            self.next_room();
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
        for v in [p.x, p.y, p.hp, p.max_hp] {
            mix(v.to_bits());
        }
        mix(self.room as u32);
        mix(self.wave as u32);
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
    /// 0 hp, 1 max hp, 2 room index, 3 rooms in a run, 4 wave index, 5 time, 6 kills,
    /// 7 choosing, 8..10 offered card ids (-1 for none), 11 game over,
    /// 12 camera x, 13 camera y, 14 enemy count, 15 running,
    /// 16..18 element on each skill button (0 none, or element id + 1), 19 what is being picked (0 nothing,
    /// 1 the starting passive, 2 a reward), 20..22 the button each offered card goes on (-1 none),
    /// 23 waves in the room, 24 run won, 25 room boss kind + 1 (0 none), 26 player x, 27 player y,
    /// 28..30 loadout skill ids, 31 camera zoom, 32.. how many of each card is held.
    fn write_state(&mut self) {
        let p = &self.player;
        let s = &mut self.state;
        s[0] = p.hp;
        s[1] = p.max_hp;
        s[2] = self.room as f32;
        s[3] = N_ROOMS as f32;
        s[4] = self.wave as f32;
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
        for c in 0..3 {
            s[16 + c] = self.mods[c].element as f32;
            s[20 + c] = self.choice_slots[c] as f32;
        }
        s[19] = match self.pick {
            Pick::None => 0.0,
            Pick::Passive => 1.0,
            Pick::Reward => 2.0,
        };
        for c in 0..N_CARDS.min(STATE_FLOATS - 32) {
            s[32 + c] = self.owned[c] as f32;
        }
        s[23] = ROOMS[self.room].waves.len() as f32;
        s[24] = self.won as u8 as f32;
        s[25] = ROOMS[self.room].boss as f32;
        s[26] = self.player.x;
        s[27] = self.player.y;
        for c in 0..3 {
            s[28 + c] = self.loadout[c] as f32;
        }
        s[31] = self.zoom;
    }
}
