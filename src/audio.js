// Synthesized sound effects. No audio files: everything is a short burst of
// filtered noise (paper and ink) or a plucked pentatonic tone (jade).
import { EVENT } from './data.js';

const PENTATONIC = [523.25, 587.33, 659.25, 783.99, 880.0, 1046.5, 1174.66, 1318.5];

export function createAudio() {
  let ctx = null;
  let master = null;
  let noiseBuf = null;
  let volume = 0.6;
  let muted = false;
  const lastPlayed = {};
  let pickupStep = 0;

  /** Browsers only allow audio after a user gesture, so this is called from one. */
  function unlock() {
    if (!ctx) {
      const AC = window.AudioContext || window.webkitAudioContext;
      if (!AC) return;
      ctx = new AC();
      master = ctx.createGain();
      master.connect(ctx.destination);
      noiseBuf = ctx.createBuffer(1, ctx.sampleRate * 0.5, ctx.sampleRate);
      const d = noiseBuf.getChannelData(0);
      for (let i = 0; i < d.length; i++) d[i] = Math.random() * 2 - 1;
      applyVolume();
    }
    if (ctx.state === 'suspended') ctx.resume();
  }

  function applyVolume() {
    if (master) master.gain.value = muted ? 0 : volume * 0.5;
  }

  function noise({ dur, type = 'bandpass', f0, f1 = f0, q = 1, gain = 0.5 }) {
    const t = ctx.currentTime;
    const src = ctx.createBufferSource();
    src.buffer = noiseBuf;
    const filter = ctx.createBiquadFilter();
    filter.type = type;
    filter.Q.value = q;
    filter.frequency.setValueAtTime(f0, t);
    filter.frequency.exponentialRampToValueAtTime(Math.max(20, f1), t + dur);
    const g = ctx.createGain();
    g.gain.setValueAtTime(gain, t);
    g.gain.exponentialRampToValueAtTime(0.001, t + dur);
    src.connect(filter).connect(g).connect(master);
    src.start(t, Math.random() * 0.3, dur + 0.02);
  }

  function tone({ dur, type = 'sine', f0, f1 = f0, gain = 0.3, delay = 0 }) {
    const t = ctx.currentTime + delay;
    const osc = ctx.createOscillator();
    osc.type = type;
    osc.frequency.setValueAtTime(f0, t);
    osc.frequency.exponentialRampToValueAtTime(Math.max(20, f1), t + dur);
    const g = ctx.createGain();
    g.gain.setValueAtTime(0.0001, t);
    g.gain.exponentialRampToValueAtTime(gain, t + 0.008);
    g.gain.exponentialRampToValueAtTime(0.001, t + dur);
    osc.connect(g).connect(master);
    osc.start(t);
    osc.stop(t + dur + 0.02);
  }

  const SOUNDS = {
    [EVENT.HIT]: () => noise({ dur: 0.05, f0: 2200, f1: 900, q: 2, gain: 0.25 }),
    [EVENT.KILL]: () => {
      noise({ dur: 0.16, type: 'lowpass', f0: 1400, f1: 200, gain: 0.4 });
      tone({ dur: 0.12, f0: 180, f1: 70, gain: 0.25 });
    },
    [EVENT.PICKUP]: () => {
      pickupStep = (pickupStep + 1) % PENTATONIC.length;
      tone({ dur: 0.09, type: 'triangle', f0: PENTATONIC[pickupStep], gain: 0.16 });
    },
    [EVENT.LEVELUP]: () => {
      [0, 2, 4, 5].forEach((n, i) => tone({ dur: 0.3, type: 'triangle', f0: PENTATONIC[n], gain: 0.25, delay: i * 0.08 }));
    },
    [EVENT.HURT]: () => {
      tone({ dur: 0.2, type: 'sawtooth', f0: 170, f1: 70, gain: 0.3 });
      noise({ dur: 0.18, f0: 700, f1: 250, gain: 0.35 });
    },
    [EVENT.SLASH]: () => noise({ dur: 0.14, type: 'highpass', f0: 900, f1: 3800, gain: 0.22 }),
    [EVENT.SHOOT]: () => noise({ dur: 0.06, f0: 3200, f1: 1800, q: 3, gain: 0.18 }),
    [EVENT.PULSE]: () => {
      tone({ dur: 0.35, f0: 130, f1: 45, gain: 0.45 });
      noise({ dur: 0.3, type: 'lowpass', f0: 900, f1: 120, gain: 0.25 });
    },
    [EVENT.DEATH]: () => {
      [5, 3, 1, 0].forEach((n, i) => tone({ dur: 0.45, type: 'triangle', f0: PENTATONIC[n] / 2, gain: 0.3, delay: i * 0.16 }));
    },
    [EVENT.HEAL]: () => {
      tone({ dur: 0.25, f0: 660, f1: 990, gain: 0.25 });
    },
    [EVENT.ELITE]: () => {
      tone({ dur: 0.7, type: 'sawtooth', f0: 90, f1: 60, gain: 0.3 });
      tone({ dur: 0.7, type: 'sawtooth', f0: 95, f1: 63, gain: 0.3 });
    },
    ui: () => noise({ dur: 0.035, f0: 2600, q: 4, gain: 0.2 }),
    confirm: () => {
      noise({ dur: 0.05, f0: 1800, q: 2, gain: 0.25 });
      tone({ dur: 0.12, type: 'triangle', f0: 659.25, gain: 0.18 });
    },
  };

  // Minimum gap between repeats of the same sound, in seconds.
  const GAP = { [EVENT.HIT]: 0.045, [EVENT.KILL]: 0.06, [EVENT.PICKUP]: 0.05 };

  function play(id) {
    if (!ctx || muted || ctx.state !== 'running' || !SOUNDS[id]) return;
    const now = ctx.currentTime;
    if (now - (lastPlayed[id] ?? -1) < (GAP[id] ?? 0.02)) return;
    lastPlayed[id] = now;
    SOUNDS[id]();
  }

  return {
    unlock,
    play,
    setVolume(v) { volume = Math.min(1, Math.max(0, v)); applyVolume(); },
    setMuted(m) { muted = !!m; applyVolume(); },
    get muted() { return muted; },
    get volume() { return volume; },
  };
}
