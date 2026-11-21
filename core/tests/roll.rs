//! The roll and the input buffer.

use jadereaper_core::defs::*;
use jadereaper_core::game::{Act, Game, Input, STEP};

fn game() -> Game {
    let mut g = Game::new(1);
    g.running = true;
    g
}

fn steps(g: &mut Game, n: u32, input: Input) {
    for _ in 0..n {
        g.input = input;
        g.update(STEP);
    }
}

const ROLL: Input = Input { mx: 1.0, my: 0.0, ax: 0.0, ay: 0.0, buttons: B_ROLL };
const RIGHT: Input = Input { mx: 1.0, my: 0.0, ax: 0.0, ay: 0.0, buttons: 0 };

fn secs(t: f32) -> u32 {
    (t / STEP).round() as u32
}

#[test]
fn roll_is_invulnerable_then_recovers() {
    let mut g = game();
    steps(&mut g, 1, ROLL);
    assert!(matches!(g.player.act, Act::Roll { .. }));
    assert!(g.player.invulnerable());
    steps(&mut g, secs(ROLL_INVULN) - 2, ROLL);
    assert!(g.player.invulnerable(), "still invulnerable just before the window ends");
    steps(&mut g, 2, RIGHT);
    assert!(!g.player.invulnerable(), "vulnerable after the window");
    assert!(matches!(g.player.act, Act::Roll { .. }), "still rolling or recovering");
    steps(&mut g, secs(ROLL_TIME + ROLL_RECOVER - ROLL_INVULN), RIGHT);
    assert_eq!(g.player.act, Act::Free);
}

#[test]
fn roll_covers_its_distance() {
    let mut g = game();
    let x0 = g.player.x;
    steps(&mut g, 1, ROLL);
    steps(&mut g, secs(ROLL_TIME), Input::default());
    assert!((g.player.x - x0 - ROLL_DIST).abs() < 1.0, "rolled {}", g.player.x - x0);
}

#[test]
fn roll_cannot_be_chained_during_recovery() {
    let mut g = game();
    steps(&mut g, 1, ROLL);
    steps(&mut g, secs(ROLL_TIME) + 1, RIGHT);
    // A fresh press during the recovery is buffered, but only acts once the recovery ends.
    steps(&mut g, 1, ROLL);
    let Act::Roll { t, .. } = g.player.act else { panic!("expected the first roll's recovery") };
    assert!(t > ROLL_TIME, "a new roll started during the recovery");
}

#[test]
fn press_just_before_recovery_ends_is_buffered() {
    let mut g = game();
    steps(&mut g, 1, ROLL);
    // Press 0.1 s before the recovery ends: within the buffer.
    let total = secs(ROLL_TIME + ROLL_RECOVER);
    steps(&mut g, total - 1 - secs(0.1), RIGHT);
    steps(&mut g, 1, ROLL);
    steps(&mut g, secs(0.1) + 1, ROLL);
    let Act::Roll { t, .. } = g.player.act else { panic!("the buffered roll did not start") };
    assert!(t < 0.1);
}

#[test]
fn press_too_early_is_dropped() {
    let mut g = game();
    steps(&mut g, 1, ROLL);
    steps(&mut g, 3, RIGHT);
    // Press well before the roll ends, then let go: the press has expired by the time the body is free.
    steps(&mut g, 1, ROLL);
    steps(&mut g, secs(ROLL_TIME + ROLL_RECOVER) + 2, RIGHT);
    assert_eq!(g.player.act, Act::Free);
}

#[test]
fn aim_follows_the_aim_point() {
    let mut g = game();
    steps(&mut g, 1, Input { ax: 0.0, ay: -50.0, ..Input::default() });
    assert!((g.player.aim.1 + 1.0).abs() < 1e-5);
    // Zero aim keeps the last one.
    steps(&mut g, 1, Input::default());
    assert!((g.player.aim.1 + 1.0).abs() < 1e-5);
}
