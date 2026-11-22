# Roadmap

This is the plan for turning the Jade Reaper prototype into a room-based action game, in the spirit of Hades, Wizard of Legend and Hyper Light Drifter. The direction, and the design pillars this plan builds on, are in decision [0005](docs/decisions/0005-room-based-action-game.md). The ideas came from the Idea hub, #41.

Step 1 is one large PR, by the maintainer's choice, built as ordered commits so it can be reviewed piece by piece. The later steps are each their own PR.

## Current state

Step 1 is built on `feat/room-action-game` (#43): Jade Reaper is now a room-based action game ([0005](docs/decisions/0005-room-based-action-game.md)). The Rust core (`core/`) runs in fixed 60 Hz steps and replays a seed exactly. The player aims, uses three skills chosen from eight, and rolls. Six rooms of waves end in two bosses, the Glutton and the Two-Element Dragon. Cards from one pool add the four elements and their reactions. Content lives in `content/game.json`, and a generator writes the ids for Rust, JavaScript and WGSL. The WebGPU renderer draws everything as flat paper cutouts. Characters are puppets of rigid paper parts, posed as a whole by a shift, a tilt in the screen plane and a squash. The traced characters have a line boil ([0003](docs/decisions/0003-character-art-pipeline.md)). The two bosses are flat placeholder ellipses. CI checks the core (tests, `cargo fmt`, `cargo clippy`, a wasm build) and the JavaScript (Biome) on every PR, runs the font check and the smoke test, and builds the wasm for the GitHub Pages deploy of `main`; the wasm is not committed ([0007](docs/decisions/0007-build-the-wasm-in-ci.md), #39).

What the maintainer found when they played it:

- **The action game lacks juice.** A skill's direction is not clear enough, and changing it is not intuitive. The charging time and the recovery time do not read, and a hit's feedback is weak enough that the timing feels off, even though the collision works as intended.
- **Balance and timings are placeholders,** tuned only against the scripted bot in `core/src/bot.rs`.

#43 fixes the first two points in part: a skill now follows the aim through its startup and locks when it hits, and the player's own skill shows as a wash that fills while it charges, with a ring at the feet through the recovery that marks the cancel point. The core also exports each character's animation state for the renderer. The rest is step 2.

## Summary

| Step | What | Size |
| --- | --- | --- |
| 1 | The action-game rebuild: controls, combat, enemies, skills and cards, rooms, two bosses (#43) | One large PR |
| 2 | Paper puppets in 3D: the new art direction, for charge, recovery and hit animations (four PRs, [0008](docs/decisions/0008-paper-puppet-art-plan.md)) | Large |
| 3 | Music and the low-health state | Small |
| 4 | Meta progression | Medium |
| 5 | More characters and stages | Medium, per character or stage |
| 6 | Touch controls | Medium to high |
| 7 | WebGL2 fallback | Medium to high |

## 1. The action-game rebuild

Built in #43. This section is the plan it was built from.

### Controls

- **Aim** with the mouse or the right stick. The core gets `jr_input(mx, my, aim_x, aim_y, buttons)`, with a bit per button.
- **Three active skills on three buttons,** plus a **roll**. Keyboard and mouse: left button, right button and Q for the skills, Space to roll. Gamepad: X, Y and RB for the skills, A or B to roll.
- **An input buffer** of about 120 ms, so a press just before an action can start still counts.
- **The roll** is every character's one fixed move: a short dodge with about 0.25 s of invulnerability, a short recovery so it cannot be chained, and no cooldown. It also breaks a grab and gets the player up faster after a knockdown. It replaces a dash; dashing is what some skills do.

### Combat

- **Skills have no cooldowns.** Each one is balanced by its timing: startup, active window and recovery. Light skills come out and recover fast; heavy skills are slow on both. The player may pick any three, including three heavy skills.
- **Cancel point.** A roll can cut a skill's recovery short only after its cancel point, which is early for light skills and late for heavy ones.
- **Hyper armor.** Heavy skills are not interrupted by normal enemies during their active window (the damage still lands). Boss hits always interrupt.
- **Hitboxes** have a shape (circle, arc, line, ring) and a timed active window, and hit each target once per use.
- **Hit feel:** hit-stop, knockback, stagger and damage numbers.
- **Aiming a skill.** A skill follows the aim through its startup, so it can be turned while it charges, and locks when its active window starts. A leap's landing point follows the aim point the same way.
- **The player's own skill is shown** like an enemy warning, in the skill's colour: a wash in the shape of the coming hit fills in through the startup. Through the recovery, a ring at the feet fills in ink until the cancel point, then turns to the skill's colour while a roll can cut the recovery short.
- **No contact damage.** Damage comes from attacks, except charges.

Starting skills (placeholder names), each defined in data by startup, active window, recovery, cancel point, damage, stagger, knockback, reach, hyper armor and movement:

| Skill | Weight | Feel |
| --- | --- | --- |
| Scythe combo | Light | A 3-hit sweep |
| Talisman throw | Light | An aimed shot |
| Lunge cut | Medium | Dashes about 140 units forward, cutting along a line |
| Spirit pulse | Medium | A nova around the player |
| Bead flail | Medium | Flings the beads into an orbit for about 2 s |
| Reaping charge | Heavy | A long wind-up, then a charge across the room with high stagger |
| Grave cleave | Heavy | A large overhead arc with high stagger |
| Crescent leap | Heavy | Leaps to the aim point and slams down |

Dash skills give no invulnerability; the roll is the only defence.

### Camera

A closer view (about 420 to 480 world units across the short side, from 640), a look-ahead of about 15% towards the aim, clamped to the room, widened in boss fights to keep the player and the boss in view. Shake stays, and a "reduce motion" setting turns off shake and hit-stop.

### Enemies

- At most about **10 enemies at once.** No hordes.
- Every enemy runs approach, wind-up with a warning, strike, recover. The recovery is the player's opening.
- **Warnings** are ink washes that grow on the ground in the shape of the coming hit. Every attack is warned at least about 0.4 s ahead, and big boss moves about 0.6 s.
- The current kinds get roles: the blot lunges, the wisp keeps its distance and fires slow ink shots, the brute slams the ground with a shock ring, and the elite is a mini-boss using some boss moves.

### Skills, cards and elements

- **One card pool.** The starting passive, the pick after each room and the skill upgrades are all cards from the same pool. Each card names what it attaches to: one skill, any skill, or the character. Most cards add an element to a skill.
- **Start of a run:** pick any 3 skills, then 1 passive from three character cards.
- **After each room:** pick one of three cards, weighted towards the skills and elements already in use.
- **One element per skill.** A new element card on that skill replaces the old one, after a confirm. Other upgrade cards, such as faster recovery or more reach, stack. Cards can also change the roll, for example a fire trail.
- **Elements:**
  - Fire: burn damage over time.
  - Ice: slows, then freezes.
  - Lightning: chains to the nearest enemy.
  - Celestial, which plays with gravity and stars: hits leave a star mark; at 3 marks a gravity well pulls nearby enemies together and a star falls on the spot. Bosses are never pulled; celestial hits add stagger to them instead.
- **Reactions** come last in this step: element hits leave tags on enemies that react with each other and are used up, for example gravity and fire making a collapsing-star blast.

This is #29's two axes in the new shape: a skill is the martial art, a card the element.

### Rooms and runs

A run is 4 to 6 fixed arena rooms of 2 to 3 waves each, a card pick after each clear, then a boss. No generated maps yet. Experience gems are removed. A run is won by beating the last boss. The save gets a version field and a migration.

### Bosses

Both are original designs that borrow move ideas from Monster Hunter: World's Deviljho and Alatreon, as #41 asks. Their names are placeholders.

Shared framework: a state machine of stalk, choose move, wind-up with warning, active, recover and staggered. Moves are data (warning shape, timings, range, weight, phases, cooldown) and picked by distance and weight. Hits fill a stagger meter; when it is full the boss falls for about 2 s and takes extra damage.

**The Glutton (Taotie 饕餮), first boss.** Relentless, combo-heavy, and more dangerous as it gets angry.

| Phase | When | Moves |
| --- | --- | --- |
| 1 | 100% to 60% health | Bite combo (2 to 3 lunging cones), tail sweep (an arc, used when the player is behind it), stomp (a circle and a shock ring), charge (a line; runs through and turns) |
| 2, enraged | Below 60% health, or after heavy damage in a short time; it stays enraged | Red ink veins, about 20% shorter wind-ups, moves chained 2 to 3 in a row. Adds ink breath, a long line swept across a wide arc, and a grab that pins the player until they press roll repeatedly |

It can summon blots and eat one nearby to heal or to enrage sooner; hitting it interrupts the eating.

**The Two-Element Dragon (Zhulong 燭龍), final boss.** It switches between fire and ice, and the right element is what keeps the player alive.

- **Stances** change at health thresholds or after about 45 s. Fire: dive-bombs, lingering fire pools, a spreading flame line. Ice: lines of ice lances, a freezing field that slows, falling shards.
- **Element check.** Each stance shows its weakness, the opposite element. Enough weak-element damage knocks it down. At the end of each stance a screen-wide finisher builds for about 3 s, and its damage drops with the weak-element damage dealt in that stance, down to a floor. The roll does not avoid it, and it never kills from full health on the first attempt.
- Left out for now: flight and a third stance.

The card offers make sure fire or ice cards appear before the final boss.

### Content and art

- One content file (skills with their timing, cards, enemies, boss moves) and a Node generator, run by `./build.sh`, that writes the ids for Rust, JavaScript and WGSL, so nothing is mirrored by hand.
- Puppet rigs posed from data: wind-up, attack, hurt and roll poses (the roll tucks and spins the cutout).
- The bosses ship as flat placeholder parts. Their real art follows the new art direction in step 2.
- Each character's animation state is exported to the host (`core/src/anim.rs`): phase, progress, cancel point, direction and the last hit. Nothing draws from it yet; step 2 poses characters from it.

### Commits

1. `refactor(core)`: fixed 60 Hz step with an accumulator in `jr_update`.
2. `feat`: aim, buttons, input buffer and the roll.
3. `feat(core)`: timing-based combat, hitboxes and hit feel; the auto weapons go.
4. `feat`: camera.
5. `feat(core)`: enemy behaviours, warnings and ink shots.
6. `build`: content file and generator.
7. `feat`: the eight skills and the skill pick.
8. `feat`: card pool, the four elements and the starting passive.
9. `feat`: rooms, waves, winning a run and the save migration.
10. `feat(core)`: puppet rigs.
11. `feat`: the Glutton.
12. `feat`: the Two-Element Dragon.
13. `feat(core)`: element tags and reactions.

After commit 5 the branch goes up as a draft PR for the maintainer to play, before the content is built on top.

### Done when

- The standard checks pass: `cargo test --release --manifest-path core/Cargo.toml`, `./build.sh`, then `npm run smoke`.
- The same seed and inputs give the same state hash twice.
- Tests cover: the roll's invulnerability window, recovery and cancel point; rolling through every attack except the finisher takes no damage; each skill's timing matches its data; hyper armor holds against normal enemies and breaks against bosses; dash skills move the set distance; each hitbox hits a target once; every attack is warned at least the minimum time ahead; each element effect and reaction; bosses are never pulled; the Glutton's phase change and enrage; the Dragon's finisher drops with weak-element damage and its knockdown triggers; the generator rejects duplicate or undefined ids; an old save migrates.
- A scripted bot that rolls clears a full run to the boss room, once with three light skills and once with three heavy skills.
- Frame time in a room is no worse than today's (about 8.3 ms median measured in the browser pane).

### Risks

Size. One PR touches nearly all of `core/` and much of `src/`. The ordered commits, the tests that come with each one and the draft PR after commit 5 are what keep it reviewable. Feel is the other risk: timing numbers start as placeholders and get tuned in play.

## 2. Paper puppets in 3D

The new art direction ([0006](docs/decisions/0006-paper-puppets-in-3d.md)). The game keeps its paper feel: characters stay flat paper cutouts with a white margin. But they move in 3D, the way a paper puppet held in the hand can turn, lean, lift and flip, and those movements carry the action game's timing so the player reads it from the body.

**Status: agreed with the maintainer, to be confirmed on #45.** The choices that shape later work are recorded in [0008](docs/decisions/0008-paper-puppet-art-plan.md); this section adds the detail for building it.

### The look

- **Stays:** the traced pencil line art, the per-stroke line boil at 4 fps, the white cutout margin, the cast shadow, and today's palette (ink `#2b2621`, paper `#f6f0df`, the muted part colours in `content/game.json`). Nothing changes on a character standing still.
- **Each part is a sheet of paper in 3D.** A part keeps its image, but it can turn about its vertical axis, lean towards or away from the camera, lift off the ground and bend a little. Turned edge-on it thins to a line, as a real card does.
- **The back of a sheet** is plain paper with the front's ink showing through faintly, mirrored. A turn or a flip then shows paper, not a mirrored drawing that pops.
- **Light:** one fixed light from the upper left. A sheet turned away from it is a little darker. The shadow stays on the ground, so a lifted sheet's shadow falls behind it and shows the height.
- **References** (for the maintainer to confirm or replace): Chinese shadow puppetry (皮影戲) for jointed flat figures moved on rods; paper theatre and pop-up books for sheets standing at different depths; stop-motion paper animation for the stepped boil. For hit feel, the action games 0005 names (Hades, Hyper Light Drifter): short hit-stop, a snap away from the hit, and a mark at the point of contact.

### What each animation does

Poses are built from the animation record the core exports (`core/src/anim.rs`): phase, progress, length, cancel point, direction and the last hit.

| Phase | Pose |
| --- | --- |
| Idle, walk | Upright, a small bob, a slight sway about the vertical axis |
| Startup (charging) | The sheet turns to face the skill's direction on the ground, in any direction rather than only left or right, and leans back as the startup runs. Over the last fifth it holds still and trembles, so "about to fire" reads |
| Active | Snaps forward, past upright, in a few frames |
| Recovery | Sags forward and a little crumpled. At the cancel point it pops upright with a small overshoot, so the moment a roll can cut in is seen on the body |
| Hurt | Spins about its vertical axis, away from the push. A hit worth less than a fifth of its max HP twists it, then it swings back and forth and the swing dies away; a heavier one spins it whole turns (one more per further fifth, up to three) before it wobbles to rest facing front. It cannot move or act until the swing settles, so the hit's damage sets how long it is stopped (`core/src/twist.rs`). Bosses have so much HP that they barely swing |
| Down | Spins two whole turns, then wobbles to rest; locked the same way |
| Grabbed | Pinned flat and shaking |
| Roll | Spins once about its vertical axis while tucking low, instead of turning in the screen plane |
| Stunned | A broken poise or a shatter spins it one whole turn, locked the same way |

Turning to face left or right becomes a quick turn through edge-on instead of an instant mirror. It is drawing only and never delays an action.

### Where poses are computed: in the core

The core already builds every instance and holds the rigs and today's 2D poses. The 3D pose becomes one function of a rig and an animation record, `pose(rig, anim) -> Pose3`, in `core/src/rigs.rs`, with each rig's 3D poses as data in `content/game.json`. Reasons: the draw list stays built in one place and one language; the pose code is tested where the timings are; and posing from the exported record means a test can feed it a made-up record. Drawing state, such as the eased facing turn, is kept out of `state_hash`, so replays stay exact. The ?debug panel gets pose sliders through a core setter, as the waddle has now, so tuning needs no wasm rebuild.

Set aside: posing on the drawing side in JavaScript from the exported record. It would iterate faster, but the instance list would then be built half in Rust and half in JavaScript, and the draw order and culling would have to be merged across the two.

### Renderer changes

- **Camera:** stays as today, an orthographic view of the ground, with the same framing, look-ahead and zoom. Only the sheets are 3D: each part is a quad oriented in 3D around its own anchor and projected orthographically. No perspective camera, so the gameplay view does not change.
- **Instance format:** from 12 to 16 floats (48 to 64 bytes): turn, lean, lift and bend join the existing in-plane rotation. `INST_FLOATS`, `INSTANCE_BYTES` and the vertex layout change together.
- **Depth order:** characters stay sorted by ground position as today. Within a puppet, the core orders the parts by their depth after the turn, so an arm behind the body goes behind it when the puppet turns. No depth buffer, because the margins and shadows are blended.
- **Bend:** the quad is split into a few strips in the vertex shader (more vertices per instance, no new buffers), so a sheet can curve.
- **The old outline wobble goes:** `stopMotionFps` and `wobble` in `LOOK`, `qtime` and `wobble` in the shader's globals, and the boil nudge on cutout parts. The line boil replaced it, and the 3D motion gives the placeholder art its life until the bosses are drawn.
- Everything stays plain WGSL with no compute passes, so the WebGL2 fallback (step 7) can still port it.

### Boss art

The Glutton and the Two-Element Dragon are drawn on paper by the artist and traced in the Lightbox, like the other characters ([0003](docs/decisions/0003-character-art-pipeline.md)). Unlike the small kinds, a boss is a jointed puppet: head, jaw, body, tail, legs and, for the Dragon, wings, each a sheet at its own depth, with a parent part and a pivot, moved per boss move (the jaw opens for a bite, the tail swings for the sweep). That needs a parent field per part in the character JSON and the Lightbox, and per-part poses in the rigs. Until the drawings exist, the bosses keep their placeholder parts and still get the 3D poses.

### Hit feedback

Once the poses are in: a torn-paper mark and a few paper scraps at the point of contact, flying away from the hit; a short push of the camera in the hit's direction (off with "reduce motion"); and hit-stop retuned against the new hurt snap. Damage numbers stay.

### PRs

Each PR is its own issue, as `CONTRIBUTING.md` asks, and each depends on the one before it. #39 (CI builds the wasm and checks the core) landed first, in #44.

| PR | What | Depends on | Size |
| --- | --- | --- | --- |
| A, #46 | Renderer: 16-float instances with turn, lean, lift and bend; the back of a sheet; the light; ordering parts by depth; the old outline wobble removed. The core sends zeros for the new fields, so the game looks as it does now, except for a ?debug toggle that spins every puppet to show the new fields work | #39 | Medium |
| B, #47 | Core: 3D poses from the animation record for the player and the four small kinds, the facing turn, pose data in `content/game.json`, pose sliders in ?debug, tests | A | Large |
| C, #48 | Hit feedback: paper marks and scraps, the camera push, hit-stop retuned | B | Small to medium |
| D, #49 | Boss art: jointed parts in the character JSON and the Lightbox, the traced Glutton and Dragon, per-part move poses | B, and the artist's drawings | Large |

A docs PR first, with the agreed version of this section and record 0008 (#45).

### Done when

- The standard checks pass: `cargo test --release --manifest-path core/Cargo.toml`, `./build.sh`, then `npm run smoke`.
- The same seed and inputs give the same state hash twice, and the hash does not change when only the drawing changes.
- Tests cover: the startup pose faces the skill's direction; the recovery pose changes at the cancel point; the hurt pose leans away from the hit and more for more damage; the down pose lies flat; a roll turns once; parts reorder when a puppet turns.
- The maintainer plays a run and can tell, from the characters alone, which way a skill points, how long until it fires, when a roll can cut its recovery, and when a hit landed.
- Frame time in a room is no worse than today's (about 8.3 ms median in the browser pane).

### Risks

The feel is the risk again: pose numbers start as placeholders and are tuned in play, which is why the ?debug sliders come in B. Boss art waits on the artist, so D may land well after the rest. Turning a traced part edge-on may show that some parts need fill behind them; the Lightbox's fill brush is the fix.

## 3. Music and the low-health state

A generative pentatonic loop in WebAudio (guqin-like plucks and a low drone) with its own volume setting, ducked during card picks. Below 25% health the vignette tints vermilion and a heartbeat plays.

## 4. Meta progression

Jade earned per run, spent in a paper "shrine" screen on small permanent bonuses (starting health, one reroll per card pick), stored in the save from `src/save.js` with a version bump.

## 5. More characters and stages

More reapers, each with a different starting set of skills and its own rig, and stages with different paper backgrounds (rice paper, indigo dyed paper, burnt edges) through the background shader's colours. Each one is its own PR.

## 6. Touch controls

Aim plus three skills and a roll on a phone is a design question of its own; settle it before building. The HUD would also need a pass for notches and safe areas.

## 7. WebGL2 fallback

Port `src/shader.wgsl` to GLSL ES 3.0 and add a second renderer behind the same `resize` and `draw` interface, so browsers without WebGPU can play. Detect WebGPU first and fall back automatically.

## Smaller items

- Pause when the gamepad disconnects mid-run.
- Show which gamepad is connected on the settings screen (`src/input.js` already tracks its name).
- Add a favicon and a social preview image.
