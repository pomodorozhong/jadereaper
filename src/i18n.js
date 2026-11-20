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
    'title.hint': 'Move with WASD or the left stick, aim with the mouse or the right stick. Mouse buttons and Q (X, Y, RB) use your three skills; Space (A) rolls. Esc or Start pauses.',
    'records.one': 'Best: {time} · {kills} cut · level {level} · {runs} run',
    'records.other': 'Best: {time} · {kills} cut · level {level} · {runs} runs',
    'settings.title': 'Settings',
    'settings.volume': 'Volume',
    'settings.volDown': 'Lower volume',
    'settings.volUp': 'Raise volume',
    'settings.soundOn': 'Sound: on',
    'settings.soundOff': 'Sound: off',
    'settings.language': 'Language: {name}',
    'settings.motionOn': 'Shake and hit-stop: on',
    'settings.motionOff': 'Shake and hit-stop: off',
    'settings.erase': 'Erase records',
    'settings.back': 'Back',
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
    'skill.scythe.name': 'Scythe Combo',
    'skill.scythe.desc': 'A quick sweep. Press again to chain up to three cuts.',
    'skill.talisman.name': 'Talisman Throw',
    'skill.talisman.desc': 'Throws a paper talisman where you aim.',
    'skill.lunge.name': 'Lunge Cut',
    'skill.lunge.desc': 'Dashes forward, cutting everything along the way.',
    'skill.pulse.name': 'Spirit Pulse',
    'skill.pulse.desc': 'A ring of force around you that shoves spirits away.',
    'skill.beads.name': 'Bead Flail',
    'skill.beads.desc': 'Flings jade beads into an orbit for two seconds.',
    'skill.charge.name': 'Reaping Charge',
    'skill.charge.desc': 'A long wind-up, then a charge across the room. Hard to stop.',
    'skill.cleave.name': 'Grave Cleave',
    'skill.cleave.desc': 'A huge overhead arc that staggers. Hard to stop.',
    'skill.leap.name': 'Crescent Leap',
    'skill.leap.desc': 'Leaps to where you aim and slams down. Hard to stop.',
    'cards.passive': 'Choose a passive',
    'cards.reward': 'Choose a card',
    'cards.on': 'On {skill}',
    'cards.character': 'Character',
    'cards.roll': 'Roll',
    'cards.replaces': 'Replaces {element}',
    'cards.confirm': 'Pick again to replace {element}',
    'element.fire': 'Fire',
    'element.ice': 'Ice',
    'element.lightning': 'Lightning',
    'element.celestial': 'Celestial',
    'card.fire.name': 'Fire',
    'card.fire.desc': 'Hits set spirits burning for a few seconds.',
    'card.ice.name': 'Ice',
    'card.ice.desc': 'Hits chill and slow; the third chill freezes.',
    'card.lightning.name': 'Lightning',
    'card.lightning.desc': 'Hits jump to the nearest other spirit for half damage.',
    'card.celestial.name': 'Celestial',
    'card.celestial.desc': 'Hits leave star marks. Three open a gravity well, and a star falls on it.',
    'card.quick_hands.name': 'Quick Hands',
    'card.quick_hands.desc': 'Recovers 20% faster.',
    'card.long_reach.name': 'Long Reach',
    'card.long_reach.desc': 'Reaches 20% further.',
    'card.heavy_blow.name': 'Heavy Blow',
    'card.heavy_blow.desc': '+25% damage.',
    'card.crushing.name': 'Crushing',
    'card.crushing.desc': '+40% stagger.',
    'card.vitality.name': 'Vitality',
    'card.vitality.desc': '+25 maximum health.',
    'card.swift.name': 'Swift Steps',
    'card.swift.desc': '+10% movement speed.',
    'card.mending.name': 'Mending',
    'card.mending.desc': 'Slowly recover health over time.',
    'card.might.name': 'Might',
    'card.might.desc': '+10% damage with every skill.',
    'card.thick_paper.name': 'Thick Paper',
    'card.thick_paper.desc': 'Take 15% less damage.',
    'card.long_roll.name': 'Long Roll',
    'card.long_roll.desc': 'Rolls 25% further.',
    'card.fire_trail.name': 'Fire Trail',
    'card.fire_trail.desc': 'Your roll leaves a trail of fire.',
    'weight.light': 'Light',
    'weight.medium': 'Medium',
    'weight.heavy': 'Heavy',
    'pick.banner': 'Choose three skills',
    'pick.begin': 'Begin ({n}/3)',
    'pick.back': 'Back',
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
    'title.hint': '用 WASD 或左搖桿移動，用滑鼠或右搖桿瞄準。滑鼠左右鍵與 Q（X、Y、RB）施放三個招式，空白鍵（A）翻滾，Esc 或 Start 暫停。',
    'records.one': '最佳：{time} · 斬殺 {kills} · 等級 {level} · 共 {runs} 局',
    'records.other': '最佳：{time} · 斬殺 {kills} · 等級 {level} · 共 {runs} 局',
    'settings.title': '設定',
    'settings.volume': '音量',
    'settings.volDown': '降低音量',
    'settings.volUp': '提高音量',
    'settings.soundOn': '音效：開',
    'settings.soundOff': '音效：關',
    'settings.language': '語言：{name}',
    'settings.motionOn': '畫面震動與頓幀：開',
    'settings.motionOff': '畫面震動與頓幀：關',
    'settings.erase': '清除紀錄',
    'settings.back': '返回',
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
    'skill.scythe.name': '連環鐮斬',
    'skill.scythe.desc': '快速揮斬。連按可接續最多三斬。',
    'skill.talisman.name': '擲符',
    'skill.talisman.desc': '朝瞄準方向擲出紙符。',
    'skill.lunge.name': '突刺斬',
    'skill.lunge.desc': '向前突進，斬開沿途的一切。',
    'skill.pulse.name': '靈波',
    'skill.pulse.desc': '在身邊震出一圈力場，推開墨靈。',
    'skill.beads.name': '玉珠流星',
    'skill.beads.desc': '甩出玉珠環繞身邊兩秒。',
    'skill.charge.name': '收割衝鋒',
    'skill.charge.desc': '長長蓄力後橫衝整個房間，難以打斷。',
    'skill.cleave.name': '斷墓劈',
    'skill.cleave.desc': '從頭頂劈下的巨大弧斬，能打出硬直，難以打斷。',
    'skill.leap.name': '新月躍',
    'skill.leap.desc': '躍向瞄準處並重重落地，難以打斷。',
    'cards.passive': '選擇一項被動',
    'cards.reward': '選擇一張卡',
    'cards.on': '裝在{skill}',
    'cards.character': '角色',
    'cards.roll': '翻滾',
    'cards.replaces': '取代{element}',
    'cards.confirm': '再選一次以取代{element}',
    'element.fire': '火',
    'element.ice': '冰',
    'element.lightning': '雷',
    'element.celestial': '星辰',
    'card.fire.name': '火',
    'card.fire.desc': '命中使墨靈燃燒數秒。',
    'card.ice.name': '冰',
    'card.ice.desc': '命中使墨靈受寒減速，第三次受寒時冰封。',
    'card.lightning.name': '雷',
    'card.lightning.desc': '命中時電流跳到最近的另一隻墨靈，造成一半傷害。',
    'card.celestial.name': '星辰',
    'card.celestial.desc': '命中留下星印。集滿三個時打開引力井，並有星辰落下。',
    'card.quick_hands.name': '快手',
    'card.quick_hands.desc': '收招快 20%。',
    'card.long_reach.name': '長臂',
    'card.long_reach.desc': '攻擊範圍遠 20%。',
    'card.heavy_blow.name': '重擊',
    'card.heavy_blow.desc': '傷害 +25%。',
    'card.crushing.name': '碎勢',
    'card.crushing.desc': '硬直 +40%。',
    'card.vitality.name': '活力',
    'card.vitality.desc': '最大生命值 +25。',
    'card.swift.name': '疾步',
    'card.swift.desc': '移動速度 +10%。',
    'card.mending.name': '癒合',
    'card.mending.desc': '隨時間緩慢回復生命值。',
    'card.might.name': '力量',
    'card.might.desc': '所有招式傷害 +10%。',
    'card.thick_paper.name': '厚紙',
    'card.thick_paper.desc': '受到的傷害減少 15%。',
    'card.long_roll.name': '長滾',
    'card.long_roll.desc': '翻滾距離遠 25%。',
    'card.fire_trail.name': '火痕',
    'card.fire_trail.desc': '翻滾時留下一道火焰。',
    'weight.light': '輕',
    'weight.medium': '中',
    'weight.heavy': '重',
    'pick.banner': '選擇三個招式',
    'pick.begin': '開始（{n}/3）',
    'pick.back': '返回',
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
