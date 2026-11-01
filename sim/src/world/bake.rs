//! Terrain baking: turns the tile grid into hand-painted-looking pixel art.
//!
//! Every pixel is shaded procedurally in world space, so there is no visible
//! tiling. Transitions between terrains are organic (bilinear corner fields
//! perturbed by noise, a "dual-grid" without a fixed tile set). All colours
//! are quantised to palette ramps with ordered dithering.
//!
//! The alpha channel carries a material code for the GPU shader:
//!   255        opaque land
//!   1..=180    water; the value encodes depth (1 = shoreline, 180 = deep)
//!   200..=215  waterfall

use super::{flag, Terrain, World, CHUNK, TILE};
use crate::noise::Noise;
use crate::palette::{self as pal, hex, ramp_at, ramp_dither, Ramp, Rgb};
use crate::rng::{hash2, hash2f};

pub const CHUNK_PX: i32 = CHUNK * TILE;

const R_DEEP: u8 = 0;
const R_SHALLOW: u8 = 1;
const R_SAND: u8 = 2;
const R_DIRT: u8 = 3;
const R_COBBLE: u8 = 4;
const R_GRASS: u8 = 5;
const R_ROCK: u8 = 6;
const R_SNOW: u8 = 7;

pub struct Baker {
    n_lo: Noise,
    n_mid: Noise,
    seed: u32,
}

#[derive(Clone, Copy)]
struct Surf {
    rank: u8,
    /// How far inside its own terrain the pixel is, in pixels (approx).
    inner: f32,
    /// Distance to the next-higher terrain's edge, in pixels (approx).
    /// Large when no higher terrain is nearby.
    outer: f32,
    /// The rank just below this one at this pixel (what an edge overhangs).
    below: u8,
}

#[inline]
fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Cheap value noise for pixel-scale detail.
#[inline]
fn vnoise(x: f32, y: f32, seed: u32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let tx = smooth(x - xi as f32);
    let ty = smooth(y - yi as f32);
    let a = hash2f(xi, yi, seed);
    let b = hash2f(xi + 1, yi, seed);
    let c = hash2f(xi, yi + 1, seed);
    let d = hash2f(xi + 1, yi + 1, seed);
    let ab = a + (b - a) * tx;
    let cd = c + (d - c) * tx;
    ab + (cd - ab) * ty
}

/// Bilinearly sample a per-tile field at pixel resolution (no tile seams).
fn field_at(w: &World, f: &[f32], px: i32, py: i32) -> f32 {
    let fx = (px as f32 + 0.5) / TILE as f32 - 0.5;
    let fy = (py as f32 + 0.5) / TILE as f32 - 0.5;
    let i0 = fx.floor() as i32;
    let j0 = fy.floor() as i32;
    let u = smooth(fx - i0 as f32);
    let v = smooth(fy - j0 as f32);
    let g = |x: i32, y: i32| f[w.idx(x.clamp(0, w.w - 1), y.clamp(0, w.h - 1))];
    let a = g(i0, j0) + (g(i0 + 1, j0) - g(i0, j0)) * u;
    let b = g(i0, j0 + 1) + (g(i0 + 1, j0 + 1) - g(i0, j0 + 1)) * u;
    a + (b - a) * v
}

fn season_grass(season: u8) -> Ramp {
    match season {
        0 => pal::GRASS_SPRING,
        1 => pal::GRASS_SUMMER,
        2 => pal::GRASS_AUTUMN,
        _ => pal::SNOW,
    }
}

impl Baker {
    pub fn new(seed: u64) -> Self {
        Baker { n_lo: Noise::new(seed ^ 0xa11ce), n_mid: Noise::new(seed ^ 0xb0b), seed: seed as u32 }
    }

    fn rank(&self, w: &World, x: i32, y: i32) -> u8 {
        let x = x.clamp(0, w.w - 1);
        let y = y.clamp(0, w.h - 1);
        let f = w.flags(x, y);
        if f & flag::PLAZA != 0 {
            return R_COBBLE;
        }
        match w.terrain(x, y) {
            Terrain::Deep => R_DEEP,
            Terrain::Shallow => R_SHALLOW,
            Terrain::Sand => R_SAND,
            Terrain::Dirt => R_DIRT,
            Terrain::Grass => R_GRASS,
            Terrain::Rock => R_ROCK,
            Terrain::Snow => R_SNOW,
        }
    }

    /// Resolve which terrain a pixel shows, with organic edges.
    fn surface(&self, w: &World, px: i32, py: i32, tx: i32, ty: i32) -> Surf {
        let fx = (px as f32 + 0.5) / TILE as f32 - 0.5;
        let fy = (py as f32 + 0.5) / TILE as f32 - 0.5;
        let i0 = fx.floor() as i32;
        let j0 = fy.floor() as i32;
        let u = fx - i0 as f32;
        let v = fy - j0 as f32;
        let own_lv = w.level(tx, ty);
        let own = self.rank(w, tx, ty);
        let mut ranks = [0u8; 4];
        let corners = [(i0, j0), (i0 + 1, j0), (i0, j0 + 1), (i0 + 1, j0 + 1)];
        for (k, &(cx, cy)) in corners.iter().enumerate() {
            // Never blend across a cliff: those corners take our own terrain.
            ranks[k] = if w.level(cx, cy) != own_lv || (w.is_face(cx, cy) && !w.is_face(tx, ty)) { own } else { self.rank(w, cx, cy) };
        }
        let wts = [(1.0 - u) * (1.0 - v), u * (1.0 - v), (1.0 - u) * v, u * v];
        let nz = (vnoise(px as f32 * 0.09, py as f32 * 0.09, self.seed ^ 0x1f) - 0.5) * 0.34
            + (vnoise(px as f32 * 0.3, py as f32 * 0.3, self.seed ^ 0x2f) - 0.5) * 0.1
            + (hash2f(px, py, self.seed ^ 0x3f) - 0.5) * 0.05;
        let field = |r: u8| -> f32 {
            let mut s = 0.0;
            for k in 0..4 {
                if ranks[k] >= r {
                    s += wts[k];
                }
            }
            s
        };
        let mut sorted = ranks;
        sorted.sort_unstable();
        let mut chosen = sorted[0];
        let mut chosen_field = 1.0;
        let mut below = sorted[0];
        for &r in sorted.iter() {
            if r == chosen {
                continue;
            }
            let f = field(r);
            // Grass creeps over paths and plazas, keeping them narrow.
            let bias = if r == R_GRASS && (chosen == R_DIRT || chosen == R_COBBLE) { 0.14 } else { 0.0 };
            let val = if f >= 0.999 { 1.0 } else if f <= 0.001 { 0.0 } else { f + nz + bias };
            if val > 0.5 {
                below = chosen;
                chosen = r;
                chosen_field = val;
            }
        }
        // Distance to the edge of the next-higher terrain, if any.
        let mut outer = 99.0;
        for &r in sorted.iter() {
            if r > chosen {
                let f = field(r);
                if f > 0.001 {
                    let val = f + nz;
                    outer = ((0.5 - val) * TILE as f32 * 1.2).max(0.0);
                    break;
                }
            }
        }
        let inner = if chosen_field >= 0.999 { 99.0 } else { (chosen_field - 0.5) * TILE as f32 * 1.2 };
        Surf { rank: chosen, inner, outer, below }
    }

    pub fn bake_chunk(&self, w: &World, cx: i32, cy: i32, season: u8, out: &mut [u8]) {
        let ox = cx * CHUNK_PX;
        let oy = cy * CHUNK_PX;
        for ly in 0..CHUNK_PX {
            for lx in 0..CHUNK_PX {
                let px = ox + lx;
                let py = oy + ly;
                let (c, a) = self.pixel(w, px, py, season);
                let o = ((ly * CHUNK_PX + lx) * 4) as usize;
                out[o] = c[0];
                out[o + 1] = c[1];
                out[o + 2] = c[2];
                out[o + 3] = a;
            }
        }
    }

    /// Colour of a single world pixel (used for the overview map).
    pub fn sample(&self, w: &World, px: i32, py: i32, season: u8) -> Rgb {
        self.pixel(w, px, py, season).0
    }

    fn pixel(&self, w: &World, px: i32, py: i32, season: u8) -> (Rgb, u8) {
        let tx = px.div_euclid(TILE);
        let ty = py.div_euclid(TILE);
        let lx = px.rem_euclid(TILE);
        let ly = py.rem_euclid(TILE);
        let f = w.flags(tx, ty);

        if w.is_face(tx, ty) {
            return self.face(w, px, py, tx, ty, lx, ly, f, season);
        }
        if f & flag::BRIDGE != 0 {
            return (self.bridge(w, px, py, tx, ty, lx, ly), 255);
        }

        let s = self.surface(w, px, py, tx, ty);
        let (mut col, alpha) = match s.rank {
            R_DEEP | R_SHALLOW => {
                let (c, a) = self.water(w, px, py, tx, ty);
                (c, a)
            }
            R_SAND => (self.sand(w, px, py, &s), 255),
            R_DIRT => (self.dirt(px, py, &s, season), 255),
            R_COBBLE => (self.cobble(px, py, &s), 255),
            R_GRASS => (self.grass(w, px, py, tx, ty, &s, season), 255),
            R_ROCK => (self.rock(w, px, py, &s, season), 255),
            _ => (self.snow(px, py, &s), 255),
        };
        if alpha == 255 && f & flag::TILLED != 0 {
            col = self.tilled(w, px, py, tx, ty, lx, ly, col, f);
        }
        if alpha == 255 {
            // Higher ground catches a touch more warm light, so terraces read.
            let lv = w.level(tx, ty) as f32;
            if lv > 0.0 && s.rank >= R_SAND {
                col = pal::mix(col, hex(0xfff0c0), lv * 0.035);
            }
            col = self.cliff_shadows(w, tx, ty, lx, ly, px, py, col);
        }
        (col, alpha)
    }

    fn darken(&self, c: Rgb, amt: f32) -> Rgb {
        // Shadows shift toward a cool violet instead of grey.
        pal::mix(c, hex(0x24203c), amt)
    }

    fn cliff_shadows(&self, w: &World, tx: i32, ty: i32, lx: i32, ly: i32, px: i32, py: i32, c: Rgb) -> Rgb {
        let lv = w.level(tx, ty);
        let mut c = c;
        // Shadow cast at the foot of a cliff face.
        if w.is_face(tx, ty - 1) {
            let depth = TILE / 4 + (hash2(px, 0, self.seed ^ 9) % 3) as i32;
            if ly < depth {
                let t = 1.0 - ly as f32 / depth as f32;
                c = self.darken(c, 0.25 + 0.35 * t);
            }
        }
        // Side shadows where a plateau rises to the east or west.
        let jag = (hash2(0, py / 2, self.seed ^ 5) % 2) as i32;
        if tx > 0 && w.level(tx - 1, ty) > lv && !w.is_face(tx - 1, ty) && lx < TILE / 5 + jag {
            c = self.darken(c, 0.32);
        }
        if w.level(tx + 1, ty) > lv && !w.is_face(tx + 1, ty) && lx > TILE - 4 - jag {
            c = self.darken(c, 0.22);
        }
        // Plateau edges: a narrow rocky side wall on east/west drops, a lip
        // on the back (north) edge.
        let lower = |dx: i32, dy: i32| w.level(tx + dx, ty + dy) < lv;
        let up_t = w.terrain(tx, ty);
        let ramp = if matches!(up_t, Terrain::Rock | Terrain::Snow) { pal::CLIFF_STONE } else { pal::CLIFF_EARTH };
        let wall = TILE / 5 + jag;
        let side = if lower(-1, 0) && lx < wall { Some(lx) } else if lower(1, 0) && lx > TILE - 1 - wall { Some(TILE - 1 - lx) } else { None };
        if let Some(d) = side {
            let (crack, shade, _) = self.rock_chunks(px, py, 4, 7);
            c = if d == 0 {
                pal::INK_SOFT
            } else if d == wall - 1 {
                ramp_at(ramp, 4)
            } else if crack {
                ramp_at(ramp, 1)
            } else {
                ramp_dither(ramp, 0.4 + shade, px, py, 0.4)
            };
        } else if lower(0, -1) && ly == 0 {
            c = pal::INK_SOFT;
        } else if lower(0, -1) && ly == 1 {
            c = pal::mix(c, hex(0xf6e8b0), 0.22);
        }
        c
    }

    fn grass(&self, w: &World, px: i32, py: i32, tx: i32, ty: i32, s: &Surf, season: u8) -> Rgb {
        let ramp = season_grass(season);
        let _ = (tx, ty);
        let forest = field_at(w, &w.forest, px, py);
        let moist = field_at(w, &w.moist, px, py);
        let x = px as f32;
        let y = py as f32;
        let lo = self.n_lo.fbm(x / 90.0, y / 90.0, 3, 2.0, 0.5);
        let mid = self.n_mid.get(x / 14.0, y / 14.0);
        let mut v = 0.58 + lo * 0.12 + mid * 0.04 - forest * 0.18 + (moist - 0.5) * 0.06;
        // Grass tufts: tiny "v" strokes placed on a jittered grid.
        let (gx, gy) = (px.div_euclid(5), py.div_euclid(4));
        let h = hash2(gx, gy, self.seed ^ 0xabc);
        let ox = (h % 5) as i32;
        let oy = ((h >> 4) % 4) as i32;
        let (dx, dy) = (px.rem_euclid(5) - ox, py.rem_euclid(4) - oy);
        let tuft = (h >> 8) % 100 < 55;
        if tuft {
            if (dx == 0 || dx == 2) && dy == 0 || dx == 1 && dy == 1 {
                v += 0.14;
            } else if dx == 1 && dy == 2 {
                v -= 0.1;
            }
        }
        // Speckle
        let sp = hash2f(px, py, self.seed ^ 0x55);
        if sp < 0.04 {
            v -= 0.08;
        } else if sp > 0.975 {
            v += 0.08;
        }
        let mut c = ramp_dither(ramp, v, px, py, 0.22);
        // Flowers in sunny meadows (none in winter).
        if season != 3 && s.inner > 2.0 {
            let meadow = self.n_mid.get(x / 70.0 + 40.0, y / 70.0 - 13.0) * 0.5 + 0.5;
            let density = (meadow - 0.55).max(0.0) * 1.6 * (1.0 - forest) + if season == 0 { 0.05 } else { 0.0 };
            let (fx, fy) = (px.div_euclid(7), py.div_euclid(7));
            let fh = hash2(fx, fy, self.seed ^ 0xf10);
            if ((fh % 1000) as f32) < density * 1000.0 * 0.5 {
                let cx = fx * 7 + 2 + ((fh >> 10) % 3) as i32;
                let cy = fy * 7 + 2 + ((fh >> 12) % 3) as i32;
                let kind = &pal::FLOWERS[((fh >> 14) as usize) % pal::FLOWERS.len()];
                let (dx, dy) = (px - cx, py - cy);
                if dx == 0 && dy == 0 {
                    c = hex(0xf6d860);
                } else if (dx.abs() + dy.abs()) == 1 {
                    c = hex(if dy == -1 { kind[2] } else { kind[1] });
                } else if dx == 0 && dy == 2 {
                    c = ramp_at(ramp, 1);
                }
            }
        }
        // Edges: a darker outline where grass overhangs lower ground, and a
        // muddy bank against water.
        if s.inner < 1.3 {
            c = if s.below <= R_SHALLOW { hex(0x3d3036) } else { ramp_at(ramp, 1) };
        } else if s.inner < 2.4 {
            c = if s.below <= R_SHALLOW { ramp_at(pal::DIRT, 1) } else { self.darken(c, 0.25) };
        }
        c
    }

    fn dirt(&self, px: i32, py: i32, s: &Surf, season: u8) -> Rgb {
        let x = px as f32;
        let y = py as f32;
        let mut v = 0.55 + self.n_mid.get(x / 11.0, y / 11.0) * 0.1 + self.n_lo.get(x / 45.0, y / 45.0) * 0.08;
        // Pebbles
        let (gx, gy) = (px.div_euclid(6), py.div_euclid(6));
        let h = hash2(gx, gy, self.seed ^ 0xd17);
        if h % 100 < 35 {
            let cx = gx * 6 + (h >> 8) as i32 % 5;
            let cy = gy * 6 + (h >> 12) as i32 % 5;
            if py == cy && (px == cx || px == cx + 1) {
                v += 0.2;
            } else if py == cy + 1 && (px == cx || px == cx + 1) {
                v -= 0.18;
            }
        }
        let mut c = ramp_dither(pal::DIRT, v, px, py, 0.3);
        if season == 3 && hash2f(px, py, self.seed ^ 0x5) < 0.25 + self.n_mid.get(x / 6.0, y / 6.0).max(0.0) {
            c = ramp_at(pal::SNOW, 4);
        }
        // Shadow cast by an overhanging higher terrain.
        if s.outer < 1.5 {
            c = self.darken(c, 0.38);
        } else if s.outer < 3.0 {
            c = self.darken(c, 0.16);
        }
        c
    }

    fn cobble(&self, px: i32, py: i32, s: &Surf) -> Rgb {
        // Irregular rounded flagstones: jittered cells, nearest-centre wins.
        let cell = 10;
        let (gx, gy) = (px.div_euclid(cell), py.div_euclid(cell));
        let mut best = (i32::MAX, i32::MAX, 0u32, 0i32, 0i32);
        for oy in -1..=1 {
            for ox in -1..=1 {
                let (cx, cy) = (gx + ox, gy + oy);
                let h = hash2(cx, cy, self.seed ^ 0xc0b);
                let sx = cx * cell + (h % cell as u32) as i32;
                let sy = cy * cell + ((h >> 8) % cell as u32) as i32;
                let d = (px - sx).pow(2) + (py - sy).pow(2);
                if d < best.0 {
                    best = (d, best.0, h, sx, sy);
                } else if d < best.1 {
                    best.1 = d;
                }
            }
        }
        let (d1, d2, h, sx, sy) = best;
        let edge = (d2 as f32).sqrt() - (d1 as f32).sqrt();
        let tint = (h >> 16) % 13;
        let ramp = if tint == 0 { pal::ROCK } else { pal::STONE_WALL };
        let base = 0.5 + ((h >> 20) % 100) as f32 / 100.0 * 0.22;
        let c = if edge < 1.0 {
            if hash2(px, py, self.seed ^ 0x6) % 5 == 0 { ramp_at(pal::GRASS_SUMMER, 2) } else { ramp_at(pal::STONE_WALL, 0) }
        } else {
            // Light from the upper left.
            let shade = ((sx - px) + (sy - py)) as f32 * 0.04;
            let v = base + shade + if edge < 2.0 && (py > sy || px > sx) { -0.15 } else { 0.0 };
            ramp_dither(ramp, v, px, py, 0.3)
        };
        if s.outer < 1.5 {
            self.darken(c, 0.35)
        } else {
            c
        }
    }

    fn sand(&self, w: &World, px: i32, py: i32, s: &Surf) -> Rgb {
        let x = px as f32;
        let y = py as f32;
        let ripple = ((x * 0.25 + y * 0.9 + self.n_mid.get(x / 20.0, y / 20.0) * 6.0).sin() * 0.5 + 0.5) * 0.1;
        let mut v = 0.55 + ripple + self.n_lo.get(x / 50.0, y / 50.0) * 0.1;
        let sp = hash2f(px, py, self.seed ^ 0x5a);
        if sp > 0.985 {
            v += 0.2;
        } else if sp < 0.02 {
            v -= 0.12;
        }
        let _ = w;
        // Wet sand toward the waterline.
        if s.below <= R_SHALLOW && s.inner < 5.0 {
            if s.inner < 2.5 {
                return ramp_dither(pal::WET_SAND, 0.4 + ripple, px, py, 0.4);
            }
            return ramp_dither(pal::SAND, v - 0.25, px, py, 0.3);
        }
        let c = ramp_dither(pal::SAND, v, px, py, 0.3);
        if s.outer < 1.5 { self.darken(c, 0.3) } else { c }
    }

    fn rock(&self, w: &World, px: i32, py: i32, s: &Surf, season: u8) -> Rgb {
        let x = px as f32;
        let y = py as f32;
        let n = self.n_mid.get(x / 9.0, y / 9.0);
        let mut v = 0.5 + self.n_lo.get(x / 30.0, y / 30.0) * 0.15 + n * 0.08;
        let crack = self.n_lo.get(x / 13.0 + 7.0, y / 13.0 - 3.0).abs();
        if crack < 0.04 {
            v -= 0.25;
        } else if crack < 0.07 {
            v += 0.1;
        }
        let mut c = ramp_dither(pal::ROCK, v, px, py, 0.3);
        let m = field_at(w, &w.moist, px, py);
        if season != 3 && self.n_mid.get(x / 7.0 - 9.0, y / 7.0 + 4.0) + m * 0.3 > 0.78 && hash2f(px, py, self.seed ^ 0x4d) < 0.6 {
            c = ramp_dither(pal::GRASS_SUMMER, 0.3 + n * 0.15, px, py, 0.4);
        }
        if season == 3 && self.n_mid.get(x / 10.0, y / 10.0) > 0.1 {
            c = ramp_dither(pal::SNOW, 0.75 + n * 0.1, px, py, 0.4);
        }
        if s.inner < 1.3 {
            c = ramp_at(pal::ROCK, 0);
        }
        c
    }

    fn snow(&self, px: i32, py: i32, s: &Surf) -> Rgb {
        let x = px as f32;
        let y = py as f32;
        let mut v = 0.72 + self.n_lo.get(x / 26.0, y / 26.0) * 0.14 + self.n_mid.get(x / 7.0, y / 7.0) * 0.04;
        if hash2f(px, py, self.seed ^ 0x51) > 0.992 {
            v = 1.0;
        }
        let c = ramp_dither(pal::SNOW, v, px, py, 0.3);
        if s.inner < 1.3 { ramp_at(pal::SNOW, 2) } else { c }
    }

    fn water(&self, w: &World, px: i32, py: i32, tx: i32, ty: i32) -> (Rgb, u8) {
        // Land weight around this pixel tells us how close the shore is.
        let fx = (px as f32 + 0.5) / TILE as f32 - 0.5;
        let fy = (py as f32 + 0.5) / TILE as f32 - 0.5;
        let i0 = fx.floor() as i32;
        let j0 = fy.floor() as i32;
        let u = fx - i0 as f32;
        let v = fy - j0 as f32;
        let land = |x: i32, y: i32| if w.terrain(x, y).is_water() { 0.0 } else { 1.0 };
        let deep = |x: i32, y: i32| if w.terrain(x, y) == Terrain::Deep { 1.0 } else { 0.0 };
        let bil = |f: &dyn Fn(i32, i32) -> f32| {
            f(i0, j0) * (1.0 - u) * (1.0 - v) + f(i0 + 1, j0) * u * (1.0 - v) + f(i0, j0 + 1) * (1.0 - u) * v + f(i0 + 1, j0 + 1) * u * v
        };
        let lw = bil(&land);
        let dw = bil(&deep);
        let river = w.has(tx, ty, flag::RIVER);
        let n = self.n_mid.get(px as f32 / 18.0, py as f32 / 18.0) * 0.08;
        let mut depth = ((0.5 - lw) * 2.0 + dw * 0.7 + n).clamp(0.0, 1.0);
        if river {
            depth = depth.min(0.45);
        }
        let ramp = pal::WATER;
        let c = ramp_dither(ramp, 0.95 - depth * 0.85, px, py, 0.45);
        let code = (1.0 + depth * 179.0) as u8;
        (c, code.max(1))
    }

    /// Voronoi rock chunks shared by cliff faces and side walls. Returns
    /// (crack, shade) where shade includes upper-left lighting.
    fn rock_chunks(&self, px: i32, py: i32, cw: i32, ch: i32) -> (bool, f32, u32) {
        let (gx, gy) = (px.div_euclid(cw), py.div_euclid(ch));
        let mut best = (i32::MAX, i32::MAX, 0u32, 0i32, 0i32);
        for oy in -1..=1 {
            for ox in -1..=1 {
                let (cx, cy) = (gx + ox, gy + oy);
                let h = hash2(cx, cy, self.seed ^ 0xc1f);
                let sx = cx * cw + (h % cw as u32) as i32;
                let sy = cy * ch + ((h >> 8) % ch as u32) as i32;
                // Stretch horizontally so chunks read as stacked strata.
                let d = ((px - sx).pow(2)) + ((py - sy).pow(2)) * 2;
                if d < best.0 {
                    best = (d, best.0, h, sx, sy);
                } else if d < best.1 {
                    best.1 = d;
                }
            }
        }
        let (d1, d2, h, sx, sy) = best;
        let edge = (d2 as f32).sqrt() - (d1 as f32).sqrt();
        let light = ((sx - px) as f32 * 0.025 + (sy - py) as f32 * 0.05).clamp(-0.2, 0.2);
        let var = ((h >> 16) % 100) as f32 / 100.0 * 0.12;
        (edge < 1.1, light + var, h)
    }

    #[allow(clippy::too_many_arguments)]
    fn face(&self, w: &World, px: i32, py: i32, tx: i32, ty: i32, lx: i32, ly: i32, f: u8, season: u8) -> (Rgb, u8) {
        let row = w.face_row(tx, ty) as i32;
        let top_ty = ty - row;
        let fy = (row - 1) * TILE + ly; // 0..2*TILE down the face
        let face_h = 2 * TILE;
        let up_terrain = w.terrain(tx, top_ty);
        let stone = matches!(up_terrain, Terrain::Rock | Terrain::Snow);
        let ramp = if stone { pal::CLIFF_STONE } else { pal::CLIFF_EARTH };

        // Waterfall where a river spills over the edge.
        if w.terrain(tx, ty).is_water() && w.terrain(tx, top_ty).is_water() {
            let streak = hash2(px, 0, self.seed ^ 0x77) % 4;
            let c = match streak {
                0 => hex(pal::FOAM[1]),
                1 => hex(pal::WATER[6]),
                _ => hex(pal::WATER[5]),
            };
            return (c, 200 + ((fy * 16 / face_h) as u8).min(15));
        }

        // Upper lip: the plateau's surface curls over the edge.
        let lip = 3 + (hash2(px / 2, top_ty, self.seed ^ 0x11) % 4) as i32;
        if fy < lip {
            let c = if season == 3 || up_terrain == Terrain::Snow {
                ramp_at(pal::SNOW, if fy == lip - 1 { 3 } else { 5 })
            } else if up_terrain == Terrain::Grass {
                let g = season_grass(season);
                if fy == lip - 1 { ramp_at(g, 1) } else { ramp_at(g, 3) }
            } else if up_terrain == Terrain::Sand {
                ramp_at(pal::SAND, if fy == lip - 1 { 1 } else { 3 })
            } else {
                ramp_at(ramp, 5)
            };
            return (c, 255);
        }

        if f & flag::STAIRS != 0 {
            return (self.stairs(lx, fy, px, py), 255);
        }

        let (crack, shade, h) = self.rock_chunks(px, py, 13, 8);
        // Lighter near the top, darker toward the base (ambient occlusion).
        let grad = 0.72 - (fy as f32 / face_h as f32) * 0.42;
        let mut c = if crack { ramp_at(ramp, 0) } else { ramp_dither(ramp, grad + shade, px, py, 0.35) };
        // Highlight along the top of each chunk.
        if !crack {
            let (above, _, h2) = self.rock_chunks(px, py - 1, 13, 8);
            if above || h2 != h {
                c = ramp_at(ramp, ((grad + shade) * ramp.len() as f32) as i32 + 1);
            }
        }
        if fy == lip {
            c = ramp_at(ramp, 0);
        }
        // Ends of a cliff run turn the corner.
        let left_end = !w.is_face(tx - 1, ty);
        let right_end = !w.is_face(tx + 1, ty);
        if left_end && lx <= 1 || right_end && lx >= TILE - 2 {
            c = if left_end && lx == 0 || right_end && lx == TILE - 1 { pal::INK_SOFT } else { ramp_at(ramp, 1) };
        }
        // Dark contact line at the base.
        if fy >= face_h - 1 {
            c = pal::INK_SOFT;
        } else if fy >= face_h - 3 {
            c = self.darken(c, 0.4);
        }
        // Moss and hanging vines on earthy cliffs.
        if !stone && season != 3 {
            let vine = hash2(px, top_ty, self.seed ^ 0x3e);
            let len = lip + 2 + (vine >> 8) as i32 % (face_h / 2);
            if vine % 100 < 8 && fy < len {
                c = ramp_at(season_grass(season), if fy == len - 1 { 3 } else { 1 + ((vine >> 16) % 2) as i32 });
            }
        }
        (c, 255)
    }

    fn stairs(&self, lx: i32, fy: i32, px: i32, py: i32) -> Rgb {
        if lx < 3 || lx > TILE - 4 {
            return if lx == 0 || lx == TILE - 1 { pal::INK_SOFT } else if lx == 1 || lx == TILE - 2 { ramp_at(pal::STONE_WALL, 3) } else { ramp_at(pal::STONE_WALL, 1) };
        }
        let step = fy.rem_euclid(6);
        let step = if step >= 4 { step - 1 } else { step };
        match step {
            0 => ramp_at(pal::STONE_WALL, 4),
            1 | 2 => ramp_dither(pal::STONE_WALL, 0.62, px, py, 0.4),
            3 => ramp_at(pal::STONE_WALL, 1),
            _ => ramp_at(pal::STONE_WALL, 0),
        }
    }

    fn bridge(&self, w: &World, px: i32, py: i32, tx: i32, ty: i32, lx: i32, ly: i32) -> Rgb {
        let link = |x: i32, y: i32| w.has(x, y, flag::BRIDGE | flag::ROAD) || !w.terrain(x, y).is_water();
        let horizontal = link(tx - 1, ty) && link(tx + 1, ty) && w.has(tx - 1, ty, flag::BRIDGE | flag::ROAD) | w.has(tx + 1, ty, flag::BRIDGE | flag::ROAD);
        let (along, across) = if horizontal { (lx, ly) } else { (ly, lx) };
        // Rails along both edges.
        if across <= 2 || across >= TILE - 3 {
            return if across == 0 || across == TILE - 1 { ramp_at(pal::WOOD, 0) } else { ramp_at(pal::WOOD, 3) };
        }
        // Planks run across the direction of travel.
        let plank = along.div_euclid(4);
        let h = hash2(plank, tx * 31 + ty, self.seed ^ 0xb7);
        if along.rem_euclid(4) == 3 {
            return ramp_at(pal::WOOD, 1);
        }
        let v = 0.55 + (h % 100) as f32 / 100.0 * 0.2;
        let c = ramp_dither(pal::WOOD, v, px, py, 0.4);
        if across == 3 { self.darken(c, 0.3) } else { c }
    }

    #[allow(clippy::too_many_arguments)]
    fn tilled(&self, w: &World, px: i32, py: i32, tx: i32, ty: i32, lx: i32, ly: i32, base: Rgb, f: u8) -> Rgb {
        let edge_l = !w.has(tx - 1, ty, flag::TILLED);
        let edge_r = !w.has(tx + 1, ty, flag::TILLED);
        let edge_t = !w.has(tx, ty - 1, flag::TILLED);
        let edge_b = !w.has(tx, ty + 1, flag::TILLED);
        let inset = 2;
        // Round the outer corners.
        let corner = (edge_l && lx < inset + 1 || edge_r && lx > TILE - 2 - inset) && (edge_t && ly < inset + 1 || edge_b && ly > TILE - 2 - inset);
        if corner || edge_l && lx < inset || edge_r && lx > TILE - 1 - inset || edge_t && ly < inset || edge_b && ly > TILE - 1 - inset {
            return base;
        }
        let watered = f & flag::WATERED != 0;
        let shift = if watered { -0.25 } else { 0.0 };
        let furrow = ly.rem_euclid(6);
        let v = match furrow {
            0 => 0.78,
            1 | 2 => 0.6,
            3 => 0.45,
            4 => 0.3,
            _ => 0.15,
        } + shift + (hash2f(px, py, self.seed ^ 0x7) - 0.5) * 0.15;
        let mut c = ramp_dither(pal::TILLED, v, px, py, 0.4);
        if edge_t && ly == inset || edge_l && lx == inset {
            c = ramp_at(pal::TILLED, 0);
        }
        if edge_b && ly == TILE - 1 - inset || edge_r && lx == TILE - 1 - inset {
            c = ramp_at(pal::TILLED, 3);
        }
        c
    }
}
