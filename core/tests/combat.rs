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
    Input {
        buttons: 1 << slot,
        ..Input::default()
    }
}

fn run(g: &mut Game, n: u32, input: Input) {
    for _ in 0..n {
        g.input = input;
        g.update(STEP);
    }
}

#[test]
fn each_skill_takes_its_data_time() {
    for (skill, def) in SKILLS.iter().enumerate() {
        let mut g = arena();
        g.loadout[0] = skill;
        run(&mut g, 1, press(0));
        let mut steps = 1;
        while g.player.act != Act::Free {
            run(&mut g, 1, Input::default());
            steps += 1;
            assert!(steps < 600);
        }
        let want = (def.total() / STEP).round() as i32;
        assert!(
            (steps - want).abs() <= 1,
            "skill {skill}: {steps} steps, data says {want}"
        );
    }
}

#[test]
fn a_skill_use_hits_each_target_once() {
    for skill in [SK_SCYTHE, SK_PULSE, SK_LUNGE, SK_CHARGE, SK_CLEAVE, SK_LEAP] {
        let mut g = arena();
        g.loadout[0] = skill;
        let i = g.add_enemy(K_BRUTE, 60.0, 0.0).unwrap();
        g.enemies[i].hp = 1e6;
        g.enemies[i].stun = 10.0; // stands still
        run(
            &mut g,
            1,
            Input {
                ax: 60.0,
                buttons: 1,
                ..Input::default()
            },
        );
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
    run(
        &mut g,
        1,
        Input {
            ax: 0.0,
            ay: -100.0,
            buttons: 1,
            ..Input::default()
        },
    );
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
    run(
        &mut g,
        1,
        Input {
            buttons: B_ROLL,
            ..Input::default()
        },
    );
    assert!(matches!(g.player.act, Act::Skill { .. }));
    // Pressed again once past the cancel point: the roll starts at once.
    let after = ((d.cancel * 0.8) / STEP) as u32 + 1;
    run(&mut g, after, Input::default());
    run(
        &mut g,
        1,
        Input {
            buttons: B_ROLL,
            ..Input::default()
        },
    );
    assert!(
        matches!(g.player.act, Act::Roll { .. }),
        "{:?}",
        g.player.act
    );
}

#[test]
fn combo_chains_on_repeated_presses() {
    let mut g = arena();
    g.loadout[0] = SK_SCYTHE;
    let mut seen = [false; 3];
    for k in 0..90 {
        run(
            &mut g,
            1,
            if k % 6 == 0 {
                press(0)
            } else {
                Input::default()
            },
        );
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
    // The hit staggers it, or spins it round so it cannot act.
    assert!(g.enemies[0].twist.locked() || g.enemies[0].stagger > 0.0);
    assert!(g.enemies[0].kx > 0.0, "knocked back away from the player");
}

#[test]
fn with_no_aim_skills_go_the_way_it_moves_or_last_moved() {
    let mut g = arena();
    // Aim right with the mouse or stick, then let go of it.
    run(
        &mut g,
        1,
        Input {
            ax: 100.0,
            ..Input::default()
        },
    );
    assert_eq!(g.player.aim, (1.0, 0.0));
    // Moving down with no aim input turns the aim down.
    run(
        &mut g,
        3,
        Input {
            my: 1.0,
            ..Input::default()
        },
    );
    assert!(g.player.aim.1 > 0.99, "{:?}", g.player.aim);
    // Standing still, a skill goes the way it last moved.
    run(&mut g, 20, Input::default());
    run(&mut g, 1, press(0));
    match g.player.act {
        Act::Skill { dy, .. } => assert!(dy > 0.99, "skill aimed {dy}"),
        _ => panic!("no skill"),
    }
    // An aim input still wins over the movement.
    let mut g = arena();
    run(
        &mut g,
        3,
        Input {
            my: 1.0,
            ax: -100.0,
            ..Input::default()
        },
    );
    assert_eq!(g.player.aim, (-1.0, 0.0));
}

#[test]
fn dash_skills_move_their_set_distance() {
    for skill in [SK_LUNGE, SK_CHARGE] {
        let mut g = arena();
        g.loadout[0] = skill;
        run(
            &mut g,
            1,
            Input {
                ax: 0.0,
                ay: 100.0,
                buttons: 1,
                ..Input::default()
            },
        );
        while g.player.act != Act::Free {
            run(&mut g, 1, Input::default());
        }
        let moved = g.player.y;
        assert!(
            (moved - SKILLS[skill].dash).abs() < 1.0,
            "skill {skill} moved {moved}"
        );
    }
}

#[test]
fn leap_lands_on_the_aim_point_within_its_reach() {
    for (aim, want) in [(150.0, 150.0), (900.0, SKILLS[SK_LEAP].dash)] {
        let mut g = arena();
        g.loadout[0] = SK_LEAP;
        run(
            &mut g,
            1,
            Input {
                ax: aim,
                buttons: 1,
                ..Input::default()
            },
        );
        while g.player.act != Act::Free {
            run(&mut g, 1, Input::default());
        }
        assert!(
            (g.player.x - want).abs() < 1.0,
            "aimed {aim}, landed at {}",
            g.player.x
        );
    }
}

#[test]
fn hyper_armor_holds_against_normal_hits_and_breaks_against_bosses() {
    for skill in [SK_CHARGE, SK_CLEAVE, SK_LEAP] {
        assert!(SKILLS[skill].hyper);
        for boss in [false, true] {
            let mut g = arena();
            g.loadout[0] = skill;
            run(&mut g, 1, press(0));
            // Into the active window.
            let n = (SKILLS[skill].startup / STEP) as u32 + 1;
            run(&mut g, n, Input::default());
            let hp = g.player.hp;
            assert!(g.hurt_player(5.0, boss, (g.player.x + 30.0, g.player.y), 1.0));
            assert_eq!(g.player.hp, hp - 5.0, "the damage lands either way");
            let still = matches!(g.player.act, Act::Skill { .. });
            assert_eq!(still, !boss, "skill {skill}, boss {boss}");
        }
    }
    // Light skills have no armor.
    let mut g = arena();
    run(&mut g, 1, press(0));
    run(
        &mut g,
        (SKILLS[SK_SCYTHE].startup / STEP) as u32 + 1,
        Input::default(),
    );
    g.hurt_player(5.0, false, (30.0, 0.0), 1.0);
    assert!(matches!(g.player.act, Act::Hurt { .. }));
}

#[test]
fn beads_orbit_and_rehit_after_a_while() {
    let mut g = arena();
    g.loadout[0] = SK_BEADS;
    let i = g.add_enemy(K_BRUTE, 70.0, 0.0).unwrap();
    g.enemies[i].hp = 1e6;
    g.enemies[i].stun = 10.0;
    run(&mut g, 1, press(0));
    let mut hits = 0;
    for _ in 0..(60 * 3) {
        run(&mut g, 1, Input::default());
        g.enemies[0].x = 70.0;
        g.enemies[0].y = 0.0;
        hits += g.events.iter().filter(|e| e[0] == EV_HIT).count();
    }
    assert!(g.orbit.is_none(), "the orbit ends");
    let most = (SKILLS[SK_BEADS].lasts / ORBIT_REHIT).ceil() as usize + 1;
    assert!(hits >= 2 && hits <= most, "{hits} hits");
}

#[test]
fn a_skill_turns_with_the_aim_while_it_charges_and_locks_when_it_hits() {
    let d = SKILLS[SK_CLEAVE];
    let mut g = arena();
    g.loadout[0] = SK_CLEAVE;
    run(
        &mut g,
        1,
        Input {
            ax: 100.0,
            buttons: 1,
            ..Input::default()
        },
    );
    // Turned straight down halfway through the startup.
    run(
        &mut g,
        (d.startup * 0.5 / STEP) as u32,
        Input {
            ay: 100.0,
            ..Input::default()
        },
    );
    let Act::Skill { dx, dy, .. } = g.player.act else {
        panic!("not using the skill")
    };
    assert!(dx.abs() < 1e-3 && (dy - 1.0).abs() < 1e-3, "({dx}, {dy})");
    // Into the active window, then aimed left: the direction stays down.
    run(
        &mut g,
        (d.startup * 0.5 / STEP) as u32 + 2,
        Input {
            ay: 100.0,
            ..Input::default()
        },
    );
    run(
        &mut g,
        2,
        Input {
            ax: -100.0,
            ..Input::default()
        },
    );
    let Act::Skill { dx, dy, .. } = g.player.act else {
        panic!("not using the skill")
    };
    assert!(
        dx.abs() < 1e-3 && (dy - 1.0).abs() < 1e-3,
        "locked at ({dx}, {dy})"
    );
}

#[test]
fn a_leap_lands_where_the_aim_point_moved_to() {
    let d = SKILLS[SK_LEAP];
    let mut g = arena();
    g.loadout[0] = SK_LEAP;
    run(
        &mut g,
        1,
        Input {
            ax: 200.0,
            buttons: 1,
            ..Input::default()
        },
    );
    run(&mut g, (d.startup * 0.3 / STEP) as u32, Input::default());
    // The host sends the aim point as an offset from where the player now is: the world point (0, 150).
    while matches!(g.player.act, Act::Skill { t, .. } if t < d.startup) {
        let (x, y) = (g.player.x, g.player.y);
        run(
            &mut g,
            1,
            Input {
                ax: -x,
                ay: 150.0 - y,
                ..Input::default()
            },
        );
    }
    assert!(
        g.player.x.abs() < 2.0 && (g.player.y - 150.0).abs() < 2.0,
        "landed at ({}, {})",
        g.player.x,
        g.player.y
    );
}

#[test]
fn the_players_skill_shows_while_it_charges_and_the_ring_marks_the_cancel_point() {
    let d = SKILLS[SK_CLEAVE];
    let jade = jadereaper_core::elements::element_color(0);
    let warn = |g: &Game, part: f32| {
        g.inst
            .iter()
            .filter(|i| i.part % PART_SLOTS == part)
            .map(|i| (i.color, i.flash))
            .collect::<Vec<_>>()
    };
    let mut g = arena();
    g.loadout[0] = SK_CLEAVE;
    run(
        &mut g,
        1,
        Input {
            ax: 100.0,
            buttons: 1,
            ..Input::default()
        },
    );
    run(&mut g, (d.startup * 0.5 / STEP) as u32, Input::default());
    let arc = warn(&g, P_WARN_ARC);
    assert_eq!(arc.len(), 1, "one wash for the coming cleave");
    assert_eq!(arc[0].0[..3], jade, "in the skill's colour");
    assert!(
        (arc[0].1 - 0.5).abs() < 0.05,
        "half filled halfway through the startup, got {}",
        arc[0].1
    );
    // Early in the recovery: an ink ring, filling towards the cancel point.
    run(
        &mut g,
        ((d.startup * 0.5 + d.active + d.cancel * 0.5) / STEP) as u32,
        Input::default(),
    );
    let ring = warn(&g, P_WARN_RING);
    assert_eq!(ring.len(), 1);
    assert_eq!(ring[0].0[..3], INK);
    // Past the cancel point: the ring turns to the skill's colour.
    run(&mut g, (d.cancel * 0.5 / STEP) as u32 + 2, Input::default());
    let ring = warn(&g, P_WARN_RING);
    assert_eq!(ring[0].0[..3], jade);
    run(&mut g, (d.recovery / STEP) as u32, Input::default());
    assert!(
        warn(&g, P_WARN_RING).is_empty() && warn(&g, P_WARN_ARC).is_empty(),
        "gone once the skill ends"
    );
}
