//! Procedural trees, bushes, rocks and plants. Canopies are built from many
//! small leaf clumps, each lit as a tiny sphere, inside a larger lit volume.
//! That gives the clustered, hand-painted look of good pixel-art foliage.

use super::canvas::Img;
use crate::palette::{self as pal, hex, ramp_at, Ramp, Rgb};
use crate::rng::{hash2, hash2f, Rng};

/// Light from the upper left, slightly toward the viewer.
const L: (f32, f32, f32) = (-0.5, -0.62, 0.6);

fn lnorm() -> (f32, f32, f32) {
    let m = (L.0 * L.0 + L.1 * L.1 + L.2 * L.2).sqrt();
    (L.0 / m, L.1 / m, L.2 / m)
}

#[inline]
pub fn shade(ramp: Ramp, v: f32) -> Rgb {
    let n = ramp.len() as f32;
    ramp_at(ramp, (v.clamp(0.0, 0.999) * n) as i32)
}

pub struct Blob {
    pub cx: f32,
    pub cy: f32,
    pub rx: f32,
    pub ry: f32,
}

struct Clump {
    x: f32,
    y: f32,
    r: f32,
    /// Global normal at the clump centre.
    n: (f32, f32, f32),
    depth: f32,
}

/// Paint a clumpy canopy into `img`. `extra(x, y, &mut colour)` can decorate
/// lit pixels (fruit, blossoms, snow).
pub fn canopy(img: &mut Img, blobs: &[Blob], ramp: Ramp, seed: u32, clump_r: f32, mut extra: impl FnMut(i32, i32, f32, &mut Rgb)) {
    let l = lnorm();
    let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for b in blobs {
        x0 = x0.min(b.cx - b.rx);
        y0 = y0.min(b.cy - b.ry);
        x1 = x1.max(b.cx + b.rx);
        y1 = y1.max(b.cy + b.ry);
    }
    let global = |x: f32, y: f32| -> Option<((f32, f32, f32), f32)> {
        let mut best: Option<((f32, f32, f32), f32)> = None;
        for b in blobs {
            let nx = (x - b.cx) / b.rx;
            let ny = (y - b.cy) / b.ry;
            let d = nx * nx + ny * ny;
            if d < 1.0 {
                let z = (1.0 - d).sqrt();
                let height = z * b.rx.max(b.ry) - b.cy * 0.05;
                if best.map_or(true, |(_, h)| height > h) {
                    best = Some(((nx, ny, z), height));
                }
            }
        }
        best
    };
    // Scatter clumps over the volume.
    let mut clumps: Vec<Clump> = Vec::new();
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
            if let Some((n, height)) = global(cx, cy) {
                if n.0 * n.0 + n.1 * n.1 < 0.9 {
                    let r = clump_r * (0.8 + ((h >> 16) % 100) as f32 / 100.0 * 0.45);
                    clumps.push(Clump { x: cx, y: cy, r, n, depth: height });
                }
            }
            gx += step;
        }
        gy += step * 0.85;
        row += 1;
    }
    let xi0 = (x0 - clump_r * 1.5) as i32;
    let yi0 = (y0 - clump_r * 1.5) as i32;
    let xi1 = (x1 + clump_r * 1.5) as i32;
    let yi1 = (y1 + clump_r * 1.5) as i32;
    for y in yi0..=yi1 {
        for x in xi0..=xi1 {
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            let mut best: Option<(&Clump, f32, f32, f32)> = None;
            let mut best_key = f32::MIN;
            for c in &clumps {
                let dx = (fx - c.x) / c.r;
                let dy = (fy - c.y) / c.r;
                let d = dx * dx + dy * dy;
                if d < 1.0 {
                    let lz = (1.0 - d).sqrt();
                    let key = c.depth + lz * c.r * 0.9;
                    if key > best_key {
                        best_key = key;
                        best = Some((c, dx, dy, lz));
                    }
                }
            }
            let Some((c, dx, dy, lz)) = best else { continue };
            let g = c.n;
            let gdot = g.0 * l.0 + g.1 * l.1 + g.2 * l.2;
            let ldot = dx * l.0 + dy * l.1 + lz * l.2;
            // Underside of the canopy falls into shadow.
            let under = ((fy - (y0 + (y1 - y0) * 0.55)) / (y1 - y0)).max(0.0) * 0.5;
            let mut v = 0.42 + gdot * 0.32 + ldot * 0.24 - under;
            // Rim of each clump slightly darker for separation.
            if dx * dx + dy * dy > 0.7 && (dx > 0.0 || dy > 0.0) {
                v -= 0.1;
            }
            let sp = hash2f(x, y, seed ^ 0x5);
            if sp < 0.06 {
                v -= 0.08;
            }
            let mut col = shade(&ramp[1..], v);
            extra(x, y, v, &mut col);
            img.set(x, y, col);
        }
    }
}

fn trunk(img: &mut Img, cx: i32, top: i32, bottom: i32, width: i32, ramp: Ramp, seed: u32, birch: bool) {
    for y in top..=bottom {
        let flare = (y - (bottom - 3)).max(0);
        let w = width + flare * 2 / 2 + if y == bottom { 2 } else { 0 };
        let x0 = cx - w / 2;
        for x in x0..x0 + w {
            let t = (x - x0) as f32 / (w - 1).max(1) as f32;
            let mut v = 0.78 - t * 0.55;
            if hash2(x, y / 3, seed) % 5 == 0 {
                v -= 0.18;
            }
            let mut col = shade(ramp, v);
            if birch && hash2(x / 2, y, seed ^ 3) % 9 == 0 {
                col = pal::INK_SOFT;
            }
            img.set(x, y, col);
        }
    }
}

fn branch_lines(img: &mut Img, cx: i32, y: i32, ramp: Ramp) {
    img.line(cx, y, cx - 5, y - 6, ramp_at(ramp, 2));
    img.line(cx + 1, y - 1, cx + 6, y - 7, ramp_at(ramp, 1));
}

pub fn leaf_ramp(season: u8) -> Ramp {
    match season {
        0 => pal::LEAF_SPRING,
        1 => pal::LEAF_SUMMER,
        2 => pal::LEAF_AUTUMN,
        _ => pal::LEAF_WINTER,
    }
}

fn bare_branches(img: &mut Img, rng: &mut Rng, x: f32, y: f32, ang: f32, len: f32, depth: u32, ramp: Ramp, snow: bool) {
    if depth == 0 || len < 2.0 {
        return;
    }
    let x2 = x + ang.cos() * len;
    let y2 = y + ang.sin() * len;
    let col = ramp_at(ramp, if depth > 2 { 2 } else { 1 });
    img.line(x as i32, y as i32, x2 as i32, y2 as i32, col);
    if depth > 3 {
        img.line(x as i32 + 1, y as i32, x2 as i32 + 1, y2 as i32, ramp_at(ramp, 1));
    }
    if snow && depth >= 3 {
        img.set(x2 as i32, y2 as i32 - 1, hex(0xe8eef6));
    }
    let n = if depth > 1 { 2 } else { 1 };
    for i in 0..n {
        let spread = rng.range_f(0.3, 0.7) * if i == 0 { -1.0 } else { 1.0 };
        let nl = len * rng.range_f(0.62, 0.78);
        bare_branches(img, rng, x2, y2, ang + spread, nl, depth - 1, ramp, snow);
    }
}

/// Deciduous tree. kind: 0 oak, 1 fruit, 2 birch.
pub fn deciduous(kind: u8, variant: u8, season: u8) -> (Img, (i32, i32)) {
    let (w, h) = if kind == 2 { (64, 108) } else { (92, 116) };
    let mut img = Img::new(w, h);
    let seed = 0x7eee_u32.wrapping_add(variant as u32 * 977 + kind as u32 * 131);
    let mut rng = Rng::new(seed as u64);
    let cx = w / 2;
    let bottom = h - 3;
    let bark = if kind == 2 { pal::BIRCH_BARK } else { pal::WOOD };
    let trunk_w = if kind == 2 { 7 } else { 11 };
    let trunk_top = if kind == 2 { 40 } else { 54 };

    if season == 3 {
        trunk(&mut img, cx, trunk_top, bottom, trunk_w, bark, seed, kind == 2);
        let mut r = Rng::new(seed as u64 ^ 9);
        let top = (cx as f32, trunk_top as f32 + 2.0);
        let base_len = if kind == 2 { 17.0 } else { 22.0 };
        for (a, l) in [(-1.57, base_len), (-2.2, base_len * 0.9), (-0.95, base_len * 0.9), (-2.7, base_len * 0.6), (-0.45, base_len * 0.6)] {
            let a2 = a + r.range_f(-0.15, 0.15);
            bare_branches(&mut img, &mut r, top.0, top.1, a2, l, 6, bark, true);
        }
        img.outline(pal::INK);
        return (img, (cx, bottom));
    }

    trunk(&mut img, cx, trunk_top, bottom, trunk_w, bark, seed, kind == 2);
    branch_lines(&mut img, cx, trunk_top + 8, bark);

    let ramp = if kind == 1 && season == 0 { pal::BLOSSOM } else { leaf_ramp(season) };
    let mut blobs = Vec::new();
    if kind == 2 {
        blobs.push(Blob { cx: cx as f32, cy: 38.0, rx: 20.0, ry: 27.0 });
        blobs.push(Blob { cx: cx as f32 - 9.0, cy: 48.0, rx: 13.0, ry: 13.0 });
        blobs.push(Blob { cx: cx as f32 + 8.0, cy: 26.0, rx: 12.0, ry: 14.0 });
    } else {
        let wob = |r: &mut Rng| r.range_f(-3.0, 3.0);
        blobs.push(Blob { cx: cx as f32 + wob(&mut rng), cy: 44.0, rx: 30.0 + (variant % 3) as f32 * 2.0, ry: 26.0 });
        blobs.push(Blob { cx: cx as f32 - 19.0 + wob(&mut rng), cy: 54.0, rx: 18.0, ry: 15.0 });
        blobs.push(Blob { cx: cx as f32 + 19.0 + wob(&mut rng), cy: 52.0, rx: 18.0, ry: 16.0 });
        blobs.push(Blob { cx: cx as f32 - 8.0 + wob(&mut rng), cy: 22.0, rx: 18.0, ry: 15.0 });
        if variant % 2 == 0 {
            blobs.push(Blob { cx: cx as f32 + 12.0, cy: 24.0, rx: 15.0, ry: 13.0 });
        } else {
            blobs.push(Blob { cx: cx as f32 + 2.0, cy: 16.0, rx: 14.0, ry: 11.0 });
        }
    }
    let fruit_season = kind == 1 && (season == 1 || season == 2);
    canopy(&mut img, &blobs, ramp, seed, if kind == 2 { 4.6 } else { 5.6 }, |x, y, v, col| {
        if fruit_season && v > 0.25 && hash2(x / 5, y / 5, seed ^ 0xf) % 7 == 0 && x % 5 < 2 && y % 5 < 2 {
            *col = hex(pal::FRUIT_RED[if v > 0.6 { 3 } else { 2 }]);
        }
        if kind == 0 && season == 2 && hash2(x, y, seed ^ 0x1) % 31 == 0 {
            *col = hex(pal::LEAF_AUTUMN[5]);
        }
    });
    img.outline(pal::INK);
    (img, (cx, bottom))
}

pub fn pine(variant: u8, season: u8) -> (Img, (i32, i32)) {
    let (w, h) = (66, 120);
    let mut img = Img::new(w, h);
    let seed = 0x91ee_u32.wrapping_add(variant as u32 * 733);
    let cx = w / 2;
    let bottom = h - 3;
    trunk(&mut img, cx, bottom - 18, bottom, 8, pal::WOOD, seed, false);
    let ramp = pal::PINE;
    let tiers = 6;
    for k in 0..tiers {
        let tb = bottom as f32 - 14.0 - k as f32 * 15.0;
        let tt = tb - 28.0 + k as f32 * 1.0;
        let hw = 29.0 - k as f32 * 4.4 + (variant % 2) as f32 * 2.0;
        for y in tt as i32..=tb as i32 {
            let t = (y as f32 - tt) / (tb - tt);
            let droop = (hash2(y, k, seed) % 4) as f32;
            let half = (t.powf(0.9) * hw + if t > 0.85 { droop - 1.0 } else { 0.0 }).max(0.5);
            for x in (cx as f32 - half) as i32..=(cx as f32 + half) as i32 {
                let u = (x as f32 - cx as f32) / half.max(1.0);
                let mut v = 0.68 - u * 0.28 - t * 0.22 + (1.0 - t) * 0.12;
                // Needle strokes.
                if hash2(x, y, seed ^ 7) % 3 == 0 || (x - cx + y) % 5 == 0 && t > 0.3 {
                    v += if (x + y) % 2 == 0 { 0.1 } else { -0.12 };
                }
                // Bottom edge of each tier falls into the shadow of the one above.
                if t > 0.92 {
                    v -= 0.15;
                }
                let mut col = shade(&ramp[1..], v);
                if season == 3 && (t < 0.18 + hash2f(x, k, seed) * 0.3) && u < 0.6 {
                    col = if u < -0.2 { hex(0xf4f8fc) } else { hex(0xc8d4e4) };
                }
                img.set(x, y, col);
            }
        }
    }
    img.outline(pal::INK);
    (img, (cx, bottom))
}

pub fn bush(variant: u8, season: u8, berries: bool) -> (Img, (i32, i32)) {
    let (w, h) = (42, 34);
    let mut img = Img::new(w, h);
    let seed = 0xb05e_u32.wrapping_add(variant as u32 * 71 + berries as u32);
    let ramp = if season == 3 { pal::LEAF_WINTER } else { leaf_ramp(season.min(1)) };
    let blobs = [
        Blob { cx: 21.0, cy: 18.0, rx: 16.5, ry: 12.5 },
        Blob { cx: 11.0, cy: 22.0, rx: 10.0, ry: 9.0 },
        Blob { cx: 31.0, cy: 21.0, rx: 10.0, ry: 10.0 },
    ];
    let berry = if variant % 2 == 0 { pal::FRUIT_RED } else { &[0x1e1a4a, 0x343a8a, 0x5a6ac8, 0x9aaaf0] };
    canopy(&mut img, &blobs, ramp, seed, 4.0, |x, y, v, col| {
        if berries && season != 3 && v > 0.2 && hash2(x / 2, y / 2, seed ^ 0xbe) % 9 == 0 {
            *col = hex(berry[if v > 0.55 { 3 } else { 2 }]);
        }
    });
    img.outline(pal::INK);
    (img, (21, 31))
}

/// Faceted stone: jittered outline, normals quantised into flat facets.
pub fn rock(size: u8, variant: u8, ore: Option<Ramp>, season: u8) -> (Img, (i32, i32)) {
    let (w, h, rx, ry) = match size {
        0 => (28, 22, 12.0, 8.5),
        _ => (52, 42, 23.0, 17.0),
    };
    let mut img = Img::new(w, h);
    let seed = 0x40c4_u32.wrapping_add(variant as u32 * 1913 + size as u32);
    let cx = w as f32 / 2.0;
    let cy = h as f32 - ry - 1.5;
    let l = lnorm();
    let ramp = pal::ROCK;
    for y in 0..h {
        for x in 0..w {
            let dx = (x as f32 + 0.5 - cx) / rx;
            let dy = (y as f32 + 0.5 - cy) / ry;
            let ang = dy.atan2(dx);
            let sector = ((ang + std::f32::consts::PI) / std::f32::consts::TAU * 7.0) as i32;
            let jit = 0.85 + (hash2(sector, 0, seed) % 100) as f32 / 100.0 * 0.25;
            let flat_bottom = if dy > 0.55 { 1.0 + (dy - 0.55) * 2.0 } else { 1.0 };
            let d = (dx * dx + dy * dy).sqrt() * flat_bottom;
            if d > jit {
                continue;
            }
            // Facets: the top plateau, then angular side faces.
            let (nx, ny, nz) = if d < 0.42 {
                (-0.15, -0.35, 0.92)
            } else {
                let a = (sector as f32 + 0.5) / 7.0 * std::f32::consts::TAU - std::f32::consts::PI;
                (a.cos() * 0.75, a.sin() * 0.75, 0.55)
            };
            let mut v = 0.45 + (nx * l.0 + ny * l.1 + nz * l.2) * 0.42;
            if hash2(x, y, seed ^ 9) % 9 == 0 {
                v -= 0.06;
            }
            let mut col = shade(&ramp[1..], v);
            if let Some(ore) = ore {
                if d < 0.75 && hash2(x / 3, y / 3, seed ^ 0x0e) % 5 == 0 {
                    col = hex(ore[if v > 0.5 { ore.len() - 1 } else { ore.len() - 2 }]);
                }
            }
            if season == 3 && dy < -0.35 && d < 0.9 {
                col = if v > 0.5 { hex(0xf4f8fc) } else { hex(0xc8d4e4) };
            }
            img.set(x, y, col);
        }
    }
    // A crack.
    let mut x = (cx + (variant as f32 % 3.0) - 1.0) as i32;
    for y in (cy - ry * 0.4) as i32..(cy + ry * 0.3) as i32 {
        if img.alpha(x, y) != 0 {
            img.set(x, y, ramp_at(ramp, 1));
        }
        if hash2(x, y, seed) % 2 == 0 {
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
        let flare = if y > 17 { y - 17 } else { 0 };
        for x in 5 - flare..25 + flare {
            let t = (x - 5) as f32 / 19.0;
            let mut v = 0.75 - t * 0.55;
            if hash2(x, y / 3, 41) % 5 == 0 {
                v -= 0.15;
            }
            img.set(x, y, shade(wood, v));
        }
    }
    img.ellipse_with(15.0, 8.5, 10.0, 4.6, |im, x, y, nx, ny| {
        let r = (nx * nx + ny * ny).sqrt();
        let ring = ((r * 5.0) as i32) % 2 == 0;
        let c = if season == 3 { if r > 0.8 { hex(0xc8d4e4) } else { hex(0xeef3f8) } } else if r > 0.88 { ramp_at(wood, 2) } else if ring { hex(0xdcb47e) } else { hex(0xbc8e5c) };
        im.set(x, y, c);
    });
    img.outline(pal::INK);
    (img, (15, 21))
}

pub fn tall_grass(variant: u8, season: u8) -> (Img, (i32, i32)) {
    let mut img = Img::new(26, 26);
    let ramp = match season {
        0 => pal::GRASS_SPRING,
        1 => pal::GRASS_SUMMER,
        _ => pal::GRASS_AUTUMN,
    };
    let seed = 0x6a55_u32 + variant as u32 * 31;
    for b in 0..11 {
        let bx = 3 + (b * 2) as i32 + (hash2(b, 0, seed) % 2) as i32;
        let len = 10 + (hash2(b, 1, seed) % 12) as i32;
        let lean = (hash2(b, 2, seed) % 5) as f32 / 4.0 - 0.5;
        for k in 0..len {
            let t = k as f32 / len as f32;
            let x = bx + (lean * t * t * 8.0) as i32;
            let y = 25 - k;
            img.set(x, y, shade(&ramp[1..], 0.2 + t * 0.75));
            if k < len / 2 {
                img.set(x + 1, y, shade(&ramp[1..], 0.1 + t * 0.5));
            }
        }
    }
    if season == 3 {
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
            img.set(bx, 39 - k, shade(&pal::GRASS_SUMMER[1..], 0.15 + t * 0.7));
            if k < len * 2 / 3 {
                img.set(bx + 1, 39 - k, shade(&pal::GRASS_SUMMER[1..], 0.05 + t * 0.5));
            }
        }
        if b % 2 == 0 {
            let top = 39 - len;
            img.rect(bx - 1, top - 6, 3, 7, hex(0x6a4030));
            img.vline(bx - 1, top - 6, top, hex(0x8c5a3c));
            img.set(bx, top - 7, hex(0x8a8a5a));
        }
    }
    img.outline(pal::INK);
    (img, (12, 39))
}

pub fn mushroom(variant: u8) -> (Img, (i32, i32)) {
    let mut img = Img::new(20, 20);
    let cap: Ramp = if variant % 2 == 0 { &[0x5c1a2a, 0x9c2c34, 0xd24c40, 0xf08a63] } else { &[0x3a2420, 0x6b4430, 0x9a6a44, 0xc89a64] };
    img.rect(8, 9, 4, 9, hex(0xe8dcc8));
    img.vline(11, 9, 17, hex(0xb8a890));
    img.ellipse_with(10.0, 8.0, 8.0, 5.5, |im, x, y, nx, ny| {
        if ny > 0.55 {
            im.set(x, y, hex(0xd8c8b0));
            return;
        }
        let v = 0.62 - nx * 0.25 - ny * 0.3;
        im.set(x, y, shade(cap, v));
    });
    if variant % 2 == 0 {
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
    if variant % 2 == 0 {
        let petals = [0xf8d0e0u32, 0xffffff, 0xe890b0];
        for (dx, dy, c) in [(0, -2, 1), (-2, 0, 0), (2, 0, 0), (-1, -1, 1), (1, -1, 1), (0, 0, 2), (-1, 1, 2), (1, 1, 2)] {
            img.set(10 + dx, 7 + dy, hex(petals[c]));
        }
        img.set(10, 6, hex(0xf6d860));
    }
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

/// Crops: kind 0 turnip, 1 wheat, 2 corn, 3 tomato, 4 pumpkin, 5 strawberry.
pub fn crop(kind: u8, stage: u8) -> (Img, (i32, i32)) {
    let mut img = Img::new(32, 44);
    let green = &pal::LEAF_SPRING[1..];
    let base = 40;
    match stage {
        0 => {
            img.ellipse(16.0, base as f32 - 2.0, 8.0, 3.0, hex(pal::TILLED[3]));
            img.hline(10, 21, base - 4, hex(pal::TILLED[4]));
            for (x, y) in [(13, base - 4), (17, base - 3), (15, base - 2)] {
                img.rect(x, y, 2, 1, hex(0xc8a868));
            }
        }
        1 => {
            img.vline(16, base - 8, base, hex(green[2]));
            img.vline(17, base - 6, base, hex(green[1]));
            img.ellipse_with(12.5, base as f32 - 9.0, 4.0, 2.2, |im, x, y, nx, _| im.set(x, y, hex(green[if nx < 0.0 { 4 } else { 3 }])));
            img.ellipse_with(20.0, base as f32 - 11.0, 4.0, 2.2, |im, x, y, nx, _| im.set(x, y, hex(green[if nx < 0.0 { 4 } else { 3 }])));
        }
        _ => {
            let ripe = stage >= 3;
            match kind {
                1 => {
                    for (i, x) in [7, 11, 15, 19, 23].iter().enumerate() {
                        let top = base - 24 - (i % 2) as i32 * 4;
                        img.vline(*x, top, base, if ripe { hex(0xa88a3c) } else { hex(green[2]) });
                        img.line(*x, base - 8, *x - 3, base - 14, hex(green[3]));
                        for k in 0..4 {
                            let c = if ripe { hex(if k == 0 { 0xf6e08a } else { 0xd8b450 }) } else { hex(green[4]) };
                            img.set(*x - 1, top - 2 - k * 2, c);
                            img.set(*x + 1, top - 3 - k * 2, c);
                        }
                    }
                }
                2 => {
                    img.rect(15, base - 34, 3, 34, hex(green[2]));
                    img.vline(15, base - 34, base, hex(green[3]));
                    for k in 0..5 {
                        let y = base - 6 - k * 6;
                        let dir = if k % 2 == 0 { -1 } else { 1 };
                        img.line(16, y, 16 + dir * 11, y - 5, hex(green[3]));
                        img.line(16, y + 1, 16 + dir * 10, y - 3, hex(green[2]));
                        img.set(16 + dir * 12, y - 4, hex(green[4]));
                    }
                    if ripe {
                        img.rect(19, base - 22, 4, 10, hex(0xeec04a));
                        img.vline(19, base - 22, base - 13, hex(0xfbe68a));
                        img.line(19, base - 23, 22, base - 26, hex(0x9ab04a));
                    }
                }
                _ => {
                    let (fruit, big): (&[u32], bool) = match kind {
                        0 => (&[0x8a4a8a, 0xc87ac0, 0xf0e8f0], true),
                        3 => (&[0x7a1a24, 0xd04040, 0xf08a6a], false),
                        4 => (&[0x8a4a1a, 0xe0802a, 0xf6b45a], true),
                        _ => (&[0x8a1a2a, 0xe04050, 0xf8a0a0], false),
                    };
                    let blobs = [
                        Blob { cx: 16.0, cy: base as f32 - 12.0, rx: 11.0, ry: 9.0 },
                        Blob { cx: 11.0, cy: base as f32 - 7.0, rx: 6.0, ry: 5.0 },
                        Blob { cx: 21.0, cy: base as f32 - 7.0, rx: 6.0, ry: 5.0 },
                    ];
                    canopy(&mut img, &blobs, pal::LEAF_SPRING, 0xc0 + kind as u32, 3.4, |_, _, _, _| {});
                    if ripe {
                        if big {
                            img.ellipse_with(16.0, base as f32 - 3.0, 8.0, 5.5, |im, x, y, nx, ny| {
                                let rib = (nx * 4.0).abs().fract() < 0.15 && kind == 4;
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
                }
            }
        }
    }
    img.outline(pal::INK);
    (img, (16, base))
}
