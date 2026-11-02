"""Anime face drawn at 128px over a Scale2x'd LPC head (3/4 top-down).

Eye anatomy copied from three references and then adapted to our 44px head:
  * .context/ref_free/Free/64X128_Idle_Free.png (32px head, 7x7 eye)
  * Sun Haven NPC walkers (3/4 top-down, 24px head, 4x4 eye)
  * Fields of Mistria walkers (3/4 top-down, 2x2 eye)
All three agree on the same rules, which are what the old eyes broke:
  1. no outline around the eye: the dark lash mass sits straight on the skin
  2. the lash is the heaviest element, 2 rows, thickest at the outer corner,
     with a warm red-brown end pixel where it meets the skin
  3. the iris hangs directly under the lash, a dark pupil column runs down its
     middle, the iris gets lighter toward the bottom, one highlight upper-inner
  4. white sclera shows only on the outer side and as a sliver on the inner
     side; a grey "sclera shadow" pixel sits just under the lash
  5. the lower lid is not a line: one or two soft dark-skin pixels at the outer
     bottom corner, the rest of the bottom is the light iris on the skin
  6. the eye is about as tall as it is wide, never a wide oval
See .context/research/anime_eyes.md for the pixel dumps.

Glyph key: . none  k lash  r lash end (red-brown)  S sclera shadow  W white
           d dark iris  m mid iris  l light iris  P pupil  H highlight
           t lower lid  B blush  L line (brows/mouth)  p mouth inside
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
LASH = (28, 22, 38)
LASH_END = (122, 50, 64)
SCLERA_SHADOW = (168, 152, 168)
WHITE = (250, 248, 245)
HI = (255, 255, 255)
LID = (138, 74, 82)
BLUSH = (236, 150, 140)
MOUTH_IN = (160, 60, 70)

EYE_W = 10
EYE_Y = 56  # top lash row at standing height (LPC eye rows 29-31 doubled = 58-62)
EYE_L_X, EYE_R_X = 50, 68  # face centre is x=64; 8px gap between the eyes
SIDE_W = 8
SIDE_X = 66  # profile eye left edge when facing right

# Front eyes, 10 wide. Outer corner on the left (character's right eye, viewer's
# left); the other eye is mirrored.
FRONT = {
    'round': [
        '..kkkkkk..',
        '.kkkkkkkkk',
        'rSSkkkkkkr',
        'rWWdPPHddW',
        '.WWdPPdddW',
        '.SWmdPPdmW',
        '..WmlddlmW',
        '..tllddll.',
        '...tllll..',
    ],
    'sharp': [  # flatter top, lash sweeps up to the outer corner, shorter
        'kkkkkkkk..',
        'rkkkkkkkkk',
        'rSSdPPkkkr',
        '.WWdPPHddW',
        '.SWmdPPdmW',
        '..WmlddlmW',
        '..tllddll.',
        '...tlll...',
    ],
    'droopy': [  # outer corner lower than the inner one
        '....kkkkk.',
        '..kkkkkkkk',
        'rkkkkkkkkr',
        'rSSdPPHddW',
        'rWWdPPdddW',
        '.SWmdPPdmW',
        '..WmlddlmW',
        '..tllddll.',
        '...tllll..',
    ],
    'cat': [  # vertical slit pupil, lighter iris, lash flicks up at the outer end
        'k.kkkkkk..',
        'kkkkkkkkkk',
        'rSSkkkkkkr',
        'rWWmdPmHmW',
        '.WWmdPmmmW',
        '.SWlmdPmlW',
        '..WllPllmW',
        '..tllPlll.',
        '...tllll..',
    ],
}
CLOSED = [
    '..........',
    '..........',
    '..........',
    '..........',
    '..........',
    '.kkkkkkkk.',
    'k........k',
]
# Profile eye, 8 wide, facing right: outer corner (front of the face) on the
# right. The iris is foreshortened to 4 wide with the white behind it.
SIDE = {
    'round': ['...kkkkk', '.kkkkkkk', 'Skkkkkkr', 'WdPPHddr', 'WdPPddd.', 'WmdPPdm.', '.mlddl..', '..llt...'],
    'sharp': ['..kkkkkk', '.kkkkkkr', 'SdPPkkkr', 'WdPPHdd.', 'WmdPPdm.', '.mlddl..', '..llt...'],
    'droopy': ['.kkkkk..', '.kkkkkkk', 'Skkkkkkr', 'WdPPHddr', 'WdPPdddr', 'WmdPPdm.', '.mlddl..', '..llt...'],
    'cat': ['...kkkkk', '.kkkkkkk', 'Skkkkkkr', 'WmdPmHmr', 'WmdPmmm.', 'WlmPlml.', '.llPll..', '..lPt...'],
}
SIDE_CLOSED = ['........', '........', '........', '........', '........', 'kkkkkkk.', '.......k']
BROWS = {  # 10 wide, 2 rows, drawn above the eye; per expression
    'neutral': ['..LLLLLL..', '.LL.....L.'],
    'angry': ['......LLL.', '...LLLL...'],
    'soft': ['...LLLLL..', '..LL....L.'],
    'none': [],
}
MOUTHS = {  # drawn with the character's centre at x=64
    'smile': ['L......L', '.LLLLLL.'],
    'neutral': ['..LLLL..'],
    'grin': ['L......L', 'LppppppL', '.LLLLLL.'],
    'open': ['..LLLL..', '.LppppL.', '..LLLL..'],
    'frown': ['.LLLLLL.', 'L......L'],
}


def pupil(dk):
    return tuple(int(c * 0.45) for c in dk)


def stamp(fr, x0, y0, rows, colors):
    h, w = fr.shape[:2]
    for j, r in enumerate(rows):
        for i, ch in enumerate(r):
            c = colors.get(ch)
            if c is not None and 0 <= y0 + j < h and 0 <= x0 + i < w:
                fr[y0 + j, x0 + i] = (*c, 255)


def mirror(rows):
    return [r[::-1] for r in rows]


def draw(fr, direction, dy, char, blink=False):
    """fr: 128x128 RGBA; dy: head bob in 128px units."""
    eye = char.get('eyes', 'brown')
    dk, m, l = EYE_RAMPS.get(eye, EYE_RAMPS['brown'])
    shape = char.get('eye_shape', 'round' if char.get('sex', 'f') == 'f' else 'sharp')
    female = char.get('sex', 'f') == 'f'
    C = {'L': LINE, 'k': LASH, 'r': LASH_END, 'S': SCLERA_SHADOW, 'W': WHITE, 'd': dk, 'm': m, 'l': l,
         'P': pupil(dk), 'H': HI, 't': LID, 'B': BLUSH}
    y = EYE_Y + dy
    if direction == 'down':
        e = CLOSED if blink else FRONT[shape]
        stamp(fr, EYE_L_X, y, e, C)
        stamp(fr, EYE_R_X, y, mirror(e), C)
        brow = BROWS[char.get('brows', 'soft' if female else 'neutral')]
        stamp(fr, EYE_L_X, y - 4, brow, C)
        stamp(fr, EYE_R_X, y - 4, mirror(brow), C)
        mouth = MOUTHS[char.get('mouth', 'smile')]
        stamp(fr, 60, y + 12, mouth, {'L': LINE, 'p': MOUTH_IN})
        if female:
            stamp(fr, 45, y + 9, ['BBBB', '.BBB'], C)
            stamp(fr, 79, y + 9, ['BBBB', 'BBB.'], C)
    elif direction in ('right', 'left'):
        e = SIDE_CLOSED if blink else SIDE[shape]
        brow = ['.LLLLL.', 'L.....L'] if not blink else []
        if direction == 'right':
            stamp(fr, SIDE_X, y, e, C)
            stamp(fr, SIDE_X, y - 4, brow, C)
            if female:
                stamp(fr, SIDE_X + 8, y + 9, ['BBB', 'BB.'], C)
        else:
            x0 = 128 - SIDE_X - SIDE_W
            stamp(fr, x0, y, mirror(e), C)
            stamp(fr, x0, y - 4, mirror(brow), C)
            if female:
                stamp(fr, x0 - 3, y + 9, ['BBB', '.BB'], C)
