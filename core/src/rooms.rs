//! A run: a chain of fixed arena rooms, each fought in waves, with a card pick after each clear. The last rooms
//! hold the bosses, and beating the last one wins the run. The rooms are in `content/game.json`.

use crate::defs::*;
use crate::game::{Fx, Game};
use crate::geom::dist;
use crate::player::PLAYER_R;

#[derive(Clone, Copy, Debug)]
pub struct Room {
    /// Size of the arena, centred on the origin.
    pub w: f32,
    pub h: f32,
    /// 0 for none, or the boss kind + 1; the room is cleared when its waves are, and the boss is in one of them.
    pub boss: u8,
    /// Each wave's enemy kinds.
    pub waves: &'static [&'static [u8]],
}

/// Pause before the first wave of a room and between waves.
pub const WAVE_DELAY: f32 = 1.2;
/// New enemies appear at least this far from the player.
pub const SPAWN_CLEARANCE: f32 = 200.0;
/// Health regained on clearing a room, as a share of the maximum.
pub const CLEAR_HEAL: f32 = 0.2;

/// Where a room stands.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Stage {
    /// Waiting to send in wave `wave`.
    Waiting,
    /// Wave `wave` is fighting.
    Fighting,
    /// All waves beaten; the card pick follows.
    Cleared,
}

impl Game {
    /// The arena's edges for a room.
    pub fn room_bounds(room: usize) -> [f32; 4] {
        let r = &ROOMS[room.min(N_ROOMS - 1)];
        [-r.w * 0.5, -r.h * 0.5, r.w * 0.5, r.h * 0.5]
    }

    /// Enter a room: clear what the last one left, put the player at the bottom edge, and wait for the first wave.
    pub fn enter_room(&mut self, room: usize) {
        self.room = room;
        self.wave = 0;
        self.stage = Stage::Waiting;
        self.wave_t = WAVE_DELAY;
        let b = Game::room_bounds(room);
        self.arena = Some(b);
        self.enemies.clear();
        self.hazards.clear();
        self.shots.clear();
        self.wells.clear();
        self.patches.clear();
        self.orbit = None;
        self.fx.clear();
        self.focus = None;
        self.player.x = 0.0;
        self.player.y = b[3] - 90.0;
        self.player.kx = 0.0;
        self.player.ky = 0.0;
        self.snap_camera();
    }

    /// Run the room: send waves in when the last is beaten, and clear the room after the last wave.
    pub(crate) fn run_room(&mut self, dt: f32) {
        if !self.auto_spawn || self.won {
            return;
        }
        let room = &ROOMS[self.room];
        match self.stage {
            Stage::Waiting => {
                self.wave_t -= dt;
                if self.wave_t <= 0.0 {
                    let wave = room.waves[self.wave];
                    for &kind in wave {
                        self.spawn_in_room(kind);
                    }
                    self.stage = Stage::Fighting;
                }
            }
            Stage::Fighting => {
                if self.enemies.is_empty() {
                    self.wave += 1;
                    if self.wave < room.waves.len() {
                        self.stage = Stage::Waiting;
                        self.wave_t = WAVE_DELAY;
                    } else {
                        self.clear_room();
                    }
                }
            }
            Stage::Cleared => {}
        }
    }

    fn clear_room(&mut self) {
        self.stage = Stage::Cleared;
        self.rooms_cleared += 1;
        let p = &mut self.player;
        p.hp = (p.hp + p.max_hp * CLEAR_HEAL).min(p.max_hp);
        self.event(EV_CLEAR, self.player.x, self.player.y, self.room as f32);
        if self.room + 1 >= N_ROOMS {
            // The last boss is down: the run is won.
            self.won = true;
            self.over = true;
            self.event(EV_WIN, self.player.x, self.player.y, 0.0);
        } else {
            self.prepare_offer();
            self.offer_reward();
        }
    }

    /// Lean the next offer towards fire and ice before the final boss room, whose element check needs them.
    fn prepare_offer(&mut self) {
        let next = self.room + 1;
        let final_boss = next + 1 == N_ROOMS && ROOMS[next].boss > 0;
        let has = |g: &Game, e: u8| g.mods.iter().any(|m| m.element == e);
        self.want_element = [0; 2];
        if final_boss || next + 2 == N_ROOMS {
            let (fire, ice) = (E_FIRE + 1, E_ICE + 1);
            if !has(self, fire) {
                self.want_element[0] = fire;
            }
            if !has(self, ice) {
                self.want_element[1] = ice;
            }
        }
    }

    /// After the card pick that follows a clear, move on to the next room.
    pub(crate) fn next_room(&mut self) {
        if self.stage == Stage::Cleared && self.room + 1 < N_ROOMS {
            self.enter_room(self.room + 1);
        }
    }

    /// Spawn an enemy somewhere in the room, away from the player.
    pub(crate) fn spawn_in_room(&mut self, kind: u8) {
        let [x0, y0, x1, y1] = self.arena.unwrap_or([-400.0, -300.0, 400.0, 300.0]);
        let m = 60.0;
        let (mut x, mut y) = (0.0, y0 + m);
        for _ in 0..12 {
            x = self.rng.range(x0 + m, x1 - m);
            y = self.rng.range(y0 + m, y1 - m);
            if dist(x, y, self.player.x, self.player.y) > SPAWN_CLEARANCE {
                break;
            }
        }
        if self.add_enemy(kind, x, y).is_some() {
            let k = &KINDS[kind as usize];
            self.add_fx(Fx { top: false, grow: 30.0, ..Fx::at(x, y, P_SPLAT, 0.6, k.r * 1.2, k.r * 0.8, [INK[0], INK[1], INK[2], 0.5]) });
        }
    }

    /// Keep the player and enemies inside the arena.
    pub(crate) fn keep_inside(&mut self) {
        let Some([x0, y0, x1, y1]) = self.arena else { return };
        let p = &mut self.player;
        p.x = p.x.clamp(x0 + PLAYER_R, x1 - PLAYER_R);
        p.y = p.y.clamp(y0 + PLAYER_R, y1 - PLAYER_R);
        for e in self.enemies.iter_mut() {
            let r = KINDS[e.kind as usize].r;
            e.x = e.x.clamp(x0 + r, x1 - r);
            e.y = e.y.clamp(y0 + r, y1 - r);
        }
    }
}
