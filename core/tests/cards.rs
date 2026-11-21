//! The card pool and the element effects.

use jadereaper_core::cards::*;
use jadereaper_core::defs::*;
use jadereaper_core::game::{Game, Input, STEP};

fn arena() -> Game {
    let mut g = Game::new(3);
    g.running = true;
    g.auto_spawn = false;
    g
}

/// An enemy that stands still and does not die.
fn dummy(g: &mut Game, kind: u8, x: f32, y: f32) -> usize {
    let i = g.add_enemy(kind, x, y).unwrap();
    g.enemies[i].hp = 1e5;
    g.enemies[i].think = 1e9;
    i
}

fn swing(g: &mut Game, slot: usize, aim: (f32, f32)) {
    g.input = Input {
        ax: aim.0,
        ay: aim.1,
        buttons: 1 << slot,
        ..Input::default()
    };
    g.update(STEP);
    g.input = Input::default();
    for _ in 0..40 {
        g.update(STEP);
    }
}

#[test]
fn starting_passive_offers_three_different_character_cards() {
    let mut g = arena();
    g.offer_passive();
    assert!(g.choosing);
    assert_eq!(g.pick, Pick::Passive);
    let c = g.choices;
    assert!(c[0] != c[1] && c[1] != c[2] && c[0] != c[2]);
    for &id in &c {
        assert!(CARDS[id as usize].passive);
        assert!(matches!(
            CARDS[id as usize].target,
            Target::Character | Target::Roll
        ));
    }
    g.choose(0);
    assert!(!g.choosing);
    assert_eq!(g.owned[c[0] as usize], 1);
}

#[test]
fn a_new_element_replaces_the_old_and_upgrades_stack() {
    let mut g = arena();
    g.apply_card(C_FIRE, 0);
    assert_eq!(g.mods[0].element, E_FIRE + 1);
    g.apply_card(C_ICE, 0);
    assert_eq!(g.mods[0].element, E_ICE + 1, "one element per skill");
    let base = g.def(0).recovery;
    g.apply_card(C_QUICK_HANDS, 0);
    g.apply_card(C_QUICK_HANDS, 0);
    assert!((g.def(0).recovery - base * 0.64).abs() < 1e-4);
    assert_eq!(
        g.mods[1],
        SkillMods::default(),
        "other buttons are untouched"
    );
    let hp = g.player.max_hp;
    g.apply_card(C_VITALITY, NO_SLOT);
    assert_eq!(g.player.max_hp, hp + 25.0);
}

#[test]
fn offers_never_repeat_an_element_a_skill_has_and_lean_towards_elements_in_use() {
    let mut g = arena();
    g.apply_card(C_FIRE, 0);
    let (mut fire_elsewhere, mut ice) = (0, 0);
    for _ in 0..400 {
        g.offer_reward();
        for c in 0..3 {
            let (card, slot) = (g.choices[c], g.choice_slots[c]);
            assert!(
                !(card == C_FIRE as i32 && slot == 0),
                "offered fire to the button that has it"
            );
            if card == C_FIRE as i32 {
                fire_elsewhere += 1;
            }
            if card == C_ICE as i32 {
                ice += 1;
            }
        }
        g.choosing = false;
    }
    assert!(fire_elsewhere > ice, "fire {fire_elsewhere}, ice {ice}");
}

#[test]
fn fire_burns_over_time() {
    let mut g = arena();
    g.apply_card(C_FIRE, 0);
    let i = dummy(&mut g, K_BLOT, 50.0, 0.0);
    swing(&mut g, 0, (100.0, 0.0));
    let hp = g.enemies[i].hp;
    assert!(g.enemies[i].status.burn > 0.0);
    for _ in 0..60 {
        g.update(STEP);
    }
    assert!(g.enemies[i].hp < hp, "burning did no damage");
}

#[test]
fn ice_slows_then_freezes() {
    let mut g = arena();
    g.apply_card(C_ICE, 0);
    let i = dummy(&mut g, K_BRUTE, 50.0, 0.0);
    swing(&mut g, 0, (100.0, 0.0));
    assert_eq!(g.enemies[i].status.chill, 1);
    assert!(g.enemies[i].status.pace() < 1.0);
    swing(&mut g, 0, (100.0, 0.0));
    g.enemies[i].x = 50.0;
    g.enemies[i].y = 0.0;
    swing(&mut g, 0, (100.0, 0.0));
    let s = g.enemies[i].status;
    assert!(s.frozen > 0.0, "{s:?}");
    assert_eq!(s.pace(), 0.0);
}

#[test]
fn lightning_chains_to_the_nearest_other_enemy() {
    let mut g = arena();
    g.apply_card(C_LIGHTNING, 1);
    g.loadout[1] = SK_TALISMAN;
    let a = dummy(&mut g, K_BLOT, 120.0, 0.0);
    let near = dummy(&mut g, K_BLOT, 200.0, 80.0);
    let far = dummy(&mut g, K_BLOT, 120.0, -260.0);
    swing(&mut g, 1, (100.0, 0.0));
    assert!(g.enemies[a].hp < 1e5);
    assert!(
        g.enemies[near].hp < 1e5,
        "the chain did not reach the nearest"
    );
    assert_eq!(g.enemies[far].hp, 1e5);
}

#[test]
fn celestial_marks_open_a_well_that_pulls_and_drops_a_star() {
    let mut g = arena();
    g.apply_card(C_CELESTIAL, 0);
    let i = dummy(&mut g, K_BRUTE, 50.0, 0.0);
    let other = dummy(&mut g, K_BLOT, 160.0, 90.0);
    for _ in 0..3 {
        g.enemies[i].x = 50.0;
        g.enemies[i].y = 0.0;
        swing(&mut g, 0, (100.0, 0.0));
    }
    // The third mark opened a well; the swings since have let it pull and fall.
    let moved = ((g.enemies[other].x - 160.0).powi(2) + (g.enemies[other].y - 90.0).powi(2)).sqrt();
    assert!(moved > 20.0, "the well did not pull, moved {moved}");
    assert!(
        g.enemies[other].hp < 1e5,
        "the star did not land on the pulled enemy"
    );
}

#[test]
fn fire_trail_roll_lights_enemies() {
    let mut g = arena();
    g.apply_card(C_FIRE_TRAIL, NO_SLOT);
    let i = dummy(&mut g, K_BLOT, 60.0, 0.0);
    g.input = Input {
        mx: 1.0,
        buttons: B_ROLL,
        ..Input::default()
    };
    for _ in 0..40 {
        g.update(STEP);
        g.input.buttons = 0;
    }
    assert!(g.enemies[i].status.burn > 0.0 || g.enemies[i].hp < 1e5);
}
