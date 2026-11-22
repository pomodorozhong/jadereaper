//! Paper debris: chips of the ground's paper torn up by a heavy landing, flying and tumbling as sheets in 3D,
//! with a puff of paper dust and a torn patch left on the ground. Looks only: the pieces are effects, drawn from
//! `fx_rng`, so they never change the run.

use crate::content::{P_CHIP, P_SCRAP, P_SPLAT};
use crate::game::{Fx, Game};
use core::f32::consts::{FRAC_PI_2, PI, TAU};

/// Downward pull on flying paper, in world units per second squared.
const GRAVITY: f32 = 1100.0;
/// A piece landing faster than this bounces once; slower, it stays down.
const BOUNCE_SPEED: f32 = 140.0;
/// The ground paper's colour (`GROUND` in `src/shader.wgsl`), a little darker so chips read against it.
const CHIP: [f32; 4] = [0.9, 0.86, 0.76, 1.0];
/// Paper dust: the colour of a cut paper edge (`EDGE` in the shader).
const DUST: [f32; 4] = [0.74, 0.69, 0.6, 0.9];
/// The torn patch: the paper's darker underside showing through.
const TORN: [f32; 4] = [0.55, 0.49, 0.4, 0.45];

impl Game {
    /// Tear up the ground at (x, y): chips flung out over about `reach`, dust, and a torn patch. `power` scales
    /// how many pieces fly and how hard (1 for the crescent leap's landing).
    pub(crate) fn ground_burst(&mut self, x: f32, y: f32, reach: f32, power: f32) {
        let chips = (14.0 * power).round() as usize;
        for i in 0..chips {
            let r = &mut self.fx_rng;
            // Spread the chips evenly round the circle, then jitter, so none of the burst is empty.
            let a = (i as f32 + r.f()) / chips as f32 * TAU;
            let out = r.range(0.25, 0.6) * reach;
            let size = r.range(10.0, 18.0) * power.sqrt();
            let life = r.range(1.4, 2.0);
            let fx = Fx {
                paper: true,
                vx: a.cos() * out * r.range(1.2, 2.2),
                vy: a.sin() * out * r.range(1.2, 2.2),
                vz: r.range(260.0, 480.0) * power.sqrt(),
                z: r.range(0.0, 8.0),
                rot: r.range(0.0, TAU),
                vr: r.range(-9.0, 9.0),
                turn: r.range(-PI, PI),
                vturn: r.range(-14.0, 14.0),
                lean: r.range(-0.6, 0.6),
                vlean: r.range(-12.0, 12.0),
                seed: r.f(),
                ..Fx::at(
                    x + a.cos() * reach * 0.2,
                    y + a.sin() * reach * 0.2,
                    P_CHIP,
                    life,
                    size,
                    size * r.range(0.7, 1.1),
                    CHIP,
                )
            };
            self.add_fx(fx);
        }
        let dust = (10.0 * power).round() as usize;
        for _ in 0..dust {
            let r = &mut self.fx_rng;
            let a = r.range(0.0, TAU);
            let s = r.range(180.0, 420.0);
            let size = r.range(2.0, 4.5);
            let fx = Fx {
                vx: a.cos() * s,
                // The camera looks down, so a puff reads flatter up and down the screen.
                vy: a.sin() * s * 0.6,
                rot: a,
                vr: r.range(-6.0, 6.0),
                seed: r.f(),
                top: false,
                ..Fx::at(x, y, P_SCRAP, r.range(0.3, 0.5), size, size, DUST)
            };
            self.add_fx(fx);
        }
        let seed = self.fx_rng.f();
        self.add_fx(Fx {
            grow: reach * 0.15,
            seed,
            top: false,
            ..Fx::at(x, y, P_SPLAT, 1.4, reach * 0.45, reach * 0.3, TORN)
        });
    }
}

/// Move one piece of flying paper by `dt`: fall, bounce once, tumble, and settle flat on the ground.
pub(crate) fn fly(f: &mut Fx, dt: f32) {
    f.x += f.vx * dt;
    f.y += f.vy * dt;
    f.rot += f.vr * dt;
    if f.z > 0.0 || f.vz > 0.0 {
        f.vz -= GRAVITY * dt;
        f.z += f.vz * dt;
        f.turn += f.vturn * dt;
        f.lean += f.vlean * dt;
        // Paper catches the air: it slows quickly sideways.
        let air = (-1.6 * dt).exp();
        f.vx *= air;
        f.vy *= air;
        if f.z <= 0.0 {
            f.z = 0.0;
            if -f.vz > BOUNCE_SPEED {
                f.vz = -f.vz * 0.3;
                f.vx *= 0.5;
                f.vy *= 0.5;
                f.vturn *= 0.5;
                f.vlean *= 0.5;
            } else {
                f.vz = 0.0;
                // Down for good: drawn under the characters, like the ground's own marks.
                f.top = false;
            }
        }
    } else {
        // On the ground: it slides to a stop and lies flat, face up or face down, whichever is nearer.
        let rest = (-10.0 * dt).exp();
        f.vx *= rest;
        f.vy *= rest;
        f.vr *= rest;
        let flat = ((f.lean - FRAC_PI_2) / PI).round() * PI + FRAC_PI_2;
        let settle = 1.0 - (-14.0 * dt).exp();
        f.lean += (flat - f.lean) * settle;
        f.turn += (turn_rest(f.turn) - f.turn) * settle;
    }
}

/// The nearest turn that shows the paper square on: a whole number of half turns.
fn turn_rest(turn: f32) -> f32 {
    (turn / PI).round() * PI
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::Rng;

    fn burst() -> Game {
        let mut g = Game::new(7);
        g.ground_burst(100.0, 50.0, 110.0, 1.0);
        g
    }

    #[test]
    fn a_burst_throws_paper_chips_dust_and_a_torn_patch() {
        let g = burst();
        let chips = g.fx.iter().filter(|f| f.paper && f.part == P_CHIP).count();
        let dust = g.fx.iter().filter(|f| f.part == P_SCRAP).count();
        let torn = g.fx.iter().filter(|f| f.part == P_SPLAT).count();
        assert_eq!((chips, dust, torn), (14, 10, 1));
        assert!(g.fx.iter().filter(|f| f.paper).all(|f| f.vz > 0.0));
    }

    #[test]
    fn chips_fly_up_then_land_and_lie_flat() {
        let mut g = burst();
        let mut highest: f32 = 0.0;
        for _ in 0..84 {
            g.move_fx(1.0 / 60.0);
            for f in g.fx.iter().filter(|f| f.paper) {
                highest = highest.max(f.z);
            }
        }
        assert!(highest > 40.0, "chips should rise, highest {highest}");
        // By 1.4 s, when the first chips fade out, every chip is down and lying flat.
        let paper: Vec<&Fx> = g.fx.iter().filter(|f| f.paper).collect();
        assert!(!paper.is_empty());
        for f in paper {
            assert_eq!(f.z, 0.0);
            let off = ((f.lean - FRAC_PI_2) / PI).fract().abs();
            assert!(off < 0.05 || off > 0.95, "lean {} is not flat", f.lean);
        }
    }

    #[test]
    fn debris_does_not_change_the_run() {
        let plain = Game::new(7);
        let g = burst();
        assert_eq!(g.state_hash(), plain.state_hash());
        assert_eq!(g.rng.0, plain.rng.0);
    }

    #[test]
    fn rest_turn_is_a_whole_number_of_half_turns() {
        let mut r = Rng(5);
        for _ in 0..20 {
            let t = r.range(-10.0, 10.0);
            let rest = turn_rest(t);
            assert!((rest / PI - (rest / PI).round()).abs() < 1e-5);
            assert!((rest - t).abs() <= PI / 2.0 + 1e-5);
        }
    }
}
