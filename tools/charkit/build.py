#!/usr/bin/env python3
"""Build the character kit from a Universal LPC Spritesheet Character Generator checkout.

Usage: python3 -I tools/charkit/build.py <ulpc checkout> [out dir = assets/charkit]

Every part sheet is stored as an indexed PNG: index 0 is transparent, 1..6 are
the steps of the material's base ramp, 7.. are the sheet's few off-ramp colours.
A character in any skin/hair/cloth colour is then one palette lookup, so one
sheet per part serves every colour. kit.json lists parts, z order, palettes and
the sheet files; CREDITS.csv and LICENSE.md carry the attribution
(CC-BY-SA 3.0 / GPL 3.0 / OGA-BY 3.0).
"""
import csv, json, os, shutil, sys
import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from manifest import MANIFEST, MATERIAL_BASE, ANIMS

CELL = 64
TOL = 1  # per-channel tolerance when matching a ramp colour
SNAP = 16  # off-ramp colours this close to a ramp step are treated as that step (antialiasing, old palettes)
GREY = 8  # a sheet whose every colour has channel spread <= GREY is a legacy greyscale ramp


def hexrgb(h):
    h = h.lstrip('#')
    return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))


def load_palettes(src):
    out = {}
    for mat in MATERIAL_BASE:
        p = json.load(open(os.path.join(src, 'palette_definitions', mat, f'{mat}_ulpc.json')))
        out[mat] = {name: [hexrgb(c) for c in ramp] for name, ramp in p.items()}
    return out


def ramp_coverage(rgb, opaque, ramp):
    """Fraction of opaque pixels within TOL of some step of ramp; also the step index map."""
    ramp = np.array(ramp, np.int16)
    dist = np.abs(rgb[..., None, :] - ramp[None, None]).max(-1)  # h, w, 6
    step = dist.argmin(-1)
    on = (dist.min(-1) <= TOL) & opaque
    return on.sum() / max(1, opaque.sum()), step, on


def sheet_candidates(d, anim):
    """All files that could be the <anim> sheet of a part dir: variant files first, then the flat one."""
    out = []
    sub = os.path.join(d, anim)
    if os.path.isdir(sub):
        out += [os.path.join(sub, n) for n in sorted(os.listdir(sub)) if n.endswith('.png') and not n.startswith('_')]
    flat = os.path.join(d, anim + '.png')
    if os.path.exists(flat):
        out.append(flat)
    return out


def pick_sheet(d, anim, mat, palettes):
    """The sheet drawn in the material's base ramp if there is one, else the best-covered variant."""
    cands = sheet_candidates(d, anim)
    if not cands:
        return None, None
    if mat is None:
        return cands[0], None
    base = MATERIAL_BASE[mat]
    for c in cands:
        if os.path.basename(c) == base + '.png':
            return c, base
    best = (0, cands[0], base)
    for c in cands:
        a = np.array(Image.open(c).convert('RGBA'))
        rgb, op = a[..., :3].astype(np.int16), a[..., 3] >= 128
        for name, ramp in palettes[mat].items():
            cov = ramp_coverage(rgb, op, ramp)[0]
            if cov > best[0]:
                best = (cov, c, name)
    return best[1], best[2]


def index_sheet(path, ramp):
    """-> (uint8 index map, extras[(r,g,b)], coverage).

    Pixels within TOL of a ramp step take that step. Colours within SNAP of a
    step are snapped to it. A sheet drawn entirely in greys (a legacy sheet
    predating the ramp palettes) is mapped onto the ramp by luminance. Whatever
    is left (buckles, ribbons, eyes) is kept as a fixed extra colour.
    """
    a = np.array(Image.open(path).convert('RGBA'))
    rgb, op = a[..., :3].astype(np.int16), a[..., 3] >= 128
    idx = np.zeros(a.shape[:2], np.uint8)
    if ramp is None or not op.any():
        on = np.zeros_like(op)
        cov = 0.0
    else:
        r = np.array(ramp, np.int16)
        cols = np.unique(a[op][:, :3], axis=0).astype(np.int16)
        dist = np.abs(cols[:, None, :] - r[None]).max(-1)  # colours x 6
        step, dmin = dist.argmin(-1), dist.min(-1)
        exact = dmin <= TOL
        cov = 0.0
        greyscale = (cols.max(-1) - cols.min(-1) <= GREY).all()
        lum = lambda c: (c @ np.array([299, 587, 114])) / 1000
        on = np.zeros_like(op)
        for c, st, d, ex in zip(cols, step, dmin, exact):
            if d <= SNAP or greyscale:
                if d > SNAP:
                    st = int(np.abs(lum(r) - lum(c)).argmin())
                m = (rgb == c).all(-1) & op
                idx[m] = st + 1
                on |= m
                if ex:
                    cov += m.sum()
        cov /= op.sum()
    extras = []
    off = op & ~on
    if off.any():
        cols, inv = np.unique(a[off][:, :3], axis=0, return_inverse=True)
        if len(cols) > 249:
            sys.exit(f'{path}: {len(cols)} off-ramp colours')
        extras = [tuple(int(v) for v in c) for c in cols]
        idx[off] = 7 + inv.reshape(-1)
    return idx, extras, cov


def save_indexed(idx, ramp, extras, out):
    pal = [(0, 0, 0)] + list(ramp or [(0, 0, 0)] * 6) + extras
    im = Image.fromarray(idx, 'P')
    im.putpalette([v for c in pal for v in c] + [0] * (3 * (256 - len(pal))))
    im.info['transparency'] = bytes([0] + [255] * 255)
    im.save(out, optimize=True, transparency=bytes([0] + [255] * 255))


def main():
    src = sys.argv[1]
    out = sys.argv[2] if len(sys.argv) > 2 else os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))), 'assets', 'charkit')
    ss = os.path.join(src, 'spritesheets')
    palettes = load_palettes(src)
    if os.path.isdir(out):
        shutil.rmtree(out)
    os.makedirs(out)
    parts, used, report = [], [], []
    for id_, path, mat, z in MANIFEST:
        d = os.path.join(ss, path)
        if not os.path.isdir(d):
            sys.exit(f'missing {path}')
        halves = [('bg', os.path.join(d, 'bg'), -100), ('fg', os.path.join(d, 'fg'), 0)] if os.path.isdir(os.path.join(d, 'fg')) else [('', d, 0)]
        layers, extras_all = [], []
        for suffix, ld, dz in halves:
            anims = {}
            for anim in ANIMS:
                f, base = pick_sheet(ld, anim, mat, palettes)
                if f is None:
                    if anim == 'walk':
                        sys.exit(f'{path}: no walk sheet')
                    continue
                ramp = palettes[mat][base] if mat else None
                idx, extras, cov = index_sheet(f, ramp)
                # extras are shared per part so every sheet of it maps the same way
                merged = list(extras_all)
                remap = {}
                for i, c in enumerate(extras):
                    if c not in merged:
                        merged.append(c)
                    remap[7 + i] = 7 + merged.index(c)
                extras_all = merged
                if remap:
                    lut = np.arange(256, dtype=np.uint8)
                    for k, v in remap.items():
                        lut[k] = v
                    idx = lut[idx]
                name = f'{id_}/{suffix}_{anim}.png' if suffix else f'{id_}/{anim}.png'
                os.makedirs(os.path.join(out, id_), exist_ok=True)
                save_indexed(idx, palettes[mat][MATERIAL_BASE[mat]] if mat else None, extras_all, os.path.join(out, name))
                anims[anim] = {'file': name, 'cols': idx.shape[1] // CELL, 'rows': idx.shape[0] // CELL}
                used.append(os.path.relpath(f, ss))
                if mat and (base != MATERIAL_BASE[mat] or extras):
                    report.append(f'{name}: drawn in {mat}/{base}, exact ramp {cov:.0%}, extras {" ".join("#%02x%02x%02x" % c for c in extras)}')
            layers.append({'z': z + dz, 'anims': anims})
        parts.append({'id': id_, 'material': mat, 'z': z, 'source': path, 'layers': layers,
                      'extras': ['#%02x%02x%02x' % c for c in extras_all]})
    # fix up palettes written to indexed files: parts whose sheets were drawn in a non-base
    # variant were re-indexed against that variant's ramp, so recolouring from the base works.
    kit = {
        'cell': CELL,
        'rows': {'up': 0, 'left': 1, 'down': 2, 'right': 3},
        'anims': ANIMS,
        'materials': {m: {'base': b, 'palettes': {n: ['#%02x%02x%02x' % c for c in r] for n, r in palettes[m].items()}} for m, b in MATERIAL_BASE.items()},
        'parts': parts,
    }
    json.dump(kit, open(os.path.join(out, 'kit.json'), 'w'), indent=1)
    # Credits: the generator's rows for every sheet directory used.
    dirs = sorted({os.path.dirname(u) for u in used})
    with open(os.path.join(src, 'CREDITS.csv'), newline='') as f, open(os.path.join(out, 'CREDITS.csv'), 'w', newline='') as o:
        r, w = csv.reader(f), csv.writer(o)
        w.writerow(next(r))
        seen = set()
        for row in r:
            key = (os.path.dirname(row[0]), row[2], row[3])
            if any(row[0].startswith(d + '/') for d in dirs) and key not in seen:
                seen.add(key)
                w.writerow([os.path.dirname(row[0]) + '/'] + row[1:])
    with open(os.path.join(out, 'LICENSE.md'), 'w') as o:
        o.write('# Liberated Pixel Cup character kit\n\n'
                'The sheets in this directory are the walk and idle animations of parts from the Universal LPC\n'
                'Spritesheet Character Generator (https://github.com/LiberatedPixelCup/Universal-LPC-Spritesheet-Character-Generator),\n'
                'stored as palette-indexed PNGs (index 1-6 = the material\'s base colour ramp) so the game can recolour\n'
                'them. Authors and licences per sheet directory are in CREDITS.csv (CC-BY-SA 3.0, GPL 3.0 and/or\n'
                'OGA-BY 3.0). Derived character art in this game is shared under CC-BY-SA 3.0.\n')
    total = sum(os.path.getsize(os.path.join(dp, f)) for dp, _, fs in os.walk(out) for f in fs)
    print(f'{len(parts)} parts, {len(used)} sheets, {total // 1024} KiB -> {out}')
    for line in report:
        print(' ', line)


if __name__ == '__main__':
    main()
