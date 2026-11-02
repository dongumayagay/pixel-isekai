//! Procedural trees, bushes, rocks and plants. Canopies are built from many
//! small leaf clumps, each lit as a tiny sphere, inside a larger lit volume.
//! That gives the clustered, hand-painted look of good pixel-art foliage.

use std::f32::consts::{PI, TAU};

use super::canvas::Img;
use super::shade;
use crate::palette::{self as pal, hex, ramp_at, Ramp, Rgb};
use crate::rng::{hash2, hash2f, Rng};

const SPRING: u8 = 0;
const SUMMER: u8 = 1;
const AUTUMN: u8 = 2;
const WINTER: u8 = 3;

/// `deciduous` kinds.
const OAK: u8 = 0;
const FRUIT: u8 = 1;
const BIRCH: u8 = 2;

/// `crop` kinds.
const TURNIP: u8 = 0;
const WHEAT: u8 = 1;
const CORN: u8 = 2;
const TOMATO: u8 = 3;
const PUMPKIN: u8 = 4;

/// Ground level of every crop sprite, which is also its pivot.
const CROP_BASE: i32 = 40;

const SNOW_LIT: Rgb = hex(0xf4f8fc);
const SNOW_SHADE: Rgb = hex(0xc8d4e4);
const BLUEBERRY: Ramp = &[0x1e1a4a, 0x343a8a, 0x5a6ac8, 0x9aaaf0];
const TOADSTOOL: Ramp = &[0x5c1a2a, 0x9c2c34, 0xd24c40, 0xf08a63];
const BROWN_CAP: Ramp = &[0x3a2420, 0x6b4430, 0x9a6a44, 0xc89a64];

type Vec3 = (f32, f32, f32);

/// Light from the upper left, slightly toward the viewer.
const LIGHT: Vec3 = (-0.5, -0.62, 0.6);

fn light_dir() -> Vec3 {
    let m = dot(LIGHT, LIGHT).sqrt();
    (LIGHT.0 / m, LIGHT.1 / m, LIGHT.2 / m)
}

#[inline]
fn dot(a: Vec3, b: Vec3) -> f32 {
    a.0 * b.0 + a.1 * b.1 + a.2 * b.2
}

/// Like `shade`, but skipping the ramp's darkest step, which would merge
/// with the ink outline.
fn shade_body(ramp: Ramp, v: f32) -> Rgb {
    shade(&ramp[1..], v)
}

fn leaf_ramp(season: u8) -> Ramp {
    match season {
        SPRING => pal::LEAF_SPRING,
        SUMMER => pal::LEAF_SUMMER,
        AUTUMN => pal::LEAF_AUTUMN,
        _ => pal::LEAF_WINTER,
    }
}

/// An ellipse of foliage volume; overlapping blobs form a canopy.
struct Blob {
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
}

struct Clump {
    x: f32,
    y: f32,
    r: f32,
    /// Global normal at the clump centre.
    n: Vec3,
    depth: f32,
}

/// Bounding box (x0, y0, x1, y1) of a set of blobs.
fn bounds(blobs: &[Blob]) -> (f32, f32, f32, f32) {
    blobs.iter().fold((f32::MAX, f32::MAX, f32::MIN, f32::MIN), |(x0, y0, x1, y1), b| {
        (x0.min(b.cx - b.rx), y0.min(b.cy - b.ry), x1.max(b.cx + b.rx), y1.max(b.cy + b.ry))
    })
}

/// Normal and height of the topmost blob at (x, y), each blob treated as a
/// hemisphere. Blobs higher up the sprite rank slightly nearer, so the
/// crown top overlaps the foliage below it.
fn volume_at(blobs: &[Blob], x: f32, y: f32) -> Option<(Vec3, f32)> {
    let mut best: Option<(Vec3, f32)> = None;
    for b in blobs {
        let nx = (x - b.cx) / b.rx;
        let ny = (y - b.cy) / b.ry;
        let d = nx * nx + ny * ny;
        if d < 1.0 {
            let z = (1.0 - d).sqrt();
            let height = z * b.rx.max(b.ry) - b.cy * 0.05;
            if best.is_none_or(|(_, h)| height > h) {
                best = Some(((nx, ny, z), height));
            }
        }
    }
    best
}

/// Scatter clumps on a jittered, staggered grid over the volume. Spots on
/// its steep rim are skipped so clumps don't bulge past the silhouette.
fn scatter_clumps(blobs: &[Blob], seed: u32, clump_r: f32) -> Vec<Clump> {
    let (x0, y0, x1, y1) = bounds(blobs);
    let mut clumps = Vec::new();
    let step = clump_r * 1.1;
    let mut gy = y0;
    let mut row = 0;
    while gy <= y1 {
        let mut gx = x0 + if row % 2 == 1 { step * 0.5 } else { 0.0 };
        while gx <= x1 {
            let h = hash2(gx as i32 * 7, gy as i32 * 13, seed);
            let jx = (h % 100) as f32 / 100.0 - 0.5;
            let jy = ((h >> 8) % 100) as f32 / 100.0 - 0.5;
            let (cx, cy) = (gx + jx * step * 0.6, gy + jy * step * 0.6);
            if let Some((n, height)) = volume_at(blobs, cx, cy)
                && n.0 * n.0 + n.1 * n.1 < 0.9
            {
                let r = clump_r * (0.8 + ((h >> 16) % 100) as f32 / 100.0 * 0.45);
                clumps.push(Clump { x: cx, y: cy, r, n, depth: height });
            }
            gx += step;
        }
        gy += step * 0.85;
        row += 1;
    }
    clumps
}

/// The frontmost clump covering a point, with the point's offset from the
/// clump centre as a unit-sphere normal (dx, dy, dz).
fn front_clump(clumps: &[Clump], fx: f32, fy: f32) -> Option<(&Clump, Vec3)> {
    let mut best = None;
    let mut best_key = f32::MIN;
    for c in clumps {
        let dx = (fx - c.x) / c.r;
        let dy = (fy - c.y) / c.r;
        let d = dx * dx + dy * dy;
        if d < 1.0 {
            let lz = (1.0 - d).sqrt();
            let key = c.depth + lz * c.r * 0.9;
            if key > best_key {
                best_key = key;
                best = Some((c, (dx, dy, lz)));
            }
        }
    }
    best
}

/// Paint a clumpy canopy into `img`. `extra(x, y, v, &mut colour)` can
/// decorate pixels by brightness `v` (fruit, blossoms, berries).
fn canopy(img: &mut Img, blobs: &[Blob], ramp: Ramp, seed: u32, clump_r: f32, mut extra: impl FnMut(i32, i32, f32, &mut Rgb)) {
    let l = light_dir();
    let (x0, y0, x1, y1) = bounds(blobs);
    let clumps = scatter_clumps(blobs, seed, clump_r);
    let margin = clump_r * 1.5;
    for y in (y0 - margin) as i32..=(y1 + margin) as i32 {
        for x in (x0 - margin) as i32..=(x1 + margin) as i32 {
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            let Some((c, local)) = front_clump(&clumps, fx, fy) else { continue };
            let (dx, dy, _) = local;
            // Underside of the canopy falls into shadow.
            let under = ((fy - (y0 + (y1 - y0) * 0.55)) / (y1 - y0)).max(0.0) * 0.5;
            let mut v = 0.42 + dot(c.n, l) * 0.32 + dot(local, l) * 0.24 - under;
            // Rim of each clump slightly darker for separation.
            if dx * dx + dy * dy > 0.7 && (dx > 0.0 || dy > 0.0) {
                v -= 0.1;
            }
            if hash2f(x, y, seed ^ 0x5) < 0.06 {
                v -= 0.08;
            }
            let mut col = shade_body(ramp, v);
            extra(x, y, v, &mut col);
            img.set(x, y, col);
        }
    }
}

/// Bark column shaded across its width, flaring at the root. Birch bark
/// gets its dark lenticel flecks.
fn trunk(img: &mut Img, cx: i32, top: i32, bottom: i32, width: i32, bark: Ramp, seed: u32) {
    let birch = bark == pal::BIRCH_BARK;
    for y in top..=bottom {
        let flare = (y - (bottom - 3)).max(0);
        let w = width + flare + if y == bottom { 2 } else { 0 };
        let x0 = cx - w / 2;
        for x in x0..x0 + w {
            let t = (x - x0) as f32 / (w - 1).max(1) as f32;
            let mut v = 0.78 - t * 0.55;
            if hash2(x, y / 3, seed).is_multiple_of(5) {
                v -= 0.18;
            }
            let mut col = shade(bark, v);
            if birch && hash2(x / 2, y, seed ^ 3).is_multiple_of(9) {
                col = pal::INK_SOFT;
            }
            img.set(x, y, col);
        }
    }
}

/// Two short limbs where the trunk disappears into the canopy.
fn branch_lines(img: &mut Img, cx: i32, y: i32, bark: Ramp) {
    img.line(cx, y, cx - 5, y - 6, ramp_at(bark, 2));
    img.line(cx + 1, y - 1, cx + 6, y - 7, ramp_at(bark, 1));
}

/// Recursive winter branching, with snow caught on the thicker forks.
fn bare_branches(img: &mut Img, rng: &mut Rng, (x, y): (f32, f32), ang: f32, len: f32, depth: u32, bark: Ramp) {
    if depth == 0 || len < 2.0 {
        return;
    }
    let x2 = x + ang.cos() * len;
    let y2 = y + ang.sin() * len;
    img.line(x as i32, y as i32, x2 as i32, y2 as i32, ramp_at(bark, if depth > 2 { 2 } else { 1 }));
    if depth > 3 {
        img.line(x as i32 + 1, y as i32, x2 as i32 + 1, y2 as i32, ramp_at(bark, 1));
    }
    if depth >= 3 {
        img.set(x2 as i32, y2 as i32 - 1, hex(0xe8eef6));
    }
    let forks = if depth > 1 { 2 } else { 1 };
    for i in 0..forks {
        let spread = rng.range_f(0.3, 0.7) * if i == 0 { -1.0 } else { 1.0 };
        let nl = len * rng.range_f(0.62, 0.78);
        bare_branches(img, rng, (x2, y2), ang + spread, nl, depth - 1, bark);
    }
}

/// Leafless crown: five main limbs fanning up from the trunk top.
fn bare_crown(img: &mut Img, top: (f32, f32), base_len: f32, bark: Ramp, seed: u32) {
    let mut r = Rng::new(seed as u64 ^ 9);
    for (a, l) in [(-1.57, base_len), (-2.2, base_len * 0.9), (-0.95, base_len * 0.9), (-2.7, base_len * 0.6), (-0.45, base_len * 0.6)] {
        let a2 = a + r.range_f(-0.15, 0.15);
        bare_branches(img, &mut r, top, a2, l, 6, bark);
    }
}

/// Foliage volume of a deciduous crown centred on `cx`.
fn crown_blobs(kind: u8, variant: u8, cx: f32, seed: u32) -> Vec<Blob> {
    if kind == BIRCH {
        return vec![
            Blob { cx, cy: 38.0, rx: 20.0, ry: 27.0 },
            Blob { cx: cx - 9.0, cy: 48.0, rx: 13.0, ry: 13.0 },
            Blob { cx: cx + 8.0, cy: 26.0, rx: 12.0, ry: 14.0 },
        ];
    }
    let mut rng = Rng::new(seed as u64);
    let mut wob = || rng.range_f(-3.0, 3.0);
    let mut blobs = vec![
        Blob { cx: cx + wob(), cy: 44.0, rx: 30.0 + (variant % 3) as f32 * 2.0, ry: 26.0 },
        Blob { cx: cx - 19.0 + wob(), cy: 54.0, rx: 18.0, ry: 15.0 },
        Blob { cx: cx + 19.0 + wob(), cy: 52.0, rx: 18.0, ry: 16.0 },
        Blob { cx: cx - 8.0 + wob(), cy: 22.0, rx: 18.0, ry: 15.0 },
    ];
    blobs.push(if variant.is_multiple_of(2) {
        Blob { cx: cx + 12.0, cy: 24.0, rx: 15.0, ry: 13.0 }
    } else {
        Blob { cx: cx + 2.0, cy: 16.0, rx: 14.0, ry: 11.0 }
    });
    blobs
}

/// Deciduous tree. kind: 0 oak, 1 fruit, 2 birch.
pub fn deciduous(kind: u8, variant: u8, season: u8) -> (Img, (i32, i32)) {
    let birch = kind == BIRCH;
    let (w, h) = if birch { (64, 108) } else { (92, 116) };
    let mut img = Img::new(w, h);
    let seed = 0x7eee_u32.wrapping_add(variant as u32 * 977 + kind as u32 * 131);
    let cx = w / 2;
    let bottom = h - 3;
    let bark = if birch { pal::BIRCH_BARK } else { pal::WOOD };
    let trunk_top = if birch { 40 } else { 54 };
    trunk(&mut img, cx, trunk_top, bottom, if birch { 7 } else { 11 }, bark, seed);

    if season == WINTER {
        bare_crown(&mut img, (cx as f32, trunk_top as f32 + 2.0), if birch { 17.0 } else { 22.0 }, bark, seed);
    } else {
        branch_lines(&mut img, cx, trunk_top + 8, bark);
        let ramp = if kind == FRUIT && season == SPRING { pal::BLOSSOM } else { leaf_ramp(season) };
        let blobs = crown_blobs(kind, variant, cx as f32, seed);
        let fruit_season = kind == FRUIT && (season == SUMMER || season == AUTUMN);
        canopy(&mut img, &blobs, ramp, seed, if birch { 4.6 } else { 5.6 }, |x, y, v, col| {
            if fruit_season && v > 0.25 && hash2(x / 5, y / 5, seed ^ 0xf).is_multiple_of(7) && x % 5 < 2 && y % 5 < 2 {
                *col = hex(pal::FRUIT_RED[if v > 0.6 { 3 } else { 2 }]);
            }
            if kind == OAK && season == AUTUMN && hash2(x, y, seed ^ 0x1).is_multiple_of(31) {
                *col = hex(pal::LEAF_AUTUMN[5]);
            }
        });
    }
    img.outline(pal::INK);
    (img, (cx, bottom))
}

/// Conifer of six stacked, drooping tiers.
pub fn pine(variant: u8, season: u8) -> (Img, (i32, i32)) {
    let (w, h) = (66, 120);
    let mut img = Img::new(w, h);
    let seed = 0x91ee_u32.wrapping_add(variant as u32 * 733);
    let cx = w / 2;
    let bottom = h - 3;
    trunk(&mut img, cx, bottom - 18, bottom, 8, pal::WOOD, seed);
    for k in 0..6 {
        let tb = bottom as f32 - 14.0 - k as f32 * 15.0;
        let tt = tb - 28.0 + k as f32;
        let hw = 29.0 - k as f32 * 4.4 + (variant % 2) as f32 * 2.0;
        for y in tt as i32..=tb as i32 {
            let t = (y as f32 - tt) / (tb - tt);
            let droop = (hash2(y, k, seed) % 4) as f32;
            let half = (t.powf(0.9) * hw + if t > 0.85 { droop - 1.0 } else { 0.0 }).max(0.5);
            for x in (cx as f32 - half) as i32..=(cx as f32 + half) as i32 {
                let u = (x as f32 - cx as f32) / half.max(1.0);
                let mut v = 0.68 - u * 0.28 - t * 0.22 + (1.0 - t) * 0.12;
                // Needle strokes.
                if hash2(x, y, seed ^ 7).is_multiple_of(3) || (x - cx + y) % 5 == 0 && t > 0.3 {
                    v += if (x + y) % 2 == 0 { 0.1 } else { -0.12 };
                }
                // Bottom edge of each tier falls into the shadow of the one above.
                if t > 0.92 {
                    v -= 0.15;
                }
                let mut col = shade_body(pal::PINE, v);
                if season == WINTER && (t < 0.18 + hash2f(x, k, seed) * 0.3) && u < 0.6 {
                    col = if u < -0.2 { SNOW_LIT } else { SNOW_SHADE };
                }
                img.set(x, y, col);
            }
        }
    }
    img.outline(pal::INK);
    (img, (cx, bottom))
}

/// Round shrub; `berries` dots it with red or blue fruit outside winter.
pub fn bush(variant: u8, season: u8, berries: bool) -> (Img, (i32, i32)) {
    let mut img = Img::new(42, 34);
    let seed = 0xb05e_u32.wrapping_add(variant as u32 * 71 + berries as u32);
    // Bushes hold their summer green through autumn.
    let ramp = if season == AUTUMN { pal::LEAF_SUMMER } else { leaf_ramp(season) };
    let blobs = [
        Blob { cx: 21.0, cy: 18.0, rx: 16.5, ry: 12.5 },
        Blob { cx: 11.0, cy: 22.0, rx: 10.0, ry: 9.0 },
        Blob { cx: 31.0, cy: 21.0, rx: 10.0, ry: 10.0 },
    ];
    let berry = if variant.is_multiple_of(2) { pal::FRUIT_RED } else { BLUEBERRY };
    canopy(&mut img, &blobs, ramp, seed, 4.0, |x, y, v, col| {
        if berries && season != WINTER && v > 0.2 && hash2(x / 2, y / 2, seed ^ 0xbe).is_multiple_of(9) {
            *col = hex(berry[if v > 0.55 { 3 } else { 2 }]);
        }
    });
    img.outline(pal::INK);
    (img, (21, 31))
}

/// Faceted stone: jittered outline, normals quantised into flat facets.
pub fn rock(size: u8, variant: u8, ore: Option<Ramp>, season: u8) -> (Img, (i32, i32)) {
    const SIDES: f32 = 7.0;
    let (w, h, rx, ry) = match size {
        0 => (28, 22, 12.0, 8.5),
        _ => (52, 42, 23.0, 17.0),
    };
    let mut img = Img::new(w, h);
    let seed = 0x40c4_u32.wrapping_add(variant as u32 * 1913 + size as u32);
    let cx = w as f32 / 2.0;
    let cy = h as f32 - ry - 1.5;
    let l = light_dir();
    for y in 0..h {
        for x in 0..w {
            let dx = (x as f32 + 0.5 - cx) / rx;
            let dy = (y as f32 + 0.5 - cy) / ry;
            let sector = ((dy.atan2(dx) + PI) / TAU * SIDES) as i32;
            let jit = 0.85 + (hash2(sector, 0, seed) % 100) as f32 / 100.0 * 0.25;
            let flat_bottom = if dy > 0.55 { 1.0 + (dy - 0.55) * 2.0 } else { 1.0 };
            let d = (dx * dx + dy * dy).sqrt() * flat_bottom;
            if d > jit {
                continue;
            }
            // Facets: the top plateau, then angular side faces.
            let normal = if d < 0.42 {
                (-0.15, -0.35, 0.92)
            } else {
                let a = (sector as f32 + 0.5) / SIDES * TAU - PI;
                (a.cos() * 0.75, a.sin() * 0.75, 0.55)
            };
            let mut v = 0.45 + dot(normal, l) * 0.42;
            if hash2(x, y, seed ^ 9).is_multiple_of(9) {
                v -= 0.06;
            }
            let mut col = shade_body(pal::ROCK, v);
            if let Some(ore) = ore
                && d < 0.75
                && hash2(x / 3, y / 3, seed ^ 0x0e).is_multiple_of(5)
            {
                col = hex(ore[if v > 0.5 { ore.len() - 1 } else { ore.len() - 2 }]);
            }
            if season == WINTER && dy < -0.35 && d < 0.9 {
                col = if v > 0.5 { SNOW_LIT } else { SNOW_SHADE };
            }
            img.set(x, y, col);
        }
    }
    // A crack wandering down and to the right.
    let mut x = (cx + (variant as f32 % 3.0) - 1.0) as i32;
    for y in (cy - ry * 0.4) as i32..(cy + ry * 0.3) as i32 {
        if img.alpha(x, y) != 0 {
            img.set(x, y, ramp_at(pal::ROCK, 1));
        }
        if hash2(x, y, seed).is_multiple_of(2) {
            x += 1;
        }
    }
    img.outline(pal::INK);
    (img, (w / 2, h - 2))
}

pub fn stump(season: u8) -> (Img, (i32, i32)) {
    let mut img = Img::new(30, 24);
    let wood = pal::WOOD;
    for y in 8..21 {
        let flare = (y - 17).max(0);
        for x in 5 - flare..25 + flare {
            let t = (x - 5) as f32 / 19.0;
            let mut v = 0.75 - t * 0.55;
            if hash2(x, y / 3, 41).is_multiple_of(5) {
                v -= 0.15;
            }
            img.set(x, y, shade(wood, v));
        }
    }
    // Cut face with growth rings, or a snow cap in winter.
    img.ellipse_with(15.0, 8.5, 10.0, 4.6, |im, x, y, nx, ny| {
        let r = (nx * nx + ny * ny).sqrt();
        let ring = ((r * 5.0) as i32) % 2 == 0;
        let c = if season == WINTER {
            if r > 0.8 { SNOW_SHADE } else { hex(0xeef3f8) }
        } else if r > 0.88 {
            ramp_at(wood, 2)
        } else if ring {
            hex(0xdcb47e)
        } else {
            hex(0xbc8e5c)
        };
        im.set(x, y, c);
    });
    img.outline(pal::INK);
    (img, (15, 21))
}

/// Unoutlined so it blends into the grass tiles beneath.
pub fn tall_grass(variant: u8, season: u8) -> (Img, (i32, i32)) {
    let mut img = Img::new(26, 26);
    let ramp = match season {
        SPRING => pal::GRASS_SPRING,
        SUMMER => pal::GRASS_SUMMER,
        _ => pal::GRASS_AUTUMN,
    };
    let seed = 0x6a55_u32 + variant as u32 * 31;
    for b in 0..11 {
        let bx = 3 + b * 2 + (hash2(b, 0, seed) % 2) as i32;
        let len = 10 + (hash2(b, 1, seed) % 12) as i32;
        let lean = (hash2(b, 2, seed) % 5) as f32 / 4.0 - 0.5;
        for k in 0..len {
            let t = k as f32 / len as f32;
            let x = bx + (lean * t * t * 8.0) as i32;
            let y = 25 - k;
            img.set(x, y, shade_body(ramp, 0.2 + t * 0.75));
            if k < len / 2 {
                img.set(x + 1, y, shade_body(ramp, 0.1 + t * 0.5));
            }
        }
    }
    if season == WINTER {
        // Snow settles on the topmost pixel of every blade.
        let src = img.clone();
        for x in 0..26 {
            for y in 0..26 {
                if src.alpha(x, y) != 0 && src.alpha(x, y - 1) == 0 {
                    img.set(x, y, hex(0xf2f6fa));
                }
            }
        }
    }
    (img, (13, 25))
}

pub fn reeds(variant: u8) -> (Img, (i32, i32)) {
    let mut img = Img::new(24, 40);
    let seed = 0x4eed_u32 + variant as u32;
    for b in 0..7 {
        let bx = 3 + b * 3;
        let len = 20 + (hash2(b, 0, seed) % 14) as i32;
        for k in 0..len {
            let t = k as f32 / len as f32;
            img.set(bx, 39 - k, shade_body(pal::GRASS_SUMMER, 0.15 + t * 0.7));
            if k < len * 2 / 3 {
                img.set(bx + 1, 39 - k, shade_body(pal::GRASS_SUMMER, 0.05 + t * 0.5));
            }
        }
        if b % 2 == 0 {
            // Cattail head.
            let top = 39 - len;
            img.rect(bx - 1, top - 6, 3, 7, hex(0x6a4030));
            img.vline(bx - 1, top - 6, top, hex(0x8c5a3c));
            img.set(bx, top - 7, hex(0x8a8a5a));
        }
    }
    img.outline(pal::INK);
    (img, (12, 39))
}

/// Even variants are spotted red toadstools, odd ones brown caps.
pub fn mushroom(variant: u8) -> (Img, (i32, i32)) {
    let mut img = Img::new(20, 20);
    let toadstool = variant.is_multiple_of(2);
    let cap = if toadstool { TOADSTOOL } else { BROWN_CAP };
    img.rect(8, 9, 4, 9, hex(0xe8dcc8));
    img.vline(11, 9, 17, hex(0xb8a890));
    img.ellipse_with(10.0, 8.0, 8.0, 5.5, |im, x, y, nx, ny| {
        let c = if ny > 0.55 { hex(0xd8c8b0) } else { shade(cap, 0.62 - nx * 0.25 - ny * 0.3) };
        im.set(x, y, c);
    });
    if toadstool {
        for (x, y) in [(6, 5), (12, 4), (9, 7), (14, 7)] {
            img.rect(x, y, 2, 2, hex(0xf8f0e0));
        }
    }
    // A smaller companion.
    img.rect(15, 14, 2, 4, hex(0xe8dcc8));
    img.ellipse_with(16.0, 13.5, 3.5, 2.5, |im, x, y, nx, _| im.set(x, y, shade(cap, 0.6 - nx * 0.3)));
    img.outline(pal::INK);
    (img, (10, 18))
}

/// Lily pad with its notch; even variants carry a pink bloom.
pub fn lily(variant: u8) -> (Img, (i32, i32)) {
    let mut img = Img::new(28, 18);
    img.ellipse_with(14.0, 9.0, 12.0, 7.0, |im, x, y, nx, ny| {
        if nx > 0.15 && ny.abs() < 0.15 + nx * 0.2 {
            return;
        }
        let vein = (nx * 6.0).abs() < 0.3 || (ny * 5.0).abs() < 0.25;
        let v = 0.62 - nx * 0.2 - ny * 0.3 - if vein { 0.12 } else { 0.0 };
        im.set(x, y, shade(&pal::LEAF_SUMMER[2..], v));
    });
    if variant.is_multiple_of(2) {
        let petals = [0xf8d0e0u32, 0xffffff, 0xe890b0];
        for (dx, dy, c) in [(0, -2, 1), (-2, 0, 0), (2, 0, 0), (-1, -1, 1), (1, -1, 1), (0, 0, 2), (-1, 1, 2), (1, 1, 2)] {
            img.set(10 + dx, 7 + dy, hex(petals[c]));
        }
        img.set(10, 6, hex(0xf6d860));
    }
    // Outlined in deep water blue rather than ink, so it sits on the pond.
    img.outline(hex(0x14303a));
    (img, (14, 11))
}

pub fn flower_patch(variant: u8) -> (Img, (i32, i32)) {
    let mut img = Img::new(28, 24);
    let seed = 0xf10e_u32 + variant as u32 * 5;
    let petals = pal::FLOWERS[variant as usize % pal::FLOWERS.len()];
    for k in 0..7 {
        let x = 4 + (hash2(k, 0, seed) % 20) as i32;
        let top = 4 + (hash2(k, 1, seed) % 10) as i32;
        img.vline(x, top + 2, 23, hex(pal::LEAF_SUMMER[3]));
        img.set(x - 1, top + 6 + k % 3, hex(pal::LEAF_SUMMER[4]));
        img.set(x + 1, top + 9, hex(pal::LEAF_SUMMER[2]));
        for (dx, dy, c) in [(-2, 0, 1), (2, 0, 1), (0, -2, 2), (0, 2, 0), (-1, -1, 2), (1, -1, 2), (-1, 1, 1), (1, 1, 0)] {
            img.set(x + dx, top + dy, hex(petals[c]));
        }
        img.set(x, top, hex(0xf6d860));
    }
    img.outline(pal::INK);
    (img, (14, 23))
}

/// Crop foliage colour: spring leaves without the darkest step.
fn crop_green(step: usize) -> Rgb {
    hex(pal::LEAF_SPRING[step + 1])
}

/// Crops: kind 0 turnip, 1 wheat, 2 corn, 3 tomato, 4 pumpkin, 5 strawberry.
/// Stage 0 is seeded soil, 1 a sprout, 2 growing and 3+ ripe.
pub fn crop(kind: u8, stage: u8) -> (Img, (i32, i32)) {
    let mut img = Img::new(32, 44);
    match stage {
        0 => seeded_soil(&mut img),
        1 => sprout(&mut img),
        _ => {
            let ripe = stage >= 3;
            match kind {
                WHEAT => wheat(&mut img, ripe),
                CORN => corn(&mut img, ripe),
                _ => leafy_crop(&mut img, kind, ripe),
            }
        }
    }
    img.outline(pal::INK);
    (img, (16, CROP_BASE))
}

fn seeded_soil(img: &mut Img) {
    let base = CROP_BASE;
    img.ellipse(16.0, base as f32 - 2.0, 8.0, 3.0, hex(pal::TILLED[3]));
    img.hline(10, 21, base - 4, hex(pal::TILLED[4]));
    for (x, y) in [(13, base - 4), (17, base - 3), (15, base - 2)] {
        img.rect(x, y, 2, 1, hex(0xc8a868));
    }
}

fn sprout(img: &mut Img) {
    let base = CROP_BASE;
    img.vline(16, base - 8, base, crop_green(2));
    img.vline(17, base - 6, base, crop_green(1));
    for (cx, rise) in [(12.5, 9.0), (20.0, 11.0)] {
        img.ellipse_with(cx, base as f32 - rise, 4.0, 2.2, |im, x, y, nx, _| im.set(x, y, crop_green(if nx < 0.0 { 4 } else { 3 })));
    }
}

fn wheat(img: &mut Img, ripe: bool) {
    let base = CROP_BASE;
    for (i, x) in [7, 11, 15, 19, 23].into_iter().enumerate() {
        let top = base - 24 - (i % 2) as i32 * 4;
        img.vline(x, top, base, if ripe { hex(0xa88a3c) } else { crop_green(2) });
        img.line(x, base - 8, x - 3, base - 14, crop_green(3));
        for k in 0..4 {
            let grain = if ripe { hex(if k == 0 { 0xf6e08a } else { 0xd8b450 }) } else { crop_green(4) };
            img.set(x - 1, top - 2 - k * 2, grain);
            img.set(x + 1, top - 3 - k * 2, grain);
        }
    }
}

fn corn(img: &mut Img, ripe: bool) {
    let base = CROP_BASE;
    img.rect(15, base - 34, 3, 34, crop_green(2));
    img.vline(15, base - 34, base, crop_green(3));
    for k in 0..5 {
        let y = base - 6 - k * 6;
        let dir = if k % 2 == 0 { -1 } else { 1 };
        img.line(16, y, 16 + dir * 11, y - 5, crop_green(3));
        img.line(16, y + 1, 16 + dir * 10, y - 3, crop_green(2));
        img.set(16 + dir * 12, y - 4, crop_green(4));
    }
    if ripe {
        img.rect(19, base - 22, 4, 10, hex(0xeec04a));
        img.vline(19, base - 22, base - 13, hex(0xfbe68a));
        img.line(19, base - 23, 22, base - 26, hex(0x9ab04a));
    }
}

/// A low leafy mound; ripe plants show one big fruit (turnip, pumpkin) or
/// several small ones (tomato, strawberry).
fn leafy_crop(img: &mut Img, kind: u8, ripe: bool) {
    let base = CROP_BASE;
    let (fruit, big): (&[u32], bool) = match kind {
        TURNIP => (&[0x8a4a8a, 0xc87ac0, 0xf0e8f0], true),
        TOMATO => (&[0x7a1a24, 0xd04040, 0xf08a6a], false),
        PUMPKIN => (&[0x8a4a1a, 0xe0802a, 0xf6b45a], true),
        _ => (&[0x8a1a2a, 0xe04050, 0xf8a0a0], false),
    };
    let blobs = [
        Blob { cx: 16.0, cy: base as f32 - 12.0, rx: 11.0, ry: 9.0 },
        Blob { cx: 11.0, cy: base as f32 - 7.0, rx: 6.0, ry: 5.0 },
        Blob { cx: 21.0, cy: base as f32 - 7.0, rx: 6.0, ry: 5.0 },
    ];
    canopy(img, &blobs, pal::LEAF_SPRING, 0xc0 + kind as u32, 3.4, |_, _, _, _| {});
    if !ripe {
        return;
    }
    if big {
        img.ellipse_with(16.0, base as f32 - 3.0, 8.0, 5.5, |im, x, y, nx, ny| {
            let rib = (nx * 4.0).abs().fract() < 0.15 && kind == PUMPKIN;
            im.set(x, y, hex(fruit[if rib { 0 } else if nx + ny < -0.4 { 2 } else if nx + ny > 0.6 { 0 } else { 1 }]));
        });
    } else {
        for (x, y) in [(9, base - 14), (20, base - 10), (14, base - 6), (22, base - 18), (12, base - 20)] {
            img.ellipse_with(x as f32, y as f32, 2.6, 2.6, |im, px, py, nx, ny| {
                im.set(px, py, hex(fruit[if nx + ny < -0.4 { 2 } else { 1 }]));
            });
        }
    }
}
