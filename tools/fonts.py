"""Build and check the cut-down UI fonts in assets/fonts/.

    python fonts.py subset   download the pinned source fonts and write the woff2 files
    python fonts.py check    fail if a committed font lacks a character the game shows

Run it through tools/subset-fonts.sh or tools/check-fonts.sh, which supply fontTools.
The character list is every CJK character in src/*.js and index.html, so it covers
all translated text, upgrade glyphs and static markup.
"""

import hashlib
import pathlib
import sys
import time
import urllib.request

from fontTools import subset
from fontTools.ttLib import TTFont

ROOT = pathlib.Path(__file__).resolve().parent.parent
OUT = ROOT / 'assets' / 'fonts'
CACHE = ROOT / '.font-cache'
SOURCES = [ROOT / 'index.html', *sorted((ROOT / 'src').glob('*.js'))]

ASCII = {chr(c) for c in range(0x20, 0x7F)}
# Characters drawn with the numbers face, in every language. Keep in sync with
# the unicode-range of "JR Numerals" in style.css.
NUMERALS = set('0123456789+%:,.×')

GF = 'https://raw.githubusercontent.com/google/fonts/4e5f06dbb274a27ebe71ed54ea706b3ee40eabd9/ofl'
BAKUDAI = 'https://cdn.jsdelivr.net/gh/max32002/bakudaifont@1.49'

# Each output font: its source, the sha256 of that source, its license, and which
# characters it keeps ('cjk' = ASCII plus every CJK character the game uses).
FONTS = {
    'bakudai-bold.woff2': {
        'url': f'{BAKUDAI}/tw/Bakudai-Bold.ttf',
        'sha256': '59c99887980e8002c4dea7c4ad7b207e9dd080cf1aef0f698750f4f806183cf8',
        'license': (f'{BAKUDAI}/SIL_Open_Font_License_1.1.txt', 'bakudai-OFL.txt'),
        'keep': 'cjk',
    },
    'lxgw-wenkai-tc-regular.woff2': {
        'url': f'{GF}/lxgwwenkaitc/LXGWWenKaiTC-Regular.ttf',
        'sha256': '4fcc5aec11cbbf737b0cfab7b63796f7f280087a7a656b2f13342cd5e5318d95',
        'license': (f'{GF}/lxgwwenkaitc/OFL.txt', 'lxgw-wenkai-tc-OFL.txt'),
        'keep': 'cjk',
    },
    'lxgw-wenkai-tc-bold.woff2': {
        'url': f'{GF}/lxgwwenkaitc/LXGWWenKaiTC-Bold.ttf',
        'sha256': '5c9feadfd928f3ae3860e7d001ad3d0d0c55b0157fff08cd4d51b51aca677f23',
        'license': (f'{GF}/lxgwwenkaitc/OFL.txt', 'lxgw-wenkai-tc-OFL.txt'),
        'keep': 'cjk',
    },
    'caveat-brush-numerals.woff2': {
        'url': f'{GF}/caveatbrush/CaveatBrush-Regular.ttf',
        'sha256': 'e565bd7bffd895ca64f8d1cb77c816b9b38d9c010246386a07c9648592f6f282',
        'license': (f'{GF}/caveatbrush/OFL.txt', 'caveat-brush-OFL.txt'),
        'keep': 'numerals',
    },
}


def is_cjk(ch):
    c = ord(ch)
    return (0x3000 <= c <= 0x303F or 0x3400 <= c <= 0x4DBF or 0x4E00 <= c <= 0x9FFF
            or 0xF900 <= c <= 0xFAFF or 0xFF00 <= c <= 0xFFEF)


def game_cjk():
    chars = set()
    for path in SOURCES:
        chars.update(ch for ch in path.read_text(encoding='utf-8') if is_cjk(ch))
    return chars


def wanted(spec):
    return NUMERALS if spec['keep'] == 'numerals' else ASCII | game_cjk()


def fetch(url, dest, sha256=''):
    if not dest.exists():
        print(f'  downloading {url}')
        dest.parent.mkdir(parents=True, exist_ok=True)
        tmp = dest.with_suffix('.part')
        urllib.request.urlretrieve(url, tmp)
        tmp.rename(dest)
    digest = hashlib.sha256(dest.read_bytes()).hexdigest()
    if sha256 and digest != sha256:
        sys.exit(f'{dest.name}: sha256 {digest} does not match the pinned {sha256}')
    return digest


def kb(n):
    return f'{n / 1024:,.0f} KB' if n < 1024 * 1024 else f'{n / 1024 / 1024:,.1f} MB'


def cmd_subset():
    start = time.time()
    for name, spec in FONTS.items():
        src = CACHE / spec['url'].rsplit('/', 1)[1]
        digest = fetch(spec['url'], src, spec['sha256'])
        if not spec['sha256']:
            print(f'  {src.name} sha256 {digest} (not pinned yet)')
        url, license_name = spec['license']
        lic = CACHE / license_name
        fetch(url, lic)
        OUT.mkdir(parents=True, exist_ok=True)
        (OUT / license_name).write_bytes(lic.read_bytes())

        chars = wanted(spec)
        # Keep the source's timestamp so the same input always gives the same bytes.
        font = TTFont(src, recalcTimestamp=False)
        missing = sorted(ch for ch in chars if ord(ch) not in font.getBestCmap())
        opts = subset.Options()
        opts.flavor = 'woff2'
        opts.hinting = False
        opts.desubroutinize = True
        opts.name_IDs = ['*']  # keep copyright and license records
        sub = subset.Subsetter(opts)
        sub.populate(text=''.join(chars))
        sub.subset(font)
        font.save(OUT / name)
        print(f'{name}: {kb(src.stat().st_size)} -> {kb((OUT / name).stat().st_size)}, '
              f'{len(chars) - len(missing)} characters')
        if missing and spec['keep'] != 'cjk':
            print(f'  source font lacks: {"".join(missing)}')
        elif missing:
            cjk_missing = [ch for ch in missing if is_cjk(ch)]
            if cjk_missing:
                print(f'  WARNING: source font lacks game characters: {"".join(cjk_missing)}')
    print(f'done in {time.time() - start:.1f} s')


def cmd_check():
    start = time.time()
    failed = False
    for name, spec in FONTS.items():
        path = OUT / name
        if not path.exists():
            print(f'{name}: missing')
            failed = True
            continue
        cmap = TTFont(path).getBestCmap()
        need = wanted(spec)
        # Only CJK characters must be present: a font may legitimately lack an
        # ASCII or numeral character, and the browser then uses the next font.
        need = {ch for ch in need if is_cjk(ch)} if spec['keep'] == 'cjk' else set()
        missing = sorted(ch for ch in need if ord(ch) not in cmap)
        if missing:
            failed = True
            print(f'{name}: missing {len(missing)} character(s): {"".join(missing)}')
        else:
            print(f'{name}: ok')
    if failed:
        print('Run tools/subset-fonts.sh and commit assets/fonts/.')
    print(f'checked in {time.time() - start:.1f} s')
    return 1 if failed else 0


if __name__ == '__main__':
    if sys.argv[1:] == ['subset']:
        cmd_subset()
    elif sys.argv[1:] == ['check']:
        sys.exit(cmd_check())
    else:
        sys.exit(__doc__)
