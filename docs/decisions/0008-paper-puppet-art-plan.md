# 0008: Paper sheets in 3D, posed in the core, with the traced line art kept

- Status: decided
- Date: 2026-11-21
- Issue: #45, where it was decided

## Context

Decision [0006](0006-paper-puppets-in-3d.md) chose to keep the paper look and move the paper puppets in 3D, and left roadmap step 2 to settle the rest: what the art looks like, what happens to the traced line art, the line boil, the shader's old outline wobble and the Lightbox, where the 3D poses are computed, and how the bosses are drawn.

## Decision

- **The look stays as it is at rest.** The traced pencil line art, the per-stroke line boil, the white cutout margin, the ground shadow and today's palette all stay. Each part becomes a sheet of paper that can turn, lean, lift and bend in 3D. The back of a sheet is plain paper with the front's ink showing through faintly, and one fixed light darkens sheets turned away from it.
- **The camera stays orthographic** with today's framing. Only the sheets are 3D, each oriented around its own anchor. Parts within a puppet are ordered by depth after the turn; there is no depth buffer.
- **The core computes the 3D poses,** as one function of a rig and the animation record from `core/src/anim.rs`, with the pose numbers in `content/game.json`. Drawing state stays out of `state_hash`, so replays stay exact.
- **The shader's old outline wobble is removed** (`stopMotionFps` and `wobble` in `LOOK`, and the matching shader globals). The line boil replaced it.
- **The Lightbox stays** the way characters get into the game ([0003](0003-character-art-pipeline.md)). The bosses are drawn on paper by the artist and traced as jointed puppets, which adds a parent part to the character JSON.

## Reasons

Keeping the traced line art and the boil keeps the paper identity 0006 asked for, and a foreshortened sheet still reads as paper. An orthographic camera leaves the gameplay view and its tuning alone. Posing in the core keeps the draw list built in one place and one language, next to the timings and their tests, and posing from the exported record lets a test feed it a made-up record.

Set aside: posing on the drawing side in JavaScript (faster to iterate, but the instance list would be built half in Rust and half in JavaScript); a perspective camera (changes the framing the combat was tuned in); keeping the outline wobble for placeholder art (#40), since the placeholder art goes once the bosses are drawn.

## Consequences

- The instance format grows from 12 to 16 floats, and `INST_FLOATS`, `INSTANCE_BYTES` and the vertex layout change together.
- Rigs in `content/game.json` gain 3D poses per animation phase, and boss rigs gain per-part poses.
- The character JSON gains an optional parent per part; the format in `docs/character-art-pipeline.md` changes with it.
- The shader must stay plain WGSL without compute passes, so the WebGL2 fallback (roadmap step 7) can port it.
- Moving the poses to the drawing side, or adding a perspective camera, needs a new record that supersedes this one.
