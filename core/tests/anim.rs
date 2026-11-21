//! The animation state the core exports for the renderer.

use jadereaper_core::anim::*;
use jadereaper_core::defs::*;
use jadereaper_core::game::{Game, Input, STEP};
use jadereaper_core::skills::*;

fn arena() -> Game {
    let mut g = Game::new(1);
    g.running = true;
    g.auto_spawn = false;
    g.reduce_motion = true; // no hit-stop, so steps are not frozen
    g
}

fn run(g: &mut Game, n: u32, input: Input) {
    for _ in 0..n {
        g.input = input;
        g.update(STEP);
    }
}

#[test]
fn a_skill_goes_through_startup_active_and_recovery_with_its_cancel_point() {
    let d = SKILLS[SK_CLEAVE];
    let mut g = arena();
    g.loadout[0] = SK_CLEAVE;
    run(&mut g, 1, Input { ay: 100.0, buttons: 1, ..Input::default() });
    let mut seen = vec![];
    let mut last_k = -1.0;
    while g.player.act != jadereaper_core::game::Act::Free {
        let a = g.anims[0];
        assert_eq!(a.who, 0.0);
        if seen.last() != Some(&a.phase) {
            seen.push(a.phase);
            last_k = -1.0;
        }
        assert!(a.k >= last_k && (0.0..=1.0).contains(&a.k), "k {} after {last_k}", a.k);
        last_k = a.k;
        assert!((a.dy - 1.0).abs() < 1e-3, "points where the skill goes");
        if a.phase == A_STARTUP {
            assert!((a.len - d.startup).abs() < 1e-4);
        }
        if a.phase == A_RECOVERY {
            assert!((a.cancel - d.cancel / d.recovery).abs() < 1e-4);
        } else {
            assert_eq!(a.cancel, -1.0);
        }
        run(&mut g, 1, Input::default());
    }
    assert_eq!(seen, [A_STARTUP, A_ACTIVE, A_RECOVERY]);
    assert_eq!(g.anims[0].phase, A_IDLE);
}

#[test]
fn every_character_has_a_record_and_hits_record_their_direction() {
    let mut g = arena();
    let i = g.add_enemy(K_BRUTE, 60.0, 0.0).unwrap();
    g.enemies[i].hp = 1e6;
    g.enemies[i].stun = 10.0;
    run(&mut g, 1, Input::default());
    assert_eq!(g.anims.len(), 2);
    let e = g.anims[1];
    assert_eq!(e.who, 1.0 + K_BRUTE as f32);
    assert_eq!(e.id, g.enemies[0].id as f32);
    assert_eq!(e.phase, A_STUNNED);
    assert!(e.hit_age > 1000.0, "never hit yet");

    run(&mut g, 1, Input { ax: 60.0, buttons: 1, ..Input::default() });
    run(&mut g, 10, Input::default());
    let e = g.anims[1];
    assert!(e.hit_dx > 0.9 && e.hit_dmg > 0.0, "hit from the left: ({}, {})", e.hit_dx, e.hit_dy);
    assert!(e.hit_age < 0.2);

    assert!(g.hurt_player(5.0, false, (g.player.x, g.player.y - 30.0), 1.0));
    run(&mut g, 1, Input::default());
    let p = g.anims[0];
    assert_eq!(p.phase, A_HURT);
    assert!(p.hit_dy > 0.9 && p.hit_dmg == 5.0, "pushed away from an attacker above");
}

#[test]
fn the_record_is_its_documented_size() {
    assert_eq!(core::mem::size_of::<Anim>(), ANIM_FLOATS * 4);
}
