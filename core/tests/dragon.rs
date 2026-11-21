//! The Two-Element Dragon.

use jadereaper_core::boss::BossState;
use jadereaper_core::defs::*;
use jadereaper_core::dragon::*;
use jadereaper_core::game::{Act, Game, Input, Phase, STEP};
use jadereaper_core::moves::*;
use jadereaper_core::rooms::Stage;

fn arena() -> (Game, usize) {
    let mut g = Game::new(21);
    g.running = true;
    g.auto_spawn = false;
    g.arena = Some([-650.0, -450.0, 650.0, 450.0]);
    let i = g.add_enemy(K_DRAGON, 0.0, -250.0).unwrap();
    (g, i)
}

fn boss(g: &Game) -> BossState {
    g.boss.expect("a boss")
}

fn dragon(g: &Game) -> usize {
    g.enemies.iter().position(|e| e.kind == K_DRAGON).unwrap()
}

fn steps(g: &mut Game, n: u32) {
    for _ in 0..n {
        g.update(STEP);
    }
}

#[test]
fn starts_in_fire_and_uses_only_that_stance_s_moves() {
    let (mut g, _) = arena();
    g.player.max_hp = 1e7;
    g.player.hp = 1e7;
    let mut seen = [false; N_MOVES];
    for _ in 0..(60 * 30) {
        g.update(STEP);
        assert_eq!(g.stance, Stance::Fire);
        let e = &g.enemies[dragon(&g)];
        if e.phase == Phase::Windup {
            seen[e.mv as usize] = true;
        }
        g.player.act = Act::Free;
    }
    assert!(!seen[M_DRAGON_LANCES] && !seen[M_DRAGON_FIELD] && !seen[M_DRAGON_SHARDS]);
    assert!(seen[M_DRAGON_DIVE] || seen[M_DRAGON_POOLS] || seen[M_DRAGON_FLAME_LINE]);
}

#[test]
fn a_stance_ends_with_a_finisher_after_its_time_then_changes() {
    let (mut g, _) = arena();
    g.player.max_hp = 1e7;
    g.player.hp = 1e7;
    let mut finisher_at = None;
    for s in 0..(60 * 60) {
        g.update(STEP);
        g.player.act = Act::Free;
        let e = &g.enemies[dragon(&g)];
        if e.mv as usize == M_DRAGON_FINISHER && e.phase == Phase::Windup && finisher_at.is_none() {
            finisher_at = Some(s as f32 * STEP);
        }
        if g.stance == Stance::Ice {
            break;
        }
    }
    let at = finisher_at.expect("no finisher");
    assert!((at - STANCE_TIME).abs() < 1.0, "finisher at {at} s");
    assert_eq!(g.stance, Stance::Ice);
    assert_eq!(boss(&g).finishers, 1);
}

#[test]
fn passing_two_thirds_health_ends_the_stance() {
    let (mut g, i) = arena();
    g.player.max_hp = 1e7;
    g.player.hp = 1e7;
    steps(&mut g, 60);
    g.enemies[i].hp = KINDS[K_DRAGON as usize].hp * 0.65;
    steps(&mut g, 1);
    let e = &g.enemies[dragon(&g)];
    assert_eq!(e.mv as usize, M_DRAGON_FINISHER);
    assert_eq!(e.phase, Phase::Windup);
}

#[test]
fn finisher_drops_with_weak_element_damage_down_to_a_floor() {
    let full = finisher_damage(100.0, 0.0, true);
    let some = finisher_damage(100.0, WEAK_FULL * 0.5, true);
    let lots = finisher_damage(100.0, WEAK_FULL * 3.0, true);
    assert!(full > some && some > lots);
    assert!((lots - 100.0 * FINISHER_FIRST * FINISHER_FLOOR).abs() < 1e-4);
    assert!(
        full < 100.0,
        "the first finisher never kills from full health"
    );
    assert!(
        finisher_damage(100.0, 0.0, false) >= 100.0,
        "later ones can"
    );
}

/// Run until the finisher lands, rolling all the way through; returns the damage taken from it.
fn take_finisher(weak: bool) -> f32 {
    let (mut g, _) = arena();
    if weak {
        // Fire stance: ice is the weakness.
        g.apply_card(C_ICE, 0);
    }
    g.boss.as_mut().unwrap().stance_t = STANCE_TIME - 0.1;
    steps(&mut g, 10);
    let j = dragon(&g);
    assert_eq!(g.enemies[j].mv as usize, M_DRAGON_FINISHER);
    if weak {
        for _ in 0..30 {
            g.hit_boss_for_test(8.0);
        }
        // Get it back up and into the finisher if the hits knocked it down.
        g.boss.as_mut().unwrap().down = 0.0;
        g.boss.as_mut().unwrap().meter = 0.0;
    }
    let mut taken = 0.0;
    for k in 0..(60 * 4) {
        let hp = g.player.hp;
        g.input = Input {
            mx: 1.0,
            buttons: if k % 20 == 0 { B_ROLL } else { 0 },
            ..Input::default()
        };
        g.update(STEP);
        taken += (hp - g.player.hp).max(0.0);
        if g.stance == Stance::Ice {
            break;
        }
    }
    assert_eq!(g.stance, Stance::Ice, "the finisher did not land");
    assert!(!g.over, "the first finisher killed from full health");
    taken
}

#[test]
fn the_roll_does_not_avoid_the_finisher_and_weak_damage_softens_it() {
    let plain = take_finisher(false);
    let weak = take_finisher(true);
    assert!(plain > 0.0, "rolling avoided the finisher");
    assert!(
        weak < plain,
        "weak-element damage did not soften it: {weak} vs {plain}"
    );
}

#[test]
fn enough_weak_element_damage_knocks_it_down() {
    let (mut g, _) = arena();
    steps(&mut g, 1);
    g.apply_card(C_FIRE, 0);
    // Fire is not the fire stance's weakness.
    for _ in 0..20 {
        g.hit_boss_for_test(10.0);
    }
    g.boss.as_mut().unwrap().meter = 0.0;
    steps(&mut g, 12);
    assert!(boss(&g).down <= 0.0);
    g.apply_card(C_ICE, 0);
    for _ in 0..((WEAK_DOWN / 10.0) as u32 + 1) {
        g.hit_boss_for_test(10.0);
    }
    // Past the hit-stop.
    steps(&mut g, 12);
    assert!(
        boss(&g).down > 0.0,
        "weak-element damage did not knock it down"
    );
}

#[test]
fn rolling_avoids_every_dragon_attack_but_the_finisher() {
    for stance in [Stance::Fire, Stance::Ice] {
        let (mut g, _) = arena();
        g.stance = stance;
        g.player.y = 100.0;
        let mut hits = 0;
        for _ in 0..(60 * 40) {
            // Keep the finisher away; it is not meant to be rolled through.
            g.boss.as_mut().unwrap().stance_t = 0.0;
            let roll = g.threat(0.07);
            let j = dragon(&g);
            let (dx, dy) = (g.enemies[j].x - g.player.x, g.enemies[j].y - g.player.y);
            let d = (dx * dx + dy * dy).sqrt().max(1e-3);
            let line =
                MOVES[g.enemies[j].mv as usize].area == jadereaper_core::skills::AreaKind::Line;
            let (mut mx, mut my) = if line {
                (-dy / d, dx / d)
            } else {
                (dx / d, dy / d)
            };
            let mut buttons = if roll { B_ROLL } else { 0 };
            if let Some(out) = g.pool_escape() {
                // Burning pools are walked out of.
                (mx, my) = out;
                buttons = 0;
            }
            g.input = Input {
                mx,
                my,
                buttons,
                ..Input::default()
            };
            let hp = g.player.hp;
            let in_pool = g.pool_escape().is_some();
            g.update(STEP);
            if g.player.hp < hp && !in_pool {
                hits += 1;
            }
            if roll {
                g.input = Input::default();
                g.update(STEP);
            }
            g.player.hp = g.player.max_hp;
        }
        assert_eq!(hits, 0, "{stance:?}: hit {hits} times");
    }
}

#[test]
fn fire_or_ice_is_offered_before_the_final_boss() {
    for seed in 1..30 {
        let mut g = Game::new(seed);
        g.running = true;
        g.enter_room(N_ROOMS - 2);
        g.enemies.clear();
        g.stage = Stage::Fighting;
        g.wave = ROOMS[N_ROOMS - 2].waves.len() - 1;
        g.update(STEP);
        assert!(g.choosing);
        let offered = g
            .choices
            .iter()
            .any(|&c| c >= 0 && [E_FIRE + 1, E_ICE + 1].contains(&CARDS[c as usize].element));
        assert!(offered, "seed {seed}: {:?}", g.choices);
    }
}
