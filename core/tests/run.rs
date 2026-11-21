//! Whole runs: the scripted bot clears the rooms up to the boss, and the run is replayable.

use jadereaper_core::bot;
use jadereaper_core::defs::*;
use jadereaper_core::game::Game;
use jadereaper_core::rooms::Stage;

/// The first room with a boss in it.
fn boss_room() -> usize {
    (0..N_ROOMS).find(|&r| ROOMS[r].boss > 0).unwrap()
}

fn start(seed: u32, loadout: [usize; 3]) -> Game {
    let mut g = Game::new(seed);
    g.set_view(410.0, 230.0);
    g.running = true;
    g.loadout = loadout;
    g.enter_room(0);
    g.offer_passive();
    g
}

fn reach_boss(loadout: [usize; 3], seed: u32) -> (usize, f32) {
    let mut g = start(seed, loadout);
    let target = boss_room();
    let steps = bot::play(&mut g, 900.0, |g| g.room >= target);
    (g.room, steps as f32 / 60.0)
}

#[test]
fn bot_with_light_skills_reaches_the_boss_room() {
    // Only the scythe combo and the talisman are light; the lunge cut is the quickest of the rest.
    for seed in [1, 2] {
        let (room, secs) = reach_boss([SK_SCYTHE, SK_TALISMAN, SK_LUNGE], seed);
        assert_eq!(
            room,
            boss_room(),
            "seed {seed}: stopped in room {room} after {secs} s"
        );
    }
}

#[test]
fn bot_with_heavy_skills_reaches_the_boss_room() {
    for seed in [1, 2] {
        let (room, secs) = reach_boss([SK_CHARGE, SK_CLEAVE, SK_LEAP], seed);
        assert_eq!(
            room,
            boss_room(),
            "seed {seed}: stopped in room {room} after {secs} s"
        );
    }
}

#[test]
fn a_run_replays_from_its_seed_and_inputs() {
    let run = || {
        let mut g = start(9, [SK_SCYTHE, SK_PULSE, SK_CLEAVE]);
        bot::play(&mut g, 120.0, |_| false);
        (g.state_hash(), g.room, g.kills)
    };
    assert_eq!(run(), run());
}

#[test]
fn clearing_a_room_offers_a_card_then_moves_on() {
    let mut g = start(4, [SK_SCYTHE, SK_TALISMAN, SK_PULSE]);
    g.choose(0);
    bot::play(&mut g, 300.0, |g| g.stage == Stage::Cleared);
    assert_eq!(g.stage, Stage::Cleared);
    assert!(g.choosing, "a card is offered after the clear");
    let room = g.room;
    g.choose(0);
    assert_eq!(g.room, room + 1);
    assert!(g.enemies.is_empty());
}

#[test]
fn beating_the_last_room_wins() {
    let mut g = start(4, [SK_SCYTHE, SK_TALISMAN, SK_PULSE]);
    g.choose(0);
    g.enter_room(N_ROOMS - 1);
    // Wait for the boss, then leave it nearly beaten.
    bot::play(&mut g, 10.0, |g| !g.enemies.is_empty());
    for e in g.enemies.iter_mut() {
        e.hp = 30.0;
    }
    bot::play(&mut g, 120.0, |_| false);
    assert!(
        g.won && g.over,
        "room {}, wave {}, {} enemies left",
        g.room,
        g.wave,
        g.enemies.len()
    );
}

/// Prints how the bot fares room by room, for tuning:
/// `cargo test --release --manifest-path core/Cargo.toml --test run report -- --ignored --nocapture`
#[test]
#[ignore]
fn report() {
    for loadout in [
        [SK_SCYTHE, SK_TALISMAN, SK_LUNGE],
        [SK_CHARGE, SK_CLEAVE, SK_LEAP],
    ] {
        let mut g = start(1, loadout);
        let mut bot = bot::Bot::default();
        let mut room = usize::MAX;
        let mut hurt = 0;
        for _ in 0..60 * 600 {
            if g.choosing {
                g.choose(0);
            }
            if g.room != room {
                room = g.room;
                println!(
                    "{loadout:?} room {room}: hp {:.0}/{:.0} at {:.0} s",
                    g.player.hp, g.player.max_hp, g.time
                );
            }
            g.input = bot.input(&g);
            g.update(1.0 / 60.0);
            hurt += g.events.iter().filter(|e| e[0] == EV_HURT).count();
            if g.over {
                let how = if g.won { "won" } else { "died" };
                println!(
                    "{loadout:?} {how} in room {} wave {} at {:.0} s, hit {hurt} times",
                    g.room, g.wave, g.time
                );
                break;
            }
        }
    }
}
