//! Skill definitions. A skill has no cooldown: it is balanced by its timing, a startup before it hits, an active
//! window while it hits, and a recovery after. A roll can cut the recovery short after the cancel point.

/// How a skill delivers its hit.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SkillKind {
    /// A hit area in front of the player during the active window.
    Melee,
    /// Throws a projectile at the start of the active window.
    Shot,
    /// A hit area around the player.
    Nova,
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
    /// Distance moved along the aim through the active window.
    pub dash: f32,
    /// How many presses chain into a combo; 1 for none.
    pub combo: u8,
    /// Movement speed while starting up and active, as a share of walking speed.
    pub walk: f32,
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

pub const SK_SCYTHE: usize = 0;
pub const SK_TALISMAN: usize = 1;
pub const SK_PULSE: usize = 2;
pub const N_SKILLS: usize = 3;

pub const SKILLS: [SkillDef; N_SKILLS] = [
    // Scythe combo: a fast 3-hit sweep.
    SkillDef {
        weight: 0, kind: SkillKind::Melee, startup: 0.07, active: 0.08, recovery: 0.22, cancel: 0.06,
        damage: 10.0, stagger: 10.0, knockback: 160.0, area: AreaKind::Arc, reach: 95.0, width: 1.1,
        hyper: false, dash: 12.0, combo: 3, walk: 0.4,
    },
    // Talisman throw: an aimed shot.
    SkillDef {
        weight: 0, kind: SkillKind::Shot, startup: 0.10, active: 0.05, recovery: 0.20, cancel: 0.05,
        damage: 9.0, stagger: 6.0, knockback: 80.0, area: AreaKind::Circle, reach: 520.0, width: 10.0,
        hyper: false, dash: 0.0, combo: 1, walk: 0.7,
    },
    // Spirit pulse: a nova around the player.
    SkillDef {
        weight: 1, kind: SkillKind::Nova, startup: 0.25, active: 0.10, recovery: 0.35, cancel: 0.20,
        damage: 16.0, stagger: 25.0, knockback: 420.0, area: AreaKind::Circle, reach: 140.0, width: 0.0,
        hyper: false, dash: 0.0, combo: 1, walk: 0.0,
    },
];

/// The third hit of a combo hits harder and reaches further.
pub const COMBO_FINISH_DAMAGE: f32 = 1.6;
pub const COMBO_FINISH_REACH: f32 = 1.15;
