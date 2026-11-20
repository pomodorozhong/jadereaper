//! Enemy attacks. Every attack is a move: a wind-up that shows a warning on the ground in the shape of the coming
//! hit, an active window when it hits, then a recovery that is the player's opening. The moves themselves are in
//! `content/game.json`.

use crate::skills::AreaKind;

/// What a move leaves behind when it strikes, besides its own hit area.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum After {
    None,
    /// A slow ink shot along the move's direction: speed, radius.
    Shot { speed: f32, r: f32 },
    /// A shock ring spreading from the strike: final radius, speed, band width.
    Wave { to: f32, speed: f32, band: f32 },
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
}

/// Every attack is warned at least this long before it can hit.
pub const MIN_WARNING: f32 = 0.4;
/// Big boss moves are warned at least this long.
pub const MIN_BIG_WARNING: f32 = 0.6;

pub use crate::content::*;
