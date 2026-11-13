// Host: loads the wasm core, runs the frame loop and ties input, sound,
// saving and the UI together. The game rules live in core/ (Rust).
import { createRenderer } from './renderer.js';
import { createInput } from './input.js';
import { createAudio } from './audio.js';
import { createUI } from './ui.js';
import { loadSave, writeSave, recordRun } from './save.js';
import { EVENT, STATE, UPGRADES } from './data.js';

const STATE_FLOATS = 32;

async function boot() {
  const canvas = document.getElementById('game');
  const save = loadSave();
  const audio = createAudio();
  audio.setVolume(save.volume);
  audio.setMuted(save.muted);

  // 'title' | 'playing' | 'levelup' | 'paused' | 'over'
  let mode = 'title';
  let settingsFrom = 'title';

  const ui = createUI({ onAction });
  const input = createInput({ onKey });

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

  const seed = () => (Math.random() * 0xffffffff) >>> 0;
  // Views are rebuilt on every read because wasm memory can grow and detach them.
  const readState = () => new Float32Array(core.memory.buffer, core.jr_state_ptr(), STATE_FLOATS);
  const upgradeLevels = (s) => Array.from(s.subarray(STATE.UPGRADE0, STATE.UPGRADE0 + UPGRADES.length));

  function toTitle() {
    core.jr_reset(seed());
    mode = 'title';
    ui.showHud(false);
    ui.records(save);
    ui.show('title');
  }

  function startRun() {
    core.jr_start(seed());
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
      case 'start': startRun(); break;
      case 'resume': setPaused(false); break;
      case 'restart': startRun(); break;
      case 'quit': toTitle(); break;
      case 'settings': settingsFrom = ui.screen; ui.show('settings'); break;
      case 'back': ui.show(settingsFrom); break;
      case 'mute': audio.setMuted(!audio.muted); saveSettings(); break;
      case 'vol-up': audio.setVolume(audio.volume + 0.1); saveSettings(); audio.play('confirm'); break;
      case 'vol-down': audio.setVolume(audio.volume - 0.1); saveSettings(); audio.play('confirm'); break;
      case 'clear-save':
        Object.assign(save, { bestTime: 0, bestKills: 0, bestLevel: 0, runs: 0, totalKills: 0 });
        writeSave(save);
        ui.records(save);
        break;
      case 'choose':
        if (mode === 'levelup') {
          core.jr_choose(Number(value));
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
      ui.levelUp(Array.from(s.subarray(STATE.CHOICE0, STATE.CHOICE0 + 3)), upgradeLevels(s));
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

  let last = performance.now();
  let clock = 0;

  function frame(now) {
    const dt = Math.min(0.05, Math.max(0, (now - last) / 1000));
    last = now;

    const view = renderer.resize();
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
    core.jr_input(mode === 'playing' ? pad.mx : 0, mode === 'playing' ? pad.my : 0);
    core.jr_update(step);

    const s = readState();
    if (mode === 'playing') {
      const count = core.jr_event_count();
      if (count) {
        const ev = new Float32Array(core.memory.buffer, core.jr_events_ptr(), count * 4);
        for (let i = 0; i < count; i++) audio.play(ev[i * 4]);
      }
      ui.hud({
        hp: s[STATE.HP], maxHp: s[STATE.MAX_HP], xp: s[STATE.XP], xpNext: s[STATE.XP_NEXT],
        level: s[STATE.LEVEL], time: s[STATE.TIME], kills: s[STATE.KILLS], upgrades: upgradeLevels(s),
      });
      if (s[STATE.OVER]) {
        mode = 'over';
        const run = { time: s[STATE.TIME], kills: s[STATE.KILLS], level: s[STATE.LEVEL] };
        ui.gameOver(run, save, recordRun(save, run));
      } else if (s[STATE.CHOOSING]) {
        mode = 'levelup';
        ui.levelUp(Array.from(s.subarray(STATE.CHOICE0, STATE.CHOICE0 + 3)), upgradeLevels(s));
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
