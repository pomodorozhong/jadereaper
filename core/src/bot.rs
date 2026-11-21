//! A scripted player for tests and balance checks: it aims at the nearest enemy, keeps a distance that suits its
//! skills, uses them when a target is in reach, rolls through attacks about to land, and takes the first card.

use crate::defs::*;
use crate::game::{Game, Input};
use crate::geom::{dir_to, dist};
use crate::skills::SkillKind;

pub struct Bot {
    /// Steps until the next skill press, so presses are fresh edges.
    wait: u32,
    next_slot: usize,
}

impl Default for Bot {
    fn default() -> Self {
        Bot {
            wait: 0,
            next_slot: 0,
        }
    }
}

impl Bot {
    /// The input for this step. Call `Game::update` with it, then `pick` if the game is choosing.
    pub fn input(&mut self, g: &Game) -> Input {
        let (px, py) = (g.player.x, g.player.y);
        let Some(e) = g.enemies.iter().min_by(|a, b| {
            dist(a.x, a.y, px, py)
                .partial_cmp(&dist(b.x, b.y, px, py))
                .unwrap_or(core::cmp::Ordering::Equal)
        }) else {
            return Input::default();
        };
        let d = dist(e.x, e.y, px, py);
        let r = KINDS[e.kind as usize].r;
        let (dx, dy) = dir_to(px, py, e.x, e.y);
        let aim = (e.x - px, e.y - py);

        if let Some((ox, oy)) = g.pool_escape() {
            // Walk out of burning ground.
            return Input {
                mx: ox,
                my: oy,
                ax: aim.0,
                ay: aim.1,
                buttons: 0,
            };
        }
        if g.threat(0.08) {
            // Roll through the attacker.
            return Input {
                mx: dx,
                my: dy,
                ax: aim.0,
                ay: aim.1,
                buttons: B_ROLL,
            };
        }

        // Pick the next skill that can reach.
        let mut buttons = 0;
        if self.wait > 0 {
            self.wait -= 1;
        } else {
            for k in 0..3 {
                let slot = (self.next_slot + k) % 3;
                let def = g.def(slot);
                let reach = match def.kind {
                    SkillKind::Shot => def.reach * 0.8,
                    SkillKind::Leap => def.dash,
                    SkillKind::Orbit => def.reach + 20.0,
                    _ => def.reach + def.dash * 0.8,
                } + r;
                // Do not start a skill that an attack would cut off before it lands.
                if d < reach && !g.threat(def.startup + def.active + 0.05) {
                    buttons = 1 << slot;
                    self.next_slot = (slot + 1) % 3;
                    self.wait = 6;
                    break;
                }
            }
        }

        // Close in to striking range, but not on top of it.
        let want = (0..3)
            .map(|s| {
                let def = g.def(s);
                match def.kind {
                    SkillKind::Shot => 220.0,
                    _ => def.reach * 0.7 + r,
                }
            })
            .fold(f32::MAX, f32::min);
        let (mx, my) = if d > want {
            (dx, dy)
        } else if d < want * 0.5 {
            (-dx, -dy)
        } else {
            (-dy * 0.4, dx * 0.4)
        };
        Input {
            mx,
            my,
            ax: aim.0,
            ay: aim.1,
            buttons,
        }
    }
}

/// Play a game with the bot until it ends, `stop` says so, or `secs` of game time pass. Takes the first card
/// offered each time. Returns the steps played.
pub fn play(g: &mut Game, secs: f32, mut stop: impl FnMut(&Game) -> bool) -> u32 {
    let mut bot = Bot::default();
    let mut steps = 0;
    while (steps as f32) * crate::game::STEP < secs && !g.over && !stop(g) {
        if g.choosing {
            g.choose(0);
        }
        g.input = bot.input(g);
        g.update(crate::game::STEP);
        steps += 1;
        // Release so the next press is a fresh edge.
        g.input.buttons = 0;
    }
    steps
}
