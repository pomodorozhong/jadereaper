//! The boss framework and the Glutton.

use jadereaper_core::boss::*;
use jadereaper_core::defs::*;
use jadereaper_core::elements::Well;
use jadereaper_core::game::{Act, Game, Input, Phase, STEP};
use jadereaper_core::moves::*;
use jadereaper_core::player::GRAB_MASH;

fn arena() -> (Game, usize) {
    let mut g = Game::new(11);
    g.running = true;
    g.auto_spawn = false;
    g.arena = Some([-600.0, -430.0, 600.0, 430.0]);
    let i = g.add_enemy(K_GLUTTON, 0.0, -200.0).unwrap();
    (g, i)
}

fn boss(g: &Game) -> BossState {
    g.boss.expect("a boss")
}

fn glutton(g: &Game) -> usize {
    g.enemies.iter().position(|e| e.kind == K_GLUTTON).unwrap()
}

fn steps(g: &mut Game, n: u32) {
    for _ in 0..n {
        g.update(STEP);
    }
}

#[test]
fn boss_moves_are_warned_long_enough_even_enraged() {
    for &mv in KINDS[K_GLUTTON as usize].moves {
        let m = &MOVES[mv];
        let floor = if m.big { MIN_BIG_WARNING } else { MIN_WARNING };
        for enraged in [false, true] {
            assert!(
                windup_for(m, enraged) >= floor - 1e-6,
                "move {mv} enraged {enraged}"
            );
        }
        assert!(windup_for(m, true) < m.windup || windup_for(m, true) == floor);
    }
}

#[test]
fn enrages_below_sixty_percent_and_stays_enraged() {
    let (mut g, i) = arena();
    steps(&mut g, 2);
    assert!(!boss(&g).enraged());
    let max = KINDS[K_GLUTTON as usize].hp;
    g.enemies[i].hp = max * 0.61;
    steps(&mut g, 2);
    assert!(!boss(&g).enraged());
    let j = glutton(&g);
    g.enemies[j].hp = max * 0.59;
    steps(&mut g, 2);
    assert!(boss(&g).enraged());
    let j = glutton(&g);
    g.enemies[j].hp = max;
    steps(&mut g, 120);
    assert!(boss(&g).enraged(), "it stays enraged");
}

#[test]
fn enrages_after_heavy_damage_in_a_short_time() {
    let (mut g, _) = arena();
    g.player.max_hp = 1e6;
    g.player.hp = 1e6;
    let max = KINDS[K_GLUTTON as usize].hp;
    // Spread out, the same damage does not enrage it.
    for _ in 0..6 {
        g.player.x = 0.0;
        g.player.y = -110.0;
        let j = glutton(&g);
        g.enemies[j].hp = max;
        g.hurt_player(0.0, false, (0.0, 0.0), 1.0);
        let b = g.boss.as_mut().unwrap();
        b.recent += max * 0.03;
        for _ in 0..300 {
            g.update(STEP);
            g.enemies.retain(|e| e.kind == K_GLUTTON);
        }
    }
    assert!(!boss(&g).enraged());
    let b = g.boss.as_mut().unwrap();
    b.recent += max * 0.13;
    steps(&mut g, 1);
    assert!(boss(&g).enraged());
}

#[test]
fn only_enraged_it_uses_breath_and_grab() {
    let (mut g, _) = arena();
    g.player.max_hp = 1e6;
    g.player.hp = 1e6;
    let mut seen = [false; N_MOVES];
    for _ in 0..(60 * 60) {
        g.update(STEP);
        let e = &g.enemies[glutton(&g)];
        if e.phase == Phase::Windup {
            seen[e.mv as usize] = true;
        }
        g.player.act = Act::Free;
        // No blots to eat, which would enrage it too.
        g.enemies.retain(|e| e.kind == K_GLUTTON);
    }
    assert!(!seen[M_GLUTTON_BREATH] && !seen[M_GLUTTON_GRAB]);
    assert!(seen[M_GLUTTON_BITE] && seen[M_GLUTTON_STOMP]);
}

#[test]
fn a_full_stagger_meter_knocks_it_down_and_it_takes_extra_damage() {
    let (mut g, i) = arena();
    g.boss.as_mut().unwrap().meter = KINDS[K_GLUTTON as usize].poise;
    steps(&mut g, 1);
    assert!(boss(&g).down > 0.0);
    // A hit while it is down does more.
    let hp = g.enemies[i].hp;
    g.player.x = g.enemies[i].x;
    g.player.y = g.enemies[i].y + 80.0;
    g.input = Input {
        ax: 0.0,
        ay: -100.0,
        buttons: B_SKILL1,
        ..Input::default()
    };
    let mut dealt = 0.0;
    for _ in 0..20 {
        g.update(STEP);
        g.input.buttons = 0;
        dealt += g
            .events
            .iter()
            .filter(|e| e[0] == EV_HIT)
            .map(|e| e[3])
            .sum::<f32>();
    }
    let base = SKILLS[SK_SCYTHE].damage;
    assert!(
        (hp - g.enemies[i].hp - base * DOWN_DAMAGE).abs() < 0.01,
        "dealt {dealt}"
    );
    steps(&mut g, (DOWN_TIME / STEP) as u32);
    assert!(boss(&g).down <= 0.0, "it gets up");
}

#[test]
fn eating_a_blot_heals_and_angers_and_a_hit_stops_it() {
    let (mut g, i) = arena();
    g.add_enemy(K_BLOT, 80.0, -200.0);
    g.enemies[1].think = 1e9;
    let max = KINDS[K_GLUTTON as usize].hp;
    g.enemies[i].hp = max * 0.8;
    // Force the eat.
    g.enemies[i].mv = M_GLUTTON_EAT as u8;
    g.enemies[i].phase = Phase::Windup;
    g.enemies[i].pt = 0.0;
    g.player.x = 400.0;
    steps(&mut g, (MOVES[M_GLUTTON_EAT].windup / STEP) as u32 + 2);
    assert!(
        g.enemies.iter().all(|e| e.kind != K_BLOT),
        "the blot was eaten"
    );
    assert!(g.enemies[glutton(&g)].hp > max * 0.8);
    assert_eq!(boss(&g).rage, EAT_RAGE);
    assert_eq!(g.kills, 0, "eating is not a kill");

    // Again, but hit it during the wind-up.
    let (mut g, i) = arena();
    g.add_enemy(K_BLOT, 80.0, -200.0);
    g.enemies[1].think = 1e9;
    g.enemies[i].mv = M_GLUTTON_EAT as u8;
    g.enemies[i].phase = Phase::Windup;
    g.enemies[i].pt = 0.0;
    g.player.x = 400.0;
    steps(&mut g, 10);
    g.hit_boss_for_test(1.0);
    steps(&mut g, (MOVES[M_GLUTTON_EAT].windup / STEP) as u32 + 2);
    assert!(
        g.enemies.iter().any(|e| e.kind == K_BLOT),
        "the hit stopped the eating"
    );
}

#[test]
fn mashing_roll_breaks_a_grab() {
    let (mut g, i) = arena();
    g.enemies[i].phase = Phase::Windup;
    g.enemies[i].mv = M_GLUTTON_GRAB as u8;
    g.enemies[i].pt = 0.0;
    g.boss.as_mut().unwrap().phase = 2;
    g.enemies[i].dx = 0.0;
    g.enemies[i].dy = 1.0;
    g.enemies[i].ox = g.enemies[i].x;
    g.enemies[i].oy = g.enemies[i].y;
    g.player.x = g.enemies[i].x;
    g.player.y = g.enemies[i].y + 100.0;
    let mut n = 0;
    while !matches!(g.player.act, Act::Grabbed { .. }) {
        g.update(STEP);
        n += 1;
        assert!(n < 120, "the grab did not land");
    }
    for k in 0..(GRAB_MASH as u32 * 2) {
        g.input = Input {
            buttons: if k % 2 == 0 { B_ROLL } else { 0 },
            ..Input::default()
        };
        g.update(STEP);
    }
    assert!(!matches!(g.player.act, Act::Grabbed { .. }), "still held");
    assert!(g.player.invulnerable());
}

#[test]
fn bosses_are_never_pulled_or_frozen() {
    let (mut g, i) = arena();
    let (x, y) = (g.enemies[i].x, g.enemies[i].y);
    g.wells.push(Well {
        x: x + 100.0,
        y,
        t: 0.8,
        dmg: 1.0,
    });
    g.enemies[i].think = 1e9;
    // Close by, so it stands its ground rather than walking.
    g.player.x = x - 60.0;
    g.player.y = y + 40.0;
    steps(&mut g, 30);
    let e = &g.enemies[glutton(&g)];
    assert!(
        (e.x - x).abs() < 1e-3 && (e.y - y).abs() < 1e-3,
        "pulled to {} {}",
        e.x,
        e.y
    );
    g.apply_card(C_ICE, 0);
    for _ in 0..4 {
        g.hit_boss_for_test(1.0);
    }
    assert_eq!(g.enemies[glutton(&g)].status.frozen, 0.0);
    assert!(boss(&g).meter > 0.0, "ice on a boss adds stagger instead");
}

#[test]
fn a_boss_hit_interrupts_hyper_armor() {
    let (mut g, i) = arena();
    g.loadout[0] = SK_CLEAVE;
    g.player.x = 0.0;
    g.player.y = 0.0;
    g.input = Input {
        ay: -100.0,
        buttons: B_SKILL1,
        ..Input::default()
    };
    g.update(STEP);
    g.input = Input::default();
    steps(&mut g, (SKILLS[SK_CLEAVE].startup / STEP) as u32);
    assert!(g.player.armored());
    let (bx, by) = (g.enemies[i].x, g.enemies[i].y);
    g.hurt_player(5.0, true, (bx, by), 1.0);
    assert!(!matches!(g.player.act, Act::Skill { .. }));
}

#[test]
fn rolling_avoids_every_glutton_attack() {
    for enraged in [false, true] {
        let (mut g, _) = arena();
        if enraged {
            g.boss.as_mut().unwrap().phase = 2;
        }
        g.player.y = 100.0;
        let mut hits = 0;
        for _ in 0..(60 * 40) {
            let roll = g.threat(0.07);
            let j = glutton(&g);
            let (dx, dy) = (g.enemies[j].x - g.player.x, g.enemies[j].y - g.player.y);
            let d = (dx * dx + dy * dy).sqrt().max(1e-3);
            // Roll sideways out of a line, through anything else.
            let line =
                MOVES[g.enemies[j].mv as usize].area == jadereaper_core::skills::AreaKind::Line;
            let (mx, my) = if line {
                (-dy / d, dx / d)
            } else {
                (dx / d, dy / d)
            };
            g.input = Input {
                mx,
                my,
                buttons: if roll { B_ROLL } else { 0 },
                ..Input::default()
            };
            let hp = g.player.hp;
            g.update(STEP);
            if g.player.hp < hp {
                hits += 1;
            }
            if roll {
                g.input = Input::default();
                g.update(STEP);
            }
            g.enemies.retain(|e| e.kind == K_GLUTTON);
            g.player.hp = g.player.max_hp;
        }
        assert_eq!(hits, 0, "enraged {enraged}: hit {hits} times");
    }
}
