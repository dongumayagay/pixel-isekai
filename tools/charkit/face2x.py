"""Anime face drawn at 128px over a Scale2x'd LPC head.

Prior art: RPG Maker MV/MZ walkers (eyes ~1/3 of face width, two highlights),
Stardew portraits (lash line heavier than lower lid), LPC Revised eye rows.
At 2x the head is ~44px wide: eyes are 12x9, with brows, lashes, a 3-tone
iris, two highlights, a mouth shape and blush.

Glyph key: . none  L line  W white  d dark iris  m mid  l light  H highlight
           B blush  k lash (darker line)  p pupil
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
LASH = (24, 12, 20)
WHITE = (250, 248, 245)
HI = (255, 255, 255)
BLUSH = (236, 150, 140)
MOUTH_IN = (160, 60, 70)

EYE_Y = 58  # top row of the eye at standing height (LPC eye rows 29-31 doubled)
EYE_L_X, EYE_R_X = 47, 69  # left edge of each 12-wide eye; face centre is x=64
SIDE_X = 66  # profile eye left edge when facing right

# Front eyes, 12 wide. Outer corner is on the left for the character's right eye
# (viewer's left); the other eye is mirrored.
FRONT = {
    'round': [
        '.kkkkkkkkkk.',
        'kkkkkkkkkkkk',
        'LWWdddddWWWL',
        'LWdHHdddmWWL',
        'LWdHHmmlmWWL',
        'LWddmllllWWL',
        '.LWdmlllWWL.',
        '..LWddddWL..',
        '...LLLLLL...',
    ],
    'sharp': [
        '.kkkkkkkkk..',
        'kkkkkkkkkkkk',
        'LWWdddddWWWL',
        'LWdHHdddmWWL',
        'LWdHHmmlmWWL',
        '.LWdmllldWL.',
        '..LWddddWL..',
        '...LLLLLL...',
    ],
    'droopy': [
        'kkkkkkkk....',
        'kkkkkkkkkkk.',
        'LWWdddddWWWL',
        'LWdHHdddmWWL',
        'LWdHHmmlmWWL',
        'LWddmllllWWL',
        '.LWdmlllWWL.',
        '..LWddddWL..',
        '...LLLLLL...',
    ],
    'cat': [
        '.kkkkkkkkkk.',
        'kkkkkkkkkkkk',
        'LWWdddddWWWL',
        'LWdHHdpdmWWL',
        'LWdHHmpmmWWL',
        'LWdmmlplllWL',
        '.LWdmlpllWL.',
        '..LWddddWL..',
        '...LLLLLL...',
    ],
}
CLOSED = [
    '............',
    '............',
    '............',
    '............',
    '.kkkkkkkkkk.',
    'kk........kk',
]
SIDE = {  # 8 wide, facing right; outer corner on the right
    'round': ['kkkkkkk.', 'kkkkkkkk', 'LddWWWWL', 'LdHHmWWL', 'LddmlWWL', 'LdmllWWL', '.LdllWL.', '..LLLL..'],
    'sharp': ['.kkkkkk.', 'kkkkkkkk', 'LddWWWWL', 'LdHHmWWL', 'LddmlWWL', '.LdllWL.', '..LLLL..'],
    'droopy': ['kkkkkk..', 'kkkkkkk.', 'LddWWWWL', 'LdHHmWWL', 'LddmlWWL', 'LdmllWWL', '.LdllWL.', '..LLLL..'],
    'cat': ['kkkkkkk.', 'kkkkkkkk', 'LddWWWWL', 'LdHpmWWL', 'LddplWWL', 'LdmplWWL', '.LdllWL.', '..LLLL..'],
}
SIDE_CLOSED = ['........', '........', '........', '........', 'kkkkkkk.', '......kk']
BROWS = {  # 12 wide, 2 rows, drawn 4 rows above the eye; per expression
    'neutral': ['..LLLLLLL...', '.LL.......L.'],
    'angry': ['.......LLLL.', '....LLLL....'],
    'soft': ['...LLLLLL...', '..LL......L.'],
    'none': [],
}
MOUTHS = {  # drawn with the character's centre at x=64, top row at EYE_Y + 20
    'smile': ['L......L', '.LLLLLL.'],
    'neutral': ['..LLLL..'],
    'grin': ['L......L', 'LppppppL', '.LLLLLL.'],
    'open': ['..LLLL..', '.LppppL.', '..LLLL..'],
    'frown': ['.LLLLLL.', 'L......L'],
}


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
    C = {'L': LINE, 'k': LASH, 'W': WHITE, 'd': dk, 'm': m, 'l': l, 'H': HI, 'p': LASH, 'B': BLUSH}
    y = EYE_Y + dy
    if direction == 'down':
        e = CLOSED if blink else FRONT[shape]
        stamp(fr, EYE_L_X, y, e, C)
        stamp(fr, EYE_R_X, y, mirror(e), C)
        brow = BROWS[char.get('brows', 'soft' if female else 'neutral')]
        stamp(fr, EYE_L_X, y - 4, brow, C)
        stamp(fr, EYE_R_X, y - 4, mirror(brow), C)
        mouth = MOUTHS[char.get('mouth', 'smile')]
        stamp(fr, 60, y + 10, mouth, {'L': LINE, 'p': MOUTH_IN})
        if female:
            stamp(fr, 43, y + 7, ['BBBB', '.BBB'], C)
            stamp(fr, 81, y + 7, ['BBBB', 'BBB.'], C)
    elif direction in ('right', 'left'):
        e = SIDE_CLOSED if blink else SIDE[shape]
        brow = ['.LLLLL.', 'L.....L'] if not blink else []
        if direction == 'right':
            stamp(fr, SIDE_X, y, e, C)
            stamp(fr, SIDE_X, y - 4, brow, C)
            if female:
                stamp(fr, SIDE_X + 8, y + 7, ['BBB', 'BB.'], C)
        else:
            x0 = 128 - SIDE_X - 8
            stamp(fr, x0, y, mirror(e), C)
            stamp(fr, x0, y - 4, mirror(brow), C)
            if female:
                stamp(fr, x0 - 3, y + 7, ['BBB', '.BB'], C)
