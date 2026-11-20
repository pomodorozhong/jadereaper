// Tables shared with the Rust core. Part ids, events, enemy kinds, skills and moves are generated from
// content/game.json into src/content.js; the rest here must match core/src/defs.rs and core/src/game.rs by hand.
export { PART_IDS, EVENT, SKILLS, KINDS, MOVES } from './content.js';

/** Upgrades, indexed by id. `glyph` is the brush mark shown on cards and HUD chips.
 * Names and descriptions are in src/i18n.js as upgrade.<id>.name and .desc. */
export const UPGRADES = [
  { glyph: '命', max: 5 },
  { glyph: '風', max: 5 },
  { glyph: '引', max: 5 },
  { glyph: '力', max: 5 },
  { glyph: '癒', max: 5 },
];

/** Which input reaches each skill slot, for the HUD. */
export const SLOT_KEYS = ['LMB', 'RMB', 'Q'];

/** Button bits sent to jr_input. Match B_* in core/src/defs.rs. */
export const BUTTON = { SKILL1: 1, SKILL2: 2, SKILL3: 4, ROLL: 8 };

/** Indices into the state array written by the core. */
export const STATE = {
  HP: 0, MAX_HP: 1, XP: 2, XP_NEXT: 3, LEVEL: 4, TIME: 5, KILLS: 6,
  CHOOSING: 7, CHOICE0: 8, OVER: 11, CAM_X: 12, CAM_Y: 13, ENEMIES: 14,
  RUNNING: 15, UPGRADE0: 16, PLAYER_X: 26, PLAYER_Y: 27, LOADOUT0: 28, ZOOM: 31,
};
