// Build and check the cut-down UI fonts in assets/fonts/.
//
//     node tools/fonts.mjs subset   download the pinned source fonts and write the woff2 files
//     node tools/fonts.mjs check    fail if a committed font lacks a character the game shows
//
// Run it through tools/subset-fonts.sh or tools/check-fonts.sh after `npm ci`.
// The character list is every CJK character in src/*.js and index.html, so it
// covers all translated text, upgrade glyphs and static markup.

import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, readdirSync, renameSync, statSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import * as fontkit from 'fontkit';
import subsetFont from 'subset-font';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');
const OUT = join(ROOT, 'assets', 'fonts');
const CACHE = join(ROOT, '.font-cache');
const SOURCES = [
  join(ROOT, 'index.html'),
  ...readdirSync(join(ROOT, 'src')).filter((f) => f.endsWith('.js')).sort().map((f) => join(ROOT, 'src', f)),
];

const ASCII = new Set(Array.from({ length: 0x7f - 0x20 }, (_, i) => String.fromCodePoint(0x20 + i)));
// Characters drawn with the numbers face, in every language. Keep in sync with
// the unicode-range of "JR Numerals" in style.css.
const NUMERALS = new Set('0123456789+%:,.×');

const GF = 'https://raw.githubusercontent.com/google/fonts/4e5f06dbb274a27ebe71ed54ea706b3ee40eabd9/ofl';
const BAKUDAI = 'https://cdn.jsdelivr.net/gh/max32002/bakudaifont@1.49';

// Each output font: its source, the sha256 of that source, its license, and which
// characters it keeps ('cjk' = ASCII plus every CJK character the game uses).
const FONTS = {
  'bakudai-bold.woff2': {
    url: `${BAKUDAI}/tw/Bakudai-Bold.ttf`,
    sha256: '59c99887980e8002c4dea7c4ad7b207e9dd080cf1aef0f698750f4f806183cf8',
    license: [`${BAKUDAI}/SIL_Open_Font_License_1.1.txt`, 'bakudai-OFL.txt'],
    keep: 'cjk',
  },
  'lxgw-wenkai-tc-regular.woff2': {
    url: `${GF}/lxgwwenkaitc/LXGWWenKaiTC-Regular.ttf`,
    sha256: '4fcc5aec11cbbf737b0cfab7b63796f7f280087a7a656b2f13342cd5e5318d95',
    license: [`${GF}/lxgwwenkaitc/OFL.txt`, 'lxgw-wenkai-tc-OFL.txt'],
    keep: 'cjk',
  },
  'lxgw-wenkai-tc-bold.woff2': {
    url: `${GF}/lxgwwenkaitc/LXGWWenKaiTC-Bold.ttf`,
    sha256: '5c9feadfd928f3ae3860e7d001ad3d0d0c55b0157fff08cd4d51b51aca677f23',
    license: [`${GF}/lxgwwenkaitc/OFL.txt`, 'lxgw-wenkai-tc-OFL.txt'],
    keep: 'cjk',
  },
  'caveat-brush-numerals.woff2': {
    url: `${GF}/caveatbrush/CaveatBrush-Regular.ttf`,
    sha256: 'e565bd7bffd895ca64f8d1cb77c816b9b38d9c010246386a07c9648592f6f282',
    license: [`${GF}/caveatbrush/OFL.txt`, 'caveat-brush-OFL.txt'],
    keep: 'numerals',
  },
};

// Every name record id, so copyright and license records are kept.
const ALL_NAME_IDS = Array.from({ length: 32768 }, (_, i) => i);

function isCjk(ch) {
  const c = ch.codePointAt(0);
  return (0x3000 <= c && c <= 0x303f) || (0x3400 <= c && c <= 0x4dbf) || (0x4e00 <= c && c <= 0x9fff)
    || (0xf900 <= c && c <= 0xfaff) || (0xff00 <= c && c <= 0xffef);
}

function gameCjk() {
  const chars = new Set();
  for (const path of SOURCES) {
    for (const ch of readFileSync(path, 'utf8')) if (isCjk(ch)) chars.add(ch);
  }
  return chars;
}

function wanted(spec) {
  return spec.keep === 'numerals' ? NUMERALS : new Set([...ASCII, ...gameCjk()]);
}

const sorted = (chars) => [...chars].sort((a, b) => a.codePointAt(0) - b.codePointAt(0));

async function fetchTo(url, dest, sha256 = '') {
  if (!existsSync(dest)) {
    console.log(`  downloading ${url}`);
    mkdirSync(dirname(dest), { recursive: true });
    const res = await fetch(url);
    if (!res.ok) throw new Error(`${url}: HTTP ${res.status}`);
    const tmp = `${dest}.part`;
    writeFileSync(tmp, Buffer.from(await res.arrayBuffer()));
    renameSync(tmp, dest);
  }
  const digest = createHash('sha256').update(readFileSync(dest)).digest('hex');
  if (sha256 && digest !== sha256) {
    console.error(`${dest.split('/').pop()}: sha256 ${digest} does not match the pinned ${sha256}`);
    process.exit(1);
  }
  return digest;
}

function kb(n) {
  return n < 1024 * 1024 ? `${Math.round(n / 1024).toLocaleString('en')} KB` : `${(n / 1024 / 1024).toFixed(1)} MB`;
}

async function cmdSubset() {
  const start = Date.now();
  for (const [name, spec] of Object.entries(FONTS)) {
    const src = join(CACHE, spec.url.split('/').pop());
    const digest = await fetchTo(spec.url, src, spec.sha256);
    if (!spec.sha256) console.log(`  ${src.split('/').pop()} sha256 ${digest} (not pinned yet)`);
    const [licenseUrl, licenseName] = spec.license;
    const lic = join(CACHE, licenseName);
    await fetchTo(licenseUrl, lic);
    mkdirSync(OUT, { recursive: true });
    writeFileSync(join(OUT, licenseName), readFileSync(lic));

    const chars = wanted(spec);
    const source = readFileSync(src);
    const font = fontkit.create(source);
    const missing = sorted([...chars].filter((ch) => !font.hasGlyphForCodePoint(ch.codePointAt(0))));
    const out = await subsetFont(source, sorted(chars).join(''), {
      targetFormat: 'woff2',
      noHinting: true,
      preserveNameIds: ALL_NAME_IDS,
    });
    writeFileSync(join(OUT, name), out);
    console.log(`${name}: ${kb(statSync(src).size)} -> ${kb(out.length)}, ${chars.size - missing.length} characters`);
    if (missing.length && spec.keep !== 'cjk') {
      console.log(`  source font lacks: ${missing.join('')}`);
    } else if (missing.length) {
      const cjkMissing = missing.filter(isCjk);
      if (cjkMissing.length) console.log(`  WARNING: source font lacks game characters: ${cjkMissing.join('')}`);
    }
  }
  console.log(`done in ${((Date.now() - start) / 1000).toFixed(1)} s`);
}

function cmdCheck() {
  const start = Date.now();
  let failed = false;
  for (const [name, spec] of Object.entries(FONTS)) {
    const path = join(OUT, name);
    if (!existsSync(path)) {
      console.log(`${name}: missing`);
      failed = true;
      continue;
    }
    const font = fontkit.create(readFileSync(path));
    // Only CJK characters must be present: a font may legitimately lack an
    // ASCII or numeral character, and the browser then uses the next font.
    const need = spec.keep === 'cjk' ? [...wanted(spec)].filter(isCjk) : [];
    const missing = sorted(need.filter((ch) => !font.hasGlyphForCodePoint(ch.codePointAt(0))));
    if (missing.length) {
      failed = true;
      console.log(`${name}: missing ${missing.length} character(s): ${missing.join('')}`);
    } else {
      console.log(`${name}: ok`);
    }
  }
  if (failed) console.log('Run tools/subset-fonts.sh and commit assets/fonts/.');
  console.log(`checked in ${((Date.now() - start) / 1000).toFixed(1)} s`);
  return failed ? 1 : 0;
}

const args = process.argv.slice(2).join(' ');
if (args === 'subset') {
  await cmdSubset();
} else if (args === 'check') {
  process.exitCode = cmdCheck();
} else {
  console.error('usage: node tools/fonts.mjs subset | check');
  process.exitCode = 2;
}
