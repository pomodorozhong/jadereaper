// Paper UI: screens, HUD and menu focus. All DOM work lives here.
import { UPGRADES } from './data.js';
import { formatTime } from './save.js';

const $ = (id) => document.getElementById(id);
const SCREENS = ['title', 'settings', 'levelup', 'pause', 'over', 'error'];
const INPUT_LOCK_MS = 300;

export function createUI({ onAction }) {
  let current = null;
  let lockedUntil = 0;
  const cache = {};

  document.addEventListener('click', (e) => {
    const btn = e.target.closest('button[data-action]');
    if (!btn || performance.now() < lockedUntil) return;
    onAction(btn.dataset.action, btn.dataset.value);
  });

  function buttons() {
    return current ? Array.from($(current).querySelectorAll('button')) : [];
  }

  /** Show one screen (or none). A short lock stops a held button from picking something by accident. */
  function show(name) {
    for (const id of SCREENS) $(id).hidden = id !== name;
    current = name || null;
    if (current) {
      lockedUntil = performance.now() + INPUT_LOCK_MS;
      const first = buttons()[0];
      if (first) first.focus({ preventScroll: true });
    } else if (document.activeElement instanceof HTMLElement) {
      document.activeElement.blur();
    }
  }

  /** Move focus through the open screen's buttons. Returns true if focus moved. */
  function navigate(dx, dy) {
    const list = buttons();
    if (!list.length) return false;
    const i = list.indexOf(document.activeElement);
    const step = dx > 0 || dy > 0 ? 1 : -1;
    const next = i < 0 ? 0 : (i + step + list.length) % list.length;
    list[next].focus({ preventScroll: true });
    return true;
  }

  /** Click the focused button (gamepad confirm). */
  function confirm() {
    const el = document.activeElement;
    if (current && el instanceof HTMLButtonElement && $(current).contains(el)) {
      el.click();
      return true;
    }
    return false;
  }

  function set(id, prop, value) {
    const key = id + prop;
    if (cache[key] === value) return;
    cache[key] = value;
    if (prop === 'text') $(id).textContent = value;
    else $(id).style.setProperty(prop, value);
  }

  function hud({ hp, maxHp, xp, xpNext, level, time, kills, upgrades }) {
    set('hpFill', 'width', `${Math.max(0, Math.min(100, (hp / maxHp) * 100)).toFixed(1)}%`);
    set('hpText', 'text', String(Math.ceil(hp)));
    set('xpFill', 'width', `${Math.max(0, Math.min(100, (xp / xpNext) * 100)).toFixed(1)}%`);
    set('levelText', 'text', `Lv ${level}`);
    set('timeText', 'text', formatTime(time));
    set('killsText', 'text', String(kills));
    const sig = upgrades.join(',');
    if (cache.chips !== sig) {
      cache.chips = sig;
      $('chips').replaceChildren(...upgrades.flatMap((lvl, id) => {
        if (!lvl) return [];
        const chip = document.createElement('div');
        chip.className = 'chip';
        chip.title = UPGRADES[id].name;
        const glyph = document.createElement('b');
        glyph.textContent = UPGRADES[id].glyph;
        chip.append(glyph, String(lvl));
        return [chip];
      }));
    }
  }

  function showHud(visible) {
    $('hud').hidden = !visible;
  }

  function levelUp(choices, upgrades) {
    $('cards').replaceChildren(...choices.flatMap((id, slot) => {
      if (id < 0) return [];
      const u = UPGRADES[id];
      const card = document.createElement('button');
      card.className = 'card';
      card.dataset.action = 'choose';
      card.dataset.value = String(slot);
      const lvl = upgrades[id];
      const parts = [
        ['key', String(slot + 1)],
        ['glyph', u.glyph],
        ['name', u.name],
        ['rank', lvl === 0 ? 'New' : `Level ${lvl} → ${lvl + 1}`],
        ['desc', u.desc],
      ];
      for (const [cls, text] of parts) {
        const span = document.createElement('span');
        span.className = cls;
        span.textContent = text;
        card.append(span);
      }
      return [card];
    }));
    show('levelup');
  }

  function gameOver({ time, kills, level }, save, beaten) {
    const rows = [
      ['Survived', formatTime(time), beaten.time],
      ['Spirits cut', String(kills), beaten.kills],
      ['Level', String(level), beaten.level],
      ['Best time', formatTime(save.bestTime), false],
    ];
    $('stats').replaceChildren(...rows.flatMap(([label, value, best]) => {
      const dt = document.createElement('dt');
      dt.textContent = label;
      const dd = document.createElement('dd');
      dd.textContent = value;
      if (best) dd.className = 'best';
      return [dt, dd];
    }));
    $('recordStamp').hidden = !(beaten.time || beaten.kills || beaten.level);
    show('over');
  }

  function records(save) {
    $('records').textContent = save.runs
      ? `Best: ${formatTime(save.bestTime)} · ${save.bestKills} cut · level ${save.bestLevel} · ${save.runs} ${save.runs === 1 ? 'run' : 'runs'}`
      : '';
  }

  function settings({ volume, muted }) {
    const on = Math.round(volume * 10);
    $('volPips').replaceChildren(...Array.from({ length: 10 }, (_, i) => {
      const pip = document.createElement('i');
      if (i < on) pip.className = 'on';
      return pip;
    }));
    for (const btn of document.querySelectorAll('button[data-action="mute"]')) {
      btn.textContent = muted ? 'Sound: off' : 'Sound: on';
    }
  }

  function error(message) {
    $('errorText').textContent = message;
    show('error');
  }

  return {
    show, navigate, confirm, hud, showHud, levelUp, gameOver, records, settings, error,
    get screen() { return current; },
    get locked() { return performance.now() < lockedUntil; },
  };
}
