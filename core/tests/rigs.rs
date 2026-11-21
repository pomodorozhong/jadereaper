//! Puppet rigs: poses come from the rig data.

use jadereaper_core::defs::*;
use jadereaper_core::game::{Game, Input, Phase, STEP};
use jadereaper_core::rigs::{Doing, Pose};

fn body(g: &Game, part: f32) -> Inst {
    *g.inst
        .iter()
        .find(|i| i.part % PART_SLOTS == part)
        .expect("part drawn")
}

#[test]
fn the_roll_tucks_and_spins_once() {
    let mut g = Game::new(1);
    g.running = true;
    g.auto_spawn = false;
    g.update(STEP);
    let rest = body(&g, P_PLAYER_BODY);
    g.input = Input {
        mx: 1.0,
        buttons: B_ROLL,
        ..Input::default()
    };
    let mut turned = 0.0f32;
    let mut last = rest.rot;
    let mut smallest = rest.hh;
    for _ in 0..((ROLL_TIME / STEP) as u32) {
        g.update(STEP);
        g.input.buttons = 0;
        let b = body(&g, P_PLAYER_BODY);
        turned += (b.rot - last).abs();
        last = b.rot;
        smallest = smallest.min(b.hh);
    }
    assert!(
        (turned - core::f32::consts::TAU).abs() < 0.8,
        "turned {turned}"
    );
    assert!(smallest < rest.hh * 0.85, "did not tuck");
}

#[test]
fn poses_come_from_the_rig() {
    let rig = &RIGS[RG_BLOT];
    let (wind, _) = Game::rig_pose(rig, Doing::Windup { k: 1.0 });
    assert_eq!(wind, rig.windup);
    let (hurt, _) = Game::rig_pose(rig, Doing::Hurt);
    assert_eq!(hurt, rig.hurt);
    let (walk, spin) = Game::rig_pose(
        rig,
        Doing::Walk {
            step: true,
            lean: 0.1,
            hop: -3.0,
        },
    );
    assert_eq!(spin, 0.0);
    assert_eq!(
        walk,
        Pose {
            tilt: 0.1,
            ..Pose::REST
        }
    );
}

#[test]
fn an_enemy_winding_up_is_drawn_in_its_windup_pose() {
    let mut g = Game::new(2);
    g.running = true;
    g.auto_spawn = false;
    g.add_enemy(K_BRUTE, 100.0, 0.0);
    let mut n = 0;
    while !(g.enemies[0].phase == Phase::Windup
        && g.enemies[0].pt > MOVES[M_BRUTE_SLAM].windup * 0.5)
    {
        g.update(STEP);
        n += 1;
        assert!(n < 600);
    }
    let b = body(&g, P_BRUTE);
    let want = RIGS[RG_BRUTE].windup.tilt * g.enemies[0].face;
    assert!((b.rot - want).abs() < 1e-3, "rot {} want {want}", b.rot);
    assert!((b.hh - 30.0 * RIGS[RG_BRUTE].windup.sy).abs() < 1e-3);
}
