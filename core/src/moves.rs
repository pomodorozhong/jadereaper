//! Enemy attacks. Every attack is a move: a wind-up that shows a warning on the ground in the shape of the coming
//! hit, an active window when it hits, then a recovery that is the player's opening. The moves themselves are in
//! `content/game.json`.

use crate::skills::AreaKind;

/// What a move leaves behind when it strikes, besides its own hit area.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum After {
    None,
    /// A slow ink shot along the move's direction: speed, radius.
    Shot {
        speed: f32,
        r: f32,
    },
    /// A shock ring spreading from the strike: final radius, speed, band width.
    Wave {
        to: f32,
        speed: f32,
        band: f32,
    },
}

#[derive(Clone, Copy, Debug)]
pub struct Move {
    /// Wind-up with the warning showing. At least `MIN_WARNING` (or `MIN_BIG_WARNING` for big boss moves).
    pub windup: f32,
    pub active: f32,
    pub recover: f32,
    pub area: AreaKind,
    /// Radius of a circle, arc or ring, length of a line.
    pub reach: f32,
    /// Half angle of an arc, half width of a line, band of a ring.
    pub width: f32,
    pub damage: f32,
    /// Distance the attacker moves along the move's direction during the active window (a lunge or a charge).
    pub dash: f32,
    pub after: After,
    /// Chosen when the target is between these distances.
    pub range: (f32, f32),
    /// Repeats that follow straight on, each with its own wind-up (a combo).
    pub chain: u8,
    /// Boss moves: how likely the move is picked among those that fit, and how long before it can be used again.
    pub weight: f32,
    pub cooldown: f32,
    /// Boss phases it is used in: bit 0 phase 1, bit 1 phase 2.
    pub phases: u8,
    /// A big boss move: warned at least `MIN_BIG_WARNING` ahead, even when the boss is enraged.
    pub big: bool,
    pub special: Special,
    /// A hit knocks the player down.
    pub knockdown: bool,
    /// The Dragon's stance the move belongs to.
    pub stance: Stance,
}

/// Boss moves that work differently from a plain wind-up and strike.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Special {
    None,
    /// Only used when the player is behind the boss.
    Behind,
    /// A line swept across the move's arc through the active window (the `reach` is its length, `width` its half
    /// width, and the warning shows the whole arc of half angle `SWEEP_HALF`).
    Sweep,
    /// A hit pins the player until they press roll enough times.
    Grab,
    /// Calls in blots; not an attack.
    Summon,
    /// Eats a nearby blot to heal and grow angrier; a hit during the wind-up stops it. Not an attack.
    Eat,
    /// Leaves burning pools where it strikes.
    Pools,
    /// A row of ice lances along a line.
    Lances,
    /// A freezing field that slows.
    Field,
    /// Shards fall on warned spots around the player.
    Shards,
    /// Dives at the player from above.
    Dive,
    /// The end-of-stance finisher that fills the screen.
    Finisher,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Stance {
    Any,
    Fire,
    Ice,
}

/// Half angle of a swept move's arc.
pub const SWEEP_HALF: f32 = 1.1;

/// Every attack is warned at least this long before it can hit.
pub const MIN_WARNING: f32 = 0.4;
/// Big boss moves are warned at least this long.
pub const MIN_BIG_WARNING: f32 = 0.6;

pub use crate::content::*;
