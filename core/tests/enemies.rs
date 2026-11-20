//! Enemy attacks: warnings come first, and a well-timed roll avoids every one.

use jadereaper_core::defs::*;
use jadereaper_core::game::{Act, Game, Input, MAX_ENEMIES, STEP};
use jadereaper_core::moves::*;

fn arena() -> Game {
    let mut g = Game::new(5);
    g.running = true;
    g.auto_spawn = false;
    g.set_view(240.0, 230.0);
    g
}

#[test]
fn every_move_is_warned_long_enough() {
    for (i, m) in MOVES.iter().enumerate() {
        assert!(m.windup >= MIN_WARNING, "move {i} winds up for {}", m.windup);
    }
}

#[test]
fn hits_on_an_idle_player_were_all_warned() {
    let mut g = Game::new(8);
    g.running = true;
    g.player.max_hp = 1e6;
    g.player.hp = 1e6;
    for _ in 0..(60 * 120) {
        g.update(STEP);
        if g.choosing {
            g.choose(0);
        }
        assert!(g.enemies.len() <= MAX_ENEMIES);
    }
    assert!(g.player.hp < 1e6, "nothing attacked");
    assert!(g.min_warned >= MIN_WARNING - STEP, "a hit came {} s after its warning", g.min_warned);
}

/// Stand still and roll whenever an attack is about to land, through the attacker.
fn dodge(g: &mut Game, secs: f32) {
    for _ in 0..(secs / STEP) as u32 {
        let roll = g.threat(0.07);
        let e = &g.enemies[0];
        let (dx, dy) = (e.x - g.player.x, e.y - g.player.y);
        let d = (dx * dx + dy * dy).sqrt().max(1e-3);
        let (mx, my) = if roll { (dx / d, dy / d) } else { (0.0, 0.0) };
        g.input = Input { mx, my, buttons: if roll { B_ROLL } else { 0 }, ..Input::default() };
        g.update(STEP);
        // Let go so the next threat is a fresh press.
        if roll {
            g.input = Input::default();
            g.update(STEP);
        }
    }
}

#[test]
fn rolling_avoids_every_enemy_attack() {
    for kind in [K_BLOT, K_WISP, K_BRUTE, K_ELITE] {
        let mut g = arena();
        g.add_enemy(kind, 110.0, 30.0);
        g.enemies[0].hp = 1e6;
        dodge(&mut g, 25.0);
        assert_eq!(g.player.hp, g.player.max_hp, "kind {kind} landed a hit");
        assert!(g.events.len() < 1000);
    }
}

#[test]
fn enemies_do_attack() {
    for kind in [K_BLOT, K_WISP, K_BRUTE, K_ELITE] {
        let mut g = arena();
        g.add_enemy(kind, 110.0, 30.0);
        for _ in 0..(60 * 15) {
            g.update(STEP);
        }
        assert!(g.player.hp < g.player.max_hp, "kind {kind} never hit a still player");
    }
}

#[test]
fn a_hit_interrupts_a_wind_up() {
    let mut g = arena();
    g.add_enemy(K_BLOT, 60.0, 0.0);
    // Wait for its wind-up, then cut it.
    let mut n = 0;
    while g.enemies[0].phase != jadereaper_core::game::Phase::Windup {
        g.update(STEP);
        n += 1;
        assert!(n < 600);
    }
    g.input = Input { ax: 100.0, buttons: B_SKILL1, ..Input::default() };
    for _ in 0..12 {
        g.update(STEP);
        g.input.buttons = 0;
    }
    assert_ne!(g.enemies[0].phase, jadereaper_core::game::Phase::Active);
    assert!(matches!(g.player.act, Act::Skill { .. } | Act::Free));
}
