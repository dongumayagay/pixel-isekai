"""Derive recolour ramps for the 64x128 kit from each material's reference ramp.

A ramp is 6 hex tones dark -> light. A derived ramp keeps the reference's lightness steps
(so the drawn shading survives) and swaps in the target hue / saturation / overall lightness.
"""
import colorsys


def hexrgb(h):
    return tuple(int(h[i:i + 2], 16) for i in (1, 3, 5))


def rgbhex(r, g, b):
    return '#%02x%02x%02x' % tuple(max(0, min(255, int(round(v)))) for v in (r, g, b))


def derive(ref, hue, sat, light, spread=1.0):
    """ref: 6 hex tones. hue 0-1, sat 0-1, light: target lightness of the base (index 4, the
    second-lightest tone, which the sprites use as the main fill). spread scales the dark->light range."""
    hls = [colorsys.rgb_to_hls(*(c / 255 for c in hexrgb(h))) for h in ref]
    base_l = hls[4][1]
    out = []
    for i, (_, l, s) in enumerate(hls):
        nl = light + (l - base_l) * spread
        nl = max(0.04, min(0.97, nl))
        # the outline tone keeps a little hue but goes much darker and duller
        ns = sat * (0.6 if i == 0 else 1.0)
        r, g, b = colorsys.hls_to_rgb(hue, nl, ns)
        out.append(rgbhex(r * 255, g * 255, b * 255))
    return out


SKIN = {  # hue, sat, light of the main fill
    'light': None,
    'fair': (0.07, 0.95, 0.90),
    'peach': (0.05, 0.90, 0.83),
    'tan': (0.07, 0.70, 0.72),
    'amber': (0.08, 0.80, 0.66),
    'olive': (0.10, 0.45, 0.62),
    'brown': (0.06, 0.55, 0.45),
    'dark': (0.05, 0.45, 0.30),
    'elf_pale': (0.09, 0.70, 0.87),
}
HAIR = {
    'blue': None,
    'black': (0.65, 0.25, 0.16),
    'brown': (0.07, 0.45, 0.30),
    'chestnut': (0.05, 0.55, 0.36),
    'auburn': (0.02, 0.60, 0.38),
    'ginger': (0.06, 0.85, 0.52),
    'red': (0.99, 0.75, 0.42),
    'blonde': (0.12, 0.80, 0.70),
    'gold': (0.11, 0.85, 0.60),
    'platinum': (0.13, 0.40, 0.78),
    'white': (0.60, 0.10, 0.84),
    'silver': (0.60, 0.08, 0.72),
    'pink': (0.92, 0.70, 0.72),
    'purple': (0.78, 0.50, 0.50),
    'lavender': (0.75, 0.55, 0.72),
    'navy': (0.64, 0.50, 0.32),
    'teal': (0.48, 0.50, 0.48),
    'green': (0.36, 0.45, 0.45),
}
CLOTH = {
    'white': None,
    'black': (0.65, 0.12, 0.20),
    'gray': (0.60, 0.05, 0.55),
    'red': (0.0, 0.75, 0.48),
    'maroon': (0.98, 0.60, 0.32),
    'pink': (0.93, 0.80, 0.78),
    'orange': (0.07, 0.90, 0.58),
    'yellow': (0.14, 0.90, 0.68),
    'green': (0.36, 0.50, 0.42),
    'forest': (0.38, 0.45, 0.30),
    'teal': (0.48, 0.55, 0.42),
    'blue': (0.60, 0.70, 0.52),
    'navy': (0.63, 0.55, 0.30),
    'sky': (0.57, 0.75, 0.74),
    'purple': (0.76, 0.55, 0.45),
    'lavender': (0.74, 0.60, 0.74),
    'brown': (0.07, 0.50, 0.36),
    'leather': (0.08, 0.55, 0.44),
    'tan': (0.10, 0.55, 0.68),
    'cream': (0.12, 0.70, 0.90),
}
EYES = {  # iris colour (extra index of '#639bff' in the eyes part)
    'blue': '#639bff', 'green': '#5fc66a', 'brown': '#8f563b', 'amber': '#e0a040', 'yellow': '#f2d14a',
    'purple': '#a06ce8', 'red': '#e05050', 'gray': '#9aa3b5', 'teal': '#4fc3c3', 'pink': '#f080b0',
}


SPREAD = {'body': 1.0, 'hair': 1.0, 'cloth': 0.55}


def all_palettes(material, ref):
    table = {'body': SKIN, 'hair': HAIR, 'cloth': CLOTH}[material]
    return {name: (list(ref) if spec is None else derive(ref, *spec, SPREAD[material])) for name, spec in table.items()}
