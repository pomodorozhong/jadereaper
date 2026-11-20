//! Skill definitions. A skill has no cooldown: it is balanced by its timing, a startup before it hits, an active
//! window while it hits, and a recovery after. A roll can cut the recovery short after the cancel point.
//! The skills themselves are in `content/game.json`.

/// How a skill delivers its hit.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SkillKind {
    /// A hit area in front of the player during the active window.
    Melee,
    /// Throws a projectile at the start of the active window.
    Shot,
    /// A hit area around the player.
    Nova,
    /// Beads orbit the player for `lasts` seconds at radius `reach`, each `width` across, hitting what they touch.
    Orbit,
    /// Leaps towards the aim point through the startup, up to `dash` away, and slams down where it lands.
    Leap,
}

/// Shape of a skill's hit area. `reach` and `width` give its size (see `SkillDef`).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AreaKind {
    Circle,
    Arc,
    Line,
    Ring,
}

#[derive(Clone, Copy, Debug)]
pub struct SkillDef {
    /// 0 light, 1 medium, 2 heavy.
    pub weight: u8,
    pub kind: SkillKind,
    pub startup: f32,
    pub active: f32,
    pub recovery: f32,
    /// Seconds into the recovery after which a roll may cancel it.
    pub cancel: f32,
    pub damage: f32,
    /// Stagger added to the target; a target whose stagger passes its poise is interrupted.
    pub stagger: f32,
    /// Knockback speed given to the target, world units per second.
    pub knockback: f32,
    pub area: AreaKind,
    /// Radius of a circle, arc or ring, length of a line, range of a shot.
    pub reach: f32,
    /// Half angle of an arc in radians, half width of a line, thickness of a ring, radius of a shot.
    pub width: f32,
    /// Not interrupted by normal enemies during the active window.
    pub hyper: bool,
    /// Distance moved along the aim through the active window (a leap: the farthest it goes, through the startup).
    pub dash: f32,
    /// How many presses chain into a combo; 1 for none.
    pub combo: u8,
    /// Movement speed while starting up and active, as a share of walking speed.
    pub walk: f32,
    /// How long an orbit lasts.
    pub lasts: f32,
}

impl SkillDef {
    pub fn total(&self) -> f32 {
        self.startup + self.active + self.recovery
    }
    /// Seconds from the start of the skill after which a roll can cancel it.
    pub fn cancel_at(&self) -> f32 {
        self.startup + self.active + self.cancel
    }
}

pub use crate::content::*;

/// An orbit hits the same target again after this long.
pub const ORBIT_REHIT: f32 = 0.5;
/// Beads in an orbit.
pub const ORBIT_BEADS: usize = 4;

/// The third hit of a combo hits harder and reaches further.
pub const COMBO_FINISH_DAMAGE: f32 = 1.6;
pub const COMBO_FINISH_REACH: f32 = 1.15;
