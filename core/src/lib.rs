//! Jade Reaper simulation core.
//!
//! Compiled to `wasm32-unknown-unknown` with no dependencies and no bindgen.
//! The host (JavaScript) calls the `jr_*` functions below and reads three
//! buffers straight out of linear memory each frame:
//!
//! - instances: `jr_instance_count()` x `defs::INST_FLOATS` (16) floats, uploaded to the GPU as is
//! - state: `STATE_FLOATS` floats for the HUD (layout in `Game::write_state`)
//! - anims: `jr_anim_count()` x `anim::ANIM_FLOATS` floats, each character's animation state (layout in `anim.rs`)
//! - events: `jr_event_count()` x 4 floats `[type, x, y, value]` for sound

pub mod anim;
pub mod boss;
pub mod bot;
pub mod cards;
#[rustfmt::skip] // written by tools/content.mjs, which owns its layout
pub mod content;
pub mod defs;
pub mod dragon;
pub mod draw;
pub mod elements;
pub mod enemy;
pub mod game;
pub mod geom;
pub mod moves;
pub mod player;
pub mod reactions;
pub mod rigs;
pub mod rooms;
pub mod skills;

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
    let (fps, poses, scale, waddle, calm, spin, sheet) = (
        g().boil_fps,
        g().boil_poses,
        g().draw_scale,
        g().waddle,
        g().reduce_motion,
        g().debug_spin,
        g().sheet_pose,
    );
    *g() = Game::new(seed);
    g().set_view(w, h);
    g().set_boil(fps, poses);
    g().draw_scale = scale;
    g().waddle = waddle;
    g().reduce_motion = calm;
    g().debug_spin = spin;
    g().sheet_pose = sheet;
}

/// Start a new run.
#[no_mangle]
pub extern "C" fn jr_start(seed: u32) {
    jr_reset(seed);
    g().running = true;
    g().enter_room(0);
}

/// Jump to a room of the run, for the ?debug panel and scripted checks.
#[no_mangle]
pub extern "C" fn jr_enter_room(room: u32) {
    g().enter_room((room as usize).min(defs::N_ROOMS - 1));
}

/// Visible area in world units (full width and height).
#[no_mangle]
pub extern "C" fn jr_set_view(w: f32, h: f32) {
    g().set_view(w * 0.5, h * 0.5);
}

/// Reduce motion: turns off camera shake and hit-stop.
#[no_mangle]
pub extern "C" fn jr_set_reduce_motion(on: u32) {
    g().reduce_motion = on != 0;
}

/// Line boil of the traced characters: poses per second, and poses per part (1 to 8).
#[no_mangle]
pub extern "C" fn jr_set_boil(fps: f32, poses: u32) {
    g().set_boil(fps, poses);
}

/// The ?debug toggle: spin every puppet in 3D (turn, lean, lift and bend), to show the renderer's sheets work.
/// Drawing only.
#[no_mangle]
pub extern "C" fn jr_set_debug_spin(on: u32) {
    g().set_debug_spin(on != 0);
}

/// The ?debug pose sliders: set one of a rig's 3D pose numbers, `field` indexing `SHEET_KEYS`. Drawing only.
#[no_mangle]
pub extern "C" fn jr_set_pose(rig: u32, field: u32, value: f32) {
    g().set_pose(rig as usize, field as usize, value);
}

/// How big a character is drawn: 0 is the player, 1 + kind an enemy kind (blot, wisp, brute, elite).
/// Only the drawing changes, not the hit area.
#[no_mangle]
pub extern "C" fn jr_set_draw_scale(who: u32, scale: f32) {
    g().set_draw_scale(who as usize, scale);
}

/// An enemy kind's waddle (blot, wisp, brute, elite): how far it tilts each way in radians, and how often it
/// changes pose per second.
#[no_mangle]
pub extern "C" fn jr_set_waddle(kind: u32, tilt: f32, rate: f32) {
    g().set_waddle(kind as usize, tilt, rate);
}

/// Input for the next steps: movement (each axis in -1..1), the aim point as an offset from the player in world
/// units (zero: skills aim the way it moves, or the way it last moved), and the held buttons (`B_*` bits: skill 1, 2, 3, roll).
#[no_mangle]
pub extern "C" fn jr_input(mx: f32, my: f32, aim_x: f32, aim_y: f32, buttons: u32) {
    g().input = game::Input {
        mx,
        my,
        ax: aim_x,
        ay: aim_y,
        buttons,
    };
}

/// Advance by a frame's time in seconds. The simulation runs in fixed 60 Hz steps (`game::STEP`) and carries
/// the remainder to the next call, so the same seed and inputs replay the same run.
#[no_mangle]
pub extern "C" fn jr_update(dt: f32) {
    g().update(dt);
}

/// Put a skill on each of the three skill buttons (ids from `SKILLS`). Unknown ids are ignored.
#[no_mangle]
pub extern "C" fn jr_set_loadout(a: u32, b: u32, c: u32) {
    for (slot, id) in [a, b, c].into_iter().enumerate() {
        if (id as usize) < skills::N_SKILLS {
            g().loadout[slot] = id as usize;
        }
    }
}

/// Offer the starting passive: three character cards. Call after `jr_start` and `jr_set_loadout`.
#[no_mangle]
pub extern "C" fn jr_offer_passive() {
    g().offer_passive();
}

/// Take one of the offered cards (0..2).
#[no_mangle]
pub extern "C" fn jr_choose(i: u32) {
    g().choose(i as usize);
}

/// A hash of the simulation state, for replay checks.
#[no_mangle]
pub extern "C" fn jr_state_hash() -> u32 {
    g().state_hash()
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
pub extern "C" fn jr_anims_ptr() -> *const f32 {
    g().anims.as_ptr() as *const f32
}

#[no_mangle]
pub extern "C" fn jr_anim_count() -> u32 {
    g().anims.len() as u32
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
        game.enter_room(0);
        let mut bot = bot::Bot::default();
        let mut rng = Rng(99);
        for frame in 0..(60 * 600) {
            game.input = bot.input(&game);
            // Now and then wander off at random, so odd positions are exercised too.
            if frame % 600 < 60 {
                game.input.mx = rng.range(-1.0, 1.0);
                game.input.my = rng.range(-1.0, 1.0);
            }
            game.update(1.0 / 60.0);
            assert!(game.inst.len() <= MAX_INST);
            if game.choosing {
                game.choose((rng.next_u32() % 3) as usize);
            }
            // Keep the run going so later rooms are exercised too.
            if game.over && !game.won {
                game.over = false;
                game.player.hp = game.player.max_hp;
            }
            for i in &game.inst {
                assert!(
                    i.x.is_finite() && i.y.is_finite() && i.hw.is_finite() && i.rot.is_finite()
                );
            }
        }
        assert!(
            game.rooms_cleared >= 2,
            "cleared {} rooms",
            game.rooms_cleared
        );
        assert!(game.kills > 10, "expected kills, got {}", game.kills);
    }

    #[test]
    fn boil_poses_differ_between_instances_and_stay_in_range() {
        let mut game = Game::new(5);
        game.set_view(640.0, 360.0);
        game.running = true;
        game.enter_room(0);
        for _ in 0..(60 * 30) {
            game.update(1.0 / 60.0);
            if game.choosing {
                game.choose(0);
            }
        }
        let poses = |g: &Game| {
            g.inst
                .iter()
                .map(|i| (i.part / PART_SLOTS).floor() as u32)
                .collect::<Vec<_>>()
        };
        let now = poses(&game);
        assert!(now.iter().all(|&p| p < MAX_POSES));
        let first = now[0];
        assert!(
            now.iter().any(|&p| p != first),
            "every instance shows the same pose"
        );
        for i in &game.inst {
            assert!(
                PARTS.contains(&(i.part % PART_SLOTS)),
                "part id {} lost its base id",
                i.part
            );
        }

        game.set_boil(4.0, 1);
        game.update(1.0 / 60.0);
        assert!(poses(&game).iter().all(|&p| p == 0));
    }

    #[test]
    fn draw_scale_changes_only_the_drawing() {
        let robe = |g: &Game| {
            *g.inst
                .iter()
                .find(|i| i.part % PART_SLOTS == P_PLAYER_BODY)
                .unwrap()
        };
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

    /// Play a scripted run: the same seed and inputs, fed through the given frame times.
    fn replay(seed: u32, frames: &[f32]) -> u32 {
        let mut game = Game::new(seed);
        game.set_view(640.0, 360.0);
        game.running = true;
        let mut t = 0.0;
        for &dt in frames {
            t += dt;
            // Inputs change on step boundaries, so different frame splits see the same input in every step.
            let k = (t * 2.0) as u32;
            let mut r = Rng((k * 2654435761) | 1);
            game.input = game::Input {
                buttons: if k.is_multiple_of(3) { B_ROLL } else { 0 },
                ..game::Input::moving(r.range(-1.0, 1.0), r.range(-1.0, 1.0))
            };
            game.update(dt);
            if game.choosing {
                game.choose(0);
            }
        }
        game.state_hash()
    }

    #[test]
    fn same_seed_and_inputs_replay_the_same_run() {
        let frames = vec![1.0 / 60.0; 60 * 40];
        let a = replay(11, &frames);
        assert_eq!(a, replay(11, &frames));
        assert_ne!(a, replay(12, &frames));
    }

    #[test]
    fn frame_rate_does_not_change_the_simulation() {
        // One 1/30 s frame runs the same two steps as two 1/60 s frames.
        let mut game = Game::new(4);
        game.running = true;
        let mut other = Game::new(4);
        other.running = true;
        game.input = game::Input::moving(0.7, -0.2);
        other.input = game::Input::moving(0.7, -0.2);
        for _ in 0..600 {
            game.update(1.0 / 30.0);
            other.update(1.0 / 60.0);
            other.update(1.0 / 60.0);
        }
        assert_eq!(game.steps, 1200);
        assert_eq!(game.state_hash(), other.state_hash());
    }

    #[test]
    fn player_dies_when_idle() {
        let mut game = Game::new(3);
        game.set_view(640.0, 360.0);
        game.running = true;
        game.enter_room(0);
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
