//! The camera: look-ahead, arena clamp, widening for a focus point, and reduced motion.

use jadereaper_core::defs::*;
use jadereaper_core::game::{Game, Input, STEP};

fn game() -> Game {
    let mut g = Game::new(1);
    g.running = true;
    g.auto_spawn = false;
    g.set_view(240.0, 230.0);
    g
}

fn settle(g: &mut Game, input: Input) {
    for _ in 0..120 {
        g.input = input;
        g.update(STEP);
    }
}

#[test]
fn leads_towards_the_aim() {
    let mut g = game();
    settle(&mut g, Input { ax: 200.0, ..Input::default() });
    let lead = g.cam.0 - g.player.x;
    assert!(lead > 50.0 && lead < 90.0, "lead {lead}");
}

#[test]
fn stays_inside_the_arena() {
    let mut g = game();
    g.arena = Some([-300.0, -300.0, 300.0, 300.0]);
    g.player.x = 290.0;
    settle(&mut g, Input { ax: 200.0, ..Input::default() });
    assert!(g.cam.0 + g.view_w <= 300.0 + 60.0 + 0.01, "camera right edge {}", g.cam.0 + g.view_w);
}

#[test]
fn widens_to_keep_a_focus_in_view() {
    let mut g = game();
    g.focus = Some((700.0, 0.0));
    settle(&mut g, Input::default());
    assert!(g.zoom > 1.2, "zoom {}", g.zoom);
    g.focus = None;
    settle(&mut g, Input::default());
    assert!(g.zoom < 1.05);
}

#[test]
fn reduce_motion_turns_off_shake_and_hit_stop() {
    let mut g = game();
    g.reduce_motion = true;
    let i = g.add_enemy(K_BLOT, 40.0, 0.0).unwrap();
    g.enemies[i].hp = 1e6;
    g.input = Input { ax: 100.0, buttons: B_SKILL1, ..Input::default() };
    for _ in 0..20 {
        g.update(STEP);
        assert_eq!(g.hitstop, 0.0);
        g.input.buttons = 0;
    }
    g.hurt_player(5.0, false, (50.0, 0.0));
    let before = g.player.y - 30.0;
    g.update(STEP);
    assert!((g.cam.1 - before).abs() < 5.0);
}
