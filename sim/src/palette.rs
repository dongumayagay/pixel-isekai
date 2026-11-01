//! The game's master palette. Every ramp is hue-shifted: shadows lean
//! cool (blue/violet), highlights lean warm (yellow). There's no pure black.

pub type Rgb = [u8; 3];

pub const fn hex(c: u32) -> Rgb {
    [(c >> 16) as u8, (c >> 8) as u8, c as u8]
}

/// Darkest ink, used for outlines. A deep violet, not black.
pub const INK: Rgb = hex(0x1a1424);
pub const INK_SOFT: Rgb = hex(0x2a2238);

pub type Ramp = &'static [u32];

// Ground
pub const GRASS_SPRING: Ramp = &[0x223c3e, 0x2a5446, 0x376e4c, 0x4c8a52, 0x6ca65c, 0x98c46c, 0xc8de8c];
pub const GRASS_SUMMER: Ramp = &[0x23403c, 0x2c5a44, 0x3a7449, 0x4f8e4c, 0x6ca654, 0x95c060, 0xc2d880];
pub const GRASS_AUTUMN: Ramp = &[0x2a2a30, 0x433a32, 0x5f5034, 0x7e6a3a, 0x9c8442, 0xb89e52, 0xd2bc6e];
pub const SNOW: Ramp = &[0x4f5a7c, 0x6c7d9f, 0x8fa2c0, 0xb3c4dc, 0xd4e0ee, 0xedf3f9, 0xffffff];
pub const DIRT: Ramp = &[0x3a2a32, 0x553c38, 0x725440, 0x8e6c4c, 0xa8865c, 0xc2a272];
pub const TILLED: Ramp = &[0x2a1c26, 0x3e2a2c, 0x573c33, 0x70503d, 0x8b6648];
pub const SAND: Ramp = &[0x7c6656, 0x9f8566, 0xc0a476, 0xd8bf8a, 0xead6a4, 0xf6ead0];
pub const WET_SAND: Ramp = &[0x5d5452, 0x7a6c5e, 0x988769];
pub const ROCK: Ramp = &[0x29273a, 0x3f3e55, 0x58586f, 0x75768b, 0x9697a9, 0xbdbdc9];
pub const CLIFF_EARTH: Ramp = &[0x251d2b, 0x3a2c36, 0x553f41, 0x72554c, 0x8f6d58, 0xab8a6c];
pub const CLIFF_STONE: Ramp = &[0x222034, 0x34344a, 0x4a4b62, 0x63657c, 0x7f8297, 0xa0a3b4];

// Water (deep to shallow), plus foam
pub const WATER: Ramp = &[0x14203f, 0x182c52, 0x1d3d66, 0x234f78, 0x2b6488, 0x387c96, 0x4f96a3, 0x6fb0b0];
pub const FOAM: Ramp = &[0x9fd3cf, 0xcdebe4, 0xf1fbf6];

// Flora
pub const WOOD: Ramp = &[0x2d1c22, 0x4a2c29, 0x6b412f, 0x8c5a38, 0xab7645, 0xc79559];
pub const BIRCH_BARK: Ramp = &[0x5a5662, 0x8c8890, 0xc4c0c0, 0xe8e4dc];
pub const LEAF_SPRING: Ramp = &[0x16302f, 0x1d4637, 0x285f3d, 0x3c7c40, 0x5c9b44, 0x8bbb52, 0xbfd96c];
pub const LEAF_SUMMER: Ramp = &[0x142a2c, 0x1a3f33, 0x235638, 0x346f3a, 0x4f8b3c, 0x78a845, 0xa9c55a];
pub const LEAF_AUTUMN: Ramp = &[0x3a1c26, 0x632a25, 0x8c3d27, 0xb3572b, 0xd27b33, 0xe6a443, 0xf2cb65];
pub const LEAF_WINTER: Ramp = &[0x2a2c3e, 0x3d4256, 0x56607a, 0x7e8ca8, 0xb3c3da, 0xdfe9f3, 0xffffff];
pub const PINE: Ramp = &[0x0f2230, 0x153336, 0x1b463e, 0x245a47, 0x346f50, 0x4d875b, 0x6fa067];
pub const BLOSSOM: Ramp = &[0x5c2e4a, 0x8f466a, 0xbf6890, 0xdf91ae, 0xf2bccc, 0xfbe1e6];
pub const FRUIT_RED: Ramp = &[0x5c1a2a, 0x9c2c34, 0xd24c40, 0xf08a63];

// Flowers (petal ramps: shadow, base, highlight)
pub const FLOWERS: &[[u32; 3]] = &[
    [0xb0b4c8, 0xe8ecf2, 0xffffff], // daisy white
    [0xc08a2a, 0xeec04a, 0xfbe68a], // buttercup yellow
    [0x9a3e6a, 0xd46a92, 0xf4a6c0], // pink
    [0x3c4a9a, 0x5e7ad0, 0x9ab8f2], // cornflower blue
    [0x6a3a8a, 0x9a5ec4, 0xc69ae6], // violet
    [0xa8402e, 0xe0663e, 0xf6a072], // poppy orange
];

// Architecture
pub const ROOF_RED: Ramp = &[0x3a1824, 0x5e2430, 0x873538, 0xad4a40, 0xcc6a4e, 0xe39468];
pub const ROOF_BLUE: Ramp = &[0x181c38, 0x232f55, 0x2f4775, 0x40629a, 0x5b84bb, 0x84aad6];
pub const ROOF_THATCH: Ramp = &[0x3a2c22, 0x5c4630, 0x806340, 0xa6844f, 0xc6a462, 0xe0c784];
pub const ROOF_SLATE: Ramp = &[0x1e1e2c, 0x2c2e40, 0x3f4258, 0x575b72, 0x737891, 0x989eb2];
pub const ROOF_GREEN: Ramp = &[0x14282a, 0x1c3c36, 0x285444, 0x3a6e50, 0x548b5c, 0x7aa86c];
pub const PLASTER: Ramp = &[0x5c5064, 0x8a7c86, 0xb3a5a2, 0xd6cabb, 0xede3d0];
pub const STONE_WALL: Ramp = &[0x2e2c3c, 0x484760, 0x66667c, 0x86879a, 0xa8a9b8];
pub const TIMBER: Ramp = &[0x24161e, 0x3c2526, 0x573730, 0x744c3a];
pub const WINDOW_DARK: Ramp = &[0x141a30, 0x1e2a48, 0x2e4466];
pub const WINDOW_LIT: Ramp = &[0xb86a2c, 0xf0a640, 0xffd77a, 0xfff2c0];
pub const METAL: Ramp = &[0x2a2a3a, 0x4a4c60, 0x7a7e94, 0xb8bccc, 0xe8ecf4];
pub const GOLD: Ramp = &[0x6a3e1c, 0xa86a24, 0xdca23a, 0xf6d870, 0xfff4c0];

// People
/// Anime skin ramps, light to sun-kissed. Shadows lean pink-red and the
/// darkest step toward mauve, the way anime cel shading reads.
pub const SKIN: &[[u32; 4]] = &[
    [0x8a5260, 0xdc9e9c, 0xf8d4c8, 0xfff2e8], // porcelain
    [0x84504e, 0xd49282, 0xf4c6ac, 0xfee8d6], // fair
    [0x7c4a48, 0xcc8870, 0xf0ba9a, 0xfcdcc2], // peach
    [0x744440, 0xc07c62, 0xe8ac88, 0xf8d0b2], // warm
    [0x683e3a, 0xb07056, 0xdc9c76, 0xf0c29c], // light tan
];
/// Exotic skins for demonfolk: lavender, crimson, slate blue, ashen pale.
pub const DEMON_SKIN: &[[u32; 4]] = &[
    [0x4a3a6a, 0x7a64a0, 0xa890cc, 0xcab8e6],
    [0x5a1e2a, 0x9a3a44, 0xc8605e, 0xe68a80],
    [0x2a3448, 0x46587a, 0x6a80a6, 0x92a8c8],
    [0x5e4a5a, 0xb8a0b0, 0xe6d6de, 0xfaf0f4], // ashen pale (demon nobility)
];
pub const HAIR: &[[u32; 4]] = &[
    [0x1c1420, 0x2c2030, 0x443444, 0x5e4c5a], // black
    [0x2e1a1a, 0x4e2c22, 0x744430, 0x98623e], // brown
    [0x3c2414, 0x6a4020, 0x9e6430, 0xc88a48], // chestnut
    [0x6e4a1c, 0xb08030, 0xe0b850, 0xf6e08a], // blonde
    [0x4e1c1c, 0x8a2e24, 0xc04a2e, 0xe47a44], // ginger
    [0x4a4a5a, 0x7a7a8a, 0xb0b0bc, 0xe0e0e6], // silver
    [0x2a1c4a, 0x44307a, 0x6a50b0, 0x9a82d8], // violet (rare, isekai)
    [0x183a3a, 0x24605a, 0x3a8a7a, 0x66b8a0], // teal (rare)
    [0x5a1c3a, 0x92305c, 0xc85284, 0xec8cb0], // pink (rare)
];
pub const CLOTH: &[[u32; 4]] = &[
    [0x1e2a44, 0x2e4a6e, 0x46709a, 0x6c98c0], // blue
    [0x3a1a24, 0x642a30, 0x92403c, 0xbc6250], // red
    [0x1c3428, 0x2c5238, 0x44784a, 0x6c9e62], // green
    [0x3a2a1e, 0x5e4430, 0x86643e, 0xac8852], // brown
    [0x3c3440, 0x5e5462, 0x86808a, 0xb2acb0], // grey
    [0x40284a, 0x643e70, 0x8c5a98, 0xb486bc], // purple
    [0x4a3418, 0x7a5a24, 0xae8634, 0xd8b450], // ochre
    [0x5e5048, 0x9a8a78, 0xcab8a0, 0xeee0c8], // linen
    [0x2a2430, 0x3e3644, 0x56505c, 0x726c78], // charcoal
    [0x1c3a40, 0x28606a, 0x3c8a90, 0x62b2b0], // teal
];

/// Sample a ramp with a continuous value in [0, 1], with ordered dithering
/// between neighbouring steps so gradients stay crisp pixel art.
#[inline]
pub fn ramp_dither(ramp: Ramp, v: f32, x: i32, y: i32, dither: f32) -> Rgb {
    let n = ramp.len();
    let f = v.clamp(0.0, 0.9999) * n as f32;
    let i = f as usize;
    let frac = f - i as f32;
    let threshold = bayer4(x, y);
    // Dither only in a narrow band around step boundaries.
    let t = 0.5 + (frac - 0.5) * (1.0 / dither.max(0.01));
    let idx = if t > threshold && i + 1 < n { i + 1 } else { i };
    hex(ramp[idx.min(n - 1)])
}

#[inline]
pub fn ramp_at(ramp: Ramp, i: i32) -> Rgb {
    hex(ramp[i.clamp(0, ramp.len() as i32 - 1) as usize])
}

#[inline]
pub fn bayer4(x: i32, y: i32) -> f32 {
    const M: [u8; 16] = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];
    (M[((y & 3) * 4 + (x & 3)) as usize] as f32 + 0.5) / 16.0
}

pub fn mix(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    [
        (a[0] as f32 + (b[0] as f32 - a[0] as f32) * t) as u8,
        (a[1] as f32 + (b[1] as f32 - a[1] as f32) * t) as u8,
        (a[2] as f32 + (b[2] as f32 - a[2] as f32) * t) as u8,
    ]
}
