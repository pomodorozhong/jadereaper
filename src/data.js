// Tables shared with the Rust core. Part ids, events, enemy kinds, skills and moves are generated from
// content/game.json into src/content.js; the rest here must match core/src/defs.rs and core/src/game.rs by hand.
export { PART_IDS, EVENT, SKILLS, KINDS, MOVES, CARDS, ELEMENTS } from './content.js';

/** Which input reaches each skill slot, for the HUD. */
export const SLOT_KEYS = ['LMB', 'RMB', 'Q'];

/** Button bits sent to jr_input. Match B_* in core/src/defs.rs. */
export const BUTTON = { SKILL1: 1, SKILL2: 2, SKILL3: 4, ROLL: 8 };

/** The state array's length (STATE_FLOATS in core/src/defs.rs). */
export const STATE_FLOATS = 64;

/** Indices into the state array written by the core. */
export const STATE = {
  HP: 0, MAX_HP: 1, ROOM: 2, ROOMS: 3, WAVE: 4, TIME: 5, KILLS: 6,
  CHOOSING: 7, CHOICE0: 8, OVER: 11, CAM_X: 12, CAM_Y: 13, ENEMIES: 14,
  RUNNING: 15, ELEMENT0: 16, PICK: 19, CHOICE_SLOT0: 20, WAVES: 23, WON: 24, BOSS: 25, PLAYER_X: 26, PLAYER_Y: 27, LOADOUT0: 28, ZOOM: 31,
  OWNED0: 32,
};
