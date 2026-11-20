//! Jade Reaper simulation core.
//!
//! Compiled to `wasm32-unknown-unknown` with no dependencies and no bindgen.
//! The host (JavaScript) calls the `jr_*` functions below and reads three
//! buffers straight out of linear memory each frame:
//!
//! - instances: `jr_instance_count()` x 12 floats, uploaded to the GPU as is
//! - state: `STATE_FLOATS` floats for the HUD (layout in `Game::write_state`)
//! - events: `jr_event_count()` x 4 floats `[type, x, y, value]` for sound

pub mod defs;
pub mod game;

use game::Game;

static mut GAME: Option<Game> = None;

fn g() -> &'static mut Game {
    // Single-threaded wasm: there is only ever one caller.
    unsafe { (*core::ptr::addr_of_mut!(GAME)).get_or_insert_with(|| Game::new(1)) }
}

/// Reset to the idle scene shown behind the title screen.
#[no_mangle]
pub extern "C" fn jr_reset(seed: u32) {
    let (w, h) = (g().view_w, g().view_h);
    let (fps, poses, scale) = (g().boil_fps, g().boil_poses, g().draw_scale);
    *g() = Game::new(seed);
    g().set_view(w, h);
    g().set_boil(fps, poses);
    g().draw_scale = scale;
}

/// Start a new run.
#[no_mangle]
pub extern "C" fn jr_start(seed: u32) {
    jr_reset(seed);
    g().running = true;
}

/// Visible area in world units (full width and height).
#[no_mangle]
pub extern "C" fn jr_set_view(w: f32, h: f32) {
    g().set_view(w * 0.5, h * 0.5);
}

/// Line boil of the traced characters: poses per second, and poses per part (1 to 8).
#[no_mangle]
pub extern "C" fn jr_set_boil(fps: f32, poses: u32) {
    g().set_boil(fps, poses);
}

/// How big a character is drawn: 0 is the player, 1 + kind an enemy kind (blot, wisp, brute, elite).
/// Only the drawing changes, not the hit area.
#[no_mangle]
pub extern "C" fn jr_set_draw_scale(who: u32, scale: f32) {
    g().set_draw_scale(who as usize, scale);
}

/// Movement input, each axis in -1..1.
#[no_mangle]
pub extern "C" fn jr_input(mx: f32, my: f32) {
    g().input = (mx, my);
}

#[no_mangle]
pub extern "C" fn jr_update(dt: f32) {
    g().update(dt);
}

/// Pick one of the offered upgrades (0..2).
#[no_mangle]
pub extern "C" fn jr_choose(i: u32) {
    g().choose(i as usize);
}

#[no_mangle]
pub extern "C" fn jr_instances_ptr() -> *const f32 {
    g().inst.as_ptr() as *const f32
}

#[no_mangle]
pub extern "C" fn jr_instance_count() -> u32 {
    g().inst.len() as u32
}

#[no_mangle]
pub extern "C" fn jr_state_ptr() -> *const f32 {
    g().state.as_ptr()
}

#[no_mangle]
pub extern "C" fn jr_events_ptr() -> *const f32 {
    g().events.as_ptr() as *const f32
}

#[no_mangle]
pub extern "C" fn jr_event_count() -> u32 {
    g().events.len() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use defs::*;

    #[test]
    fn long_run_does_not_panic() {
        let mut game = Game::new(7);
        game.set_view(640.0, 360.0);
        game.running = true;
        let mut rng = Rng(99);
        let mut picks = 0;
        for frame in 0..(60 * 600) {
            if frame % 30 == 0 {
                game.input = (rng.range(-1.0, 1.0), rng.range(-1.0, 1.0));
            }
            game.update(1.0 / 60.0);
            assert!(game.inst.len() <= MAX_INST);
            if game.choosing {
                game.choose((rng.next() % 3) as usize);
                picks += 1;
            }
            // Keep the run going so late-game spawning is exercised too.
            if game.over {
                game.over = false;
                game.player.hp = game.player.max_hp;
            }
            for i in &game.inst {
                assert!(i.x.is_finite() && i.y.is_finite() && i.hw.is_finite() && i.rot.is_finite());
            }
        }
        assert!(picks > 5, "expected level ups, got {picks}");
        assert!(game.kills > 50, "expected kills, got {}", game.kills);
    }

    #[test]
    fn boil_poses_differ_between_instances_and_stay_in_range() {
        let mut game = Game::new(5);
        game.set_view(640.0, 360.0);
        game.running = true;
        for _ in 0..(60 * 30) {
            game.update(1.0 / 60.0);
            if game.choosing {
                game.choose(0);
            }
        }
        let poses = |g: &Game| g.inst.iter().map(|i| (i.part / PART_SLOTS).floor() as u32).collect::<Vec<_>>();
        let now = poses(&game);
        assert!(now.iter().all(|&p| p < MAX_POSES));
        let first = now[0];
        assert!(now.iter().any(|&p| p != first), "every instance shows the same pose");
        for i in &game.inst {
            assert!(i.part % PART_SLOTS < 20.0, "part id {} lost its base id", i.part);
        }

        game.set_boil(4.0, 1);
        game.update(1.0 / 60.0);
        assert!(poses(&game).iter().all(|&p| p == 0));
    }

    #[test]
    fn draw_scale_changes_only_the_drawing() {
        let robe = |g: &Game| *g.inst.iter().find(|i| i.part % PART_SLOTS == P_ROBE).unwrap();
        let mut game = Game::new(9);
        game.set_view(640.0, 360.0);
        game.update(1.0 / 60.0);
        let before = robe(&game);
        game.set_draw_scale(0, 2.0);
        game.update(0.0);
        let after = robe(&game);
        assert_eq!(after.hw.abs(), before.hw.abs() * 2.0);
        assert_eq!(after.hh, before.hh * 2.0);
        game.set_draw_scale(9, 2.0); // out of range is ignored
    }

    #[test]
    fn player_dies_when_idle() {
        let mut game = Game::new(3);
        game.set_view(640.0, 360.0);
        game.running = true;
        for _ in 0..(60 * 900) {
            game.update(1.0 / 60.0);
            if game.choosing {
                game.choose(0);
            }
            if game.over {
                break;
            }
        }
        assert!(game.over);
        assert_eq!(game.state[11], 1.0);
    }
}
