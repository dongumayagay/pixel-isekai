#!/usr/bin/env python3
"""Compose characters from the 64x128 kit (assets/charkit64, built by extract.py).

char = dict(skin='light', hair='hair_long', hair_color='blue', eyes='blue', wear=[('underwear','white'), ...])
Kit64().frame(char, anim, direction, col) -> 64x128 RGBA uint8.
"""
import json, os, sys
import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import palettes

HERE = os.path.dirname(os.path.abspath(__file__))
KIT_DIR = os.path.join(os.path.dirname(os.path.dirname(HERE)), 'assets', 'charkit64')


class Kit64:
    def __init__(self, kit_dir=KIT_DIR):
        self.dir = kit_dir
        self.meta = json.load(open(os.path.join(kit_dir, 'kit.json')))
        self.W, self.H = self.meta['cell']
        self.rows = self.meta['rows']
        self.parts = {p['id']: p for p in self.meta['parts']}
        self.ramps = {m: palettes.all_palettes(m, v['palettes'][v['base']]) for m, v in self.meta['materials'].items()}
        self._sheets = {}

    def sheet(self, file):
        if file not in self._sheets:
            self._sheets[file] = np.array(Image.open(os.path.join(self.dir, file)))
        return self._sheets[file]

    def palette(self, part, color=None, iris=None):
        p = self.parts[part]
        pal = np.zeros((256, 4), np.uint8)
        if p['material']:
            ramps = self.ramps[p['material']]
            ramp = ramps.get(color) or ramps[self.meta['materials'][p['material']]['base']]
            pal[1:7, :3] = [palettes.hexrgb(h) for h in ramp]
            pal[1:7, 3] = 255
        for i, h in enumerate(p['extras']):
            if iris and h == '#639bff':
                h = palettes.EYES.get(iris, h)
            pal[7 + i, :3] = palettes.hexrgb(h)
            pal[7 + i, 3] = 255
        return pal

    def layers(self, c):
        out = [('body', self.palette('body', c.get('skin'))),
               ('eyes', self.palette('eyes', iris=c.get('eyes')))]
        if c.get('hair'):
            out.append((c['hair'], self.palette(c['hair'], c.get('hair_color'))))
        for part, color in c.get('wear', []):
            out.append((part, self.palette(part, color)))
        return sorted(out, key=lambda t: self.parts[t[0]]['z'])

    def frame(self, c, anim, direction, col):
        row = self.rows[direction]
        fr = np.zeros((self.H, self.W, 4), np.uint8)
        for part, pal in self.layers(c):
            a = self.parts[part]['anims'].get(anim) or self.parts[part]['anims']['idle']
            idx = self.sheet(a['file'])
            col_ = col % a['cols']
            cell = idx[row * self.H:(row + 1) * self.H, col_ * self.W:(col_ + 1) * self.W]
            px = pal[cell]
            m = px[..., 3] > 0
            fr[m] = px[m]
        return fr


def to_image(a):
    return Image.fromarray(a, 'RGBA')
