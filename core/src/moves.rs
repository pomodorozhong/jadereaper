//! Enemy attacks. Every attack is a move: a wind-up that shows a warning on the ground in the shape of the coming
//! hit, an active window when it hits, then a recovery that is the player's opening.

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

pub const M_BLOT_LUNGE: usize = 0;
pub const M_WISP_SHOT: usize = 1;
pub const M_BRUTE_SLAM: usize = 2;
pub const M_ELITE_BITE: usize = 3;
pub const M_ELITE_STOMP: usize = 4;
pub const N_MOVES: usize = 5;

pub const MOVES: [Move; N_MOVES] = [
    // Blot: lunges along a line.
    Move {
        windup: 0.55, active: 0.18, recover: 0.75, area: AreaKind::Line, reach: 120.0, width: 18.0, damage: 8.0,
        dash: 110.0, after: After::None, range: (0.0, 130.0), chain: 0,
    },
    // Wisp: a slow ink shot from a distance.
    Move {
        windup: 0.6, active: 0.05, recover: 0.9, area: AreaKind::Line, reach: 360.0, width: 10.0, damage: 7.0,
        dash: 0.0, after: After::Shot { speed: 200.0, r: 9.0 }, range: (0.0, 380.0), chain: 0,
    },
    // Brute: slams the ground, then a shock ring spreads out.
    Move {
        windup: 0.75, active: 0.1, recover: 1.0, area: AreaKind::Circle, reach: 80.0, width: 0.0, damage: 14.0,
        dash: 0.0, after: After::Wave { to: 230.0, speed: 260.0, band: 22.0 }, range: (0.0, 120.0), chain: 0,
    },
    // Elite: two lunging bites.
    Move {
        windup: 0.5, active: 0.15, recover: 0.8, area: AreaKind::Arc, reach: 120.0, width: 0.65, damage: 16.0,
        dash: 60.0, after: After::None, range: (0.0, 160.0), chain: 1,
    },
    // Elite: a stomp and a shock ring.
    Move {
        windup: 0.65, active: 0.1, recover: 0.9, area: AreaKind::Circle, reach: 100.0, width: 0.0, damage: 18.0,
        dash: 0.0, after: After::Wave { to: 300.0, speed: 280.0, band: 24.0 }, range: (0.0, 240.0), chain: 0,
    },
];

/// The moves each enemy kind can use (blot, wisp, brute, elite).
pub const KIND_MOVES: [&[usize]; 4] = [&[M_BLOT_LUNGE], &[M_WISP_SHOT], &[M_BRUTE_SLAM], &[M_ELITE_BITE, M_ELITE_STOMP]];
