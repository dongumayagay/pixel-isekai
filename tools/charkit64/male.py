#!/usr/bin/env python3
"""Male body for the 64x128 kit: python3 -I tools/charkit64/male.py

Writes assets/charkit64/male/{body,underwear}/<anim>.png. Angeler's own male (reference
.context/ref/angeler_promo/basic-character-ver-64/00.gif) is the female base with a different
torso: same skull, same arms, same legs and feet, flat chest with a pec line and abs, shoulders a
touch wider, straight hips, thinner thighs. We build it the same way, frame for frame:

  head      the base sheet's head rows, untouched
  torso     one hand-drawn template per view (front / side / back), neck to briefs, translated
            with the head bob and sway
  arms      the base frame's arm pixels: in the front and back views the skin beside the torso
            span, in the side view the skin blobs sitting in front of (near arm) or behind (far
            arm) the torso, found by their outline ring
  legs      the base frame's leg pixels below the waistband, thighs slimmed one column on the
            outside in the front and back views

Characters pick it with body='male' (kit64.py looks up assets/charkit64/male/<part>/ first).
"""
import os
import sys
import numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sheet import KIT, W, H, ROWS, ANIMS, load, save, cell_top, outline_fix

OUT = os.path.join(KIT, 'male')
REF = {'down': 19, 'right': 18, 'up': 19}        # head top the templates were drawn at
HEAD_ROWS = 28                                   # rows from the head top down to the chin line
T = {'.': 0, 'a': 1, 'b': 2, 'c': 3, 'd': 4, 'e': 5, 'f': 6}
U = {'u': 5, 'v': 4, 'w': 6}                     # underwear tones (cloth ramp)

# ---- torso templates (rows 47..80 at REF top), mirrored halves for front/back --------------------
FRONT = [  # x 13..32, mirrored to 33..51
    '..............adffff',  # 47 neck
    '..............adffff',
    '.............aadffff',  # 49 trapezius
    '...........aadffffff',
    '..........aadfffffff',
    '........aadfffffffff',
    '......aadfffffffffff',
    '....aadfffffffffffff',
    '...adffffffffffeeeee',  # 55 shoulder line, collarbone light
    '..adffffffffffffffff',
    '..adffffffffeffffffe',  # 57 deltoid crease
    '..adfffffffeffffffff',
    '...adffffffeffffffff',
    '......adffffeffffffe',  # 60 chest (arms are capsules from here down)
    '......adffffeffffffe',
    '......adfffffffffffe',
    '......adffffeeeeefff',  # 63 pec line
    '.......adfffffffeeef',
    '.......adfffffffffff',
    '........adffffffffff',  # 66 waist
    '........adfffffeffff',  # 67 abs
    '........adffffffffff',
    '........adffffffffff',
    '........adfffffeffff',  # 70 abs
    '........adffffffffff',
    '.......adfffwwwwwwww',  # 72 waistband
    '.......adffuuuuuuuuu',
    '.......adffuuuuuuuuu',
    '.......adfuuuuuuuuuu',
    '.......aduuuuuuuuuuu',
    '.......aduuuuuuuuuuu',
    '.......aduuuuuuuuuuu',
    '........auuuuuuuuuuu',
    '.........auuuuuuuuuu',  # 80 crotch
]
BACK = [
    '..............adffff',
    '..............adffff',
    '.............aadffff',
    '...........aadffffff',
    '..........aadfffffff',
    '........aadfffffffff',
    '......aadfffffffffff',
    '....aadfffffffffffff',
    '...adfffffffffffffff',
    '..adffffffffffffffff',  # 56
    '..adffffffffefffffff',  # 57 shoulder blade
    '..adffffffffffefffff',
    '...adfffffffffffffff',
    '......adffffffffffff',
    '......adffffffffffff',
    '......adffffffffffff',
    '......adfffffffffffe',  # 63 spine
    '.......adffffffffffe',
    '.......adffffffffffe',
    '........adfffffffffe',
    '........adfffffffffe',
    '........adfffffffffe',
    '........adffffffffff',
    '........adffffffefff',  # 70 dimples
    '........adffffffffff',
    '.......adfffwwwwwwww',
    '.......adffuuuuuuuuu',
    '.......adffuuuuuuuuu',
    '.......adfuuuuuuuuuu',
    '.......aduuuuuuuuuuu',
    '.......aduuuuuuuuuuu',
    '.......aduuuuuuuuuuu',
    '........auuuuuuuuuuu',
    '.........auuuuuuuuuu',  # 80 crotch
]
SIDE = [  # x 22..45, facing right (chest at the right edge); e column = shade down the back
    '.....adffffffa..........',  # 47 neck
    '.....adffffffa..........',
    '...aadffffffffda........',  # 49
    '..aadfffffffffffdda.....',
    '..adffffffffffffffffda..',  # 51 shoulder
    '..aefffffffffffffffffda.',
    '..aeffffffefffffffffffa.',
    '..aeffffffefffffffffffa.',
    '..aeffffffeffffffffeefa.',  # 55 pec light
    '..aeffffffefffffffffffa.',
    '..aeffffffeffffffffeeda.',  # 57 under the pec
    '..aeffffffefffffffffea..',
    '..aeffffffeffffffffffa..',
    '..aeffffffefffffffffa...',
    '..aeffffffefffffffffa...',  # 61
    '..aeffffffeffffffffa....',
    '..aeffffffeffffffffa....',
    '..aeffffffeffffffffa....',  # 64 waist
    '..aeffffffeffffffffa....',
    '..aeffffffefffffffffa...',
    '..aeffffffefffffffffa...',
    '..aeffffffefffffffffa...',  # 68 hips
    '..aeffffffefffffffffa...',
    '..aeffffffefffffffffa...',
    '..aeffffffefffffffffa...',
    '..awwwwwwwwwwwwwwwwwa...',  # 72 waistband
    '..avvuuuuuuuuuuuuuuua...',
    '.avvvuuuuuuuuuuuuuuua...',  # 74 seat
    '.avvvuuuuuuuuuuuuuuua...',
    '.avvvuuuuuuuuuuuuuuua...',
    '.avvvuuuuuuuuuuuuuua....',
    '..avvuuuuuuuuuuuuua.....',
    '...avuuuuuuuuuuuua......',
    '....auuuuuuuuuuua.......',  # 80
]
# ---- drawing primitives ---------------------------------------------------------------------------
def template(tpl, x0, y0, mirror):
    body = np.zeros((H, W), np.uint8)
    under = np.zeros((H, W), np.uint8)
    for j, row in enumerate(tpl):
        if mirror:
            row = row + row[-2::-1]
        for i, ch in enumerate(row):
            if ch in T and ch != '.':
                body[y0 + j, x0 + i] = T[ch]
            elif ch in U:
                body[y0 + j, x0 + i] = 6
                under[y0 + j, x0 + i] = U[ch]
    return body, under


WAIST = 72          # template row of the waistband (same frame of reference as REF)


def waist_row(u, top):
    """First row of the briefs: the top of the lowest run of cloth rows. The hips rise up to 4 rows
    when the base crouches in the run."""
    y = top + 70
    while y > top + 44 and (u[y] > 0).sum() < 4:
        y -= 1                           # thighs below the briefs
    while y > top + 44 and (u[y - 1] > 0).sum() >= 4:
        y -= 1                           # briefs
    return y if top + 48 <= y <= top + 58 else top + 54


def fit_waist(body, under, y0, waist):
    """Keep the template neck where the head put it and the waistband where the base's briefs are;
    the torso rows between are resampled (the base crouches up to 4 rows in the run)."""
    y_neck, y_waist = y0, y0 + (WAIST - 47)
    if waist == y_waist:
        return body, under
    n_src, n_dst = y_waist - y_neck, waist - y_neck
    src = [y_neck + int(i * n_src / n_dst) for i in range(n_dst)]
    src += list(range(y_waist, H))
    src = src[:H]
    order = list(range(y_neck)) + src
    order = order[:H] + [H - 1] * (H - len(order))
    return body[order], under[order]


def dilate(m):
    p = np.pad(m, 1)
    return p[:-2, 1:-1] | p[2:, 1:-1] | p[1:-1, :-2] | p[1:-1, 2:] | m


def components(mask, min_size=6):
    out = []
    seen = np.zeros_like(mask, bool)
    for y, x in zip(*np.nonzero(mask)):
        if seen[y, x]:
            continue
        stack = [(y, x)]; seen[y, x] = True; pts = []
        while stack:
            cy, cx = stack.pop(); pts.append((cy, cx))
            for ny, nx in ((cy - 1, cx), (cy + 1, cx), (cy, cx - 1), (cy, cx + 1)):
                if 0 <= ny < H and 0 <= nx < W and mask[ny, nx] and not seen[ny, nx]:
                    seen[ny, nx] = True; stack.append((ny, nx))
        if len(pts) >= min_size:
            out.append(np.array(pts))
    return out




# ---- limbs lifted from the base frame -------------------------------------------------------------
_YS, _XS = np.mgrid[0:H, 0:W]


def spans(u, top):
    """Cumulative widest underwear span per row from the bra down: the torso and hips."""
    span, last = {}, None
    for y in range(top + 28, H):
        xs = np.nonzero(u[y])[0]
        if len(xs) >= 8 and y < top + 72:
            l, r = int(xs.min()), int(xs.max())
            last = (min(last[0], l), max(last[1], r)) if last else (l, r)
        if last:
            span[y] = last
    return span


def grow(mask, region):
    """Flood mask into region (4-connected)."""
    while True:
        nxt = dilate(mask) & region
        if (nxt == mask).all():
            return mask
        mask = nxt


def limbs(b, u, top):
    """Arms are the skin blobs bounded by their own outline ring once the underwear-covered torso is
    masked out. A blob reaching the shoulder rows is an arm seen from the shoulder (near); one that
    starts lower and pokes outside the torso span is the other arm's forearm swinging out (far).
    Fists sit in front of the thighs and are not always outlined, so each blob is found down to the
    waistband first and then grown into the fist rows no wider than its forearm."""
    span = spans(u, top)
    inner = (b > 1) & (u == 0)
    inner[:top + 36] = False
    upper = inner.copy(); upper[top + 54:] = False
    near, far = [], []
    for c in components(upper, 10):
        y0 = int(c[:, 0].min())
        if y0 <= top + 42:
            tgt = near
        elif y0 <= top + 52 and any(x < span[y][0] or x > span[y][1] for y, x in c if y in span):
            tgt = far
        else:
            continue
        fore = c[c[:, 0] >= top + 46]
        if not len(fore):
            fore = c
        x0, x1 = int(fore[:, 1].min()) - 2, int(fore[:, 1].max()) + 2
        mask = np.zeros_like(inner); mask[c[:, 0], c[:, 1]] = True
        region = inner & (_XS >= x0) & (_XS <= x1) | mask
        mask = dilate(grow(mask, region)) & (b > 0)
        tgt.append(mask)
    legs = np.zeros_like(b)
    legs[top + 53:] = b[top + 53:]
    legs[u > 0] = 0                      # the base's hips under her briefs are not legs
    for m in near + far:
        legs[m] = 0
    return near, far, legs


def lifted(b, u, top, waist, ref):
    """A thigh raised toward the viewer covers the briefs: skin where the idle frame had cloth
    (idle briefs aligned on this frame's waistband). The whole blob around it (bounded by its
    outline) goes in front of the template."""
    ru, rwaist, rx = ref
    ru = np.roll(np.roll(ru, waist - rwaist, axis=0), int(round(head_x(b, top) - rx)), axis=1)
    covered = (ru > 0) & (u == 0) & (b > 1)
    covered[:waist + 2] = False; covered[waist + 12:] = False
    if covered.sum() < 6:
        return np.zeros_like(b, bool)
    inner = (b > 1) & (u == 0)
    inner[:waist] = False
    return dilate(grow(covered, inner)) & (b > 0)


def keep_main(body, under):
    """Drop slivers not connected to the figure (a far hand the base shows past her bust)."""
    comps = sorted(components(body > 0, 1), key=len, reverse=True)
    for c in comps[1:]:
        body[c[:, 0], c[:, 1]] = 0
        under[c[:, 0], c[:, 1]] = 0


def slim_thighs(legs, top):
    """One column off the outside of each thigh (outline moves in, the shade column survives)."""
    m = legs > 1
    m[:top + 54] = False; m[top + 71:] = False
    comps = sorted(components(m, 20), key=len, reverse=True)[:2]
    if len(comps) < 2:
        return
    comps.sort(key=lambda c: c[:, 1].mean())
    for side, c in zip((-1, +1), comps):
        for y in np.unique(c[:, 0]):
            xs = c[c[:, 0] == y, 1]
            x = int(xs.min()) if side < 0 else int(xs.max())
            if legs[y, x - side] == 1:
                legs[y, x - side] = 0
                legs[y, x + side] = legs[y, x]
                legs[y, x] = 1


# ---- frame assembly -------------------------------------------------------------------------------
def head_crop(b, top):
    out = np.zeros((H, W), np.uint8)
    out[top:top + HEAD_ROWS] = b[top:top + HEAD_ROWS]
    return out


def head_x(b, top):
    xs = np.nonzero(b[top + 20:top + 27].any(axis=0))[0]
    return (int(xs.min()) + int(xs.max())) / 2


HEAD_X = {'down': 32.0, 'right': 34.0, 'up': 32.0}   # idle col 0, where the templates were drawn


def assemble(b, u, direction, ref):
    top = cell_top(b)
    dy = top - REF[direction]
    dx = int(round(head_x(b, top) - HEAD_X[direction]))
    if direction == 'down':
        body, under = template(FRONT, 13 + dx, 47 + dy, True)
    elif direction == 'up':
        body, under = template(BACK, 13 + dx, 47 + dy, True)
    else:
        body, under = template(SIDE, 22 + dx, 47 + dy, False)
    waist = waist_row(u, top)
    delta = max(-5, min(2, waist - ref[1] - dy))          # hips rise vs the idle pose
    body, under = fit_waist(body, under, 47 + dy, WAIST + dy + delta)
    near, far, legs = limbs(b, u, top)
    if direction != 'right':
        slim_thighs(legs, top)
    m = (body == 0) & (legs > 0)
    body[m] = legs[m]
    m = lifted(b, u, top, waist, ref) & (legs > 0)
    body[m] = legs[m]
    under[m] = 0
    # over the shoulder slope the template wins; from the chest down the arms sit in front (the far
    # arm's forearm swings out past the chest or the back; what the torso hides is not in the mask)
    hi = np.zeros_like(b, bool); hi[:top + (36 if direction == 'right' else 39)] = True
    for a in near + far:
        xs = np.nonzero(a[top + 36:top + 44].any(axis=0))[0]
        at_back = direction != 'right' or (len(xs) > 0 and xs.min() < 34)
        m = a & (~hi | (body == 0)) if at_back else a
        body[m] = b[m]
        under[m] = 0
    head = head_crop(b, top)
    body[head > 0] = head[head > 0]
    keep_main(body, under)
    outline_fix(under)
    return body, under


def build():
    for part in ('body', 'underwear'):
        os.makedirs(os.path.join(OUT, part), exist_ok=True)
    refs = {}
    ib, iu = load('body', 'idle'), load('underwear', 'idle')
    for d in ('down', 'right', 'up'):
        ys = slice(ROWS[d] * H, (ROWS[d] + 1) * H)
        top = cell_top(ib[ys, :W])
        refs[d] = (iu[ys, :W], waist_row(iu[ys, :W], top), head_x(ib[ys, :W], top))
    for anim, cols in ANIMS.items():
        body, under = load('body', anim), load('underwear', anim)
        nb = np.zeros_like(body); nu = np.zeros_like(under)
        for d in ('down', 'right', 'up'):
            r = ROWS[d]
            for c in range(cols):
                ys, xs = slice(r * H, (r + 1) * H), slice(c * W, (c + 1) * W)
                nb[ys, xs], nu[ys, xs] = assemble(body[ys, xs], under[ys, xs], d, refs[d])
        rl, rr = ROWS['left'], ROWS['right']
        for c in range(cols):
            xs = slice(c * W, (c + 1) * W)
            nb[rl * H:(rl + 1) * H, xs] = nb[rr * H:(rr + 1) * H, xs][:, ::-1]
            nu[rl * H:(rl + 1) * H, xs] = nu[rr * H:(rr + 1) * H, xs][:, ::-1]
        for part, arr in (('body', nb), ('underwear', nu)):
            save(arr, os.path.join(KIT, part, f'{anim}.png'), os.path.join(OUT, part, f'{anim}.png'))
        print('wrote', anim)



if __name__ == '__main__':
    build()
