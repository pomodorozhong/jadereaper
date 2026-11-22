//! Paper effects: pieces of paper that fly and settle, for impacts, kills, hits, elements and reactions. A paper
//! effect is an `Fx` with `paper` set: a sheet facing the camera at a fixed angle, with a height. It falls under
//! gravity (scaled by `fall`, so a curl of steam can rise), bounces once, and once down for good slides to a stop
//! under the characters. It only pans: it never turns, leans or spins. Looks only: every random number comes from
//! `fx_rng`, never the run's `rng`, so effects never change the run.

use crate::content::{P_BOLT, P_CHIP, P_CRACK, P_SHRED, P_SPLAT};
use crate::game::{Fx, Game};
use core::f32::consts::TAU;

/// Downward pull on flying paper, in world units per second squared.
const GRAVITY: f32 = 1100.0;
/// A piece landing faster than this bounces once; slower, it stays down.
const BOUNCE_SPEED: f32 = 140.0;
/// The ground paper's colour (`GROUND` in `src/shader.wgsl`), a little darker so chips read against it.
pub const GROUND_CHIP: [f32; 4] = [0.9, 0.86, 0.76, 1.0];
/// The puppets' paper (`PAPER` in the shader): shreds torn from a character.
pub const PUPPET: [f32; 3] = [0.965, 0.94, 0.875];
/// The crescent leap's landing throws the white shreds a blot used to tear into when it died: this many, this
/// big (half size), from this high.
pub const LEAP_SHREDS: (usize, f32, f32) = (8, 9.6, 12.8);
/// Paper dust: the colour of a cut paper edge (`EDGE` in the shader).
const DUST: [f32; 4] = [0.74, 0.69, 0.6, 1.0];
/// The torn patch: the paper's darker underside showing through.
const TORN: [f32; 4] = [0.55, 0.49, 0.4, 0.45];

/// A burst of paper pieces from a spot. Ranges are (low, high), picked per piece.
#[derive(Clone, Copy)]
pub struct Burst {
    /// The part each piece is cut as (`P_CHIP`, `P_SHRED`, `P_SLIP`, `P_SHARD`, ...) and its colour.
    pub part: f32,
    pub color: [f32; 4],
    pub count: usize,
    /// Half size of a piece, in world units; pieces are a little taller or shorter than wide.
    pub size: (f32, f32),
    /// Speed outward along the ground (negative flies inward), and upward.
    pub out: (f32, f32),
    pub up: (f32, f32),
    pub life: (f32, f32),
    /// Pieces start this far from the spot (as a ring), and this high.
    pub from: f32,
    pub z: f32,
    /// Gravity's share: 1 falls, 0 floats, below 0 rises.
    pub fall: f32,
    /// Pieces fade out over their last this many seconds.
    pub fade: f32,
}

impl Burst {
    /// A burst of `count` pieces of `part` in `color`, with defaults for a small knock: pieces 4 to 7 units, out
    /// 60 to 160, up 160 to 300, living 0.9 to 1.3 s, falling.
    pub fn of(part: f32, color: [f32; 4], count: usize) -> Burst {
        Burst {
            part,
            color,
            count,
            size: (4.0, 7.0),
            out: (60.0, 160.0),
            up: (160.0, 300.0),
            life: (0.9, 1.3),
            from: 0.0,
            z: 0.0,
            fall: 1.0,
            fade: 0.35,
        }
    }
}

impl Game {
    /// Throw a burst of paper pieces from (x, y), spread evenly round the circle with some jitter.
    pub(crate) fn paper_burst(&mut self, x: f32, y: f32, b: Burst) {
        for i in 0..b.count {
            let r = &mut self.fx_rng;
            let a = (i as f32 + r.f()) / b.count.max(1) as f32 * TAU;
            let (ca, sa) = (a.cos(), a.sin());
            let out = r.range(b.out.0, b.out.1);
            let size = r.range(b.size.0, b.size.1);
            let fx = Fx {
                paper: true,
                fall: b.fall,
                fade: b.fade,
                vx: ca * out,
                vy: sa * out,
                vz: r.range(b.up.0, b.up.1),
                z: b.z + r.range(0.0, 6.0),
                rot: r.range(-0.6, 0.6),
                seed: r.f(),
                ..Fx::at(
                    x + ca * b.from,
                    y + sa * b.from,
                    b.part,
                    r.range(b.life.0, b.life.1),
                    size,
                    size * r.range(0.7, 1.15),
                    b.color,
                )
            };
            self.add_fx(fx);
        }
    }

    /// Tear up the ground at (x, y): chips of the ground's paper flung out over about `reach`, paper dust, a
    /// torn patch and ink cracks. `power` scales how many pieces fly and how hard (1 for the crescent leap).
    pub(crate) fn ground_burst(&mut self, x: f32, y: f32, reach: f32, power: f32) {
        let k = power.sqrt();
        self.paper_burst(
            x,
            y,
            Burst {
                size: (10.0 * k, 18.0 * k),
                out: (0.3 * reach, 1.3 * reach),
                up: (260.0 * k, 480.0 * k),
                life: (1.4, 2.0),
                from: reach * 0.2,
                z: 2.0,
                ..Burst::of(P_CHIP, GROUND_CHIP, (14.0 * power).round() as usize)
            },
        );
        // Dust: small flecks of the paper's edge that hardly leave the ground.
        self.paper_burst(
            x,
            y,
            Burst {
                size: (2.5, 4.5),
                out: (180.0, 420.0),
                up: (40.0, 140.0),
                life: (0.4, 0.7),
                ..Burst::of(P_CHIP, DUST, (10.0 * power).round() as usize)
            },
        );
        self.ground_marks(x, y, reach, k);
    }

    /// The marks a heavy landing leaves at (x, y): a torn patch of the ground and ink cracks, about `reach`
    /// across, scaled by `k`.
    pub(crate) fn ground_marks(&mut self, x: f32, y: f32, reach: f32, k: f32) {
        let seed = self.fx_rng.f();
        self.add_fx(Fx {
            grow: reach * 0.15,
            seed,
            top: false,
            ..Fx::at(x, y, P_SPLAT, 1.4, reach * 0.45, reach * 0.3, TORN)
        });
        let seed = self.fx_rng.f();
        self.add_fx(Fx {
            seed,
            top: false,
            ..Fx::at(
                x,
                y,
                P_CRACK,
                1.2,
                reach * 0.6 * k,
                reach * 0.42 * k,
                [0.17, 0.15, 0.13, 0.6],
            )
        });
    }

    /// A lightning bolt cut from paper, from `a` to `b` (`d` apart, at angle `rot`), held up at chest height.
    pub(crate) fn paper_bolt(
        &mut self,
        a: (f32, f32),
        b: (f32, f32),
        rot: f32,
        d: f32,
        c: [f32; 3],
        life: f32,
    ) {
        let seed = self.fx_rng.f();
        self.add_fx(Fx {
            paper: true,
            fall: 0.0,
            z: 24.0,
            rot,
            fade: life * 0.5,
            seed,
            ..Fx::at(
                (a.0 + b.0) * 0.5,
                (a.1 + b.1) * 0.5,
                P_BOLT,
                life,
                d * 0.5,
                11.0,
                [c[0], c[1], c[2], 1.0],
            )
        });
    }

    /// Torn paper from a puppet at (x, y): `count` shreds in `color` thrown from `z` up its body.
    pub(crate) fn shred_burst(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        color: [f32; 3],
        count: usize,
        size: f32,
    ) {
        self.paper_burst(
            x,
            y,
            Burst {
                size: (size * 0.7, size * 1.3),
                out: (60.0, 200.0),
                up: (150.0, 340.0),
                life: (1.0, 1.5),
                z,
                ..Burst::of(P_SHRED, [color[0], color[1], color[2], 1.0], count)
            },
        );
    }

    /// Chips lifted where a shock ring at (x, y) of radius `r` is passing, about one per `gap` units of its
    /// edge per second: the ring runs through the ground's paper.
    pub(crate) fn ring_chips(&mut self, x: f32, y: f32, r: f32, dt: f32) {
        let want = TAU * r * dt / 26.0;
        let mut n = want.floor() as usize;
        if self.fx_rng.f() < want.fract() {
            n += 1;
        }
        for _ in 0..n {
            let rg = &mut self.fx_rng;
            let a = rg.range(0.0, TAU);
            let size = rg.range(5.0, 10.0);
            let fx = Fx {
                paper: true,
                fall: 1.0,
                vx: a.cos() * rg.range(40.0, 110.0),
                vy: a.sin() * rg.range(40.0, 110.0),
                vz: rg.range(140.0, 280.0),
                z: 1.0,
                rot: rg.range(-0.6, 0.6),
                seed: rg.f(),
                ..Fx::at(
                    x + a.cos() * r,
                    y + a.sin() * r,
                    P_CHIP,
                    rg.range(0.8, 1.2),
                    size,
                    size * 0.85,
                    GROUND_CHIP,
                )
            };
            self.add_fx(fx);
        }
    }
}

/// Move one piece of paper by `dt`: fall (or float, or rise), bounce once, and slide to a stop on the ground.
/// Its angle only changes by `vr`, which pieces leave at zero (a scythe's crescent sweeps round with it).
pub(crate) fn fly(f: &mut Fx, dt: f32) {
    f.x += f.vx * dt;
    f.y += f.vy * dt;
    f.rot += f.vr * dt;
    f.hw += f.grow * dt;
    f.hh += f.grow * dt;
    if f.z > 0.0 || f.vz > 0.0 {
        f.vz -= GRAVITY * f.fall * dt;
        f.z += f.vz * dt;
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
            } else {
                f.vz = 0.0;
                // Down for good: drawn under the characters, like the ground's own marks.
                f.top = false;
            }
        }
    } else {
        // On the ground: it slides to a stop.
        let rest = (-10.0 * dt).exp();
        f.vx *= rest;
        f.vy *= rest;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::P_SCRAP;

    fn burst() -> Game {
        let mut g = Game::new(7);
        g.ground_burst(100.0, 50.0, 110.0, 1.0);
        g
    }

    #[test]
    fn a_ground_burst_throws_chips_dust_a_torn_patch_and_cracks() {
        let g = burst();
        let chips = g.fx.iter().filter(|f| f.paper && f.part == P_CHIP).count();
        let torn = g.fx.iter().filter(|f| f.part == P_SPLAT).count();
        let cracks = g.fx.iter().filter(|f| f.part == P_CRACK).count();
        assert_eq!((chips, torn, cracks), (24, 1, 1));
        assert!(g.fx.iter().filter(|f| f.paper).all(|f| f.vz > 0.0));
        assert!(!g.fx.iter().any(|f| f.part == P_SCRAP));
    }

    #[test]
    fn chips_fly_up_then_land_and_stop_without_turning() {
        let mut g = burst();
        let mut highest: f32 = 0.0;
        for _ in 0..84 {
            g.move_fx(1.0 / 60.0);
            for f in g.fx.iter().filter(|f| f.paper) {
                highest = highest.max(f.z);
                // Paper only pans: it never turns, leans or spins.
                assert_eq!((f.turn, f.lean, f.vr), (0.0, 0.0, 0.0));
            }
        }
        assert!(highest > 40.0, "chips should rise, highest {highest}");
        // By 1.4 s, when the first chips fade out, every chip is down and nearly still.
        let paper: Vec<&Fx> = g.fx.iter().filter(|f| f.paper).collect();
        assert!(!paper.is_empty());
        for f in paper {
            assert_eq!(f.z, 0.0);
            assert!(!f.top, "a chip lying down is drawn under the characters");
            assert!(
                f.vx.hypot(f.vy) < 20.0,
                "still sliding at {}",
                f.vx.hypot(f.vy)
            );
        }
    }

    #[test]
    fn rising_paper_rises_and_floating_paper_stays_up() {
        let mut g = Game::new(3);
        g.paper_burst(
            0.0,
            0.0,
            Burst {
                fall: -0.2,
                up: (60.0, 80.0),
                life: (2.0, 2.0),
                ..Burst::of(P_SHRED, [1.0; 4], 4)
            },
        );
        g.paper_burst(
            0.0,
            0.0,
            Burst {
                fall: 0.0,
                up: (0.0, 0.0),
                z: 30.0,
                life: (2.0, 2.0),
                ..Burst::of(P_CHIP, [1.0; 4], 4)
            },
        );
        for _ in 0..60 {
            g.move_fx(1.0 / 60.0);
        }
        for f in &g.fx {
            if f.part == P_SHRED {
                assert!(f.z > 80.0, "steam should keep rising, z {}", f.z);
            } else {
                assert!(f.z >= 30.0, "floating paper should not fall, z {}", f.z);
            }
        }
    }

    #[test]
    fn effects_do_not_change_the_run() {
        let plain = Game::new(7);
        let mut g = burst();
        g.shred_burst(0.0, 0.0, 20.0, [0.2, 0.4, 0.3], 6, 6.0);
        g.ring_chips(0.0, 0.0, 120.0, 0.1);
        assert_eq!(g.state_hash(), plain.state_hash());
        assert_eq!(g.rng.0, plain.rng.0);
    }

    #[test]
    fn a_passing_ring_lifts_chips_along_its_edge() {
        let mut g = Game::new(9);
        g.ring_chips(0.0, 0.0, 200.0, 0.5);
        let n = g.fx.len();
        // About one chip per 26 units of edge per second: 2π·200·0.5/26 ≈ 24.
        assert!((20..=28).contains(&n), "{n} chips");
        for f in &g.fx {
            let r = (f.x * f.x + f.y * f.y).sqrt();
            assert!((r - 200.0).abs() < 1.0);
        }
    }

    #[test]
    fn every_paper_effect_draws_whole_finite_paper() {
        use crate::content::{P_BOLT, P_FLAME, P_SHARD, P_SLIP};
        use crate::defs::PART_SLOTS;
        use crate::reactions::*;
        let mut g = Game::new(11);
        g.set_view(900.0, 600.0);
        g.running = true;
        g.enter_room(2);
        for _ in 0..900 {
            g.update(1.0 / 60.0);
            if g.choosing {
                g.choose(0);
            }
            if g.enemies.len() >= 2 {
                break;
            }
        }
        assert!(g.enemies.len() >= 2, "no enemies to react on");
        for e in g.enemies.iter_mut() {
            e.hp = 1e6;
        }
        for r in [
            R_STEAM, R_OVERLOAD, R_COLLAPSE, R_SHATTER, R_STASIS, R_STARFALL,
        ] {
            g.react(0, r, 5.0);
        }
        g.enemies[1].status.burn = 3.0;
        g.enemies[1].status.frozen = 2.0;
        let (x, y) = (g.enemies[0].x, g.enemies[0].y);
        g.paper_bolt((x, y), (x + 120.0, y), 0.0, 120.0, [0.5, 0.5, 0.9], 0.2);
        g.ground_burst(x, y, 110.0, 1.0);
        g.shred_burst(x, y, 20.0, PUPPET, 8, 9.0);
        for _ in 0..6 {
            g.update(1.0 / 60.0);
        }
        let parts: Vec<f32> = g.inst.iter().map(|i| i.part % PART_SLOTS).collect();
        for p in [P_CHIP, P_SHRED, P_SLIP, P_SHARD, P_FLAME, P_BOLT, P_CRACK] {
            assert!(parts.contains(&p), "part {p} is not drawn");
        }
        for i in &g.inst {
            let v = [
                i.x, i.y, i.hw, i.hh, i.rot, i.turn, i.lean, i.lift, i.bend, i.color[3],
            ];
            assert!(v.iter().all(|f| f.is_finite()), "part {} has {v:?}", i.part);
        }
    }
}
