#!/usr/bin/env python3
"""Bust S / L variants of the female base: python3 -I tools/charkit64/bust.py

Writes assets/charkit64/bust_s/ and bust_l/ {body,underwear}/<anim>.png. The base sheet is the M
bust. Per frame, in the bra rows: S drops 3 rows and 1 column per side and gives them back to the
belly; L duplicates 3 rows and 1 column per side and takes them from the belly, so the head, feet
and frame height never move. Side views flatten (S) or push out (L) the chest profile by up to 3px.
Body and underwear get the identical row/column edits so the bra stays glued to the bust.
Characters pick one with body='bust_s' | 'bust_l' (see kit64.py).
"""
import os
import numpy as np
from PIL import Image
import sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sheet import KIT, W, H, ROWS, CX, ANIMS, load, save, cell_top, outline_fix

SIDE_ROWS = (33, 44)   # rows (relative to the head top) that carry the side-view bust bump


def bra_rows(u, ytop):
    ys = [y for y in range(ytop + 30, ytop + 49) if (u[y] > 0).sum() >= 8]
    return (ys[0], ys[-1]) if ys else (ytop + 36, ytop + 45)


def resample_rows(arrs, drop, dup):
    """Rebuild each array with the listed rows dropped and the listed rows duplicated."""
    keep = [y for y in range(H) if y not in drop]
    order = []
    for y in keep:
        order.append(y)
        if y in dup:
            order.append(y)
    assert len(order) == H, (len(order), drop, dup)
    return [a[order] for a in arrs]


def torso_span(u, b, y):
    xs = np.nonzero(u[y])[0]
    if len(xs) >= 8:
        return int(xs.min()), int(xs.max())
    xs = np.nonzero(b[y])[0]
    return (int(xs.min()), int(xs.max())) if len(xs) else (CX, CX)


def narrow_row(row, l, r):
    out = row.copy()
    out[1:l + 2] = row[:l + 1]; out[0] = 0          # drop column l+1, left part moves right
    out[r - 1:W - 1] = row[r:]; out[W - 1] = 0       # drop column r-1, right part moves left
    return out


def widen_row(row, l, r):
    out = row.copy()
    out[:l + 1] = np.concatenate([row[1:l + 2]])      # left part moves left, column l+1 repeats
    out[r:] = np.concatenate([row[r - 1:W - 1]])      # right part moves right, column r-1 repeats
    return out


def front_back(b, u, ytop, size):
    top, bot = bra_rows(u, ytop)
    n = bot - top + 1
    inner = [top + int(n * f) for f in (0.25, 0.5, 0.75)]
    belly = [bot + 1, bot + 2, bot + 3]
    if size == 's':
        drop, dup = inner, belly
    else:
        drop, dup = belly, inner
    nb, nu = resample_rows([b, u], set(drop), set(dup))
    top, bot = bra_rows(nu, ytop)
    for y in range(top + 1, bot):
        l, r = torso_span(nu, nb, y)
        if r - l < 10:
            continue
        f = narrow_row if size == 's' else widen_row
        nb[y] = f(nb[y], l, r); nu[y] = f(nu[y], l, r)
    return nb, nu


def side(b, u, ytop, direction, size):
    sign = 1 if direction == 'right' else -1
    nb, nu = b.copy(), u.copy()
    y0, y1 = ytop + SIDE_ROWS[0], ytop + SIDE_ROWS[1]
    for y in range(y0, y1 + 1):
        xs = np.nonzero(nu[y])[0]   # the bra wraps the bump; the body layer would also catch the fist
        if len(xs) < 4:
            continue
        fx = int(xs.max()) if sign > 0 else int(xs.min())
        n = 3 if y0 + 2 <= y <= y1 - 2 else (2 if y0 + 1 <= y <= y1 - 1 else 1)
        for arr in (nb, nu):
            row = arr[y].copy()
            if size == 's':
                if sign > 0:
                    row[fx - n:fx + 1] = 0
                else:
                    row[fx:fx + n + 1] = 0
            else:
                if sign > 0:
                    row[fx - 1 + n:W] = arr[y][fx - 1:W - n]; row[fx - 1:fx - 1 + n] = arr[y][fx - 1]
                else:
                    row[0:fx + 2 - n] = arr[y][n:fx + 2]; row[fx + 2 - n:fx + 2] = arr[y][fx + 1]
            arr[y] = row
    return nb, nu


def build():
    for size in ('s', 'l'):
        out = os.path.join(KIT, f'bust_{size}')
        for part in ('body', 'underwear'):
            os.makedirs(os.path.join(out, part), exist_ok=True)
        for anim, cols in ANIMS.items():
            body, under = load('body', anim), load('underwear', anim)
            nb, nu = body.copy(), under.copy()
            for d, r in ROWS.items():
                for c in range(cols):
                    ys, xs = slice(r * H, (r + 1) * H), slice(c * W, (c + 1) * W)
                    b, u = body[ys, xs], under[ys, xs]
                    ytop = cell_top(b)
                    if d == 'down':
                        b2, u2 = front_back(b, u, ytop, size)
                    elif d == 'up':
                        continue
                    else:
                        b2, u2 = side(b, u, ytop, d, size)
                    outline_fix(b2)
                    nb[ys, xs], nu[ys, xs] = b2, u2
            for part, arr in (('body', nb), ('underwear', nu)):
                save(arr, os.path.join(KIT, part, f'{anim}.png'), os.path.join(out, part, f'{anim}.png'))
        print('wrote', out)


if __name__ == '__main__':
    build()
