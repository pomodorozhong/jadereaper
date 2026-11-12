// Keyboard and gamepad input.
//
// `poll()` is called once per frame and returns the movement vector plus
// one-shot menu actions (navigate, confirm, back, pause) from the gamepad.
// Keyboard menu actions are delivered through the `onKey` callback, because
// Enter and Space already click the focused button natively.

const DEADZONE = 0.22;
const REPEAT_FIRST = 0.35;
const REPEAT_NEXT = 0.14;

const MOVE_KEYS = {
  KeyW: [0, -1], ArrowUp: [0, -1],
  KeyS: [0, 1], ArrowDown: [0, 1],
  KeyA: [-1, 0], ArrowLeft: [-1, 0],
  KeyD: [1, 0], ArrowRight: [1, 0],
};

export function createInput({ onKey }) {
  const held = new Set();
  let prevButtons = [];
  let navHeld = 0;
  let navDir = '';
  let padName = '';

  window.addEventListener('keydown', (e) => {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    if (e.code in MOVE_KEYS) {
      held.add(e.code);
      e.preventDefault();
      if (!e.repeat || e.code.startsWith('Arrow')) onKey({ nav: MOVE_KEYS[e.code] });
    } else if (e.code === 'Escape' || e.code === 'KeyP') {
      if (!e.repeat) onKey({ pause: true });
      e.preventDefault();
    } else if (/^Digit[1-3]$/.test(e.code)) {
      onKey({ pick: Number(e.code.slice(5)) - 1 });
    } else if (e.code === 'KeyM' && !e.repeat) {
      onKey({ mute: true });
    }
  });
  window.addEventListener('keyup', (e) => held.delete(e.code));
  window.addEventListener('blur', () => held.clear());
  window.addEventListener('gamepadconnected', (e) => { padName = e.gamepad.id; });
  window.addEventListener('gamepaddisconnected', () => { padName = ''; });

  function poll(dt) {
    let mx = 0, my = 0;
    for (const code of held) {
      mx += MOVE_KEYS[code][0];
      my += MOVE_KEYS[code][1];
    }
    const out = { mx, my, nav: null, confirm: false, back: false, pause: false, pad: padName };

    const pads = navigator.getGamepads ? navigator.getGamepads() : [];
    const pad = Array.from(pads).find((p) => p && p.connected);
    if (pad) {
      let ax = pad.axes[0] || 0, ay = pad.axes[1] || 0;
      if (Math.hypot(ax, ay) < DEADZONE) { ax = 0; ay = 0; }
      const b = (i) => !!(pad.buttons[i] && pad.buttons[i].pressed);
      const dx = (b(15) ? 1 : 0) - (b(14) ? 1 : 0);
      const dy = (b(13) ? 1 : 0) - (b(12) ? 1 : 0);
      if (ax || ay || dx || dy) {
        mx += ax + dx;
        my += ay + dy;
      }
      const pressed = (i) => b(i) && !prevButtons[i];
      out.confirm = pressed(0);
      out.back = pressed(1);
      out.pause = pressed(9);

      // Menu navigation from stick or d-pad, with key-repeat.
      const sx = ax + dx, sy = ay + dy;
      let dir = '';
      if (Math.abs(sx) > 0.5 || Math.abs(sy) > 0.5) {
        dir = Math.abs(sx) > Math.abs(sy) ? (sx > 0 ? 'r' : 'l') : (sy > 0 ? 'd' : 'u');
      }
      if (dir && dir !== navDir) {
        navHeld = REPEAT_FIRST;
        out.nav = dir;
      } else if (dir) {
        navHeld -= dt;
        if (navHeld <= 0) { navHeld = REPEAT_NEXT; out.nav = dir; }
      }
      navDir = dir;
      if (out.nav) out.nav = { l: [-1, 0], r: [1, 0], u: [0, -1], d: [0, 1] }[out.nav];
      prevButtons = pad.buttons.map((x) => x.pressed);
    } else {
      prevButtons = [];
      navDir = '';
    }

    const len = Math.hypot(mx, my);
    if (len > 1) { mx /= len; my /= len; }
    out.mx = mx;
    out.my = my;
    return out;
  }

  return { poll };
}
