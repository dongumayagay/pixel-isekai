#!/usr/bin/env python3
"""Render the cast on the 64x128 base: python3 -I tools/charkit64/roster64.py [scale]

Writes .context/wip/base64_cast.png (idle 4 dirs + idle frames + walk frames per character)
and .context/wip/anim/base64_{idle,walk,run}.gif.
"""
import os, sys
import numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from kit64 import Kit64, to_image
from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
WIP = os.path.join(ROOT, '.context', 'wip')

CAST = {
    'Reincarnated Hero': dict(skin='light', hair='hair_long', hair_color='black', eyes='blue', wear=[('underwear', 'navy')]),
    'Guild Receptionist': dict(skin='light', hair='hair_long', hair_color='chestnut', eyes='green', wear=[('underwear', 'white')]),
    'Elf Mage': dict(skin='elf_pale', hair='hair_long', hair_color='platinum', eyes='purple', wear=[('underwear', 'lavender')]),
    'Cat-girl Adventurer': dict(skin='amber', hair='hair_long', hair_color='ginger', eyes='yellow', wear=[('underwear', 'brown')]),
    'Female Knight': dict(skin='light', hair='hair_long', hair_color='blonde', eyes='blue', wear=[('underwear', 'black')]),
    'Maid': dict(skin='fair', hair='hair_long', hair_color='black', eyes='brown', wear=[('underwear', 'pink')]),
}
BG = (40, 44, 52)


def grid(cells, cols, W, H):
    rows = (len(cells) + cols - 1) // cols
    sheet = np.zeros((rows * H, cols * W, 4), np.uint8)
    sheet[..., :3] = BG; sheet[..., 3] = 255
    for i, cell in enumerate(cells):
        r, c = divmod(i, cols)
        m = cell[..., 3] > 0
        sheet[r * H:(r + 1) * H, c * W:(c + 1) * W][m] = cell[m]
    return sheet


def render(kit, chars, scale=2):
    W, H = kit.W, kit.H
    cells = []
    for c in chars:
        cells += [kit.frame(c, 'idle', d, 0) for d in ('down', 'right', 'up', 'left')]
        cells += [kit.frame(c, 'walk', 'down', k) for k in range(10)]
        cells += [kit.frame(c, 'run', 'right', k) for k in range(8)]
    im = to_image(grid(cells, 22, W, H))
    return im.resize((im.width * scale, im.height * scale), Image.NEAREST)


def gif(kit, chars, anim, path, scale=2, ms=120):
    W, H = kit.W, kit.H
    n = kit.meta['anims'][anim]['cols']
    frames = []
    for k in range(n):
        cells = [kit.frame(c, anim, d, k) for c in chars for d in ('down', 'right', 'up', 'left')]
        im = to_image(grid(cells, 4 * len(chars), W, H)).convert('RGB')
        frames.append(im.resize((im.width * scale, im.height * scale), Image.NEAREST))
    frames[0].save(path, save_all=True, append_images=frames[1:], duration=ms, loop=0)


if __name__ == '__main__':
    scale = int(sys.argv[1]) if len(sys.argv) > 1 else 2
    kit = Kit64()
    chars = list(CAST.values())
    os.makedirs(os.path.join(WIP, 'anim'), exist_ok=True)
    render(kit, chars, scale).save(os.path.join(WIP, 'base64_cast.png'))
    for anim, ms in (('idle', 140), ('walk', 90), ('run', 70)):
        gif(kit, chars[:3], anim, os.path.join(WIP, 'anim', f'base64_{anim}.gif'), 3, ms)
    print('wrote', WIP)
