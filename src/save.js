// Records and settings, kept in localStorage. The game still works if
// storage is blocked: values just do not persist.

const KEY = 'jadereaper.save.v1';

const DEFAULTS = {
  bestTime: 0,
  bestKills: 0,
  bestLevel: 0,
  runs: 0,
  totalKills: 0,
  volume: 0.6,
  muted: false,
  lang: null, // null: use DEFAULT_LANG from i18n.js
};

export function loadSave() {
  try {
    const raw = localStorage.getItem(KEY);
    const data = raw ? JSON.parse(raw) : {};
    return { ...DEFAULTS, ...(data && typeof data === 'object' ? data : {}) };
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
export function recordRun(save, { time, kills, level }) {
  const beaten = {
    time: time > save.bestTime,
    kills: kills > save.bestKills,
    level: level > save.bestLevel,
  };
  save.bestTime = Math.max(save.bestTime, time);
  save.bestKills = Math.max(save.bestKills, kills);
  save.bestLevel = Math.max(save.bestLevel, level);
  save.runs += 1;
  save.totalKills += kills;
  writeSave(save);
  return beaten;
}

export function formatTime(seconds) {
  const s = Math.floor(seconds);
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`;
}
