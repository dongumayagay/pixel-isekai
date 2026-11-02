"""Shared helpers for scripts that derive body variants from the extracted 64x128 kit sheets.

Sheets are indexed 'P' PNGs: index 0 transparent, 1..6 the material ramp dark -> light. One cell per
frame, W x H; rows are directions (ROWS), columns are frames (ANIMS gives the count per animation).
"""
import os
import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
KIT = os.path.join(os.path.dirname(os.path.dirname(HERE)), 'assets', 'charkit64')
W, H = 64, 128
ROWS = {'down': 0, 'left': 1, 'right': 2, 'up': 3}
ANIMS = {'idle': 8, 'walk': 10, 'run': 8}
CX = 32


def load(part, anim):
    return np.array(Image.open(os.path.join(KIT, part, f'{anim}.png')))


def save(idx, src_path, path):
    """Write an indexed sheet with the palette of the sheet it was derived from."""
    src = Image.open(src_path)
    im = Image.fromarray(idx.astype(np.uint8), 'P')
    im.putpalette(src.getpalette())
    im.save(path, transparency=0, optimize=True)


def cell_top(b):
    rows = np.nonzero((b > 0).any(axis=1))[0]
    return int(rows.min()) if len(rows) else None


def outline_fix(b):
    """Every opaque pixel on the silhouette edge becomes the outline tone (1)."""
    full = b > 0
    p = np.pad(full, 1)
    edge = full & ~(p[:-2, 1:-1] & p[2:, 1:-1] & p[1:-1, :-2] & p[1:-1, 2:])
    b[edge & (b > 1)] = 1
