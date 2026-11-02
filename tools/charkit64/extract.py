#!/usr/bin/env python3
"""Split the 64x128 anime base (Angeler1234 "Basic Character Ver 64", free version) into kit layers.

Source: .context/ref_free/Free/64X128_{Idle,Walking,Runing}_Free.png (not in git).
Licence: personal + commercial use, modification allowed, NO redistribution/resale, no AI.
Output: assets/charkit64/<part>/<anim>.png indexed (index 0 transparent, 1..6 material ramp,
7.. extras) + assets/charkit64/kit.json. Output dir is gitignored for the licence reason.

Layers produced from the baked sheet:
  body        skin tones; the scalp under the hair is rebuilt so other hair can go on top
  hair_long   the baked long hair, 6-tone ramp
  underwear   white bra + panties (cloth ramp)
  eyes        lashes, iris, highlight, pupil, mouth; iris recolourable
The ground shadow (alpha 125) is dropped; the game draws its own.
"""
import json, os
import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
SRC = os.path.join(ROOT, '.context', 'ref_free', 'Free')
OUT = os.path.join(ROOT, 'assets', 'charkit64')
W, H = 64, 128
ROWS = {'down': 0, 'left': 1, 'right': 2, 'up': 3}
ANIMS = {'idle': ('64X128_Idle_Free.png', 8), 'walk': ('64X128_Walking_Free.png', 10), 'run': ('64X128_Runing_Free.png', 8)}


def rgb(h):
    return tuple(int(h[i:i + 2], 16) for i in (1, 3, 5))


# ramp index = position in these lists (1-based). Ordered dark -> light like the LPC kit.
SKIN = ['#45212d', '#954a51', '#cb7173', '#f5a489', '#f9bf99', '#ffe0b1']
HAIR = ['#222034', '#333a6c', '#46426c', '#4f5aa5', '#616fcb', '#a6aee1']
HAIR_ALIAS = {'#3c447c': '#333a6c', '#6a78ac': '#616fcb', '#7c8db5': '#a6aee1', '#9ca2cc': '#a6aee1'}
CLOTH = ['#222034', '#6a78ac', '#7c8db5', '#a6aee1', '#e1eafd', '#ffffff']
EYE_EXTRAS = ['#222034', '#000000', '#639bff', '#fff3ab', '#8f563b', '#9c859f', '#3f3f74']  # lash, pupil, iris, glint, mouth, lip, sclera shade
IRIS = '#639bff'


def classify(a):
    """Per-pixel class map for one RGBA sheet: 0 none, 1 skin, 2 hair, 3 cloth, 4 face."""
    opaque = a[..., 3] == 255
    cls = np.zeros(a.shape[:2], np.uint8)
    col = np.zeros(a.shape[:2], np.int32)
    for h in SKIN:
        m = opaque & np.all(a[..., :3] == rgb(h), axis=2); cls[m] = 1; col[m] = SKIN.index(h) + 1
    for h in HAIR[1:] + list(HAIR_ALIAS):
        m = opaque & np.all(a[..., :3] == rgb(h), axis=2); cls[m] = 2; col[m] = HAIR.index(HAIR_ALIAS.get(h, h)) + 1
    for h in ('#ffffff', '#e1eafd'):
        m = opaque & np.all(a[..., :3] == rgb(h), axis=2); cls[m] = 3; col[m] = CLOTH.index(h) + 1
    for h in ('#000000', IRIS, '#fff3ab', '#8f563b', '#9c859f', '#3f3f74'):
        m = opaque & np.all(a[..., :3] == rgb(h), axis=2); cls[m] = 4; col[m] = 7 + EYE_EXTRAS.index(h)
    # #222034 is the hair outline AND the eyelash: lashes are the ones next to iris/pupil/glint
    dark = opaque & np.all(a[..., :3] == rgb('#222034'), axis=2)
    eye = np.isin(col, [7 + EYE_EXTRAS.index(h) for h in ('#000000', IRIS, '#fff3ab')])
    near = eye.copy()
    for _ in range(2):
        p = np.pad(near, 1)
        near |= p[:-2, 1:-1] | p[2:, 1:-1] | p[1:-1, :-2] | p[1:-1, 2:] | p[:-2, :-2] | p[2:, 2:] | p[:-2, 2:] | p[2:, :-2]
    lash = dark & near
    cls[dark] = 2; col[dark] = 1
    cls[lash] = 4; col[lash] = 7
    # light blues next to white cloth are cloth shading, not hair highlight
    cloth = cls == 3
    p = np.pad(cloth, 1)
    nearc = p[:-2, 1:-1] | p[2:, 1:-1] | p[1:-1, :-2] | p[1:-1, 2:]
    for h in ('#a6aee1', '#7c8db5', '#6a78ac'):
        m = opaque & np.all(a[..., :3] == rgb(h), axis=2) & nearc
        cls[m] = 3; col[m] = CLOTH.index(h) + 1
    return cls, col


def fill_scalp(body, cls, cell_y0, cell_x0):
    """Rebuild the skull under the hair in one cell. The hair hugs the cranium only on its upper rows and
    flares below the ears, so the skull is: (hair|face silhouette eroded 2px) restricted to an ellipse
    that spans the head height, plus whatever face skin is already drawn. Outlined, with a shade row."""
    y0, x0 = cell_y0, cell_x0
    c = cls[y0:y0 + H, x0:x0 + W]
    b = body[y0:y0 + H, x0:x0 + W]
    hair = c == 2
    face = (c == 1) | (c == 4)
    rows = np.nonzero(hair.any(axis=1))[0]
    if not len(rows):
        return
    ytop = int(rows.min())
    # the cranium rows: hair top .. ~12px down is where the hair hugs the skull
    sil = hair | face
    er = sil.copy()
    for _ in range(2):
        p = np.pad(er, 1)
        er = p[:-2, 1:-1] & p[2:, 1:-1] & p[1:-1, :-2] & p[1:-1, 2:]
    band = er[ytop + 6:ytop + 14]
    xs = np.nonzero(band.any(axis=0))[0]
    cx = (xs.min() + xs.max()) / 2.0
    rx = (xs.max() - xs.min()) / 2.0 + 0.5
    HEAD_H = 34                                   # hair top -> chin at this scale
    cy = ytop + HEAD_H / 2.0
    ry = HEAD_H / 2.0
    yy, xx = np.mgrid[0:H, 0:W]
    ell = ((xx - cx) / rx) ** 2 + ((yy - cy) / ry) ** 2 <= 1.0
    skull = (er & ell)
    skull[ytop + HEAD_H:] = False
    # fill the cranium solid between its left/right edges (hair partings leave holes)
    for y in range(ytop, ytop + HEAD_H):
        r = np.nonzero(skull[y])[0]
        if len(r):
            skull[y, r.min():r.max() + 1] = True
    new = skull & (b == 0)
    b[new] = 6
    p = np.pad(skull | (b > 0), 1)
    full = skull | (b > 0)
    edge = full & ~(p[:-2, 1:-1] & p[2:, 1:-1] & p[1:-1, :-2] & p[1:-1, 2:])
    b[edge & new] = 1
    sh = np.zeros_like(skull); sh[1:] = (edge & new)[:-1]
    b[sh & new & ~edge] = 4


def outline_new(b, new):
    """Outline + one shade row for freshly filled skin pixels `new` inside cell array b."""
    full = b > 0
    p = np.pad(full, 1)
    edge = full & ~(p[:-2, 1:-1] & p[2:, 1:-1] & p[1:-1, :-2] & p[1:-1, 2:])
    b[edge & new] = 1
    sh = np.zeros_like(new); sh[1:] = (edge & new)[:-1]
    b[sh & new & ~edge] = 4


def fill_under_hair(body, cls, cell_y0, cell_x0, direction, down_sil):
    """Rebuild skin hidden by the baked hair below the skull: trapezius/shoulders (down), back of the neck
    (sides) and the whole upper back (up). Down/side: fill each row between its outermost drawn pixels.
    Up: the back has the same outline as the front, so borrow the down cell's silhouette.
    Sides additionally carry the torso's back line straight up to the skull."""
    y0, x0 = cell_y0, cell_x0
    c = cls[y0:y0 + H, x0:x0 + W]
    b = body[y0:y0 + H, x0:x0 + W]
    hidden = (c == 2) & (b == 0)
    rows = np.nonzero((c == 2).any(axis=1))[0]
    if not len(rows):
        return
    ytop = int(rows.min())
    ya, yb = ytop + 30, ytop + 52                 # chin-ish .. mid torso
    cand = np.zeros_like(hidden)
    drawn = (b > 0) | (c == 3) | (c == 4)
    if direction == 'up':
        cand[ya:yb] = down_sil[ya:yb]
    else:
        for y in range(ya, yb):
            xs = np.nonzero(drawn[y])[0]
            if len(xs):
                cand[y, xs.min():xs.max() + 1] = True
        if direction in ('left', 'right'):
            sh = drawn[ytop + 36:ytop + 41]
            xs = np.nonzero(sh.any(axis=0))[0]
            if len(xs):
                lo, hi = int(xs.min()), int(xs.max())
                for y in range(ya, ytop + 40):
                    r = np.nonzero(drawn[y])[0]
                    if len(r):
                        if direction == 'right':
                            cand[y, lo:r.max() + 1] = True
                        else:
                            cand[y, r.min():hi + 1] = True
    new = cand & hidden
    b[new] = 6
    outline_new(b, new)


def save(idx, path, ramp, extras=()):
    im = Image.fromarray(idx.astype(np.uint8), 'P')
    pal = [0, 0, 0]
    for h in ramp:
        pal += list(rgb(h))
    for h in extras:
        pal += list(rgb(h))
    pal += [0, 0, 0] * (256 - len(pal) // 3)
    im.putpalette(pal)
    im.save(path, transparency=0, optimize=True)


def build():
    parts = {'body': ('body', SKIN, ()), 'hair_long': ('hair', HAIR, ()), 'underwear': ('cloth', CLOTH, ()),
             'eyes': (None, [], EYE_EXTRAS)}
    kit = {'cell': [W, H], 'rows': ROWS, 'anims': {a: {'cols': n} for a, (_, n) in ANIMS.items()},
           'materials': {'body': {'base': 'light', 'palettes': {'light': SKIN}},
                         'hair': {'base': 'blue', 'palettes': {'blue': HAIR}},
                         'cloth': {'base': 'white', 'palettes': {'white': CLOTH}}},
           'parts': []}
    for pid, (mat, ramp, extras) in parts.items():
        os.makedirs(os.path.join(OUT, pid), exist_ok=True)
    for anim, (fn, cols) in ANIMS.items():
        a = np.array(Image.open(os.path.join(SRC, fn)).convert('RGBA'))
        assert a.shape == (4 * H, cols * W, 4), (fn, a.shape)
        cls, col = classify(a)
        body = np.where(cls == 1, col, 0).astype(np.uint8)
        hair = np.where(cls == 2, col, 0).astype(np.uint8)
        cloth = np.where(cls == 3, col, 0).astype(np.uint8)
        eyes = np.where(cls == 4, col, 0).astype(np.uint8)
        for r in range(4):
            for c in range(cols):
                fill_scalp(body, cls, r * H, c * W)
        drawn = (cls == 1) | (cls == 3) | (cls == 4)
        for r, d in ((0, 'down'), (1, 'left'), (2, 'right'), (3, 'up')):
            for c in range(cols):
                down_sil = drawn[0:H, c * W:(c + 1) * W] | (body[0:H, c * W:(c + 1) * W] > 0)
                fill_under_hair(body, cls, r * H, c * W, d, down_sil)
        # skin under the eyes/mouth (so other eye sets can replace them) and under the underwear
        # (bust/hip shape stays as a soft mid-tone line where the cloth outline was)
        body[(cls == 4) & (body == 0)] = 6
        body[(cls == 3) & (body == 0) & (col == 1)] = 4
        body[(cls == 3) & (body == 0)] = 6
        save(body, os.path.join(OUT, 'body', f'{anim}.png'), SKIN)
        save(hair, os.path.join(OUT, 'hair_long', f'{anim}.png'), HAIR)
        save(cloth, os.path.join(OUT, 'underwear', f'{anim}.png'), CLOTH)
        save(eyes, os.path.join(OUT, 'eyes', f'{anim}.png'), [], EYE_EXTRAS)
        left = (a[..., 3] > 0) & (a[..., 3] == 255) & (cls == 0)
        if left.sum():
            print(anim, 'unclassified opaque px:', int(left.sum()))
    z = {'body': 10, 'underwear': 20, 'eyes': 30, 'hair_long': 40}
    for pid, (mat, ramp, extras) in parts.items():
        kit['parts'].append({'id': pid, 'material': mat, 'z': z[pid], 'extras': list(extras),
                             'anims': {a: {'file': f'{pid}/{a}.png', 'cols': n} for a, (_, n) in ANIMS.items()}})
    json.dump(kit, open(os.path.join(OUT, 'kit.json'), 'w'), indent=1)
    print('wrote', OUT)


if __name__ == '__main__':
    build()
