//! Behaviour tests that pin down how the weapons, passives and level-up
//! choices work today. They document current numbers, oddities included, so a
//! rewrite can be checked against them. Included from `game.rs` under
//! `#[cfg(test)]` so the tests can reach private fields.

use super::*;

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

/// A running game at time 0 with every weapon timer parked far away, so a test
/// only sees the weapon it arms. Scythe is at level 1 (as in a real run).
fn base() -> Game {
    let mut g = Game::new(1);
    g.running = true;
    g.t_scythe = 100.0;
    g.t_talisman = 100.0;
    g.t_pulse = 100.0;
    g
}

/// Place a tanky enemy (hp 1000) so damage is the hp difference.
fn put(g: &mut Game, kind: u8, x: f32, y: f32) -> usize {
    let id = g.enemies.len() as u32 + 1;
    g.enemies.push(Enemy {
        x,
        y,
        kx: 0.0,
        ky: 0.0,
        hp: 1000.0,
        kind,
        flash: 0.0,
        seed: 0.5,
        bead_cd: 0.0,
        id,
        face: 1.0,
    });
    g.enemies.len() - 1
}

fn dealt(g: &Game, i: usize) -> f32 {
    1000.0 - g.enemies[i].hp
}

// ------------------------------------------------------------------ scythe

#[test]
fn scythe_level1_reach_arc_and_damage() {
    let mut g = base();
    assert_eq!(g.up[U_SCYTHE], 1);
    // Reach is 105 + 14 = 119 plus the blot radius 16: hit below 135.
    let near_e = put(&mut g, K_BLOT, 134.0, 0.0);
    let far_e = put(&mut g, K_BLOT, 136.0, 0.0);
    // Aim is (1, 0). Directly behind is outside the arc (cos -1 < -0.15).
    let behind = put(&mut g, K_BLOT, -100.0, 0.0);
    // Sideways is inside the arc (cos 0 > -0.15).
    let side = put(&mut g, K_BLOT, 0.0, 100.0);
    // Very close enemies are hit even behind the player.
    let close_behind = put(&mut g, K_BLOT, -20.0, 0.0);
    g.t_scythe = 0.0;
    g.weapons(0.016);
    assert!(near(dealt(&g, near_e), 20.0)); // 14 + 6 * 1
    assert!(near(dealt(&g, far_e), 0.0));
    assert!(near(dealt(&g, behind), 0.0));
    assert!(near(dealt(&g, side), 20.0));
    assert!(near(dealt(&g, close_behind), 20.0));
    // Knockback pushes away from the player at 240.
    assert!(near(g.enemies[near_e].kx, 240.0));
    // One slash effect and an attack pose.
    assert_eq!(g.fx.len(), 1);
    assert!(near(g.player.attack, 0.25));
    assert!(g.events.iter().any(|e| e[0] == EV_SLASH));
}

#[test]
fn scythe_max_level_reach_full_circle_and_damage() {
    let mut g = base();
    g.up[U_SCYTHE] = UPGRADE_MAX[U_SCYTHE];
    // Reach is 105 + 84 = 189 plus 16: hit below 205.
    let near_e = put(&mut g, K_BLOT, 204.0, 0.0);
    let far_e = put(&mut g, K_BLOT, 206.0, 0.0);
    // Level 5 and up hit all around, including directly behind.
    let behind = put(&mut g, K_BLOT, -150.0, 0.0);
    g.t_scythe = 0.0;
    g.weapons(0.016);
    assert!(near(dealt(&g, near_e), 50.0)); // 14 + 6 * 6
    assert!(near(dealt(&g, far_e), 0.0));
    assert!(near(dealt(&g, behind), 50.0));
    // Two slash arcs at the top levels.
    assert_eq!(g.fx.len(), 2);
}

#[test]
fn scythe_full_circle_starts_at_level_5() {
    for (lvl, expect_hit) in [(4u8, false), (5u8, true)] {
        let mut g = base();
        g.up[U_SCYTHE] = lvl;
        let behind = put(&mut g, K_BLOT, -100.0, 0.0);
        g.t_scythe = 0.0;
        g.weapons(0.016);
        assert_eq!(dealt(&g, behind) > 0.0, expect_hit, "level {lvl}");
    }
}

#[test]
fn scythe_follows_aim_direction() {
    let mut g = base();
    g.player.aim = (0.0, 1.0);
    let down = put(&mut g, K_BLOT, 0.0, 100.0);
    let up = put(&mut g, K_BLOT, 0.0, -100.0);
    g.t_scythe = 0.0;
    g.weapons(0.016);
    assert!(near(dealt(&g, down), 20.0));
    assert!(near(dealt(&g, up), 0.0));
}

#[test]
fn scythe_cooldown_is_0_9_seconds() {
    let mut g = base();
    let e = put(&mut g, K_BLOT, 50.0, 0.0);
    g.t_scythe = 0.0;
    g.weapons(0.016);
    assert!(near(g.t_scythe, 0.9));
    assert!(near(dealt(&g, e), 20.0));
    g.weapons(0.5); // 0.4 left, no swing
    assert!(near(dealt(&g, e), 20.0));
    g.weapons(0.5); // expired, swings again
    assert!(near(dealt(&g, e), 40.0));
}

#[test]
fn scythe_knockback_is_reduced_on_heavy_enemies() {
    let mut g = base();
    let blot = put(&mut g, K_BLOT, 50.0, 0.0);
    let brute = put(&mut g, K_BRUTE, 50.0, 0.0);
    g.t_scythe = 0.0;
    g.weapons(0.016);
    assert!(near(g.enemies[blot].kx, 240.0));
    assert!(near(g.enemies[brute].kx, 240.0 * 0.35));
}

// ---------------------------------------------------------------- talisman

#[test]
fn talisman_is_off_until_taken() {
    let mut g = base();
    put(&mut g, K_BLOT, 100.0, 0.0);
    assert_eq!(g.up[U_TALISMAN], 0);
    g.t_talisman = 0.0;
    g.weapons(0.016);
    assert!(g.shots.is_empty());
}

#[test]
fn talisman_level1_one_shot_damage_pierce_and_cooldown() {
    let mut g = base();
    g.up[U_TALISMAN] = 1;
    let e = put(&mut g, K_BLOT, 100.0, 0.0);
    put(&mut g, K_BLOT, 0.0, 200.0);
    g.t_talisman = 0.0;
    g.weapons(0.016);
    assert_eq!(g.shots.len(), 1); // 1 + 1 / 2
    let s = &g.shots[0];
    assert!(near(s.dmg, 12.0)); // 8 + 4 * 1
    assert_eq!(s.pierce, 1); // 1 + 1 / 3
    assert!(near(s.vx, 540.0) && near(s.vy, 0.0)); // towards the nearest
    assert!(near(s.life, 1.3));
    assert!(near(g.t_talisman, 0.95));
    assert!(g.events.iter().any(|e| e[0] == EV_SHOOT));
    // The shot hits and, with pierce 1, is used up.
    for _ in 0..10 {
        g.move_shots(0.05);
    }
    assert!(near(dealt(&g, e), 12.0));
    assert!(g.shots.is_empty());
}

#[test]
fn talisman_max_level_four_shots_and_pierce() {
    let mut g = base();
    g.up[U_TALISMAN] = UPGRADE_MAX[U_TALISMAN];
    // Five enemies, only the four nearest are targeted.
    for k in 1..=5 {
        put(&mut g, K_BLOT, 100.0 * k as f32, 0.0);
    }
    g.t_talisman = 0.0;
    g.weapons(0.016);
    assert_eq!(g.shots.len(), 4); // 1 + 6 / 2 = 4
    for s in &g.shots {
        assert!(near(s.dmg, 32.0)); // 8 + 4 * 6
        assert_eq!(s.pierce, 3); // 1 + 6 / 3
    }
}

#[test]
fn talisman_max_level_pierces_through_enemies() {
    let mut g = base();
    g.up[U_TALISMAN] = UPGRADE_MAX[U_TALISMAN];
    let a = put(&mut g, K_BLOT, 100.0, 0.0);
    g.t_talisman = 0.0;
    g.weapons(0.016);
    assert_eq!(g.shots.len(), 1);
    let b = put(&mut g, K_BLOT, 250.0, 0.0);
    for _ in 0..40 {
        g.move_shots(0.016);
    }
    // Hit both, never the same enemy twice in a row.
    assert!(near(dealt(&g, a), 32.0));
    assert!(near(dealt(&g, b), 32.0));
}

#[test]
fn talisman_range_and_idle_retry() {
    let mut g = base();
    g.up[U_TALISMAN] = 1;
    put(&mut g, K_BLOT, 521.0, 0.0); // just out of the 520 range
    g.t_talisman = 0.0;
    g.weapons(0.016);
    assert!(g.shots.is_empty());
    assert!(near(g.t_talisman, 0.15)); // retries soon instead of 0.95
    put(&mut g, K_BLOT, 519.0, 0.0);
    g.t_talisman = 0.0;
    g.weapons(0.016);
    assert_eq!(g.shots.len(), 1);
}

// ------------------------------------------------------------------- beads

#[test]
fn beads_are_off_until_taken() {
    let mut g = base();
    assert_eq!(g.bead_count(), 0);
    let e = put(&mut g, K_BLOT, 84.0, 0.0);
    g.weapons(0.016);
    assert!(near(dealt(&g, e), 0.0));
}

#[test]
fn beads_level1_two_beads_orbit_radius_and_damage() {
    let mut g = base();
    g.up[U_BEADS] = 1;
    assert_eq!(g.bead_count(), 2);
    // At time 0 the beads sit at +/- radius on x (78 + 6 = 84).
    let (b0x, b0y) = g.bead_pos(0, 2);
    let (b1x, b1y) = g.bead_pos(1, 2);
    assert!(near(b0x, 84.0) && near(b0y, 0.0));
    assert!(near(b1x, -84.0) && near(b1y, 0.0));
    let on = put(&mut g, K_BLOT, 84.0, 0.0);
    // Hit radius is enemy radius + 12 = 28.
    let off = put(&mut g, K_BLOT, -84.0 - 29.0, 0.0);
    g.weapons(0.016);
    assert!(near(dealt(&g, on), 9.0)); // 6 + 3 * 1
    assert!(near(dealt(&g, off), 0.0));
    // Knocked away from the player at 160.
    assert!(near(g.enemies[on].kx, 160.0));
    assert!(near(g.enemies[on].bead_cd, 0.45));
}

#[test]
fn beads_per_enemy_cooldown() {
    let mut g = base();
    g.up[U_BEADS] = 1;
    let e = put(&mut g, K_BLOT, 84.0, 0.0);
    g.weapons(0.016);
    g.weapons(0.016);
    assert!(near(dealt(&g, e), 9.0)); // second call blocked by bead_cd
    g.enemies[e].bead_cd = 0.0;
    g.weapons(0.016);
    assert!(near(dealt(&g, e), 18.0));
}

#[test]
fn beads_max_level_seven_beads_and_damage() {
    let mut g = base();
    g.up[U_BEADS] = UPGRADE_MAX[U_BEADS];
    assert_eq!(g.bead_count(), 7); // 1 + 6
    let radius = 78.0 + 36.0;
    let mut idx = Vec::new();
    for k in 0..7 {
        let (bx, by) = g.bead_pos(k, 7);
        // Orbit is squashed vertically to 0.8.
        let a = k as f32 * TAU / 7.0;
        assert!(near(bx, a.cos() * radius) && near(by, a.sin() * radius * 0.8));
        idx.push(put(&mut g, K_BLOT, bx, by));
    }
    g.weapons(0.016);
    for i in idx {
        assert!(near(dealt(&g, i), 24.0), "bead target {i}"); // 6 + 3 * 6
    }
}

#[test]
fn beads_orbit_moves_with_time() {
    let mut g = base();
    g.up[U_BEADS] = 1;
    g.time = 1.0;
    let (x, y) = g.bead_pos(0, 2);
    assert!(near(x, 84.0 * 2.8f32.cos()));
    assert!(near(y, 84.0 * 2.8f32.sin() * 0.8));
}

// ------------------------------------------------------------------- pulse

#[test]
fn pulse_is_off_until_taken() {
    let mut g = Game::new(1);
    g.running = true;
    let e = put(&mut g, K_BLOT, 50.0, 0.0);
    g.t_scythe = 100.0;
    g.weapons(10.0);
    assert!(near(dealt(&g, e), 0.0));
}

#[test]
fn pulse_level1_first_fire_radius_damage_and_cooldown() {
    let mut g = base();
    g.up[U_PULSE] = 1;
    g.t_pulse = Game::new(1).t_pulse;
    assert!(near(g.t_pulse, 2.0)); // first pulse after 2 seconds
    let inside = put(&mut g, K_BLOT, 179.0, 0.0); // radius 150 + 30
    let outside = put(&mut g, K_BLOT, 181.0, 0.0);
    g.weapons(1.9);
    assert!(near(dealt(&g, inside), 0.0));
    g.weapons(0.2);
    assert!(near(dealt(&g, inside), 16.0)); // 10 + 6 * 1
    assert!(near(dealt(&g, outside), 0.0));
    assert!(near(g.enemies[inside].kx, 520.0));
    assert!(near(g.t_pulse, 4.2));
    assert!(g.events.iter().any(|e| e[0] == EV_PULSE));
    // Not again until 4.2 seconds pass.
    g.weapons(4.0);
    assert!(near(dealt(&g, inside), 16.0));
    g.weapons(0.3);
    assert!(near(dealt(&g, inside), 32.0));
}

#[test]
fn pulse_max_level_radius_and_damage() {
    let mut g = base();
    g.up[U_PULSE] = UPGRADE_MAX[U_PULSE];
    let inside = put(&mut g, K_BLOT, 329.0, 0.0); // radius 150 + 180
    let outside = put(&mut g, K_BLOT, 331.0, 0.0);
    g.t_pulse = 0.0;
    g.weapons(0.016);
    assert!(near(dealt(&g, inside), 46.0)); // 10 + 6 * 6
    assert!(near(dealt(&g, outside), 0.0));
    // The ring effect grows out to the radius in 0.35 seconds.
    assert_eq!(g.fx.len(), 1);
    assert!(near(g.fx[0].grow, (330.0 - 20.0) / 0.35));
}

#[test]
fn pulse_hits_in_every_direction() {
    let mut g = base();
    g.up[U_PULSE] = 1;
    let pts = [(100.0, 0.0), (-100.0, 0.0), (0.0, 100.0), (0.0, -100.0)];
    let idx: Vec<usize> = pts.iter().map(|&(x, y)| put(&mut g, K_BLOT, x, y)).collect();
    g.t_pulse = 0.0;
    g.weapons(0.016);
    for i in idx {
        assert!(near(dealt(&g, i), 16.0));
    }
}

// ---------------------------------------------------------------- passives

#[test]
fn swift_raises_move_speed() {
    let run = |lvl: u8| {
        let mut g = base();
        g.up[U_SWIFT] = lvl;
        g.input = (1.0, 0.0);
        for _ in 0..10 {
            g.step(0.05);
        }
        g.player.x
    };
    assert!(near(run(0), 190.0 * 0.5)); // 190 * 0.5 s
    assert!(near(run(5), 190.0 * 1.4 * 0.5)); // +8% per level
}

#[test]
fn swift_does_not_exceed_unit_input() {
    let mut g = base();
    g.input = (3.0, 4.0); // clamped to length 1
    g.step(0.05);
    let d = (g.player.x * g.player.x + g.player.y * g.player.y).sqrt();
    assert!(near(d, 190.0 * 0.05));
}

#[test]
fn lure_widens_gem_pull_radius() {
    let gem = |x: f32| Gem { x, y: 0.0, value: 1.0, seed: 0.0, pull: false, heart: false };
    // Level 0: pull radius 95.
    let mut g = base();
    g.gems.push(gem(100.0));
    g.gems.push(gem(90.0));
    g.move_gems(0.016);
    assert!(!g.gems[0].pull);
    assert!(g.gems[1].pull);
    // Level 5: 95 * 2.5 = 237.5.
    let mut g = base();
    g.up[U_LURE] = 5;
    g.gems.push(gem(230.0));
    g.gems.push(gem(240.0));
    g.move_gems(0.016);
    assert!(g.gems[0].pull);
    assert!(!g.gems[1].pull);
    // A pulled gem moves 460 per second towards the player.
    assert!(near(g.gems[0].x, 230.0 - 460.0 * 0.016));
}

#[test]
fn might_scales_weapon_damage() {
    let mut g = base();
    g.up[U_MIGHT] = 5; // x1.6
    g.up[U_PULSE] = 1;
    g.up[U_BEADS] = 1;
    g.up[U_TALISMAN] = 1;
    let e = put(&mut g, K_BLOT, 84.0, 0.0);
    g.t_scythe = 0.0;
    g.t_pulse = 0.0;
    g.t_talisman = 0.0;
    g.weapons(0.016);
    // scythe 20 + pulse 16 + bead 9, each x1.6
    assert!(near(dealt(&g, e), (20.0 + 16.0 + 9.0) * 1.6));
    assert!(near(g.shots[0].dmg, 12.0 * 1.6));
}

#[test]
fn haste_shortens_cooldowns() {
    let mut g = base();
    g.up[U_HASTE] = 5; // 0.92 ^ 5 per cooldown
    g.up[U_TALISMAN] = 1;
    g.up[U_PULSE] = 1;
    put(&mut g, K_BLOT, 100.0, 0.0);
    g.t_scythe = 0.0;
    g.t_talisman = 0.0;
    g.t_pulse = 0.0;
    g.weapons(0.016);
    let h = 0.92f32.powi(5);
    assert!(near(g.t_scythe, 0.9 * h));
    assert!(near(g.t_talisman, 0.95 * h));
    assert!(near(g.t_pulse, 4.2 * h));
}

#[test]
fn mending_regenerates_health() {
    let run = |lvl: u8| {
        let mut g = base();
        g.up[U_MENDING] = lvl;
        g.player.hp = 50.0;
        for _ in 0..10 {
            g.step(0.05); // 0.5 s, short enough that nothing spawns
        }
        assert!(g.enemies.is_empty());
        g.player.hp
    };
    assert!(near(run(0), 50.0));
    assert!(near(run(5), 51.0)); // 0.4 * 5 per second
}

#[test]
fn mending_does_not_overheal() {
    let mut g = base();
    g.up[U_MENDING] = 5;
    g.player.hp = 99.9;
    g.step(0.05);
    assert!(near(g.player.hp, 100.0));
}

#[test]
fn vitality_raises_max_and_current_health_on_choose() {
    let mut g = base();
    g.player.hp = 50.0;
    g.choosing = true;
    g.pending = 1;
    g.choices = [U_VITALITY as i32, -1, -1];
    g.choose(0);
    assert_eq!(g.up[U_VITALITY], 1);
    assert!(near(g.player.max_hp, 120.0));
    assert!(near(g.player.hp, 70.0));
}

// ---------------------------------------------------------------- level up

#[test]
fn passive_and_weapon_ids_and_caps() {
    assert_eq!(N_UPGRADES, 10);
    assert_eq!(UPGRADE_MAX, [6, 6, 6, 6, 5, 5, 5, 5, 5, 5]);
    assert_eq!(
        [U_SCYTHE, U_TALISMAN, U_BEADS, U_PULSE, U_VITALITY, U_SWIFT, U_LURE, U_MIGHT, U_HASTE, U_MENDING],
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
    );
    let g = Game::new(1);
    let mut expect = [0u8; N_UPGRADES];
    expect[U_SCYTHE] = 1;
    assert_eq!(g.up, expect);
}

#[test]
fn offer_gives_three_distinct_valid_choices() {
    for seed in 1..200u32 {
        let mut g = Game::new(seed);
        g.pending = 1;
        g.offer();
        assert!(g.choosing);
        assert!(g.events.iter().any(|e| e[0] == EV_LEVELUP));
        let c = g.choices;
        for k in 0..3 {
            assert!(c[k] >= 0 && (c[k] as usize) < N_UPGRADES, "seed {seed}: {c:?}");
            for j in 0..k {
                assert_ne!(c[k], c[j], "seed {seed}: {c:?}");
            }
        }
    }
}

#[test]
fn offer_never_includes_maxed_upgrades() {
    for seed in 1..100u32 {
        let mut g = Game::new(seed);
        g.up = UPGRADE_MAX;
        // Leave three upgrades one level short.
        for id in [U_PULSE, U_SWIFT, U_MENDING] {
            g.up[id] -= 1;
        }
        g.pending = 1;
        g.offer();
        let mut got: Vec<i32> = g.choices.to_vec();
        got.sort();
        assert_eq!(got, vec![U_PULSE as i32, U_SWIFT as i32, U_MENDING as i32], "seed {seed}");
    }
}

#[test]
fn offer_pads_with_empty_slots_when_few_remain() {
    let mut g = Game::new(5);
    g.up = UPGRADE_MAX;
    g.up[U_HASTE] -= 1;
    g.up[U_LURE] -= 1;
    g.pending = 1;
    g.offer();
    let mut shown: Vec<i32> = g.choices[..2].to_vec();
    shown.sort();
    assert_eq!(shown, vec![U_LURE as i32, U_HASTE as i32]);
    assert_eq!(g.choices[2], -1);
    // An empty slot cannot be chosen.
    g.choose(2);
    assert!(g.choosing);
}

#[test]
fn choose_raises_level_and_closes_the_offer() {
    let mut g = Game::new(9);
    g.pending = 1;
    g.offer();
    let id = g.choices[1] as usize;
    let before = g.up[id];
    g.choose(1);
    assert_eq!(g.up[id], before + 1);
    assert!(!g.choosing);
    assert_eq!(g.choices, [-1; 3]);
    assert_eq!(g.pending, 0);
}

#[test]
fn choose_chains_into_the_next_pending_offer() {
    let mut g = Game::new(9);
    g.pending = 2;
    g.offer();
    g.choose(0);
    assert!(g.choosing, "second level up is offered straight away");
    assert_eq!(g.pending, 1);
    g.choose(0);
    assert!(!g.choosing);
    assert_eq!(g.pending, 0);
}

#[test]
fn choose_is_ignored_when_not_choosing_or_out_of_range() {
    let mut g = Game::new(9);
    g.choices = [U_MIGHT as i32, -1, -1];
    g.choose(0); // not choosing
    assert_eq!(g.up[U_MIGHT], 0);
    g.choosing = true;
    g.pending = 1;
    g.choose(3); // out of range
    assert_eq!(g.up[U_MIGHT], 0);
    assert!(g.choosing);
}

#[test]
fn choose_on_a_maxed_upgrade_changes_nothing_but_still_consumes_the_pick() {
    // Oddity: a stale choice that is already maxed is not raised, yet the
    // offer is closed and a pending level up is spent.
    let mut g = Game::new(9);
    g.up[U_SWIFT] = UPGRADE_MAX[U_SWIFT];
    g.choosing = true;
    g.pending = 1;
    g.choices = [U_SWIFT as i32, -1, -1];
    g.choose(0);
    assert_eq!(g.up[U_SWIFT], UPGRADE_MAX[U_SWIFT]);
    assert!(!g.choosing);
    assert_eq!(g.pending, 0);
}

#[test]
fn vitality_at_full_health_overheals_current_health() {
    // Oddity: hp is raised by 20 without clamping, so it stays equal to max.
    let mut g = Game::new(9);
    g.choosing = true;
    g.pending = 1;
    g.choices = [U_VITALITY as i32, -1, -1];
    g.choose(0);
    assert!(near(g.player.max_hp, 120.0));
    assert!(near(g.player.hp, 120.0));
}

#[test]
fn offer_heals_when_everything_is_maxed() {
    let mut g = Game::new(9);
    g.up = UPGRADE_MAX;
    g.player.hp = 10.0;
    g.pending = 2;
    g.offer();
    assert!(!g.choosing);
    assert_eq!(g.pending, 0);
    assert!(near(g.player.hp, 70.0)); // 30 per pending level up
    assert_eq!(g.choices, [-1; 3]);

    g.player.hp = 90.0;
    g.pending = 3;
    g.offer();
    assert!(near(g.player.hp, 100.0)); // capped at max health
}

#[test]
fn collecting_enough_xp_triggers_a_level_up_offer() {
    let mut g = base();
    g.player.xp = g.player.xp_next; // 4 + 3.5 + 0.4 = 7.9 at level 1
    assert!(near(g.player.xp_next, 7.9));
    g.step(0.016);
    assert_eq!(g.player.level, 2);
    assert!(near(g.player.xp, 0.0));
    assert!(near(g.player.xp_next, 12.6)); // 4 + 7 + 1.6
    assert!(g.choosing);
    assert_eq!(g.pending, 1);
}

// --------------------------------------------------------------- whole run

/// Fold the observable state into one number (FNV-1a style).
fn fold(g: &Game) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in [g.kills as u64, g.player.level as u64, g.player.hp.to_bits() as u64, g.enemies.len() as u64] {
        h ^= v;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn whole_run() -> (u32, u32, f32, usize, u64) {
    let mut g = Game::new(42);
    g.set_view(640.0, 360.0);
    g.running = true;
    g.input = (1.0, 0.5);
    for _ in 0..(60 * 45) {
        g.update(1.0 / 60.0);
        if g.choosing {
            g.choose(0);
        }
    }
    (g.kills, g.player.level, g.player.hp, g.enemies.len(), fold(&g))
}

#[test]
fn whole_run_is_deterministic_and_matches_the_recorded_summary() {
    let a = whole_run();
    let b = whole_run();
    assert_eq!(a, b, "same seed and input must give the same run");
    // Recorded from the current code (seed 42, input (1, 0.5), 2700 steps of
    // 1/60 s, always picking the first offered upgrade).
    assert_eq!(a, RECORDED, "whole-run summary changed: {a:?}");
}

const RECORDED: (u32, u32, f32, usize, u64) = (39, 3, 46.0, 48, 13057946936710345287);
