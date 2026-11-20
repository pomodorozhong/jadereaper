// Host: loads the wasm core, runs the frame loop and ties input, sound,
// saving and the UI together. The game rules live in core/ (Rust).
import { createRenderer, LOOK } from './renderer.js';
import { createInput } from './input.js';
import { createAudio } from './audio.js';
import { createUI } from './ui.js';
import { loadSave, writeSave, recordRun } from './save.js';
import { EVENT, STATE, STATE_FLOATS, SKILLS, CARDS, ELEMENTS } from './data.js';
import { pickLang, setLang, getLang, nextLang } from './i18n.js';
import { BAKE, SIZE, SIZE_ORDER } from './characters.js';


/** Each monster's waddle: how far it tilts each way (degrees) and how often it changes pose (per second).
 * Tuned by the maintainer. Mirrors KINDS in core/src/defs.rs, which the core starts from; the ?debug panel sends
 * changes (jr_set_waddle). */
const WADDLE = {
  blotTilt: 2.25, blotRate: 2, wispTilt: 3.75, wispRate: 9, bruteTilt: 7.5, bruteRate: 3, eliteTilt: 7.5, eliteRate: 2.5,
};
const WADDLE_KINDS = ['blot', 'wisp', 'brute', 'elite'];

/** ?debug: sliders for every visual setting. Bake settings re-draw the traced characters; look settings apply live. */
const DEBUG_SLIDERS = [
  ['Bake (re-draws the characters)'],
  ['bake', 'lineWidth', 'Line width', 0.3, 4, 0.1],
  ['bake', 'boilAmp', 'Line boil', 0, 2, 0.05],
  ['bake', 'poses', 'Boil poses', 1, 8, 1],
  ['bake', 'texels', 'Sharpness', 1, 6, 0.5],
  ['Look (live)'],
  ['look', 'boilFps', 'Boil fps', 0, 24, 1],
  ['look', 'margin', 'Paper margin', 0, 6, 0.1],
  ['look', 'shadowX', 'Shadow x', -8, 8, 0.5],
  ['look', 'shadowY', 'Shadow y', -8, 8, 0.5],
  ['look', 'shadowAlpha', 'Shadow darkness', 0, 1, 0.05],
  ['Size (drawn only; re-draws the characters)'],
  ['size', 'player', 'Player', 0.5, 2, 0.05],
  ['size', 'blot', 'Blot', 0.5, 2, 0.05],
  ['size', 'wisp', 'Wisp', 0.5, 2, 0.05],
  ['size', 'brute', 'Brute', 0.5, 2, 0.05],
  ['size', 'elite', 'Elite', 0.5, 2, 0.05],
  ['Waddle (live): tilt each way in degrees, and pose changes per second'],
  ['waddle', 'blotTilt', 'Blot tilt', 0, 30, 0.25],
  ['waddle', 'blotRate', 'Blot rate', 0, 15, 0.5],
  ['waddle', 'wispTilt', 'Wisp tilt', 0, 30, 0.25],
  ['waddle', 'wispRate', 'Wisp rate', 0, 15, 0.5],
  ['waddle', 'bruteTilt', 'Brute tilt', 0, 30, 0.25],
  ['waddle', 'bruteRate', 'Brute rate', 0, 15, 0.5],
  ['waddle', 'eliteTilt', 'Elite tilt', 0, 30, 0.25],
  ['waddle', 'eliteRate', 'Elite rate', 0, 15, 0.5],
];
function addDebugPanel(renderer, core) {
  const sets = { bake: BAKE, look: LOOK, size: SIZE, waddle: WADDLE };
  const defaults = { bake: { ...BAKE }, look: { ...LOOK }, size: { ...SIZE }, waddle: { ...WADDLE } };
  const panel = document.createElement('div');
  panel.style.cssText = 'position:fixed;left:12px;bottom:12px;z-index:50;background:#fffdf6ee;border:1px solid #b9ae95;border-radius:6px;padding:6px 10px;font:12px system-ui,sans-serif;color:#2b2622;max-height:calc(100vh - 24px);overflow:auto';
  const btn = 'all:unset;cursor:pointer;padding:0 6px;border:1px solid #b9ae95;border-radius:4px';
  const row = (set, key, label, min, max, step) => `<div style="display:grid;grid-template-columns:9.5em 8em 2.6em auto;gap:6px;align-items:center">
    <label for="dbg_${set}_${key}">${label}</label><input id="dbg_${set}_${key}" data-set="${set}" data-key="${key}" type="range" min="${min}" max="${max}" step="${step}" value="${sets[set][key]}">
    <output style="font-variant-numeric:tabular-nums">${sets[set][key]}</output><button type="button" data-reset="${key}" title="Reset to ${defaults[set][key]}" style="${btn}">↺</button></div>`;
  panel.innerHTML = `<div style="display:flex;justify-content:space-between;gap:8px;margin-bottom:4px"><b>Debug</b><span>
    <button type="button" id="dbgCopy" title="Copy the current values as JSON" style="${btn}">Copy values</button>
    <button type="button" id="dbgResetAll" style="${btn}">Reset all</button>
    <button type="button" id="dbgFold" style="${btn}">Hide</button></span></div>
    <div id="dbgBody" style="display:flex;flex-direction:column;gap:3px">${DEBUG_SLIDERS.map((r) => (r.length === 1 ? `<div style="margin-top:4px;color:#7a6f5c">${r[0]}</div>` : row(...r))).join('')}</div>`;
  // Keep game keys (WASD, Esc) from firing while a slider has focus.
  panel.addEventListener('keydown', (e) => e.stopPropagation());
  document.body.append(panel);

  // Re-bakes run one at a time; while one runs, only the latest wanted settings are kept.
  let baking = false, again = false;
  const rebake = async () => {
    if (baking) { again = true; return; }
    baking = true;
    do { again = false; await renderer.rebake({ ...BAKE }); } while (again);
    baking = false;
  };
  const set = (el, v) => {
    const { set: which, key } = el.dataset;
    sets[which][key] = v; el.value = v; el.nextElementSibling.textContent = v;
    if (key === 'boilFps' || key === 'poses') core.jr_set_boil(LOOK.boilFps, BAKE.poses);
    if (which === 'size') core.jr_set_draw_scale(SIZE_ORDER.indexOf(key), v);
    if (which === 'waddle') {
      const kind = WADDLE_KINDS.findIndex((k) => key.startsWith(k));
      const name = WADDLE_KINDS[kind];
      core.jr_set_waddle(kind, WADDLE[name + 'Tilt'] * Math.PI / 180, WADDLE[name + 'Rate']);
    }
    if (which === 'bake' || which === 'size') rebake();
  };
  const sliders = [...panel.querySelectorAll('input[type=range]')];
  for (const el of sliders) {
    el.oninput = () => set(el, +el.value);
    el.parentElement.querySelector('[data-reset]').onclick = () => set(el, defaults[el.dataset.set][el.dataset.key]);
  }
  panel.querySelector('#dbgResetAll').onclick = () => {
    for (const el of sliders) set(el, defaults[el.dataset.set][el.dataset.key]);
  };
  panel.querySelector('#dbgCopy').onclick = () => navigator.clipboard?.writeText(JSON.stringify({ bake: BAKE, look: LOOK, size: SIZE, waddle: WADDLE }, null, 1));
  panel.querySelector('#dbgFold').onclick = (e) => {
    const body = panel.querySelector('#dbgBody');
    body.hidden = !body.hidden; e.target.textContent = body.hidden ? 'Show' : 'Hide';
  };
}

async function boot() {
  const canvas = document.getElementById('game');
  const save = loadSave();
  setLang(pickLang(save.lang));
  const audio = createAudio();
  audio.setVolume(save.volume);
  audio.setMuted(save.muted);

  // 'title' | 'pick' | 'playing' | 'levelup' | 'paused' | 'over'
  let mode = 'title';
  // Skills picked for the run, in button order (left, right, Q).
  let picked = Array.isArray(save.loadout) ? save.loadout.filter((id) => SKILLS[id]).slice(0, 3) : [];
  let settingsFrom = 'title';

  const ui = createUI({ onAction });
  const input = createInput({ onKey, canvas });

  let renderer, core;
  try {
    renderer = await createRenderer(canvas);
    const bytes = await (await fetch('wasm/jadereaper_core.wasm', { cache: 'no-cache' })).arrayBuffer();
    core = (await WebAssembly.instantiate(bytes, {})).instance.exports;
  } catch (err) {
    console.error(err);
    ui.error(err.message || String(err));
    return;
  }

  // The core picks each traced part's line boil pose, so it needs the rate and the number of baked poses.
  core.jr_set_boil(LOOK.boilFps, BAKE.poses);
  core.jr_set_reduce_motion(save.reduceMotion ? 1 : 0);
  SIZE_ORDER.forEach((who, i) => core.jr_set_draw_scale(i, SIZE[who]));
  if (new URLSearchParams(location.search).has('debug')) addDebugPanel(renderer, core);

  const seed = () => (Math.random() * 0xffffffff) >>> 0;
  // Views are rebuilt on every read because wasm memory can grow and detach them.
  const readState = () => new Float32Array(core.memory.buffer, core.jr_state_ptr(), STATE_FLOATS);
  const three = (s, at) => Array.from(s.subarray(at, at + 3));
  // A card offer waiting for a confirm: the slot of an element card that would replace another element.
  let armed = -1;
  const offer = (s) => ({
    choices: three(s, STATE.CHOICE0), slots: three(s, STATE.CHOICE_SLOT0), kind: s[STATE.PICK],
    loadout: three(s, STATE.LOADOUT0), elements: three(s, STATE.ELEMENT0), armed,
  });
  /** True when taking offered card i would replace another element on its skill, so it needs a second pick. */
  function needsConfirm(s, i) {
    const card = CARDS[s[STATE.CHOICE0 + i]];
    const slot = s[STATE.CHOICE_SLOT0 + i];
    if (!card || card.element === 'none' || slot < 0) return false;
    const old = s[STATE.ELEMENT0 + slot];
    return old > 0 && ELEMENTS[old - 1].name !== card.element;
  }

  function toTitle() {
    core.jr_reset(seed());
    mode = 'title';
    ui.showHud(false);
    ui.records(save);
    ui.show('title');
  }

  function pickSkills() {
    mode = 'pick';
    ui.showHud(false);
    ui.skillPick(picked);
  }

  function startRun() {
    if (picked.length !== 3) { pickSkills(); return; }
    core.jr_start(seed());
    core.jr_set_loadout(...picked);
    core.jr_offer_passive();
    core.jr_update(0);
    armed = -1;
    mode = 'playing';
    ui.showHud(true);
    ui.show(null);
  }

  function setPaused(paused) {
    if (paused && mode === 'playing') {
      mode = 'paused';
      ui.show('pause');
    } else if (!paused && mode === 'paused') {
      mode = 'playing';
      ui.show(null);
    }
  }

  function saveSettings() {
    save.volume = audio.volume;
    save.muted = audio.muted;
    writeSave(save);
    ui.settings(save);
  }

  function onAction(action, value) {
    audio.unlock();
    audio.play(action === 'choose' || action === 'start' ? 'confirm' : 'ui');
    switch (action) {
      case 'start': pickSkills(); break;
      case 'pick-skill': {
        const id = Number(value);
        const at = picked.indexOf(id);
        if (at >= 0) picked.splice(at, 1);
        else if (picked.length < 3) picked.push(id);
        ui.skillPick(picked);
        break;
      }
      case 'pick-begin':
        save.loadout = [...picked];
        writeSave(save);
        startRun();
        break;
      case 'again': startRun(); break;
      case 'resume': setPaused(false); break;
      case 'restart': startRun(); break;
      case 'quit': toTitle(); break;
      case 'settings': settingsFrom = ui.screen; ui.show('settings'); break;
      case 'back': ui.show(settingsFrom); break;
      case 'lang':
        setLang(nextLang(getLang()));
        save.lang = getLang();
        saveSettings();
        ui.records(save);
        break;
      case 'mute': audio.setMuted(!audio.muted); saveSettings(); break;
      case 'motion':
        save.reduceMotion = !save.reduceMotion;
        core.jr_set_reduce_motion(save.reduceMotion ? 1 : 0);
        saveSettings();
        break;
      case 'vol-up': audio.setVolume(audio.volume + 0.1); saveSettings(); audio.play('confirm'); break;
      case 'vol-down': audio.setVolume(audio.volume - 0.1); saveSettings(); audio.play('confirm'); break;
      case 'clear-save':
        Object.assign(save, { bestRoom: 0, wins: 0, fastestWin: 0, runs: 0, totalKills: 0, legacy: null });
        writeSave(save);
        ui.records(save);
        break;
      case 'choose':
        if (mode === 'levelup') {
          const i = Number(value);
          if (needsConfirm(readState(), i) && armed !== i) {
            armed = i;
            ui.cards(offer(readState()));
            break;
          }
          armed = -1;
          core.jr_choose(i);
          afterChoice();
        }
        break;
    }
  }

  /** The core may offer another choice straight away if several levels were gained at once. */
  function afterChoice() {
    core.jr_update(0);
    const s = readState();
    if (s[STATE.CHOOSING]) {
      ui.cards(offer(s));
    } else {
      mode = 'playing';
      ui.show(null);
    }
  }

  function back() {
    if (ui.screen === 'settings') onAction('back');
    else if (mode === 'paused') setPaused(false);
  }

  function onKey(key) {
    audio.unlock();
    if (key.pause) {
      if (mode === 'playing') setPaused(true);
      else back();
    } else if (key.nav && ui.screen) {
      if (ui.navigate(key.nav[0], key.nav[1])) audio.play('ui');
    } else if (key.pick !== undefined && mode === 'levelup' && !ui.locked) {
      const card = document.querySelectorAll('#cards .card')[key.pick];
      if (card) card.click();
    } else if (key.mute) {
      audio.setMuted(!audio.muted);
      saveSettings();
    }
  }

  window.addEventListener('pointerdown', () => audio.unlock());
  window.addEventListener('blur', () => setPaused(true));
  document.addEventListener('visibilitychange', () => { if (document.hidden) setPaused(true); });

  ui.settings(save);
  toTitle();

  /** The aim as an offset from the player in world units: the mouse's ground point, or the stick's direction. */
  function aimOffset(aim, s, view) {
    if (!aim) return [0, 0];
    if (aim.kind === 'stick') return [aim.x * 200, aim.y * 200];
    const r = canvas.getBoundingClientRect();
    const wx = s[STATE.CAM_X] + ((aim.x - r.left) / r.width - 0.5) * view.w;
    const wy = s[STATE.CAM_Y] + ((aim.y - r.top) / r.height - 0.5) * view.h;
    return [wx - s[STATE.PLAYER_X], wy - s[STATE.PLAYER_Y]];
  }

  let last = performance.now();
  let clock = 0;

  function frame(now) {
    const dt = Math.min(0.05, Math.max(0, (now - last) / 1000));
    last = now;

    const view = renderer.resize(readState()[STATE.ZOOM]);
    core.jr_set_view(view.w, view.h);

    const pad = input.poll(dt);
    if (pad.pause) onKey({ pause: true });
    if (pad.back) back();
    if (ui.screen) {
      if (pad.nav && ui.navigate(pad.nav[0], pad.nav[1])) audio.play('ui');
      if (pad.confirm && !ui.locked) { audio.unlock(); ui.confirm(); }
    }

    // Paused freezes everything. Other menus keep the paper puppets moving.
    const step = mode === 'paused' ? 0 : dt;
    clock += step;
    if (mode === 'playing') {
      const [ax, ay] = aimOffset(pad.aim, readState(), view);
      core.jr_input(pad.mx, pad.my, ax, ay, pad.buttons);
    } else {
      core.jr_input(0, 0, 0, 0, 0);
    }
    core.jr_update(step);

    const s = readState();
    if (mode === 'playing') {
      const count = core.jr_event_count();
      if (count) {
        const ev = new Float32Array(core.memory.buffer, core.jr_events_ptr(), count * 4);
        const r = canvas.getBoundingClientRect();
        for (let i = 0; i < count; i++) {
          audio.play(ev[i * 4]);
          if (ev[i * 4] === EVENT.HIT) {
            // Lift the number from the ground point to about chest height.
            const sx = r.left + ((ev[i * 4 + 1] - s[STATE.CAM_X]) / view.w + 0.5) * r.width;
            const sy = r.top + ((ev[i * 4 + 2] - 36 - s[STATE.CAM_Y]) / view.h + 0.5) * r.height;
            ui.damage(sx, sy, ev[i * 4 + 3]);
          }
        }
      }
      ui.hud({
        hp: s[STATE.HP], maxHp: s[STATE.MAX_HP], room: s[STATE.ROOM], rooms: s[STATE.ROOMS], wave: s[STATE.WAVE],
        waves: s[STATE.WAVES], boss: s[STATE.BOSS], time: s[STATE.TIME], kills: s[STATE.KILLS],
        loadout: three(s, STATE.LOADOUT0), elements: three(s, STATE.ELEMENT0),
        owned: Array.from(s.subarray(STATE.OWNED0, STATE.OWNED0 + CARDS.length)),
        bossBar: s[STATE.BOSS_KIND] ? {
          kind: s[STATE.BOSS_KIND] - 1, health: s[STATE.BOSS_HP], max: s[STATE.BOSS_MAX], meter: s[STATE.BOSS_METER], flags: s[STATE.BOSS_FLAGS],
          stance: s[STATE.STANCE], finisher: s[STATE.FINISHER],
        } : null,
        grabbed: s[STATE.ACT] === 5,
      });
      if (s[STATE.OVER]) {
        mode = 'over';
        const won = !!s[STATE.WON];
        // Rooms cleared: the room reached, plus the last one when the run is won.
        const run = { rooms: s[STATE.ROOM] + (won ? 1 : 0), kills: s[STATE.KILLS], time: s[STATE.TIME], won };
        ui.gameOver(run, save, recordRun(save, run));
      } else if (s[STATE.CHOOSING]) {
        mode = 'levelup';
        armed = -1;
        ui.cards(offer(s));
      }
    }

    renderer.draw(core.memory.buffer, core.jr_instances_ptr(), core.jr_instance_count(), s[STATE.CAM_X], s[STATE.CAM_Y], clock);
    requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);

  // Handy in the console: jadereaper.core, jadereaper.state().
  window.jadereaper = { core, state: () => Array.from(readState()), get mode() { return mode; } };
}

boot();
