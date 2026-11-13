// Player-facing text in every language. Static text in index.html names its
// key with data-i18n (text), data-i18n-aria (aria-label) or data-i18n-title
// (title); code calls t(key, vars). A "\n" in a string becomes a line break.

/** Languages in the order the settings button cycles through them. */
export const LANGS = ['en', 'zh-Hant'];

const STRINGS = {
  en: {
    'lang.name': 'English',
    'doc.title': 'Jade Reaper',
    'game.view': 'Game view',
    'hud.xp': 'Experience',
    'hud.level': 'Lv {n}',
    'title.name': 'Jade Reaper',
    'title.sub': 'Cut down the ink spirits. Last as long as you can.',
    'title.begin': 'Begin',
    'title.settings': 'Settings',
    'title.hint': 'Move with WASD, arrow keys or a gamepad stick. Attacks are automatic. Esc or Start pauses.',
    'records.one': 'Best: {time} · {kills} cut · level {level} · {runs} run',
    'records.other': 'Best: {time} · {kills} cut · level {level} · {runs} runs',
    'settings.title': 'Settings',
    'settings.volume': 'Volume',
    'settings.volDown': 'Lower volume',
    'settings.volUp': 'Raise volume',
    'settings.soundOn': 'Sound: on',
    'settings.soundOff': 'Sound: off',
    'settings.language': 'Language: {name}',
    'settings.erase': 'Erase records',
    'settings.back': 'Back',
    'levelup.banner': 'Level up — choose one',
    'levelup.new': 'New',
    'levelup.rank': 'Level {from} → {to}',
    'pause.title': 'Paused',
    'pause.resume': 'Resume',
    'pause.restart': 'Restart',
    'pause.quit': 'Quit to title',
    'over.title': 'The ink closes in',
    'over.record': 'New\nrecord',
    'over.survived': 'Survived',
    'over.kills': 'Spirits cut',
    'over.level': 'Level',
    'over.bestTime': 'Best time',
    'over.again': 'Try again',
    'over.quit': 'Title',
    'error.title': 'Cannot start',
    'error.hint': 'Jade Reaper needs a browser with WebGPU, such as a current Chrome, Edge or Safari, opened through http://localhost or https.',
    'error.noWebgpu': 'WebGPU is not available in this browser.',
    'error.noAdapter': 'No WebGPU adapter was found.',
    'error.shader': 'Shader failed to compile: {detail}',
    'upgrade.0.name': "Reaper's Scythe",
    'upgrade.0.desc': 'A sweeping arc where you face. Grows wider and hits harder.',
    'upgrade.1.name': 'Paper Talisman',
    'upgrade.1.desc': 'Flies at the nearest spirit. More slips at higher levels.',
    'upgrade.2.name': 'Jade Beads',
    'upgrade.2.desc': 'Beads circle you and strike what they touch.',
    'upgrade.3.name': 'Spirit Pulse',
    'upgrade.3.desc': 'A ring of force that hurts and shoves spirits away.',
    'upgrade.4.name': 'Vitality',
    'upgrade.4.desc': '+20 maximum health.',
    'upgrade.5.name': 'Swift Steps',
    'upgrade.5.desc': '+8% movement speed.',
    'upgrade.6.name': 'Jade Lure',
    'upgrade.6.desc': '+30% pickup range.',
    'upgrade.7.name': 'Might',
    'upgrade.7.desc': '+12% damage.',
    'upgrade.8.name': 'Haste',
    'upgrade.8.desc': 'All attacks come 8% faster.',
    'upgrade.9.name': 'Mending',
    'upgrade.9.desc': 'Slowly recover health over time.',
  },
  'zh-Hant': {
    'lang.name': '繁體中文',
    'doc.title': '玉鐮',
    'game.view': '遊戲畫面',
    'hud.xp': '經驗值',
    'hud.level': '等級 {n}',
    'title.name': '玉鐮',
    'title.sub': '斬除墨靈，撐得越久越好。',
    'title.begin': '開始',
    'title.settings': '設定',
    'title.hint': '用 WASD、方向鍵或手把搖桿移動。攻擊會自動施放。按 Esc 或 Start 暫停。',
    'records.one': '最佳：{time} · 斬殺 {kills} · 等級 {level} · 共 {runs} 局',
    'records.other': '最佳：{time} · 斬殺 {kills} · 等級 {level} · 共 {runs} 局',
    'settings.title': '設定',
    'settings.volume': '音量',
    'settings.volDown': '降低音量',
    'settings.volUp': '提高音量',
    'settings.soundOn': '音效：開',
    'settings.soundOff': '音效：關',
    'settings.language': '語言：{name}',
    'settings.erase': '清除紀錄',
    'settings.back': '返回',
    'levelup.banner': '升級：選擇一項',
    'levelup.new': '新',
    'levelup.rank': '等級 {from} → {to}',
    'pause.title': '暫停',
    'pause.resume': '繼續',
    'pause.restart': '重新開始',
    'pause.quit': '回到標題',
    'over.title': '墨色吞沒了你',
    'over.record': '新\n紀錄',
    'over.survived': '存活時間',
    'over.kills': '斬殺墨靈',
    'over.level': '等級',
    'over.bestTime': '最佳時間',
    'over.again': '再試一次',
    'over.quit': '標題',
    'error.title': '無法啟動',
    'error.hint': '玉鐮需要支援 WebGPU 的瀏覽器（例如最新版的 Chrome、Edge 或 Safari），並透過 http://localhost 或 https 開啟。',
    'error.noWebgpu': '這個瀏覽器不支援 WebGPU。',
    'error.noAdapter': '找不到 WebGPU 介面卡。',
    'error.shader': '著色器編譯失敗：{detail}',
    'upgrade.0.name': '死神之鐮',
    'upgrade.0.desc': '朝你面向的方向揮出弧形斬擊。範圍更廣，威力更強。',
    'upgrade.1.name': '紙符',
    'upgrade.1.desc': '飛向最近的墨靈。等級越高，符紙越多。',
    'upgrade.2.name': '玉珠',
    'upgrade.2.desc': '玉珠環繞著你，擊中碰到的一切。',
    'upgrade.3.name': '靈波',
    'upgrade.3.desc': '一圈力場，傷害並震退墨靈。',
    'upgrade.4.name': '活力',
    'upgrade.4.desc': '最大生命值 +20。',
    'upgrade.5.name': '疾步',
    'upgrade.5.desc': '移動速度 +8%。',
    'upgrade.6.name': '玉引',
    'upgrade.6.desc': '拾取範圍 +30%。',
    'upgrade.7.name': '力量',
    'upgrade.7.desc': '傷害 +12%。',
    'upgrade.8.name': '迅捷',
    'upgrade.8.desc': '所有攻擊加快 8%。',
    'upgrade.9.name': '癒合',
    'upgrade.9.desc': '隨時間緩慢回復生命值。',
  },
};

let lang = 'en';

/** The language a new player starts in, whatever the browser prefers. */
export const DEFAULT_LANG = 'zh-Hant';

/** The saved language if it is known, else the default. */
export function pickLang(saved) {
  return LANGS.includes(saved) ? saved : DEFAULT_LANG;
}

export function getLang() {
  return lang;
}

export function setLang(next) {
  lang = LANGS.includes(next) ? next : 'en';
  applyStatic();
}

/** The language after `current` in LANGS, wrapping around. */
export function nextLang(current) {
  return LANGS[(LANGS.indexOf(current) + 1) % LANGS.length];
}

/** Look up a string, falling back to English and then to the key, and fill {name} slots. */
export function t(key, vars = {}) {
  const raw = STRINGS[lang][key] ?? STRINGS.en[key] ?? key;
  return raw.replace(/\{(\w+)\}/g, (m, name) => (name in vars ? String(vars[name]) : m));
}

/** Set an element's text, turning "\n" into <br>. */
export function setText(el, text) {
  const lines = text.split('\n');
  el.replaceChildren(...lines.flatMap((line, i) => (i ? [document.createElement('br'), line] : [line])));
}

/** Re-apply every data-i18n key in the page for the current language. */
export function applyStatic(root = document) {
  document.documentElement.lang = lang;
  document.title = t('doc.title');
  for (const el of root.querySelectorAll('[data-i18n]')) setText(el, t(el.dataset.i18n));
  for (const el of root.querySelectorAll('[data-i18n-aria]')) el.setAttribute('aria-label', t(el.dataset.i18nAria));
  for (const el of root.querySelectorAll('[data-i18n-title]')) el.title = t(el.dataset.i18nTitle);
}
