//! Skill timing, hit areas and the roll cancel.

use jadereaper_core::defs::*;
use jadereaper_core::game::{Act, Game, Input, STEP};
use jadereaper_core::skills::*;

pub fn arena() -> Game {
    let mut g = Game::new(1);
    g.running = true;
    g.auto_spawn = false;
    g
}

fn press(slot: usize) -> Input {
    Input { buttons: 1 << slot, ..Input::default() }
}

fn run(g: &mut Game, n: u32, input: Input) {
    for _ in 0..n {
        g.input = input;
        g.update(STEP);
    }
}

#[test]
fn each_skill_takes_its_data_time() {
    for skill in 0..N_SKILLS {
        let mut g = arena();
        g.loadout[0] = skill;
        run(&mut g, 1, press(0));
        let mut steps = 1;
        while g.player.act != Act::Free {
            run(&mut g, 1, Input::default());
            steps += 1;
            assert!(steps < 600);
        }
        let want = (SKILLS[skill].total() / STEP).round() as i32;
        assert!((steps as i32 - want).abs() <= 1, "skill {skill}: {steps} steps, data says {want}");
    }
}

#[test]
fn a_skill_use_hits_each_target_once() {
    for skill in [SK_SCYTHE, SK_PULSE] {
        let mut g = arena();
        g.loadout[0] = skill;
        let i = g.add_enemy(K_BRUTE, 60.0, 0.0).unwrap();
        g.enemies[i].hp = 1e6;
        g.enemies[i].stun = 10.0; // stands still
        g.player.aim = (1.0, 0.0);
        run(&mut g, 1, press(0));
        let mut hits = 0;
        for _ in 0..60 {
            g.update(STEP);
            hits += g.events.iter().filter(|e| e[0] == EV_HIT).count();
            g.input = Input::default();
        }
        assert_eq!(hits, 1, "skill {skill}");
    }
}

#[test]
fn shot_flies_at_the_aim() {
    let mut g = arena();
    g.loadout[0] = SK_TALISMAN;
    let i = g.add_enemy(K_BLOT, 0.0, -300.0).unwrap();
    g.add_enemy(K_BLOT, 300.0, 0.0);
    g.enemies[i].hp = 1e6;
    run(&mut g, 1, Input { ax: 0.0, ay: -100.0, buttons: 1, ..Input::default() });
    let mut hit = false;
    for _ in 0..90 {
        run(&mut g, 1, Input::default());
        hit |= g.events.iter().any(|e| e[0] == EV_HIT && e[2] < -100.0);
    }
    assert!(hit);
}

#[test]
fn roll_cancels_recovery_only_after_the_cancel_point() {
    let d = SKILLS[SK_PULSE];
    let mut g = arena();
    g.loadout[0] = SK_PULSE;
    run(&mut g, 1, press(0));
    // Roll pressed in the recovery but before the cancel point: held in the buffer, not yet rolling.
    let before = ((d.startup + d.active + d.cancel * 0.3) / STEP) as u32;
    run(&mut g, before, Input::default());
    run(&mut g, 1, Input { buttons: B_ROLL, ..Input::default() });
    assert!(matches!(g.player.act, Act::Skill { .. }));
    // Pressed again once past the cancel point: the roll starts at once.
    let after = ((d.cancel * 0.8) / STEP) as u32 + 1;
    run(&mut g, after, Input::default());
    run(&mut g, 1, Input { buttons: B_ROLL, ..Input::default() });
    assert!(matches!(g.player.act, Act::Roll { .. }), "{:?}", g.player.act);
}

#[test]
fn combo_chains_on_repeated_presses() {
    let mut g = arena();
    g.loadout[0] = SK_SCYTHE;
    let mut seen = [false; 3];
    for k in 0..90 {
        run(&mut g, 1, if k % 6 == 0 { press(0) } else { Input::default() });
        if let Act::Skill { step, .. } = g.player.act {
            seen[step as usize] = true;
        }
    }
    assert_eq!(seen, [true; 3]);
}

#[test]
fn hits_stagger_and_freeze() {
    let mut g = arena();
    let i = g.add_enemy(K_BLOT, 40.0, 0.0).unwrap();
    g.enemies[i].hp = 1e6;
    run(&mut g, 1, press(0));
    run(&mut g, 8, Input::default());
    assert!(g.enemies[0].stun > 0.0 || g.enemies[0].stagger > 0.0);
    assert!(g.enemies[0].kx > 0.0, "knocked back away from the player");
}
