//! Element tags and reactions.

use jadereaper_core::defs::*;
use jadereaper_core::game::{Game, STEP};
use jadereaper_core::reactions::*;

fn arena() -> Game {
    let mut g = Game::new(5);
    g.running = true;
    g.auto_spawn = false;
    g
}

fn dummy(g: &mut Game, kind: u8, x: f32, y: f32) -> usize {
    let i = g.add_enemy(kind, x, y).unwrap();
    g.enemies[i].hp = 1e5;
    g.enemies[i].think = 1e9;
    i
}

/// Hit enemy `i` with a skill of the given element (element id), through the second skill button's card.
fn hit(g: &mut Game, i: usize, element: u8) {
    let card = [C_FIRE, C_ICE, C_LIGHTNING, C_CELESTIAL][element as usize];
    g.apply_card(card, 1);
    g.loadout[1] = SK_TALISMAN;
    let (x, y) = (g.enemies[i].x, g.enemies[i].y);
    g.player.x = x - 120.0;
    g.player.y = y;
    g.input = jadereaper_core::game::Input { ax: 120.0, buttons: B_SKILL2, ..Default::default() };
    g.update(STEP);
    g.input = Default::default();
    for _ in 0..40 {
        g.update(STEP);
        if g.events.iter().any(|e| e[0] == EV_HIT) {
            break;
        }
    }
    // Let the hit-stop pass.
    for _ in 0..8 {
        g.update(STEP);
    }
}

fn reactions(g: &Game) -> Vec<u8> {
    g.events.iter().filter(|e| e[0] == EV_REACTION).map(|e| e[3] as u8).collect()
}

#[test]
fn every_pair_has_its_reaction_and_same_elements_do_not_react() {
    let names = [R_STEAM, R_OVERLOAD, R_COLLAPSE, R_SHATTER, R_STASIS, R_STARFALL];
    let mut seen = vec![];
    for a in 0..4u8 {
        assert_eq!(reaction(a, a), 0);
        for b in (a + 1)..4 {
            let r = reaction(a, b);
            assert_eq!(r, reaction(b, a));
            assert!(names.contains(&r));
            seen.push(r);
        }
    }
    seen.sort();
    assert_eq!(seen, names.to_vec());
}

#[test]
fn a_hit_tags_and_a_second_element_reacts_using_the_tag_up() {
    let mut g = arena();
    let i = dummy(&mut g, K_BRUTE, 0.0, 0.0);
    hit(&mut g, i, E_FIRE);
    assert_eq!(g.enemies[i].status.tag, E_FIRE + 1);
    // The same element again only refreshes the tag.
    let hp = g.enemies[i].hp;
    hit(&mut g, i, E_FIRE);
    assert_eq!(g.enemies[i].status.tag, E_FIRE + 1);
    let mut got = vec![];
    let before = g.enemies[i].hp;
    let card = C_LIGHTNING;
    g.apply_card(card, 1);
    g.input = jadereaper_core::game::Input { ax: 120.0, buttons: B_SKILL2, ..Default::default() };
    for _ in 0..60 {
        g.update(STEP);
        g.input = Default::default();
        got.extend(reactions(&g));
    }
    assert_eq!(got, vec![R_OVERLOAD]);
    assert_eq!(g.enemies[i].status.tag, 0, "the tag is used up");
    assert!(g.enemies[i].hp < before && before < hp);
}

#[test]
fn tags_wear_off() {
    let mut g = arena();
    let i = dummy(&mut g, K_BRUTE, 0.0, 0.0);
    hit(&mut g, i, E_ICE);
    assert_eq!(g.enemies[i].status.tag, E_ICE + 1);
    for _ in 0..((TAG_TIME / STEP) as u32 + 2) {
        g.update(STEP);
    }
    assert_eq!(g.enemies[i].status.tag, 0);
}

#[test]
fn collapsing_star_pulls_enemies_in_but_not_bosses() {
    let mut g = arena();
    let i = dummy(&mut g, K_BRUTE, 0.0, 0.0);
    let small = dummy(&mut g, K_BLOT, 0.0, 150.0);
    let boss = g.add_enemy(K_GLUTTON, 150.0, 0.0).unwrap();
    g.enemies[boss].think = 1e9;
    g.enemies[i].status.tag = E_CELESTIAL + 1;
    g.enemies[i].status.tag_t = TAG_TIME;
    let boss_at = (g.enemies[boss].x, g.enemies[boss].y);
    g.react_for_test(i, E_FIRE + 1);
    assert!(g.enemies[small].y < 100.0, "the blot was not pulled in: {}", g.enemies[small].y);
    assert_eq!((g.enemies[boss].x, g.enemies[boss].y), boss_at, "the boss was pulled");
    assert!(g.enemies[small].hp < 1e5, "the blast missed");
}

#[test]
fn stasis_freezes_enemies_and_staggers_bosses() {
    let mut g = arena();
    let i = dummy(&mut g, K_BRUTE, 0.0, 0.0);
    let small = dummy(&mut g, K_WISP, 60.0, 0.0);
    let boss = g.add_enemy(K_GLUTTON, -100.0, 0.0).unwrap();
    g.enemies[i].status.tag = E_ICE + 1;
    g.enemies[i].status.tag_t = TAG_TIME;
    g.react_for_test(i, E_CELESTIAL + 1);
    assert!(g.enemies[small].status.frozen > 0.0);
    assert_eq!(g.enemies[boss].status.frozen, 0.0);
    assert!(g.boss.unwrap().meter > 0.0);
}

#[test]
fn shatter_steam_and_starfall_hurt() {
    for (tag, by, want) in [(E_ICE, E_LIGHTNING, R_SHATTER), (E_FIRE, E_ICE, R_STEAM), (E_LIGHTNING, E_CELESTIAL, R_STARFALL)] {
        let mut g = arena();
        let i = dummy(&mut g, K_BRUTE, 0.0, 0.0);
        let others: Vec<usize> = (0..3).map(|k| dummy(&mut g, K_BLOT, 70.0 + 40.0 * k as f32, 30.0)).collect();
        g.enemies[i].status.tag = tag + 1;
        g.enemies[i].status.tag_t = TAG_TIME;
        assert_eq!(g.react_for_test(i, by + 1), want);
        assert!(g.enemies[i].hp < 1e5 || others.iter().any(|&j| g.enemies[j].hp < 1e5), "reaction {want} did nothing");
    }
}
