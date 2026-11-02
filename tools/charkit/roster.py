#!/usr/bin/env python3
"""Render the core cast: python3 -I tools/charkit/roster.py out.png [scale]

Per character: standing down/right/up/left, blink, then the 8 walk frames facing down.
"""
import os, sys
import numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from kit import Kit, CELL, to_image
from PIL import Image

CAST = {
    'Reincarnated Hero': dict(sex='m', skin='light', hair='spiked', hair_color='black', eyes='blue',
                              torso=('longsleeve_male', 'blue'), legs=('pants_male', 'navy'), feet=('boots_male', 'brown'),
                              held='sword_longsword'),
    'Guild Receptionist': dict(sex='f', skin='light', hair='ponytail', hair_color='chestnut', eyes='green',
                               torso=('blouse_female', 'white'), legs=('skirt_female', 'maroon'), feet=('shoes_female', 'black')),
    'Elf Mage': dict(sex='f', skin='light', ears='elven', hair='long', hair_color='platinum', eyes='purple',
                     torso=('robe_female', 'purple'), feet=('shoes_female', 'brown'), hat=('hat_wizard', 'purple'),
                     held=['staff_simple', 'staff_simple_bg']),
    'Cat-girl Adventurer': dict(sex='f', skin='amber', ears='cat', tail='cat', tail_color='amber', hair='bob', hair_color='ginger', eyes='yellow',
                                torso=('leather_female', 'brown'), legs=('shorts_female', 'brown'), feet=('boots_female', 'brown'),
                                held='sword_dagger'),
    'Female Knight': dict(sex='f', skin='light', hair='ponytail', hair_color='blonde', eyes='blue',
                          torso=('plate_female', 'steel'), legs=('legplate_female', 'steel'), feet=('footplate_female', 'steel'),
                          shoulders=('pauldrons_female', 'steel'), held='sword_longsword'),
    'Maid': dict(sex='f', skin='light', hair='bangs', hair_color='black', eyes='brown',
                 torso=[('blouse_female', 'black'), ('apron_full_female', 'white')], legs=('skirt_female', 'black'),
                 feet=('shoes_female', 'black'), hat=('hat_headband', 'white')),
}


def render(kit, chars, scale=3):
    cols = 5 + 8
    sheet = np.zeros((len(chars) * CELL, cols * CELL, 4), np.uint8)
    sheet[..., :3] = (40, 44, 52)
    sheet[..., 3] = 255
    for r, c in enumerate(chars):
        cells = [kit.frame(c, 'idle', d, 0) for d in ('down', 'right', 'up', 'left')]
        cells.append(kit.frame(c, 'idle', 'down', 0, blink=True))
        cells += [kit.frame(c, 'walk', 'down', k) for k in range(1, 9)]
        for i, cell in enumerate(cells):
            m = cell[..., 3] > 0
            sheet[r * CELL:(r + 1) * CELL, i * CELL:(i + 1) * CELL][m] = cell[m]
    im = to_image(sheet)
    return im.resize((im.width * scale, im.height * scale), Image.NEAREST)


if __name__ == '__main__':
    out = sys.argv[1]
    scale = int(sys.argv[2]) if len(sys.argv) > 2 else 3
    render(Kit(), list(CAST.values()), scale).save(out)
    print('wrote', out)
