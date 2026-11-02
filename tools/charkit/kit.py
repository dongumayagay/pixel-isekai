"""Load the character kit and compose characters from it.

A character is a dict of attributes (see roster.py for examples); `layers()`
turns it into [(z, index map, rgba palette)] and `frame()` composites one
64x64 cell of one animation and direction, with the anime face stamped on.
"""
import json, os
import numpy as np
from PIL import Image

import face
import face2x
from upscale import scale2x

HERE = os.path.dirname(os.path.abspath(__file__))
KIT_DIR = os.path.join(os.path.dirname(os.path.dirname(HERE)), 'assets', 'charkit')
CELL = 64
HEAD_TOP = 15  # y of the standing head's top row in the LPC head sheet; the face follows the head's bob
BLINK = 'blink'


def hexrgb(h):
    h = h.lstrip('#')
    return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))


class Kit:
    def __init__(self, kit_dir=KIT_DIR):
        self.dir = kit_dir
        self.meta = json.load(open(os.path.join(kit_dir, 'kit.json')))
        self.parts = {p['id']: p for p in self.meta['parts']}
        self.rows = self.meta['rows']
        self._sheets = {}

    def ramp(self, material, name):
        m = self.meta['materials'][material]
        return [hexrgb(c) for c in m['palettes'][name if name in m['palettes'] else m['base']]]

    def names(self, material):
        return list(self.meta['materials'][material]['palettes'])

    def sheet(self, file):
        """uint8 index map of a kit sheet (cached)."""
        if file not in self._sheets:
            self._sheets[file] = np.array(Image.open(os.path.join(self.dir, file)))
        return self._sheets[file]

    def palette(self, part, color):
        """256x4 RGBA lookup for a part recoloured to `color` (a palette name of its material)."""
        p = self.parts[part]
        pal = np.zeros((256, 4), np.uint8)
        if p['material']:
            pal[1:7, :3] = self.ramp(p['material'], color)
            pal[1:7, 3] = 255
        for i, c in enumerate(p['extras']):
            pal[7 + i, :3] = hexrgb(c)
            pal[7 + i, 3] = 255
        return pal

    def layers(self, char):
        """[(z, part id, palette)] for a character dict, bottom first."""
        out = []
        for part, color in character_parts(char):
            p = self.parts[part]
            pal = self.palette(part, color)
            for layer in p['layers']:
                out.append((layer['z'], part, layer, pal))
        out.sort(key=lambda t: t[0])
        return out

    def frame(self, char, anim, direction, col, blink=False, scale=2):
        """One composited RGBA cell: 64x64 at scale 1, 128x128 (Scale2x + 2x face) at scale 2."""
        row = self.rows[direction]
        fr = np.zeros((CELL, CELL, 4), np.uint8)
        head_dy = 0
        for z, part, layer, pal in self.layers(char):
            a = layer['anims'].get(anim) or layer['anims']['walk']
            idx = self.sheet(a['file'])
            if col >= a['cols'] or row >= a['rows']:
                continue
            cell = idx[row * CELL:(row + 1) * CELL, col * CELL:(col + 1) * CELL]
            over(fr, pal[cell])
            if part.startswith('head_'):
                ys = np.nonzero(cell)[0]
                head_dy = (int(ys.min()) - HEAD_TOP) if len(ys) else 0
                if scale == 1:
                    face.draw(fr, direction, head_dy, char.get('eyes', 'brown'), char.get('sex', 'f') == 'f', blink)
                else:
                    fr = scale2x(fr)
                    face2x.draw(fr, direction, head_dy * 2, char, blink)
                    fr = fr[::2, ::2].copy()  # back to 64 for the layers above; see below
        if scale == 2:
            return self._frame2x(char, anim, direction, col, blink)
        return fr

    def _frame2x(self, char, anim, direction, col, blink):
        """Layers below the head composited, upscaled, face drawn, then the layers above upscaled on top."""
        row = self.rows[direction]
        layers = self.layers(char)
        below = np.zeros((CELL, CELL, 4), np.uint8)
        above = np.zeros((CELL, CELL, 4), np.uint8)
        head_dy, seen_head = 0, False
        for z, part, layer, pal in layers:
            a = layer['anims'].get(anim) or layer['anims']['walk']
            idx = self.sheet(a['file'])
            if col >= a['cols'] or row >= a['rows']:
                continue
            cell = idx[row * CELL:(row + 1) * CELL, col * CELL:(col + 1) * CELL]
            over(above if seen_head else below, pal[cell])
            if part.startswith('head_'):
                ys = np.nonzero(cell)[0]
                head_dy = (int(ys.min()) - HEAD_TOP) if len(ys) else 0
                seen_head = True
        fr = scale2x(below)
        face2x.draw(fr, direction, head_dy * 2, char, blink)
        over(fr, scale2x(above))
        return fr


def over(dst, src):
    """Source-over for binary alpha."""
    m = src[..., 3] > 0
    dst[m] = src[m]


def character_parts(c):
    """Attribute dict -> [(part id, colour name)], in no particular order."""
    f = c.get('sex', 'f') == 'f'
    g = 'female' if f else 'male'
    skin, hair = c.get('skin', 'light'), c.get('hair_color', 'black')
    out = [(c.get('body') or ('body_female' if f else 'body_male'), skin)]
    out.append((c.get('head') or (f'head_{g}_elderly' if c.get('old') else f'head_{g}'), skin))
    if c.get('ears'):
        out.append((f"ears_{c['ears']}", skin))
    if c.get('tail'):
        out.append((f"tail_{c['tail']}", c.get('tail_color', hair)))
    if c.get('hair'):
        out.append((f"hair_{c['hair']}", hair))
    for slot in ('torso', 'legs', 'feet', 'shoulders', 'hat', 'over'):
        v = c.get(slot)
        if not v:
            continue
        items = v if isinstance(v, list) else [v]
        for item in items:
            part, color = (item, None) if isinstance(item, str) else item
            if part.endswith('_$'):
                part = part[:-2] + '_' + g
            out.append((part, color))
    for h in c.get('held', []) if isinstance(c.get('held'), list) else ([c['held']] if c.get('held') else []):
        out.append((h, None))
        if h + '_bg' in KIT_PARTS_WITH_BG:
            out.append((h + '_bg', None))
    return out


KIT_PARTS_WITH_BG = {'staff_simple_bg'}


def to_image(a):
    return Image.fromarray(a, 'RGBA')
