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
    *g() = Game::new(seed);
    g().set_view(w, h);
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
