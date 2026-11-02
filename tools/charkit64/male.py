#!/usr/bin/env python3
"""Derive the male body from the female 64x128 kit: python3 -I tools/charkit64/male.py

Writes assets/charkit64/male/{body,underwear}/<anim>.png. Head, arms, hands and legs are the
female base's; the torso (trapezius -> crotch) is a hand-drawn template fitted per frame, the arms
are moved 2px outward to sit on the wider shoulders, idle legs get a gap opened between them.
Side views cut the bust back to a male chest profile.

The female run (hands tucked up at the chest, dainty kick) reads wrong on a man, so the male run
sheet is the walk cycle resampled to 8 frames with a forward lean; the game plays it faster. A
proper male run is still to be drawn.

Design language from Angeler's own male promo (standby_male_frames.png, reference only): square
shoulders wider than the head, collarbone ticks, two shallow pec arcs, a sternum/ab centre line with
two ab ticks, a navel dot, narrow hips, legs standing apart. Everything here is a programmatic edit
of the licensed base so the output stays gitignored like the rest of assets/charkit64.
"""
import os
import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
KIT = os.path.join(os.path.dirname(os.path.dirname(HERE)), 'assets', 'charkit64')
OUT = os.path.join(KIT, 'male')
W, H = 64, 128
ROWS = {'down': 0, 'left': 1, 'right': 2, 'up': 3}
ANIMS = {'idle': 8, 'walk': 10}
CX = 32
TONES = {'.': 0, 'a': 1, 'b': 2, 'c': 3, 'd': 4, 'e': 5, 'f': 6}

# Female idle frame 0 landmarks (head top 19): trapezius top 48, shoulder line 52, chest 56-62,
# waist 64-67, hips 68-76, crotch 77, feet 105. Template rows are indexed by these numbers.
TPL_Y0, TPL_X0 = 48, 14
# Left half of each row, 19 columns (x 14..32, col 18 = body centre), mirrored to the right.
# '?' keeps the female pixel, '.' is transparent, a-f are skin tones dark -> light.
FRONT = [
    '.....adfffe????????',  # 48 trapezius
    '....adffffff???????',  # 49
    '...adfffffff???????',  # 50
    '..adffffffff???????',  # 51
    '.adfffffffffffeeeef',  # 52 shoulder line
    'adffffffffffddddfee',  # 53 collarbones
    'adfffffffddddffffee',  # 54
    'adfffeefffffffffffe',  # 55 deltoid crease
    '?????affffffffffffe',  # 56 chest (cols 0-4: arm)
    '?????affffffffffffe',  # 57
    '?????afffffffffffde',  # 58
    '?????afddfffffffffe',  # 59 pec arc
    '?????afeeddddffffee',  # 60
    '?????affeeeedddddef',  # 61
    '?????affffffeeeeeff',  # 62
    '??????afffffffffffe',  # 63
    '???????affffffffffe',  # 64 waist
    '???????afffffffddfe',  # 65 ab tick
    '???????affffffffffe',  # 66
    '???????affffffffffe',  # 67
    '??????afffffffddffe',  # 68 ab tick
    '??????afffffffffffc',  # 69 navel
    '??????afffffffffffe',  # 70 hips
    '??????afffffffffffe',  # 71
    '??????affffffffffff',  # 72
    '??????affffffffffff',  # 73
    '??????affffffffffff',  # 74
    '??????affffffffffff',  # 75
    '??????affffffffffff',  # 76
]
BACK = [
    '.....adfffe????????',
    '....adffffff???????',
    '...adfffffff???????',
    '..adffffffff???????',
    '.adfffffffffffffffe',
    'adffffffffffffffffe',  # 53
    'adffffffffffffffffe',
    'adfffeefffffffeffee',  # 55 shoulder blade hint
    '?????affffffeefffee',
    '?????afffffeeffffee',
    '?????afffffffffffee',
    '?????affffffffffffe',  # 59 spine
    '?????affffffffffffe',
    '?????affffffffffffe',
    '?????affffffffffffe',
    '??????afffffffffffe',
    '???????affffffffffe',  # 64 waist
    '???????affffffffffe',
    '???????affffffffffe',
    '???????afffffffffff',
    '??????afffffffffdff',  # 68 dimples
    '??????afffffffffdff',
    '??????afffffffffdff',  # 70 hips
    '??????afffffffffeff',
    '??????affffffffffff',
    '??????affffffffffff',
    '??????affffffffffff',
    '??????affffffffffff',
    '??????affffffffffff',
]


def expand(tpl):
    """Mirror the half-rows into full 37-wide rows of tone indices (-1 = keep female pixel)."""
    out = np.full((len(tpl), 2 * 19 - 1), -1, np.int16)
    for i, half in enumerate(tpl):
        assert len(half) == 19, (i, half)
        full = half + half[-2::-1]
        for j, ch in enumerate(full):
            out[i, j] = -1 if ch == '?' else TONES[ch]
    return out


def load(part, anim):
    return np.array(Image.open(os.path.join(KIT, part, f'{anim}.png')))


def save(idx, src_path, path):
    src = Image.open(src_path)
    im = Image.fromarray(idx.astype(np.uint8), 'P')
    im.putpalette(src.getpalette())
    im.save(path, transparency=0, optimize=True)


def cell_top(b):
    rows = np.nonzero((b > 0).any(axis=1))[0]
    return int(rows.min()) if len(rows) else None


def centre_run(row, cx=CX):
    if not row[cx]:
        return None
    l = cx
    while l > 0 and row[l - 1]:
        l -= 1
    r = cx
    while r < W - 1 and row[r + 1]:
        r += 1
    return l, r


def crotch_row(b, ytop):
    """First leg row: the row after the last hip row whose centre is transparent."""
    last = None
    for y in range(ytop + 50, min(H, ytop + 70)):
        if b[y, CX] == 0 and b[y, CX - 1] == 0:
            last = y
    return (last + 1) if last is not None else ytop + 58


def arm_mask(b, u, y0, y1):
    """Pixels of the arms/hands in rows y0..y1: body outside the torso span, where the torso span is
    the underwear's extent on that row (or the centre run where there is no cloth)."""
    m = np.zeros_like(b, bool)
    for y in range(y0, y1):
        xs = np.nonzero(u[y])[0]
        if len(xs):
            l, r = int(xs.min()), int(xs.max())
        else:
            run = centre_run(b[y])
            if run is None:
                continue
            l, r = run
        row = b[y] > 0
        m[y, :l] = row[:l]
        m[y, r + 1:] = row[r + 1:]
    return m


def shift_sides(src, mask, dx):
    """Move masked pixels left of centre by -dx and right of centre by +dx."""
    out = np.zeros_like(src)
    left = mask.copy(); left[:, CX:] = False
    right = mask.copy(); right[:, :CX] = False
    out[:, :W - dx][left[:, dx:]] = src[:, dx:][left[:, dx:]]
    out[:, dx:][right[:, :W - dx]] = src[:, :W - dx][right[:, :W - dx]]
    return out


def split_legs(row):
    """Open a 1px gap between the two legs (centre seam -> outline, gap, outline)."""
    run = centre_run(row)
    if run is None:
        return row
    l, r = run
    seg = row[l:r + 1]
    mid = len(seg) // 2
    seg2 = np.concatenate([seg[:mid], np.array([1, 0, 1], row.dtype), seg[mid + 1:]])
    out = row.copy()
    nl = l - 1
    out[nl:nl + len(seg2)] = seg2
    return out


def fit_template(tpl, dy, y_bottom):
    """Template rows shifted by dy; if the frame's torso is shorter/taller than the template
    (breathing squash), drop or repeat belly rows (66-70) so it ends exactly at y_bottom."""
    rows = list(range(len(tpl)))
    want = y_bottom - (TPL_Y0 + dy)
    belly = [i for i in rows if 66 <= TPL_Y0 + i <= 70]
    while len(rows) > want and belly:
        rows.remove(belly.pop())
    while len(rows) < want:
        k = belly[-1] if belly else rows[-1]
        rows.insert(rows.index(k), k)
    return [(TPL_Y0 + dy + j, tpl[i]) for j, i in enumerate(rows)]


def male_front_back(b, u, ytop, tpl, anim):
    """Down / up views."""
    dy = ytop - 19
    crotch = crotch_row(b, ytop)
    arms = arm_mask(b, u, 53 + dy, H)
    nb, nu = b.copy(), u.copy()
    if anim == 'idle':
        for y in range(crotch, H):
            nb[y] = split_legs(nb[y])
            if nu[y, CX]:
                nu[y] = split_legs(nu[y])
    nb[arms] = 0
    for y, trow in fit_template(tpl, dy, crotch):
        for j, v in enumerate(trow):
            if v >= 0:
                nb[y, TPL_X0 + j] = v
    # arms stay straight: one shift per frame, chosen so the upper arm touches the chest outline
    # (template col 5 = x 19 on the left) and hangs clear of the narrower waist below it
    chest = [y for y in range(56 + dy, 63 + dy) if arms[y, :CX].any()]
    if chest:
        ar = max(int(np.nonzero(arms[y, :CX])[0].max()) for y in chest)
        dx = max(0, min(3, (TPL_X0 + 5 - 1) - ar))
    else:
        dx = 1
    arms_shifted = shift_sides(np.where(arms, b, 0), arms, dx) if dx else np.where(arms, b, 0)
    m = arms_shifted > 0
    m[:56 + dy] = False
    nb[m] = arms_shifted[m]
    # briefs: drop the bra and the panties' upper band so the waistband sits on the hips
    nu[:70 + dy] = 0
    return nb, nu


def male_side(b, u, ytop, direction):
    """Side views: cut the bust back to a male chest line and flatten the torso behind the arm."""
    y0, y1 = ytop + 33, ytop + 54
    sign = 1 if direction == 'right' else -1
    prof = [1, 2, 3, 4, 4, 4, 4, 3, 3, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]
    nb, nu = b.copy(), u.copy()
    for i, y in enumerate(range(y0, y1)):
        fx = CX + sign * (3 + prof[i])
        if sign > 0:
            nb[y, fx + 1:] = 0; nu[y, fx + 1:] = 0
            xs = range(CX + 1, fx + 1)
        else:
            nb[y, :fx] = 0; nu[y, :fx] = 0
            xs = range(fx, CX)
        for x in xs:
            if nb[y, x] not in (0, 1):
                nb[y, x] = 6
        if nb[y, fx]:
            nb[y, fx] = 1
            if nb[y, fx - sign] == 6:
                nb[y, fx - sign] = 5
    nu[:ytop + 51] = 0
    return nb, nu


def outline_fix(b):
    full = b > 0
    p = np.pad(full, 1)
    edge = full & ~(p[:-2, 1:-1] & p[2:, 1:-1] & p[1:-1, :-2] & p[1:-1, 2:])
    b[edge & (b > 1)] = 1


RUN_FROM_WALK = [0, 1, 3, 4, 5, 6, 8, 9]   # 10-frame walk resampled to the run sheet's 8 columns


def run_from_walk(sheet, tops):
    """Male run sheet: the walk cycle resampled to 8 frames with the upper body sheared toward the
    facing direction (2px at the head, 1px at the chest) so it reads as a lean, not a stroll."""
    out = np.zeros((4 * H, 8 * W), sheet.dtype)
    for d, r in ROWS.items():
        dx = {'left': -1, 'right': 1}.get(d, 0)
        for k, c in enumerate(RUN_FROM_WALK):
            cell = sheet[r * H:(r + 1) * H, c * W:(c + 1) * W].copy()
            if dx:
                ytop = tops[(r, c)]
                cell[ytop:ytop + 30] = np.roll(cell[ytop:ytop + 30], 2 * dx, axis=1)
                cell[ytop + 30:ytop + 52] = np.roll(cell[ytop + 30:ytop + 52], dx, axis=1)
            out[r * H:(r + 1) * H, k * W:(k + 1) * W] = cell
    return out


def build():
    front, back = expand(FRONT), expand(BACK)
    for part in ('body', 'underwear'):
        os.makedirs(os.path.join(OUT, part), exist_ok=True)
    for anim, cols in ANIMS.items():
        body, under = load('body', anim), load('underwear', anim)
        nb, nu = body.copy(), under.copy()
        tops = {}
        for d, r in ROWS.items():
            for c in range(cols):
                ys, xs = slice(r * H, (r + 1) * H), slice(c * W, (c + 1) * W)
                b, u = body[ys, xs], under[ys, xs]
                ytop = tops[(r, c)] = cell_top(b)
                if d == 'down':
                    b2, u2 = male_front_back(b, u, ytop, front, anim)
                elif d == 'up':
                    b2, u2 = male_front_back(b, u, ytop, back, anim)
                else:
                    b2, u2 = male_side(b, u, ytop, d)
                outline_fix(b2)
                nb[ys, xs], nu[ys, xs] = b2, u2
        for part, arr in (('body', nb), ('underwear', nu)):
            save(arr, os.path.join(KIT, part, f'{anim}.png'), os.path.join(OUT, part, f'{anim}.png'))
        if anim == 'walk':
            for part, arr in (('body', nb), ('underwear', nu)):
                save(run_from_walk(arr, tops), os.path.join(KIT, part, 'walk.png'), os.path.join(OUT, part, 'run.png'))
    print('wrote', OUT)


if __name__ == '__main__':
    build()
