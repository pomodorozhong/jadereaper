# Architecture of the core

How the Rust simulation in `core/` is put together since roadmap step 1, the room-based action game ([0005](decisions/0005-room-based-action-game.md)). How the host talks to it is in the root [README.md](../README.md#how-core-and-host-talk).

## Content is data

`content/game.json` holds the parts, events, rigs, enemy kinds, skills, elements, cards, rooms and moves. `tools/content.mjs` checks it and writes `core/src/content.rs`, `src/content.js` and `src/content.wgsl`, so ids exist once. Each table's fields and types are in `TABLES` in that script; a field can have a default (the boss-only move fields do). The generated files are committed, like the wasm.

The code keeps the types and behaviour: `SkillDef` and `SkillKind` in `skills.rs`, `Move`, `After`, `Special` and `Stance` in `moves.rs`, `Card`, `Effect` and `Target` in `cards.rs`, `Room` in `rooms.rs`, `Rig`, `Pose`, `SheetPose` and `Pose3` in `rigs.rs`. A new value that only changes numbers needs no code; a new kind of behaviour (a skill delivery, a boss special) does.

## Modules

| Module | What it does |
| --- | --- |
| `game.rs` | `Game`, the fixed 60 Hz step, the camera, effects, the HUD state layout and the replay hash |
| `player.rs` | Input, the input buffer, the roll, skills (startup, active, recovery, cancel point, combos, dashes, leaps, orbits), and taking hits (hyper armor, knockdowns, grabs) |
| `geom.rs` | Hit area shapes: circle, arc, line, ring |
| `enemy.rs` | Enemy approach, wind-up, strike and recover; hazards (ink shots, shock rings, lingering pools); taking hits; the `threat` check the bot rolls on |
| `boss.rs` | The boss state machine, stagger meter and knockdown, move picking, and the Glutton's enrage, eating, summons, breath and grab |
| `dragon.rs` | The Dragon's stances, weakness, finisher and its multi-spot moves |
| `elements.rs` | Burn, chill and freeze, lightning chains, celestial marks and gravity wells, the fire trail |
| `reactions.rs` | Element tags and the six reactions |
| `cards.rs` | The card pool, offers, picks, and the per-button and character modifiers |
| `rooms.rs` | Rooms, waves, clearing, winning, the arena walls |
| `rigs.rs`, `draw.rs` | Puppet posing (`pose` turns a rig and an animation record into a flat pose and a 3D sheet) and the instance list for the renderer, including the player's own skill wash and recovery ring |
| `paper.rs` | Paper effects: pieces of paper that fly facing the camera, bounce once and slide to a stop, only panning, never turning or spinning (`fly`), thrown in bursts (`paper_burst`) for skills, kills, hits, reactions and boss moves; ground chips, dust, a torn patch and cracks for heavy landings (`ground_burst`); chips lifted by passing shock rings; paper lightning bolts. Looks only: it draws from its own `fx_rng`, never the run's `rng`, so it does not change the replay hash |
| `twist.rs` | The spin a hit puts on a character about its vertical axis: a damped swing for a light hit, whole turns for a hit worth a fifth of its max HP or more, two turns for a knockdown, one for a broken poise or a shatter. The character cannot move or act while it is locked, which replaces fixed hurt, stun and knockdown times |
| `anim.rs` | Each character's animation state for the host: phase, progress, cancel point, direction, last hit |
| `bot.rs` | The scripted player used by the tests |

## Rules the tests hold the code to

- **Replays.** The step is fixed at 1/60 s and every random choice comes from `Game::rng`, so the same seed and inputs give the same `state_hash`, whatever the frame rate.
- **Fair warnings.** Every attack winds up for at least `MIN_WARNING` (0.4 s), and big boss moves for `MIN_BIG_WARNING` (0.6 s), even when the Glutton is enraged (`windup_for`). `Game::min_warned` records the shortest warning any hit had.
- **The roll answers every attack but the finisher.** A lunge or charge along a line hits with the attacker's body as it passes (`strike_area`), a swept breath with the line where it is, a line of lances only where it erupts. A roll timed to the arrival therefore avoids it. Lingering pools are walked out of, not rolled through.
- **A skill turns with the aim until it hits.** Its direction, and a leap's landing point, follow the aim through the startup and lock when the active window starts.
- **Hits land once per use.** A skill use keeps the ids it hit; an orbit can hit again after `ORBIT_REHIT`.
- **Bosses are never pulled or frozen.** Gravity wells, the collapsing star and stasis skip them and add stagger instead.

## The scripted bot

`bot::Bot` aims at the nearest enemy, closes to the reach of its skills, attacks only when no attack would land before its own, rolls through attacks about to hit (`Game::threat`), walks out of pools, and takes the first card. `core/tests/run.rs` has it reach the first boss room with light skills and with heavy skills; the ignored `report` test prints how it fares room by room, for tuning:

```bash
cargo test --release --manifest-path core/Cargo.toml --test run report -- --ignored --nocapture
```

Only the scythe combo and the talisman are light, so the "light" run uses the lunge cut as its third skill.
