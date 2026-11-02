"""Anime face stamped over the LPC head (which has small realistic eyes we cover).

Prior art: RPG Maker MV walkers put 6px-wide eyes on a ~22px head at eye row
~60% down the face; LPC Revised (ElizaWy) draws eyes on rows 29-31 of the 64px
cell. We stamp a 6x4 eye at rows 28-31 (front) and a 4x4 eye in profile.
"""
EYE_RAMPS = {  # dark, mid, light
    'blue': ((28, 50, 110), (58, 110, 200), (140, 200, 250)),
    'green': ((30, 80, 40), (60, 150, 80), (150, 220, 150)),
    'purple': ((70, 30, 110), (130, 70, 190), (200, 160, 240)),
    'yellow': ((120, 70, 10), (220, 160, 30), (250, 230, 130)),
    'brown': ((50, 25, 10), (120, 70, 30), (200, 150, 100)),
    'red': ((110, 20, 20), (200, 50, 50), (250, 150, 150)),
    'grey': ((50, 50, 60), (120, 120, 135), (200, 200, 210)),
}
LINE = (40, 24, 32)
WHITE = (250, 248, 245)
BLUSH = (236, 150, 140)
_ = None

EYE_Y = 28  # top row of the eye when the head is at its standing height


def stamp(fr, x0, y0, rows):
    for j, r in enumerate(rows):
        for i, c in enumerate(r):
            if c is not None and 0 <= y0 + j < fr.shape[0] and 0 <= x0 + i < fr.shape[1]:
                fr[y0 + j, x0 + i] = (*c, 255)


def front_eye(ramp, closed):
    dk, m, l = ramp
    L, W = LINE, WHITE
    if closed:
        return [[_] * 6, [_] * 6, [L, L, L, L, L, L], [_, L, _, _, L, _]]
    return [[L, L, L, L, L, L], [W, dk, W, dk, dk, W], [W, m, l, l, m, W], [_, dk, m, m, dk, _]]


def side_eye(ramp, closed):
    dk, m, l = ramp
    L, W = LINE, WHITE
    if closed:
        return [[_] * 4, [_] * 4, [L, L, L, L], [_, L, _, _]]
    return [[L, L, L, L], [L, dk, W, W], [W, m, l, W], [_, dk, dk, _]]


def draw(fr, direction, dy, eye, female, blink=False):
    ramp = EYE_RAMPS.get(eye, EYE_RAMPS['brown'])
    y = EYE_Y + dy
    if direction == 'down':
        e = front_eye(ramp, blink)
        stamp(fr, 25, y, e)
        stamp(fr, 33, y, e)
        fr[y + 5, 31] = fr[y + 5, 32] = (*LINE, 255)  # mouth
        if female:
            for x in (23, 24, 39, 40):
                fr[y + 4, x] = (*BLUSH, 255)
    elif direction in ('right', 'left'):
        e = side_eye(ramp, blink)
        if direction == 'left':
            e = [list(reversed(r)) for r in e]
        x0 = 33 if direction == 'right' else 27
        stamp(fr, x0, y, e)
        if female:
            for x in ((x0 + 4, x0 + 5) if direction == 'right' else (x0 - 2, x0 - 1)):
                fr[y + 4, x] = (*BLUSH, 255)
