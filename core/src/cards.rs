//! The card pool. The starting passive, the pick after each room and every upgrade are cards from one pool. A card
//! attaches to one skill, to any skill (the offer binds it to one of the three), to the character or to the roll.
//! Most cards add an element to a skill; a skill carries one element at most, and a new one replaces the old.
//! The cards themselves are in `content/game.json`.

use crate::defs::*;
use crate::game::Game;
use crate::skills::SkillDef;

/// What a card attaches to.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Target {
    /// One of the three skills, chosen when the card is offered.
    AnySkill,
    /// Only this skill, offered when it is on a button.
    Skill(usize),
    Character,
    Roll,
}

/// What a card does besides adding an element.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Effect {
    None,
    /// Recovery (and cancel point) times this amount.
    Recovery,
    /// Reach times this amount.
    Reach,
    /// Damage times this amount.
    Damage,
    /// Stagger times this amount.
    Stagger,
    /// Maximum health plus this amount.
    MaxHp,
    /// Walking speed plus this share.
    Speed,
    /// Health regained per second.
    Regen,
    /// All damage plus this share.
    Might,
    /// Damage taken minus this share.
    Armor,
    /// Roll distance plus this share.
    RollDist,
    /// The roll leaves a trail of the card's element.
    Trail,
}

#[derive(Clone, Copy, Debug)]
pub struct Card {
    pub target: Target,
    /// 0 for none, or an element id + 1.
    pub element: u8,
    pub effect: Effect,
    pub amount: f32,
    /// Can be the starting passive.
    pub passive: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Element {
    pub color: [f32; 3],
}

/// The cards on one skill button.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkillMods {
    /// 0 for none, or an element id + 1.
    pub element: u8,
    pub recovery: f32,
    pub reach: f32,
    pub damage: f32,
    pub stagger: f32,
    /// Cards attached so far.
    pub cards: u8,
}

impl Default for SkillMods {
    fn default() -> Self {
        SkillMods {
            element: 0,
            recovery: 1.0,
            reach: 1.0,
            damage: 1.0,
            stagger: 1.0,
            cards: 0,
        }
    }
}

/// Cards on the character and the roll.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stats {
    pub might: f32,
    pub speed: f32,
    pub regen: f32,
    pub armor: f32,
    pub roll: f32,
    /// 0 for no trail, or the trail's element id + 1.
    pub trail: u8,
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            might: 1.0,
            speed: 1.0,
            regen: 0.0,
            armor: 0.0,
            roll: 1.0,
            trail: 0,
        }
    }
}

/// What is being picked.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Pick {
    None,
    /// The starting passive, from three character cards.
    Passive,
    /// A card after a room clear.
    Reward,
}

/// The slot value for a card that does not attach to a skill.
pub const NO_SLOT: i32 = -1;

impl Game {
    /// A skill button's skill with its cards applied.
    pub fn def(&self, slot: usize) -> SkillDef {
        let mut d = SKILLS[self.loadout[slot]];
        let m = &self.mods[slot];
        d.recovery *= m.recovery;
        d.cancel *= m.recovery;
        d.reach *= m.reach;
        d.damage *= m.damage * self.stats.might;
        d.stagger *= m.stagger;
        d
    }

    /// Offer the starting passive: three different character cards.
    pub fn offer_passive(&mut self) {
        let mut pool: Vec<(usize, i32, f32)> = (0..N_CARDS)
            .filter(|&c| CARDS[c].passive)
            .map(|c| (c, NO_SLOT, 1.0))
            .collect();
        self.deal(&mut pool, Pick::Passive);
    }

    /// Offer three cards after a room clear, weighted towards the skills and elements already in use.
    pub fn offer_reward(&mut self) {
        let mut pool = Vec::with_capacity(N_CARDS * 3);
        let used = |e: u8, mods: &[SkillMods; 3]| mods.iter().any(|m| m.element == e);
        for c in 0..N_CARDS {
            let card = &CARDS[c];
            let slots: &[usize] = match card.target {
                Target::AnySkill => &[0, 1, 2],
                Target::Skill(s) => match self.loadout.iter().position(|&l| l == s) {
                    Some(0) => &[0],
                    Some(1) => &[1],
                    Some(2) => &[2],
                    _ => &[],
                },
                Target::Character | Target::Roll => {
                    let w = if card.effect == Effect::Trail && self.stats.trail == card.element {
                        0.0
                    } else {
                        1.0
                    };
                    pool.push((c, NO_SLOT, w));
                    &[]
                }
            };
            for &slot in slots {
                let m = &self.mods[slot];
                let mut w = 1.0 + 0.3 * m.cards as f32;
                if card.element > 0 {
                    if m.element == card.element {
                        w = 0.0;
                    } else {
                        if m.element > 0 {
                            w *= 0.6;
                        }
                        if used(card.element, &self.mods) {
                            w *= 2.0;
                        }
                        if self.want_element.contains(&card.element) {
                            w *= 6.0;
                        }
                    }
                }
                pool.push((c, slot as i32, w));
            }
        }
        let wanted = pool.clone();
        self.deal(&mut pool, Pick::Reward);
        self.make_sure_of_wanted(&wanted);
    }

    /// When an element is wanted (fire and ice before the final boss) and no offered card brings one, the last
    /// offer is swapped for the likeliest wanted element card.
    fn make_sure_of_wanted(&mut self, pool: &[(usize, i32, f32)]) {
        let wanted = |c: i32| {
            c >= 0
                && CARDS[c as usize].element > 0
                && self.want_element.contains(&CARDS[c as usize].element)
        };
        if self.want_element == [0, 0] || self.choices.iter().any(|&c| wanted(c)) {
            return;
        }
        let best = pool
            .iter()
            .filter(|p| p.2 > 0.0 && wanted(p.0 as i32))
            .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(core::cmp::Ordering::Equal));
        if let Some(&(card, slot, _)) = best {
            self.choices[2] = card as i32;
            self.choice_slots[2] = slot;
        }
    }

    /// Draw three different cards from a weighted pool and wait for a choice.
    fn deal(&mut self, pool: &mut Vec<(usize, i32, f32)>, pick: Pick) {
        self.choices = [-1; 3];
        self.choice_slots = [NO_SLOT; 3];
        for c in 0..3 {
            let total: f32 = pool.iter().map(|p| p.2).sum();
            if total <= 0.0 {
                break;
            }
            let mut r = self.rng.f() * total;
            let mut at = pool.len() - 1;
            for (i, p) in pool.iter().enumerate() {
                if r < p.2 {
                    at = i;
                    break;
                }
                r -= p.2;
            }
            let (card, slot, _) = pool[at];
            self.choices[c] = card as i32;
            self.choice_slots[c] = slot;
            // The same card is not offered twice in one pick.
            pool.retain(|p| p.0 != card);
        }
        self.pick = pick;
        self.choosing = self.choices[0] >= 0;
        if self.choosing {
            self.event(EV_LEVELUP, self.player.x, self.player.y, 0.0);
        }
    }

    /// Take one of the offered cards (0..2).
    pub fn choose(&mut self, i: usize) {
        if !self.choosing || i >= 3 || self.choices[i] < 0 {
            return;
        }
        let card = self.choices[i] as usize;
        let slot = self.choice_slots[i];
        self.apply_card(card, slot);
        self.choices = [-1; 3];
        self.choice_slots = [NO_SLOT; 3];
        self.choosing = false;
        let pick = self.pick;
        self.pick = Pick::None;
        self.after_pick(pick);
    }

    /// Attach a card. `slot` is the skill button for skill cards, or `NO_SLOT`.
    pub fn apply_card(&mut self, card: usize, slot: i32) {
        let c = CARDS[card];
        self.owned[card] = self.owned[card].saturating_add(1);
        if slot >= 0 {
            let m = &mut self.mods[slot as usize];
            m.cards += 1;
            if c.element > 0 {
                m.element = c.element;
            }
            match c.effect {
                Effect::Recovery => m.recovery *= c.amount,
                Effect::Reach => m.reach *= c.amount,
                Effect::Damage => m.damage *= c.amount,
                Effect::Stagger => m.stagger *= c.amount,
                _ => {}
            }
            return;
        }
        let s = &mut self.stats;
        match c.effect {
            Effect::MaxHp => {
                self.player.max_hp += c.amount;
                self.player.hp += c.amount;
            }
            Effect::Speed => s.speed += c.amount,
            Effect::Regen => s.regen += c.amount,
            Effect::Might => s.might += c.amount,
            Effect::Armor => s.armor = (s.armor + c.amount).min(0.6),
            Effect::RollDist => s.roll += c.amount,
            Effect::Trail => s.trail = c.element,
            _ => {}
        }
    }
}
