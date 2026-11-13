// Tables shared with the Rust core. Ids must match core/src/defs.rs.

/** Part names you can override with an image in assets/parts.json. */
export const PART_IDS = {
  ellipse: 0,
  player_body: 1,
  player_head: 2,
  scythe: 3,
  blot: 4,
  wisp: 5,
  brute: 6,
  gem: 7,
  slash: 8,
  talisman: 9,
  bead: 10,
  splat: 11,
  grass: 12,
  stone: 13,
  heart: 14,
  ring: 15,
  scrap: 16,
  ground_shadow: 18,
};

/** Upgrades, indexed by id. `glyph` is the brush mark shown on cards and HUD chips.
 * Names and descriptions are in src/i18n.js as upgrade.<id>.name and .desc. */
export const UPGRADES = [
  { glyph: '鐮', max: 6 },
  { glyph: '符', max: 6 },
  { glyph: '珠', max: 6 },
  { glyph: '波', max: 6 },
  { glyph: '命', max: 5 },
  { glyph: '風', max: 5 },
  { glyph: '引', max: 5 },
  { glyph: '力', max: 5 },
  { glyph: '速', max: 5 },
  { glyph: '癒', max: 5 },
];

export const EVENT = {
  HIT: 1, KILL: 2, PICKUP: 3, LEVELUP: 4, HURT: 5, SLASH: 6,
  SHOOT: 7, PULSE: 8, DEATH: 9, HEAL: 10, ELITE: 11,
};

/** Indices into the state array written by the core. */
export const STATE = {
  HP: 0, MAX_HP: 1, XP: 2, XP_NEXT: 3, LEVEL: 4, TIME: 5, KILLS: 6,
  CHOOSING: 7, CHOICE0: 8, OVER: 11, CAM_X: 12, CAM_Y: 13, ENEMIES: 14,
  RUNNING: 15, UPGRADE0: 16,
};
