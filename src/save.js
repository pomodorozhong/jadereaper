// Records and settings, kept in localStorage. The game still works if
// storage is blocked: values just do not persist.
//
// The save carries a version. loadSave() migrates older saves, including the first one, which had no version and
// lived under its own key, so records and settings carry over.

const KEY = 'jadereaper.save';
/** Where the first save (version 1, from the survivor game) was kept. */
const KEY_V1 = 'jadereaper.save.v1';
export const VERSION = 2;

const DEFAULTS = {
  version: VERSION,
  bestRoom: 0, // most rooms cleared in one run
  wins: 0,
  fastestWin: 0, // seconds, 0 for none yet
  runs: 0,
  totalKills: 0,
  volume: 0.6,
  muted: false,
  reduceMotion: false, // no camera shake or hit-stop
  loadout: [], // the skill ids picked last time, in button order
  lang: null, // null: use DEFAULT_LANG from i18n.js
  legacy: null, // version 1's records (best time, kills, level), kept but no longer shown
};

/** Bring a stored save up to the current version. Unknown or broken data gives the defaults. */
export function migrate(data) {
  if (!data || typeof data !== 'object' || Array.isArray(data)) return { ...DEFAULTS };
  let save = { ...data };
  if (!save.version) {
    // Version 1: the survivor game's records. Runs, kills and settings carry over; its best time, kills and level
    // measured a different game, so they are kept aside.
    const { bestTime = 0, bestKills = 0, bestLevel = 0, ...rest } = save;
    save = { ...rest, version: 2, legacy: { bestTime, bestKills, bestLevel } };
  }
  return { ...DEFAULTS, ...save, version: VERSION };
}

export function loadSave() {
  try {
    const raw = localStorage.getItem(KEY) ?? localStorage.getItem(KEY_V1);
    return migrate(raw ? JSON.parse(raw) : {});
  } catch {
    return { ...DEFAULTS };
  }
}

export function writeSave(save) {
  try {
    localStorage.setItem(KEY, JSON.stringify(save));
  } catch {
    // Private window or storage disabled.
  }
}

/** Fold a finished run into the save. Returns which records were beaten. */
export function recordRun(save, { rooms, kills, time, won }) {
  const beaten = {
    rooms: rooms > save.bestRoom,
    win: won && (!save.fastestWin || time < save.fastestWin),
  };
  save.bestRoom = Math.max(save.bestRoom, rooms);
  if (won) {
    save.wins += 1;
    if (beaten.win) save.fastestWin = time;
  }
  save.runs += 1;
  save.totalKills += kills;
  writeSave(save);
  return beaten;
}

export function formatTime(seconds) {
  const s = Math.floor(seconds);
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`;
}
