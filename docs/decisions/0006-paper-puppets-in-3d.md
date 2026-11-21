# 0006: Keep the paper look, and animate the paper puppets in 3D

- Status: decided
- Date: 2026-11-21
- Issue: none. The maintainer decided this directly after playing #43.

## Context

#43 rebuilt the game as a room-based action game ([0005](0005-room-based-action-game.md)). Characters are flat paper puppets: rigid parts posed as a whole by a shift, a tilt in the screen plane and a squash. When the maintainer played it, the game lacked the juice an action game needs. A skill's direction was not clear and was hard to change, the charging and recovery times did not read, and hits gave so little feedback that the timing felt off, although the collision worked as intended. The 2D poses have too little room to show any of that on the body.

## Decision

- **The paper feel stays.** Characters remain flat paper cutouts.
- **The cutouts move in 3D.** Like a paper puppet held in the hand, a cutout can turn, lean towards or away from the camera, lift and flip. The charging, recovery and getting-hit animations are built from these movements, so the timing reads from the character's body.
- **The core keeps the timings.** It exports each character's animation state (phase, progress, cancel point, direction, last hit) in `core/src/anim.rs`, and the animations are posed from that state. Replays stay exact.
- **Readability fixes that need no new art came first, in #43.** A skill follows the aim through its startup and locks when it hits. The player's own skill shows as a ground wash while it charges, with a ring at the feet that marks the cancel point through the recovery.

The 3D animation work itself is roadmap step 2, its own PR. That step settles what the art looks like beyond this: references, palette, line style, the fate of the traced line art, the line boil, the old shader wobble and the Lightbox, and where 3D poses are computed.

## Reasons

The maintainer wants to keep the paper look, and moving the paper in 3D gives the body the range to show direction, charge, recovery and impact without turning the game into 3D models or frame-by-frame sprites. Exporting the animation state from the core keeps the timing rules in one place, where the tests hold them, and lets the drawing side change freely.

Set aside, for now: only adding ground marks and effects on top of the 2D poses (they help, and #43 adds some, but they do not put the timing on the character), and leaving the art as it is until balance is tuned (the feel problems make tuning in play unreliable).

## Consequences

- The renderer needs a 3D view of flat parts: perspective or a tilted orthographic camera, a depth order, and an instance format that carries a 3D rotation.
- Rigs and poses in `content/game.json` will gain 3D fields, or be replaced by poses computed on the drawing side; step 2 decides which.
- The traced-art pipeline ([0003](0003-character-art-pipeline.md)) still applies until step 2 decides otherwise.
- Changing away from flat paper cutouts needs a new record that supersedes this one.
