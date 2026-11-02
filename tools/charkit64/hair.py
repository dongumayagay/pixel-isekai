#!/usr/bin/env python3
"""Bake hairstyles onto every frame of the 64x128 kit: python3 -I tools/charkit64/hair.py

The skull in the base only translates between frames (checked: the top-26-row head crop differs by
a few neck pixels only), so a hairstyle is three drawings (down, right, up; left is the mirror,
the base's left row is exactly the mirrored right row) placed per frame at the skull anchor.

A drawing is a dict of row -> fill spans plus shading pixels, authored against the idle-down /
idle-right / idle-up column-0 skull (see dump_skull()). Silhouette edges get the outline tone
automatically (same rule the base uses: every hair pixel on the silhouette edge is the darkest tone).

Writes assets/charkit64/hair_<style>/<anim>.png (indexed, hair ramp) and registers the part in
kit.json (material hair, z 40, same layer slot as the baked hair_long). A drawing may return
(front, back): the back cell becomes hair_<style>_back at z 5, behind the body (ponytail tail).
"""
import json
import os
import sys
import numpy as np
from PIL import Image
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sheet import KIT, W, H, ROWS, ANIMS, load, cell_top, outline_fix

TONE = {'A': 1, 'B': 2, 'C': 3, 'D': 4, 'E': 5, 'F': 6}


# ----------------------------------------------------------------------------- skull anchors
def anchor(cell):
    """(cx, top) of the skull: top = first body row, cx = centre of the widest of the next 14 rows."""
    top = cell_top(cell)
    best = None
    for y in range(top, top + 14):
        xs = np.nonzero(cell[y])[0]
        if len(xs) and (best is None or xs.max() - xs.min() > best[1] - best[0]):
            best = (int(xs.min()), int(xs.max()))
    return ((best[0] + best[1]) // 2, top)


def anchors():
    out = {}
    for anim, cols in ANIMS.items():
        body = load('body', anim)
        for d, r in ROWS.items():
            for c in range(cols):
                out[(anim, d, c)] = anchor(body[r * H:(r + 1) * H, c * W:(c + 1) * W])
    return out


def dump_skull(direction='down'):
    """Ascii of the idle column-0 skull with the eyes layer, for authoring drawings."""
    b = load('body', 'idle'); e = load('eyes', 'idle'); r = ROWS[direction]
    cell, eye = b[r * H:(r + 1) * H, :W], e[r * H:(r + 1) * H, :W]
    print(direction, 'anchor', anchor(cell))
    for y in range(14, 56):
        print(f'{y:3d}', ''.join('*' if eye[y, x] else ('abcdef'[cell[y, x] - 1] if 0 < cell[y, x] <= 6 else '.') for x in range(10, 54)))


# ----------------------------------------------------------------------------- drawing DSL
def draw(spans, shade=(), lines=()):
    """spans: {y: [(x0, x1), ...]} filled with the main tone E.
    shade: [(y, x0, x1, tone)] overrides inside the fill. lines: [(x, y, tone)] single pixels.
    Returns a 64x128 uint8 cell with the outline applied."""
    cell = np.zeros((H, W), np.uint8)
    for y, ss in spans.items():
        for x0, x1 in ss:
            cell[y, x0:x1 + 1] = TONE['E']
    for y, x0, x1, t in shade:
        m = cell[y, x0:x1 + 1] > 0
        cell[y, x0:x1 + 1][m] = TONE[t]
    for x, y, t in lines:
        if cell[y, x]:
            cell[y, x] = TONE[t]
    outline_fix(cell)
    return cell


def rows(y0, y1, x0, x1):
    """Helper: the same span on rows y0..y1."""
    return {y: [(x0, x1)] for y in range(y0, y1 + 1)}


def merge(*dicts):
    out = {}
    for d in dicts:
        for y, ss in d.items():
            out.setdefault(y, []).extend(ss)
    return out


# ----------------------------------------------------------------------------- styles
# Authored against idle col 0: down anchor (32, 19), right anchor (33, 18), up anchor (32, 19).
# Skull (down): top 19 at x 28-36, widest rows 30-34 at x 20-44, eyes rows 34-42, ears x 17-18 /
# 46-47 rows 35-38, chin row 46 x 26-38. Hair sits 2px above and 2px outside the skull like the base.

def strand(points, tone='D'):
    """Continuous 1px polyline through the points (x, y)."""
    out = []
    for (x0, y0), (x1, y1) in zip(points, points[1:]):
        n = max(abs(x1 - x0), abs(y1 - y0), 1)
        for i in range(n + 1):
            out.append((round(x0 + (x1 - x0) * i / n), round(y0 + (y1 - y0) * i / n), tone))
    return out


def bob_down():
    cap = merge({17: [(28, 36)], 18: [(26, 38)], 19: [(24, 40)], 20: [(23, 41)], 21: [(22, 42)]},
                rows(22, 23, 21, 43), rows(24, 25, 20, 44), rows(26, 29, 19, 45), rows(30, 33, 18, 46))
    # fringe: full to row 33, then spikes ending on rows 34 / 35 with forehead showing between
    fringe = {34: [(18, 22), (23, 26), (28, 31), (33, 36), (38, 40), (42, 46)],
              35: [(18, 22), (24, 25), (29, 30), (34, 35), (39, 39), (42, 46)]}
    sides = merge(rows(36, 42, 18, 22), rows(36, 42, 42, 46), rows(43, 44, 19, 22), rows(43, 44, 42, 45),
                  {45: [(20, 23), (41, 44)], 46: [(21, 24), (40, 43)]})
    # tone: lit crown E, mid-tone D from the widest rows down, the side curtains C/B at the bottom
    shade = [(y, 18, 23, 'D') for y in range(28, 36)] + [(y, 41, 46, 'D') for y in range(28, 36)]
    shade += [(y, 18, 22, 'D') for y in range(36, 47)] + [(y, 42, 46, 'D') for y in range(36, 47)]
    shade += [(y, 19, 21, 'C') for y in range(40, 47)] + [(y, 43, 45, 'C') for y in range(40, 47)]
    shade += [(y, 20, 21, 'B') for y in range(44, 47)] + [(y, 43, 44, 'B') for y in range(44, 47)]
    shade += [(33, 23, 41, 'D'), (34, 23, 41, 'D'), (35, 24, 39, 'C')]      # fringe underside
    shade += [(19, 29, 34, 'F'), (20, 27, 32, 'F'), (21, 26, 29, 'F')]      # sheen on the crown
    lines = strand([(31, 18), (28, 21), (26, 24), (25, 27), (24, 31)]) + strand([(33, 18), (36, 21), (38, 24), (39, 27), (40, 31)])
    lines += strand([(32, 19), (32, 23)]) + strand([(29, 22), (28, 26), (28, 30)]) + strand([(35, 22), (36, 26), (36, 30)])
    lines += [(x, 33, 'C') for x in (24, 27, 30, 32, 35, 37, 41)]            # fringe strand ends
    return draw(merge(cap, fringe, sides), shade, lines)


# Skull (right): top 18 at x 28-36, widest rows 29-31 at x 21-46, face front x 44-46, eye x 36-44
# rows 33-41, ear x 26-30 rows 35-43, chin row 44 x 24-43, neck rows 45-47 x 24-33.
def bob_right():
    cap = merge({16: [(28, 36)], 17: [(26, 38)], 18: [(25, 40)], 19: [(24, 42)], 20: [(23, 43)],
                 21: [(22, 44)], 22: [(22, 45)]}, rows(23, 25, 21, 46), rows(26, 28, 20, 47), rows(29, 31, 19, 48))
    # front: fringe over the forehead to row 33, a swept lock down the temple to row 36
    front = {32: [(19, 48)], 33: [(19, 30), (33, 48)], 34: [(19, 30), (40, 47)], 35: [(19, 30), (42, 46)], 36: [(20, 30), (44, 45)]}
    back = merge(rows(37, 42, 20, 30), rows(43, 44, 21, 29), {45: [(22, 28)], 46: [(23, 27)]})
    shade = [(y, 19, 27, 'D') for y in range(27, 33)] + [(y, 19, 30, 'D') for y in range(33, 47)]
    shade += [(y, 19, 24, 'C') for y in range(36, 47)] + [(y, 20, 22, 'B') for y in range(42, 47)]
    shade += [(y, 26, 29, 'C') for y in range(44, 47)]
    shade += [(33, 33, 47, 'D'), (34, 40, 47, 'D'), (35, 42, 46, 'C')]
    shade += [(18, 29, 35, 'F'), (19, 27, 33, 'F'), (20, 26, 29, 'F')]
    lines = strand([(30, 17), (27, 20), (25, 24), (24, 28)]) + strand([(34, 17), (37, 20), (39, 24), (41, 28), (43, 32)])
    lines += strand([(32, 18), (33, 22), (34, 26), (35, 30)]) + strand([(28, 24), (27, 29), (27, 34), (26, 39)], 'C')
    lines += [(x, 33, 'C') for x in (35, 38, 42, 45)]
    return draw(merge(cap, front, back), shade, lines)


# Skull (up): same silhouette as down without the face; the bob covers the whole back of the head
# and ends in a wavy edge on rows 46-48 above the shoulders.
def bob_up():
    cap = merge({17: [(28, 36)], 18: [(26, 38)], 19: [(24, 40)], 20: [(23, 41)], 21: [(22, 42)]},
                rows(22, 23, 21, 43), rows(24, 25, 20, 44), rows(26, 29, 19, 45), rows(30, 38, 18, 46),
                rows(39, 43, 19, 45), rows(44, 45, 20, 44),
                {46: [(21, 26), (28, 31), (33, 36), (38, 43)], 47: [(22, 25), (29, 30), (34, 35), (39, 42)], 48: [(23, 24), (40, 41)]})
    shade = [(y, 18, 46, 'D') for y in range(34, 49)] + [(y, 19, 45, 'C') for y in range(42, 49)] + [(y, 20, 44, 'B') for y in range(46, 49)]
    shade += [(y, 18, 21, 'D') for y in range(26, 34)] + [(y, 43, 46, 'D') for y in range(26, 34)]
    shade += [(19, 28, 33, 'F'), (20, 26, 31, 'F'), (21, 25, 28, 'F')]
    lines = strand([(31, 18), (28, 21), (26, 25), (25, 30), (24, 36), (24, 42)]) + strand([(33, 18), (36, 21), (38, 25), (39, 30), (40, 36), (40, 42)])
    lines += strand([(32, 20), (32, 26), (31, 32), (31, 40)]) + strand([(29, 23), (28, 29), (28, 36)], 'C') + strand([(35, 23), (36, 29), (36, 36)], 'C')
    return draw(cap, shade, lines)


# Short messy cut (hero, cat-girl, dwarf): hugs the skull 1px out, spikes on the crown and at the
# nape, a big uneven fringe, tufts flicking out over the ears (Angeler's own 48px short hair is the
# reference). Shading: only diagonal strands, the lower third in the mid tone.
def short_down():
    cap = merge({16: [(29, 30), (36, 36)], 17: [(27, 32), (34, 38)], 18: [(26, 39)],
                 19: [(25, 40)], 20: [(24, 41)], 21: [(23, 42)]},
                rows(22, 23, 22, 43), rows(24, 25, 21, 44), rows(26, 29, 20, 45), rows(30, 32, 19, 46),
                {24: [(16, 20), (45, 49)], 25: [(15, 20), (45, 50)], 26: [(16, 20), (45, 49)], 27: [(18, 20), (45, 47)]})
    fringe = {33: [(19, 25), (27, 31), (33, 36), (38, 41), (43, 46)],
              34: [(19, 24), (28, 30), (34, 35), (39, 40), (43, 46)], 35: [(19, 23), (29, 29), (34, 34), (44, 46)],
              36: [(19, 22), (45, 46)], 37: [(20, 21), (46, 46)], 38: [(20, 20)]}
    shade = [(y, 19, 24, 'D') for y in range(29, 39)] + [(y, 41, 46, 'D') for y in range(29, 39)]
    shade += [(y, 19, 21, 'C') for y in range(34, 39)] + [(y, 44, 46, 'C') for y in range(34, 39)]
    shade += [(32, 25, 40, 'D'), (33, 26, 42, 'D'), (34, 27, 41, 'C'), (35, 28, 35, 'C')]
    shade += [(y, 16, 18, 'D') for y in range(24, 27)] + [(y, 47, 49, 'D') for y in range(24, 27)]
    shade += [(19, 30, 35, 'F'), (20, 28, 32, 'F')]
    lines = strand([(29, 19), (26, 23), (23, 27)]) + strand([(36, 19), (39, 23), (42, 27)]) + strand([(33, 20), (31, 25), (30, 30)], 'C')
    lines += [(x, 32, 'C') for x in (27, 31, 33, 36, 38, 41)]
    return draw(merge(cap, fringe), shade, lines)


def short_right():
    # cap over the skull, then the back of the head covered down to the nape (skull back x 21-24
    # rows 32-44, neck from row 45); small flicks at the temple and the nape, nothing beyond x 18
    cap = merge({15: [(30, 31), (36, 36)], 16: [(28, 33), (35, 38)], 17: [(26, 39)], 18: [(25, 41)],
                 19: [(24, 42)], 20: [(23, 43)], 21: [(22, 44)], 22: [(21, 45)]},
                rows(23, 25, 21, 46), rows(26, 28, 20, 47), rows(29, 31, 20, 47))
    back = merge(rows(32, 36, 20, 30), rows(37, 40, 20, 29), rows(41, 43, 21, 28),
                 {44: [(21, 23), (25, 27)], 45: [(22, 22), (26, 26)]},
                 {24: [(18, 20)], 25: [(18, 20)], 40: [(18, 20)], 41: [(19, 20)]})
    front = {32: [(31, 47)], 33: [(33, 46)], 34: [(38, 46)], 35: [(41, 45)], 36: [(43, 44)]}
    shade = [(y, 20, 29, 'D') for y in range(27, 46)] + [(y, 20, 24, 'C') for y in range(34, 46)]
    shade += [(y, 21, 22, 'B') for y in range(41, 46)]
    shade += [(33, 33, 46, 'D'), (34, 38, 46, 'D'), (35, 41, 45, 'C')]
    shade += [(17, 29, 35, 'F'), (18, 28, 33, 'F')]
    lines = strand([(30, 17), (27, 21), (24, 25), (23, 30)]) + strand([(36, 17), (39, 21), (42, 26), (44, 31)]) + strand([(33, 18), (35, 23), (37, 28)], 'C')
    lines += strand([(27, 33), (26, 38), (25, 43)], 'C') + [(x, 33, 'C') for x in (35, 38, 41, 44)]
    return draw(merge(cap, back, front), shade, lines)


def short_up():
    # from behind the cut covers the whole back of the skull (widest x 19-45, narrowing to x 21-43
    # at row 46) and ends in a ragged nape line on rows 46-47 over the neck
    cap = merge({16: [(28, 29), (35, 36)], 17: [(27, 32), (34, 38)], 18: [(26, 39)],
                 19: [(25, 40)], 20: [(24, 41)], 21: [(23, 42)]},
                rows(22, 23, 22, 43), rows(24, 25, 21, 44), rows(26, 29, 20, 45), rows(30, 38, 19, 46),
                rows(39, 43, 20, 45), rows(44, 45, 21, 44),
                {46: [(22, 24), (27, 29), (32, 34), (37, 39), (41, 43)], 47: [(23, 23), (28, 28), (33, 33), (38, 38), (42, 42)]},
                {24: [(17, 20), (45, 48)], 25: [(16, 20), (45, 49)], 26: [(18, 20), (45, 47)]})
    shade = [(y, 19, 46, 'D') for y in range(33, 48)] + [(y, 20, 45, 'C') for y in range(41, 48)] + [(y, 21, 44, 'B') for y in range(46, 48)]
    shade += [(y, 16, 21, 'D') for y in range(24, 33)] + [(y, 44, 49, 'D') for y in range(24, 33)]
    shade += [(19, 29, 34, 'F'), (20, 27, 31, 'F')]
    lines = strand([(30, 19), (27, 23), (25, 28), (24, 34), (24, 40)]) + strand([(35, 19), (38, 23), (40, 28), (41, 34), (41, 40)])
    lines += strand([(32, 21), (31, 27), (30, 33), (30, 40)], 'C') + strand([(34, 24), (36, 30), (37, 36), (37, 42)], 'C')
    return draw(cap, shade, lines)


def tail(x_centre, y0, y1, width, wave=1.5, period=9.0):
    """Spans of a hanging tail: `width` at the top tapering to 2px, swaying sideways with a sine."""
    import math
    out = {}
    for y in range(y0, y1 + 1):
        t = (y - y0) / max(y1 - y0, 1)
        w = max(2, round(width * (1 - 0.55 * t)))
        cx = x_centre + round(wave * math.sin((y - y0) / period * math.pi))
        out[y] = [(cx - w // 2, cx - w // 2 + w - 1)]
    return out


def tail_shade(spans, dark_side):
    """Darker column on `dark_side` ('l'|'r') of each span, darkest at the tip."""
    out = []
    ys = sorted(spans)
    for y in ys:
        x0, x1 = spans[y][0]
        t = (y - ys[0]) / max(ys[-1] - ys[0], 1)
        n = 1 if x1 - x0 < 4 else 2
        out.append((y, x0, x0 + n - 1, 'D') if dark_side == 'l' else (y, x1 - n + 1, x1, 'D'))
        if t > 0.6:
            out.append((y, x0, x1, 'C') if x1 - x0 < 3 else (y, x0, x0, 'C') if dark_side == 'l' else (y, x1, x1, 'C'))
    return out


# High ponytail (tavern girl): smooth cap pulled back, side-swept bangs, the tail's puff shows above
# the crown from the front (behind the head), hangs behind the body from the side (second part,
# z 5) and down the spine from behind.
def pony_down():
    cap = merge({17: [(28, 36)], 18: [(26, 38)], 19: [(24, 40)], 20: [(23, 41)], 21: [(22, 42)]},
                rows(22, 23, 21, 43), rows(24, 25, 20, 44), rows(26, 29, 19, 45), rows(30, 31, 18, 46),
                {32: [(18, 23), (25, 46)], 33: [(18, 22), (27, 46)], 34: [(18, 21), (30, 45)], 35: [(18, 21), (33, 44)],
                 36: [(19, 21), (36, 43)], 37: [(19, 20), (39, 42)], 38: [(19, 20), (41, 42)]})                   # bangs sweep right
    shade = [(y, 18, 22, 'D') for y in range(28, 39)] + [(y, 42, 46, 'D') for y in range(28, 36)]
    shade += [(y, 18, 20, 'C') for y in range(33, 39)]
    shade += [(32, 25, 41, 'D'), (33, 27, 42, 'D'), (34, 30, 42, 'C'), (35, 33, 42, 'C'), (36, 36, 42, 'C')]
    shade += [(19, 28, 33, 'F'), (20, 26, 31, 'F'), (21, 25, 28, 'F')]
    lines = strand([(31, 18), (28, 21), (25, 24), (23, 27), (22, 31)]) + strand([(33, 18), (36, 21), (39, 24), (41, 27), (42, 30)])
    lines += strand([(32, 19), (33, 23), (35, 27), (38, 31)], 'C') + strand([(30, 20), (31, 24), (33, 28), (36, 32)], 'C')
    front = draw(cap, shade, lines)
    puff = {13: [(30, 34)], 14: [(28, 36)], 15: [(27, 37)], 16: [(26, 38)], 17: [(25, 39)], 18: [(25, 39)]}
    back = draw(puff, [(y, 25, 39, 'D') for y in range(16, 19)], strand([(29, 14), (28, 17)], 'C') + strand([(35, 14), (36, 17)], 'C'))
    return front, back


def pony_right():
    cap = merge({16: [(28, 36)], 17: [(26, 38)], 18: [(25, 40)], 19: [(24, 42)], 20: [(23, 43)],
                 21: [(22, 44)], 22: [(22, 45)]}, rows(23, 25, 21, 46), rows(26, 28, 20, 47), rows(29, 31, 19, 48),
                {32: [(20, 48)], 33: [(21, 30), (35, 48)], 34: [(22, 29), (40, 47)], 35: [(23, 28), (43, 46)], 36: [(24, 27), (45, 45)]},
                rows(37, 38, 24, 27), {39: [(25, 26)]},                                                  # ear tuft
                {19: [(21, 22)], 20: [(19, 22)], 21: [(18, 22)], 22: [(18, 21)], 23: [(19, 21)]})                # tie
    shade = [(y, 19, 27, 'D') for y in range(26, 40)] + [(y, 19, 23, 'C') for y in range(30, 40)]
    shade += [(33, 35, 47, 'D'), (34, 40, 47, 'D'), (35, 43, 46, 'C')]
    shade += [(y, 18, 22, 'B') for y in range(19, 24)]
    shade += [(18, 29, 35, 'F'), (19, 27, 33, 'F'), (20, 26, 29, 'F')]
    lines = strand([(34, 17), (37, 20), (39, 24), (41, 28), (43, 32)]) + strand([(30, 17), (26, 20), (23, 23), (21, 24)])
    lines += strand([(33, 18), (31, 21), (27, 25), (24, 27)], 'C') + strand([(36, 20), (34, 24), (32, 28), (30, 31)], 'C')
    front = draw(cap, shade, lines)
    puff = {18: [(14, 20)], 19: [(12, 20)], 20: [(11, 18)], 21: [(10, 17)], 22: [(10, 17)], 23: [(10, 17)], 24: [(11, 17)]}
    t = tail(13, 25, 54, 7, wave=2.0, period=10.0)
    back = draw(merge(puff, t), tail_shade(t, 'l') + [(y, 10, 12, 'D') for y in range(20, 25)],
                strand([(15, 20), (14, 26), (13, 34), (13, 42), (14, 50)], 'D') + strand([(17, 23), (16, 30), (15, 38)], 'C'))
    return front, back


def pony_up():
    cap = merge({17: [(28, 36)], 18: [(26, 38)], 19: [(24, 40)], 20: [(23, 41)], 21: [(22, 42)]},
                rows(22, 23, 21, 43), rows(24, 25, 20, 44), rows(26, 29, 19, 45), rows(30, 38, 18, 46),
                rows(39, 43, 19, 45), rows(44, 45, 20, 44), {46: [(21, 43)], 47: [(23, 41)]})
    shade = [(y, 18, 46, 'D') for y in range(33, 48)] + [(y, 20, 44, 'C') for y in range(41, 48)] + [(y, 21, 43, 'B') for y in range(46, 48)]
    shade += [(y, 18, 21, 'D') for y in range(26, 33)] + [(y, 43, 46, 'D') for y in range(26, 33)]
    shade += [(19, 28, 33, 'F'), (20, 26, 31, 'F')]
    lines = strand([(24, 24), (27, 27), (29, 29)]) + strand([(40, 24), (37, 27), (35, 29)]) + strand([(22, 32), (26, 30), (28, 30)]) + strand([(42, 32), (38, 30), (36, 30)])
    lines += strand([(21, 38), (25, 34), (28, 33)], 'C') + strand([(43, 38), (39, 34), (36, 33)], 'C')
    front = draw(cap, shade, lines)
    # from behind: tie puff at the crown, the tail hangs down the spine in front of the body
    puff = merge({25: [(29, 35)]}, rows(26, 29, 28, 36), rows(30, 32, 28, 36), {33: [(29, 35)]})
    t = tail(32, 34, 62, 9, wave=1.0, period=12.0)
    tail_cell = draw(merge(puff, t), tail_shade(t, 'r') + [(y, 28, 30, 'D') for y in range(26, 34)] + [(y, 29, 35, 'B') for y in range(27, 29)],
                     strand([(31, 35), (31, 43), (30, 51), (31, 58)], 'D') + strand([(34, 36), (34, 44), (33, 52)], 'C'))
    front = np.where(tail_cell > 0, tail_cell, front)
    return front, np.zeros((H, W), np.uint8)


STYLES = {
    'hair_bob': {'down': bob_down, 'right': bob_right, 'up': bob_up},
    'hair_short': {'down': short_down, 'right': short_right, 'up': short_up},
    'hair_ponytail': {'down': pony_down, 'right': pony_right, 'up': pony_up},
}



# ----------------------------------------------------------------------------- bake
REF = {'down': (32, 19), 'right': (33, 18), 'up': (32, 19)}   # anchors the drawings were authored at


def bake(style, fns, anc):
    """Returns the part ids written: the style and, if any drawing has a behind-body part, style_back."""
    drawn = {d: fn() for d, fn in fns.items()}
    fronts = {d: (v[0] if isinstance(v, tuple) else v) for d, v in drawn.items()}
    backs = {d: v[1] for d, v in drawn.items() if isinstance(v, tuple)}
    for d in fronts:
        assert anc[('idle', d, 0)] == REF[d], (d, anc[('idle', d, 0)], REF[d])
    ids = [style]
    write_part(style, fronts, anc)
    if backs and any(b.any() for b in backs.values()):
        write_part(style + '_back', {d: backs.get(d, np.zeros((H, W), np.uint8)) for d in fronts}, anc)
        ids.append(style + '_back')
    return ids


def write_part(part, cells, anc):
    out = os.path.join(KIT, part)
    os.makedirs(out, exist_ok=True)
    pal = json.load(open(os.path.join(KIT, 'kit.json')))['materials']['hair']['palettes']['blue']
    for anim, cols in ANIMS.items():
        sheet = np.zeros((4 * H, cols * W), np.uint8)
        for d, r in ROWS.items():
            src = 'right' if d == 'left' else d
            base = cells[src]
            for c in range(cols):
                cx, top = anc[(anim, src, c)]
                dx, dy = cx - REF[src][0], top - REF[src][1]
                cell = np.roll(np.roll(base, dy, axis=0), dx, axis=1)
                if d == 'left':
                    cell = cell[:, ::-1]
                sheet[r * H:(r + 1) * H, c * W:(c + 1) * W] = cell
        im = Image.fromarray(sheet, 'P')
        palette = [0, 0, 0] + sum(([int(h[1:3], 16), int(h[3:5], 16), int(h[5:7], 16)] for h in pal), [])
        im.putpalette(palette + [0] * (768 - len(palette)))
        im.save(os.path.join(out, f'{anim}.png'), transparency=0, optimize=True)


def register(parts):
    path = os.path.join(KIT, 'kit.json')
    kit = json.load(open(path))
    kit['parts'] = [p for p in kit['parts'] if p['id'] not in parts]
    for s in parts:
        kit['parts'].append({'id': s, 'material': 'hair', 'z': 5 if s.endswith('_back') else 40, 'extras': [],
                             'anims': {a: {'file': f'{s}/{a}.png', 'cols': n} for a, n in ANIMS.items()}})
    json.dump(kit, open(path, 'w'), indent=1)


if __name__ == '__main__':
    if len(sys.argv) > 1 and sys.argv[1] == 'skull':
        dump_skull(sys.argv[2] if len(sys.argv) > 2 else 'down')
        sys.exit()
    anc = anchors()
    parts = []
    for style, fns in STYLES.items():
        parts += bake(style, fns, anc)
        print('wrote', style)
    register(parts)
