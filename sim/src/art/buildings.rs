//! Buildings and village furniture. Every house is generated from its own
//! seed, so no two are identical, while each village keeps a consistent
//! architectural culture.

use super::canvas::Img;
use super::shade;
use crate::palette::{self as pal, hex, mix, ramp_at, Ramp, Rgb};
use crate::rng::{hash2, Rng};
use crate::world::villages::BuildingKind;
use crate::world::TILE;

/// Awnings and banners draw from the original ten cloth dyes (the later
/// white, black, pink and navy are for outfits).
const AWNING_CLOTHS: usize = 10;

/// How far the roof overhangs each side wall.
const OVERHANG: i32 = 8;

/// Shadows tint toward a cool violet rather than black, to stay luminous.
const SHADOW: Rgb = hex(0x24203c);
const WHITE: Rgb = hex(0xffffff);
/// Undyed canvas between awning stripes.
const CANVAS: Rgb = hex(0xede3d0);

/// Visual metadata the renderer needs for lights and smoke, in pixels
/// relative to the sprite pivot (bottom-left of the footprint).
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct HouseFx {
    pub windows: Vec<(i16, i16)>,
    pub chimney: Option<(i16, i16)>,
    pub door: (i16, i16),
}

/// A village's architectural tradition, chosen by `culture % 4`.
#[derive(Clone, Copy, PartialEq)]
enum Culture {
    /// Plaster panels between dark timber framing.
    HalfTimbered,
    /// Coursed stone walls.
    Stone,
    /// Plain plaster over a painted board wainscot.
    Plaster,
    /// Log cabins with round log ends and arched doors.
    Log,
}

impl Culture {
    fn of(culture: u8) -> Culture {
        match culture % 4 {
            0 => Culture::HalfTimbered,
            1 => Culture::Stone,
            2 => Culture::Plaster,
            _ => Culture::Log,
        }
    }

    /// Odds that a house turns its gable end to the street.
    fn front_gable_odds(self) -> f32 {
        match self {
            Culture::HalfTimbered => 0.5,
            Culture::Stone => 0.35,
            Culture::Plaster => 0.2,
            Culture::Log => 0.65,
        }
    }
}

struct Style {
    culture: Culture,
    wall: Ramp,
    roof: Ramp,
    trim: Ramp,
}

/// Repeated entries in `roofs` weight the pick toward the usual roof.
fn style(culture: Culture, r: &mut Rng) -> Style {
    let (wall, roofs, trim) = match culture {
        Culture::HalfTimbered => (pal::PLASTER, [pal::ROOF_RED, pal::ROOF_THATCH, pal::ROOF_RED], pal::TIMBER),
        Culture::Stone => (pal::STONE_WALL, [pal::ROOF_SLATE, pal::ROOF_SLATE, pal::ROOF_RED], pal::TIMBER),
        Culture::Plaster => (pal::PLASTER, [pal::ROOF_BLUE, pal::ROOF_BLUE, pal::ROOF_THATCH], pal::WOOD),
        Culture::Log => (pal::WOOD, [pal::ROOF_GREEN, pal::ROOF_THATCH, pal::ROOF_GREEN], pal::TIMBER),
    };
    Style { culture, wall, roof: r.pick(&roofs), trim }
}

/// Roofing material, which sets the course pattern.
#[derive(Clone, Copy, PartialEq)]
enum Roofing {
    Tile,
    Thatch,
    Slate,
}

impl Roofing {
    fn of(roof: Ramp) -> Roofing {
        if roof == pal::ROOF_THATCH {
            Roofing::Thatch
        } else if roof == pal::ROOF_SLATE {
            Roofing::Slate
        } else {
            Roofing::Tile
        }
    }

    /// Height of one course, in pixels.
    fn course(self) -> i32 {
        match self {
            Roofing::Slate => 5,
            Roofing::Thatch => 6,
            Roofing::Tile => 8,
        }
    }
}

/// Shingle / thatch / slate surface colour for a roof pixel. `u` runs along
/// the courses, `row_px` across them, `t` from the ridge (0) to the eave (1).
fn roof_px(rr: Ramp, roofing: Roofing, u: i32, row_px: i32, t: f32, seed: u32, lit: f32) -> Rgb {
    let course = roofing.course();
    let row = row_px.div_euclid(course);
    let ry = row_px.rem_euclid(course);
    let base = 0.66 - t * 0.2 + lit;
    let v = match roofing {
        Roofing::Thatch => {
            // Fibrous streaks, ragged course edges.
            let streak = hash2(u, row_px / 3, seed) % 5;
            let mut v = base
                + match streak {
                    0 => -0.16,
                    1 => 0.07,
                    2 => -0.05,
                    _ => 0.0,
                };
            if ry == course - 1 && !hash2(u, row, seed).is_multiple_of(3) {
                v -= 0.18;
            }
            v
        }
        Roofing::Tile | Roofing::Slate => {
            let tile_w = if roofing == Roofing::Slate { 8 } else { 12 };
            let off = if row % 2 == 0 { 0 } else { tile_w / 2 };
            let tx = (u + off).rem_euclid(tile_w);
            let tile = hash2((u + off).div_euclid(tile_w), row, seed ^ 0x5);
            let mut v = base + (tile % 100) as f32 / 100.0 * 0.1 - 0.05;
            // Each course's lower edge is shadowed (two rows deep on tile).
            if ry == course - 1 || ry == course - 2 && roofing != Roofing::Slate {
                v -= if ry == course - 1 { 0.26 } else { 0.12 };
            } else if ry == 0 {
                v += 0.08;
            }
            if tx == 0 {
                v -= 0.14;
            } else if tx == 1 && ry < course - 2 {
                v += 0.04;
            }
            v
        }
    };
    shade(rr, v)
}

/// Geometry shared by every part of one house sprite, in sprite pixels.
struct Frame {
    w: i32,
    wall_h: i32,
    wall_top: i32,
    base_y: i32,
    /// Leftmost and rightmost wall columns.
    wx0: i32,
    wx1: i32,
    front_gable: bool,
    /// Footprint depth as seen in the 3/4 projection.
    depth: f32,
    gable_h: i32,
    roof_h: i32,
    /// The roof overhangs the top of the wall down to here.
    eave_y: i32,
    ridge_y: i32,
    pivot: (i32, i32),
}

impl Frame {
    fn new(kind: BuildingKind, fw: i32, fh: i32, front_gable: bool) -> Frame {
        let chapel = kind == BuildingKind::Chapel;
        let w = fw * TILE + OVERHANG * 2;
        let wall_h = match kind {
            BuildingKind::Inn => 140,
            BuildingKind::Chapel => 148,
            _ => 124,
        };
        let depth = (fh * TILE) as f32 * 0.55;
        let rise = 32.0 + if chapel { 16.0 } else { 0.0 };
        let gable_h = ((fw * TILE) as f32 * 0.38) as i32;
        let roof_h = if front_gable { depth as i32 + gable_h } else { (depth + rise) as i32 };
        // Headroom for the chimney, finial or bell tower.
        let extra_top = if chapel { 46 } else { 22 };
        let h = wall_h + roof_h + extra_top;
        let base_y = h - 1;
        let wall_top = base_y - wall_h + 1;
        let eave_y = wall_top + 10;
        Frame {
            w,
            wall_h,
            wall_top,
            base_y,
            wx0: OVERHANG,
            wx1: w - OVERHANG - 1,
            front_gable,
            depth,
            gable_h,
            roof_h,
            eave_y,
            ridge_y: eave_y - roof_h,
            pivot: (OVERHANG, h),
        }
    }

    fn height(&self) -> i32 {
        self.base_y + 1
    }

    /// Top of the street-facing gable triangle.
    fn apex_y(&self) -> i32 {
        self.eave_y - self.gable_h
    }

    /// A sprite point relative to the pivot, as stored in `HouseFx`.
    fn rel(&self, x: i32, y: i32) -> (i16, i16) {
        ((x - self.pivot.0) as i16, (y - self.pivot.1) as i16)
    }
}

/// Front door rectangle, excluding its frame.
#[derive(Clone, Copy)]
struct Door {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

/// Generate a building sprite. Returns the image, its pivot (bottom-left of
/// the footprint, in sprite pixels) and fx metadata.
pub fn house(kind: BuildingKind, culture: u8, seed: u32, fw: i32, fh: i32, lit: bool) -> (Img, (i32, i32), HouseFx) {
    // Order matters twice over: later parts overpaint earlier ones, and the
    // RNG must be drawn from in the same order for a seed to stay stable.
    let mut r = Rng::new(seed as u64 ^ 0x40053);
    let st = style(Culture::of(culture), &mut r);
    let front_gable = kind != BuildingKind::Inn && r.chance(st.culture.front_gable_odds());
    let f = Frame::new(kind, fw, fh, front_gable);
    let mut img = Img::new(f.w, f.height());
    let mut fx = HouseFx::default();

    walls(&mut img, &f, &st, seed);
    foundation(&mut img, &f, seed);
    let door = front_door(&mut img, &f, &st, kind, lit, &mut fx);
    front_windows(&mut img, &f, &st, door, lit, &mut r, &mut fx);
    if kind == BuildingKind::Chapel {
        rose_window(&mut img, &f, &st, lit, &mut fx);
    }
    if kind != BuildingKind::Home {
        let sx = door.x - 30;
        if sx > f.wx0 + 4 {
            sign_icon(&mut img, kind, sx, door.y + 6);
        }
    }
    if kind == BuildingKind::Store {
        store_awning(&mut img, door, pal::CLOTH[r.below(AWNING_CLOTHS)]);
    }

    let roofing = Roofing::of(st.roof);
    if front_gable {
        front_gabled_roof(&mut img, &f, &st, roofing, lit, seed, &mut fx);
    } else {
        side_gabled_roof(&mut img, &f, &st, roofing, seed);
    }
    eave_shadow(&mut img, &f);
    if !front_gable && (fh >= 4 || kind == BuildingKind::Inn) {
        dormer(&mut img, &f, &st, lit, &mut fx);
    }
    if kind == BuildingKind::Chapel {
        bell_tower(&mut img, &f, &st);
    } else {
        chimney(&mut img, &f, &mut r, &mut fx);
    }

    img.outline(pal::INK);
    (img, f.pivot, fx)
}

// -------------------------------------------------------------------- walls

fn walls(img: &mut Img, f: &Frame, st: &Style, seed: u32) {
    for y in f.wall_top..=f.base_y {
        for x in f.wx0..=f.wx1 {
            let t = (y - f.wall_top) as f32 / f.wall_h as f32;
            // Light falls from the upper left across the facade.
            let lx = (x - f.wx0) as f32 / (f.wx1 - f.wx0) as f32 * -0.06;
            let c = match st.culture {
                Culture::Stone => {
                    let row = (y - f.wall_top) / 11;
                    let off = if row % 2 == 0 { 0 } else { 9 };
                    let bx = (x - f.wx0 + off) % 18;
                    let by = (y - f.wall_top) % 11;
                    let hsh = hash2((x - f.wx0 + off) / 18, row, seed);
                    let v = 0.52 + (hsh % 100) as f32 / 100.0 * 0.2 - t * 0.08 + lx;
                    if bx == 0 || by == 0 {
                        ramp_at(st.wall, 0)
                    } else if by == 1 || bx == 1 {
                        shade(st.wall, v + 0.16)
                    } else if by == 10 || bx == 17 {
                        shade(st.wall, v - 0.12)
                    } else if hash2(x, y, seed).is_multiple_of(17) {
                        shade(st.wall, v - 0.07)
                    } else {
                        shade(st.wall, v)
                    }
                }
                Culture::Log => ramp_at(st.wall, [0, 2, 3, 4, 4, 4, 3, 3, 2, 1][((y - f.wall_top) % 10) as usize]),
                Culture::Plaster if y >= f.base_y - 30 => match (f.base_y - y) % 7 {
                    0 => ramp_at(pal::ROOF_BLUE, 1),
                    1 => ramp_at(pal::ROOF_BLUE, 4),
                    _ => ramp_at(pal::ROOF_BLUE, 3),
                },
                Culture::HalfTimbered | Culture::Plaster => {
                    let grain = hash2(x / 2, y, seed).is_multiple_of(23);
                    shade(st.wall, 0.76 - t * 0.12 + lx + if grain { -0.05 } else { 0.0 })
                }
            };
            img.set(x, y, c);
        }
    }
    match st.culture {
        Culture::Log => log_ends(img, f),
        Culture::HalfTimbered => timber_frame(img, f, st.trim),
        _ => {}
    }
    // Corner posts.
    img.rect(f.wx0, f.wall_top, 3, f.wall_h, ramp_at(st.trim, 1));
    img.vline(f.wx0, f.wall_top, f.base_y, ramp_at(st.trim, 2));
    img.rect(f.wx1 - 2, f.wall_top, 3, f.wall_h, ramp_at(st.trim, 0));
}

/// Round log ends stacked at both corners.
fn log_ends(img: &mut Img, f: &Frame) {
    for y in (f.wall_top..f.base_y - 10).step_by(10) {
        for cx in [f.wx0 + 2, f.wx1 - 2] {
            img.ellipse_with(cx as f32 + 0.5, y as f32 + 5.0, 4.5, 4.5, |im, x, yy, nx, ny| {
                let ring = ((nx * nx + ny * ny).sqrt() * 3.5) as i32 % 2 == 0;
                im.set(x, yy, if ring { hex(0xc89a64) } else { hex(0xa87a48) });
            });
        }
    }
}

/// Beams, a post every bay and a diagonal brace in every other bay.
fn timber_frame(img: &mut Img, f: &Frame, trim: Ramp) {
    let tc = ramp_at(trim, 1);
    let tc2 = ramp_at(trim, 2);
    for yy in [f.wall_top + 3, f.base_y - 16] {
        img.rect(f.wx0, yy, f.wx1 - f.wx0 + 1, 4, tc);
        img.hline(f.wx0, f.wx1, yy, tc2);
    }
    for (bay, x) in (f.wx0..=f.wx1 - 3).step_by(34).enumerate() {
        img.rect(x, f.wall_top, 4, f.wall_h, tc);
        img.vline(x, f.wall_top, f.base_y, tc2);
        if bay % 2 == 1 && x + 34 < f.wx1 {
            for k in 0..3 {
                img.line(x + 4 + k, f.base_y - 17, x + 30 + k, f.wall_top + 7, tc);
            }
        }
    }
    img.rect(f.wx1 - 3, f.wall_top, 4, f.wall_h, tc);
}

fn foundation(img: &mut Img, f: &Frame, seed: u32) {
    let base_y = f.base_y;
    for x in f.wx0..=f.wx1 {
        for y in base_y - 11..=base_y {
            let row = (base_y - y) / 6;
            let bx = (x - f.wx0 + row * 7) % 14;
            let by = (base_y - y) % 6;
            let v = 0.45 + (hash2((x - f.wx0 + row * 7) / 14, row, seed) % 100) as f32 / 300.0;
            let c = if bx == 0 || by == 5 || y == base_y - 11 {
                ramp_at(pal::STONE_WALL, 0)
            } else if by == 4 {
                shade(pal::STONE_WALL, v + 0.18)
            } else {
                shade(pal::STONE_WALL, v)
            };
            img.set(x, y, c);
        }
    }
    img.hline(f.wx0, f.wx1, base_y - 12, ramp_at(pal::STONE_WALL, 4));
}

// ----------------------------------------------------------- doors, windows

/// Centred front door with its step and, where it fits, a lantern.
fn front_door(img: &mut Img, f: &Frame, st: &Style, kind: BuildingKind, lit: bool, fx: &mut HouseFx) -> Door {
    let grand = kind == BuildingKind::Inn || kind == BuildingKind::Chapel;
    let (w, h) = if grand { (40, 80) } else { (34, 74) };
    let d = Door { x: f.w / 2 - w / 2, y: f.base_y - h, w, h };
    door(img, d, st.trim, kind == BuildingKind::Chapel || st.culture == Culture::Log);
    img.rect(d.x - 6, f.base_y - 2, d.w + 12, 3, ramp_at(pal::STONE_WALL, 3));
    img.hline(d.x - 6, d.x + d.w + 5, f.base_y - 2, ramp_at(pal::STONE_WALL, 4));
    fx.door = ((d.x + d.w / 2 - f.pivot.0) as i16, 0);

    let lx = d.x + d.w + 6;
    if lx + 9 < f.wx1 - 4 {
        img.rect(lx, d.y + 4, 8, 2, ramp_at(pal::METAL, 1));
        img.rect(lx + 3, d.y + 6, 8, 11, ramp_at(pal::METAL, 0));
        img.rect(lx + 4, d.y + 7, 6, 9, if lit { hex(pal::WINDOW_LIT[3]) } else { hex(pal::WINDOW_LIT[1]) });
        img.rect(lx + 5, d.y + 8, 2, 4, if lit { WHITE } else { hex(pal::WINDOW_LIT[2]) });
        img.hline(lx + 2, lx + 11, d.y + 5, ramp_at(pal::METAL, 2));
        fx.windows.push(f.rel(lx + 7, d.y + 11));
    }
    d
}

fn door(img: &mut Img, d: Door, trim: Ramp, arch: bool) {
    let Door { x, y, w, h } = d;
    // Frame and lintel.
    img.rect(x - 4, y - 5, w + 8, h + 5, ramp_at(trim, 1));
    img.hline(x - 4, x + w + 3, y - 5, ramp_at(trim, 3));
    img.vline(x - 4, y - 5, y + h - 1, ramp_at(trim, 2));
    img.rect(x - 1, y - 1, w + 2, h + 1, ramp_at(trim, 0));
    let r = w as f32 / 2.0;
    for yy in 0..h {
        for xx in 0..w {
            let dx = xx as f32 + 0.5 - r;
            let dy = r - yy as f32 - 0.5;
            let c = if arch && dy > 0.0 && dx * dx + dy * dy > r * r {
                ramp_at(trim, 0)
            } else {
                match xx % 6 {
                    5 => ramp_at(pal::WOOD, 1),
                    0 => ramp_at(pal::WOOD, 4),
                    _ => ramp_at(pal::WOOD, 3),
                }
            };
            img.set(x + xx, y + yy, c);
        }
    }
    // Iron straps with rivets.
    for by in [y + h / 5, y + h * 3 / 5] {
        img.rect(x, by, w, 3, ramp_at(pal::METAL, 1));
        img.hline(x, x + w - 1, by, ramp_at(pal::METAL, 3));
        for rx in (x + 2..x + w - 1).step_by(5) {
            img.set(rx, by + 1, ramp_at(pal::METAL, 4));
        }
    }
    // Ring handle.
    img.ellipse_with((x + w - 5) as f32, (y + h / 2 + 3) as f32, 2.2, 2.2, |im, px, py, nx, ny| {
        if nx * nx + ny * ny > 0.3 {
            im.set(px, py, hex(pal::GOLD[if nx + ny < 0.0 { 3 } else { 1 }]));
        }
    });
    // Shadowed top of the doorway.
    for xx in 0..w {
        if let Some(c) = img.get(x + xx, y) {
            img.set(x + xx, y, mix(c, SHADOW, 0.5));
        }
    }
}

/// How a house dresses its windows.
struct Glazing {
    lit: bool,
    trim: Ramp,
    flowers: Option<[u32; 3]>,
    shutters: bool,
}

/// One window either side of the door, where the wall is wide enough.
fn front_windows(img: &mut Img, f: &Frame, st: &Style, d: Door, lit: bool, r: &mut Rng, fx: &mut HouseFx) {
    let flowers = if r.chance(0.65) { Some(pal::FLOWERS[r.below(pal::FLOWERS.len())]) } else { None };
    let shutters = st.culture == Culture::HalfTimbered || st.culture == Culture::Plaster && r.chance(0.6);
    let glazing = Glazing { lit, trim: st.trim, flowers, shutters };
    let (ww, wh) = (24, 30);
    let win_y = f.base_y - 12 - 32 - wh;
    let need = ww + if shutters { 26 } else { 12 };
    let left_space = d.x - 4 - f.wx0;
    let right_space = f.wx1 - (d.x + d.w + 4);
    let left = (left_space >= need).then(|| f.wx0 + left_space / 2 - ww / 2 + 2);
    let right = (right_space >= need).then(|| d.x + d.w + 4 + right_space / 2 - ww / 2);
    for sx in [left, right].into_iter().flatten() {
        window(img, sx, win_y, ww, wh, &glazing);
        fx.windows.push(f.rel(sx + ww / 2, win_y + wh / 2));
    }
}

fn window(img: &mut Img, x: i32, y: i32, w: i32, h: i32, g: &Glazing) {
    let trim = g.trim;
    // Recessed frame: dark inner reveal, lit outer trim.
    img.rect(x - 3, y - 3, w + 6, h + 6, ramp_at(trim, 1));
    img.hline(x - 3, x + w + 2, y - 3, ramp_at(trim, 3));
    img.vline(x - 3, y - 3, y + h + 2, ramp_at(trim, 2));
    img.rect(x - 1, y - 1, w + 2, h + 2, ramp_at(trim, 0));
    for yy in 0..h {
        for xx in 0..w {
            let c = if g.lit {
                // Lamplight glows brightest at the centre of the pane.
                let d = ((xx * 2 - w + 1).abs() as f32 / w as f32 + (yy * 2 - h + 1).abs() as f32 / h as f32) * 0.5;
                shade(pal::WINDOW_LIT, 0.98 - d * 0.85)
            } else if (xx + (h - yy)) % 11 < 2 && xx < w * 2 / 3 {
                // Sky reflection streaks.
                ramp_at(pal::WINDOW_DARK, 2)
            } else {
                ramp_at(pal::WINDOW_DARK, if yy < h / 3 { 1 } else { 0 })
            };
            img.set(x + xx, y + yy, c);
        }
    }
    if !g.lit {
        // Curtains tied back; at night the glow hides them.
        for yy in 0..h {
            let cw = if yy < h / 2 { 3 } else { 2 + (yy - h / 2) / 4 };
            for k in 0..cw.min(w / 3) {
                img.set(x + k, y + yy, if k == cw - 1 { hex(0x6a2e36) } else { hex(0x8e4048) });
                img.set(x + w - 1 - k, y + yy, if k == cw - 1 { hex(0x5a2630) } else { hex(0x7a3640) });
            }
        }
    }
    // Mullions.
    for k in 0..2 {
        img.vline(x + w / 2 - 1 + k, y, y + h - 1, ramp_at(trim, 2 - k));
        img.hline(x, x + w - 1, y + h / 2 - 1 + k, ramp_at(trim, 2 - k));
    }
    // Sill with a shadow beneath.
    img.rect(x - 4, y + h + 2, w + 8, 3, ramp_at(trim, 3));
    img.hline(x - 4, x + w + 3, y + h + 2, ramp_at(trim, 4));
    img.hline(x - 3, x + w + 2, y + h + 5, ramp_at(trim, 0));
    if g.shutters {
        shutters(img, x, y, w, h, trim);
    }
    if let Some(petals) = g.flowers {
        flower_box(img, x, y, w, h, petals);
    }
}

/// Louvred shutters folded open either side; the left one catches light.
fn shutters(img: &mut Img, x: i32, y: i32, w: i32, h: i32, trim: Ramp) {
    for (sx, light) in [(x - 11, true), (x + w + 4, false)] {
        for yy in -3..h + 3 {
            for xx in 0..7 {
                let slat = (yy + 3) % 5 == 4;
                let c = if xx == 0 || xx == 6 || slat {
                    ramp_at(trim, 1)
                } else if light && xx == 1 {
                    ramp_at(trim, 4)
                } else {
                    ramp_at(trim, 3)
                };
                img.set(sx + xx, y + yy, c);
            }
        }
    }
}

/// Planter under the sill, spilling leaves and blooms over its rim.
fn flower_box(img: &mut Img, x: i32, y: i32, w: i32, h: i32, petals: [u32; 3]) {
    img.rect(x - 3, y + h + 5, w + 6, 6, hex(0x5c3626));
    img.hline(x - 3, x + w + 2, y + h + 5, hex(0x8c5a38));
    img.hline(x - 3, x + w + 2, y + h + 10, hex(0x3a2420));
    for k in 0..w + 6 {
        let leaf = if (k * 7) % 3 == 0 { hex(pal::LEAF_SUMMER[5]) } else { hex(pal::LEAF_SUMMER[3]) };
        img.set(x - 3 + k, y + h + 4, leaf);
        if k % 2 == 0 {
            img.set(x - 3 + k, y + h + 3, hex(pal::LEAF_SUMMER[4]));
        }
        if k % 4 == 1 {
            img.set(x - 3 + k, y + h + 2, hex(petals[1]));
            img.set(x - 3 + k, y + h + 1, hex(petals[2]));
            img.set(x - 2 + k, y + h + 2, hex(petals[0]));
        }
    }
}

/// Chapel rose window: stained glass by day, glowing by night.
fn rose_window(img: &mut Img, f: &Frame, st: &Style, lit: bool, fx: &mut HouseFx) {
    let (cx, cy) = (f.w / 2, f.wall_top + 18);
    img.ellipse_with(cx as f32, cy as f32, 12.0, 12.0, |im, x, y, nx, ny| {
        let d = (nx * nx + ny * ny).sqrt();
        let spoke = ((ny.atan2(nx) * 4.0 / std::f32::consts::PI).fract().abs() < 0.1) && d < 0.82;
        let c = if d > 0.84 {
            ramp_at(st.trim, 1)
        } else if spoke {
            ramp_at(st.trim, 2)
        } else if lit {
            shade(pal::WINDOW_LIT, 1.0 - d * 0.7)
        } else {
            hex([0x3f6496, 0x8c5a98, 0xad4a40, 0x5a9a6a][((x + y) / 3).rem_euclid(4) as usize])
        };
        im.set(x, y, c);
    });
    fx.windows.push(f.rel(cx, cy));
}

/// Wrought-iron bracket and wooden board bearing the trade's emblem.
fn sign_icon(img: &mut Img, kind: BuildingKind, x: i32, y: i32) {
    img.rect(x - 3, y - 7, 28, 3, ramp_at(pal::METAL, 1));
    img.hline(x - 3, x + 24, y - 7, ramp_at(pal::METAL, 3));
    img.vline(x + 2, y - 4, y - 1, ramp_at(pal::METAL, 2));
    img.vline(x + 19, y - 4, y - 1, ramp_at(pal::METAL, 2));
    img.rect(x - 1, y, 24, 18, ramp_at(pal::WOOD, 1));
    img.rect(x, y + 1, 22, 16, ramp_at(pal::WOOD, 3));
    img.hline(x, x + 21, y + 1, ramp_at(pal::WOOD, 4));
    img.hline(x, x + 21, y + 16, ramp_at(pal::WOOD, 2));
    let g = hex(pal::GOLD[2]);
    let g2 = hex(pal::GOLD[3]);
    let g3 = hex(pal::GOLD[1]);
    match kind {
        BuildingKind::Inn => {
            // A frothing tankard.
            img.rect(x + 6, y + 5, 8, 9, g);
            img.vline(x + 6, y + 5, y + 13, g2);
            img.rect(x + 14, y + 7, 3, 5, g3);
            img.rect(x + 15, y + 8, 1, 3, ramp_at(pal::WOOD, 3));
            img.rect(x + 5, y + 3, 10, 3, hex(0xf8f0e0));
        }
        BuildingKind::Store => {
            // A coin purse.
            img.rect(x + 5, y + 7, 12, 8, g);
            img.vline(x + 5, y + 7, y + 14, g2);
            img.hline(x + 8, x + 13, y + 4, g3);
            img.hline(x + 7, x + 14, y + 5, g3);
            img.hline(x + 6, x + 15, y + 6, g3);
            img.rect(x + 10, y + 9, 2, 3, hex(0xfff4c0));
        }
        BuildingKind::Smithy => {
            // An anvil.
            img.rect(x + 3, y + 5, 16, 4, ramp_at(pal::METAL, 3));
            img.hline(x + 3, x + 18, y + 5, ramp_at(pal::METAL, 4));
            img.rect(x + 8, y + 9, 6, 3, ramp_at(pal::METAL, 2));
            img.rect(x + 5, y + 12, 12, 3, ramp_at(pal::METAL, 1));
        }
        BuildingKind::Chapel => {
            img.rect(x + 10, y + 3, 2, 12, g2);
            img.rect(x + 6, y + 6, 10, 2, g2);
        }
        BuildingKind::Home => {}
    }
}

/// Striped awning over the shop door with a scalloped hem.
fn store_awning(img: &mut Img, d: Door, dye: [u32; 4]) {
    for y in 0..10 {
        for x in d.x - 8..d.x + d.w + 8 {
            let stripe = ((x - d.x + 8) / 6) % 2 == 0;
            let c = if stripe { hex(dye[2]) } else { CANVAS };
            let c = if y > 7 {
                mix(c, SHADOW, 0.3)
            } else if y < 2 {
                mix(c, WHITE, 0.2)
            } else {
                c
            };
            img.set(x, d.y - 14 + y, c);
        }
    }
    for x in (d.x - 8..d.x + d.w + 8).step_by(6) {
        img.rect(x + 1, d.y - 4, 4, 2, hex(dye[1]));
    }
}

// -------------------------------------------------------------------- roofs

/// Side-gabled roof: we see its front slope, courses parallel to the eave.
fn side_gabled_roof(img: &mut Img, f: &Frame, st: &Style, roofing: Roofing, seed: u32) {
    let (w, rr, fascia) = (f.w, st.roof, st.trim);
    let (ridge_y, eave_y) = (f.ridge_y, f.eave_y);
    let top = ridge_y + 6;
    for y in top..=eave_y {
        let t = (y - top) as f32 / (eave_y - top) as f32;
        for x in 0..w {
            let lit_k = (0.5 - x as f32 / w as f32) * 0.08;
            img.set(x, y, roof_px(rr, roofing, x, y - top, t, seed, lit_k));
        }
    }
    // Back slope sliver beyond the ridge, in shadow.
    for y in ridge_y..top {
        for x in 3..w - 3 {
            img.set(x, y, shade(rr, 0.28 + (hash2(x / 6, y, seed) % 3) as f32 * 0.03));
        }
    }
    // Ridge cap with rounded segments.
    let last = rr.len() as i32 - 1;
    for x in 2..w - 2 {
        img.set(x, top - 2, ramp_at(rr, last));
        img.set(x, top - 1, if x % 9 == 0 { ramp_at(rr, 1) } else { ramp_at(rr, last - 1) });
        img.set(x, top, ramp_at(rr, 2));
        img.set(x, top + 1, ramp_at(rr, 1));
    }
    // Bargeboards on both ends.
    for y in ridge_y..=eave_y + 2 {
        for k in 0..3 {
            img.set(k, y, ramp_at(fascia, [2, 3, 1][k as usize]));
            img.set(w - 1 - k, y, ramp_at(fascia, [0, 1, 2][k as usize]));
        }
    }
    // Fascia board along the eave.
    img.rect(0, eave_y + 1, w, 4, ramp_at(fascia, 2));
    img.hline(0, w - 1, eave_y + 1, ramp_at(fascia, 3));
    img.hline(0, w - 1, eave_y + 4, ramp_at(fascia, 0));
    if roofing == Roofing::Thatch {
        // Thatch hangs over the fascia in a ragged fringe.
        for x in 0..w {
            let hang = (hash2(x, 3, seed) % 4) as i32;
            for k in 0..hang {
                img.set(x, eave_y + 1 + k, shade(rr, 0.42 - k as f32 * 0.06));
            }
        }
    }
}

/// Front-gabled roof: the gable triangle faces us; two slopes recede back.
fn front_gabled_roof(img: &mut Img, f: &Frame, st: &Style, roofing: Roofing, lit: bool, seed: u32, fx: &mut HouseFx) {
    let (w, rr, fascia) = (f.w, st.roof, st.trim);
    let (cx, apex_y, eave_y, gable_h) = (f.w / 2, f.apex_y(), f.eave_y, f.gable_h);
    let back = f.depth as i32;
    // Left slope (lit) and right slope (shade): parallelograms from the
    // gable edge back by `back` pixels.
    for y in apex_y - back - 2..=eave_y {
        for x in 0..w {
            let left = x < cx;
            let edge_x = if left { cx - x } else { x - cx };
            let half = cx as f32;
            // Height of the gable edge at this column.
            let edge_y = apex_y as f32 + edge_x as f32 / half * gable_h as f32;
            if (y as f32) > edge_y + 1.0 || (y as f32) < edge_y - back as f32 {
                continue;
            }
            let t = (edge_x as f32 / half).clamp(0.0, 1.0);
            let u = (y as f32 - (edge_y - back as f32)) as i32;
            let lit_k = if left { 0.06 } else { -0.16 };
            img.set(x, y, roof_px(rr, roofing, u, edge_x, t, seed, lit_k));
        }
    }
    // Ridge running back from the apex.
    let last = rr.len() as i32 - 1;
    for y in apex_y - back..=apex_y {
        img.set(cx - 1, y, ramp_at(rr, last));
        img.set(cx, y, ramp_at(rr, last - 1));
        img.set(cx + 1, y, ramp_at(rr, 1));
    }
    gable_end(img, f, st, lit, seed, fx);
    // Bargeboards along both slopes, overhanging the walls.
    for k in 0..5 {
        let col = match k {
            0 => ramp_at(fascia, 3),
            4 => ramp_at(fascia, 0),
            _ => ramp_at(fascia, 2),
        };
        img.line(cx, apex_y - 4 + k, 0, eave_y + k - 1, col);
        img.line(cx + 1, apex_y - 4 + k, w - 1, eave_y + k - 1, col);
    }
    // Gilded finial.
    img.rect(cx - 1, apex_y - 6, 3, 3, hex(pal::GOLD[2]));
}

/// The street-facing gable wall with its round window.
fn gable_end(img: &mut Img, f: &Frame, st: &Style, lit: bool, seed: u32, fx: &mut HouseFx) {
    let (cx, apex_y, eave_y, gable_h) = (f.w / 2, f.apex_y(), f.eave_y, f.gable_h);
    let inside = |x: i32, y: i32| -> bool {
        let ex = (x - cx).abs() as f32;
        y > apex_y + 2 && y <= eave_y + 1 && x > f.wx0 && x < f.wx1 && (y - apex_y) as f32 > ex / cx as f32 * gable_h as f32 + 2.0
    };
    for y in apex_y..=eave_y + 1 {
        for x in f.wx0..=f.wx1 {
            if !inside(x, y) {
                continue;
            }
            let c = match st.culture {
                Culture::Log => ramp_at(st.wall, [0, 2, 3, 4, 4, 3, 3, 2, 1][((y - apex_y).rem_euclid(9)) as usize]),
                Culture::Stone => shade(st.wall, 0.55 + (hash2(x / 9, y / 6, seed) % 100) as f32 / 500.0),
                Culture::HalfTimbered | Culture::Plaster => shade(st.wall, 0.8 - (y - apex_y) as f32 / gable_h as f32 * 0.06),
            };
            img.set(x, y, c);
        }
    }
    if matches!(st.culture, Culture::HalfTimbered | Culture::Plaster) {
        // King post, tie beam and struts.
        let tc = ramp_at(st.trim, 1);
        for k in 0..3 {
            img.vline(cx - 1 + k, apex_y + 4, eave_y + 1, if k == 0 { ramp_at(st.trim, 2) } else { tc });
        }
        let mid = apex_y + gable_h * 2 / 3;
        for x in f.wx0..=f.wx1 {
            if inside(x, mid) {
                img.vline(x, mid, mid + 2, tc);
            }
        }
        for k in 0..3 {
            img.line(cx - 3 - k, mid - 1, cx - gable_h / 2, eave_y, tc);
            img.line(cx + 3 + k, mid - 1, cx + gable_h / 2, eave_y, tc);
        }
    }
    let gy = apex_y + gable_h / 2 + 1;
    img.ellipse_with(cx as f32 + 0.5, gy as f32, 7.0, 7.0, |im, x, y, nx, ny| {
        let d = (nx * nx + ny * ny).sqrt();
        let c = if d > 0.78 {
            ramp_at(st.trim, 1)
        } else if lit {
            shade(pal::WINDOW_LIT, 1.0 - d)
        } else {
            ramp_at(pal::WINDOW_DARK, if ny < -0.2 { 2 } else { 0 })
        };
        im.set(x, y, c);
    });
    img.vline(cx, gy - 5, gy + 5, ramp_at(st.trim, 2));
    img.hline(cx - 5, cx + 5, gy, ramp_at(st.trim, 2));
    fx.windows.push(f.rel(cx, gy));
}

/// Shadow cast by the eave onto the wall, fading over four rows.
fn eave_shadow(img: &mut Img, f: &Frame) {
    for x in f.wx0..=f.wx1 {
        for (k, a) in [(5, 0.5), (6, 0.38), (7, 0.26), (8, 0.14)] {
            let y = f.eave_y + k;
            if let Some(c) = img.get(x, y) {
                img.set(x, y, mix(c, SHADOW, a));
            }
        }
    }
}

/// Dormer window breaking a deep side-gabled roof.
fn dormer(img: &mut Img, f: &Frame, st: &Style, lit: bool, fx: &mut HouseFx) {
    let (x, y) = (f.w / 2 - 14, f.ridge_y + f.roof_h / 2 - 6);
    for k in 0..12 {
        img.hline(x - 8 + k, x + 35 - k, y - 8 - k, ramp_at(st.roof, 4 - (k / 4).min(3)));
    }
    img.rect(x - 5, y - 8, 38, 30, ramp_at(st.wall, 3));
    img.vline(x - 5, y - 8, y + 21, ramp_at(st.wall, 4));
    img.vline(x + 32, y - 8, y + 21, ramp_at(st.wall, 1));
    window(img, x + 5, y, 18, 14, &Glazing { lit, trim: st.trim, flowers: None, shutters: false });
    fx.windows.push(f.rel(x + 14, y + 7));
}

/// Brick chimney rising above the ridge, on a random side.
fn chimney(img: &mut Img, f: &Frame, r: &mut Rng, fx: &mut HouseFx) {
    let stone = pal::STONE_WALL;
    let cx = if r.chance(0.5) { f.w / 4 - 4 } else { f.w * 3 / 4 - 12 };
    let ctop = if f.front_gable { f.apex_y() - f.depth as i32 + 4 } else { f.ridge_y - 14 };
    for y in ctop..ctop + 34 {
        for x in cx..cx + 16 {
            let row = (y - ctop) / 5;
            let bx = (x - cx + row * 4) % 8;
            let c = if y < ctop + 4 {
                if y == ctop { ramp_at(stone, 4) } else { ramp_at(stone, 2) }
            } else if bx == 0 || (y - ctop) % 5 == 0 {
                ramp_at(stone, 1)
            } else if x < cx + 4 {
                ramp_at(stone, 3)
            } else if x > cx + 12 {
                ramp_at(stone, 1)
            } else {
                ramp_at(stone, 2)
            };
            img.set(x, y, c);
        }
    }
    // Cap and flue opening.
    img.rect(cx - 2, ctop, 20, 4, ramp_at(stone, 3));
    img.hline(cx - 2, cx + 17, ctop, ramp_at(stone, 4));
    img.rect(cx + 3, ctop + 1, 10, 2, pal::INK_SOFT);
    fx.chimney = Some(f.rel(cx + 8, ctop));
}

/// Chapel bell tower with a golden bell and a spire.
fn bell_tower(img: &mut Img, f: &Frame, st: &Style) {
    let (w, rr) = (f.w, st.roof);
    let (tx, tw) = (w / 2 - 16, 32);
    let ttop = f.ridge_y - 30;
    img.rect(tx, ttop + 18, tw, 34, ramp_at(st.wall, 3));
    img.vline(tx, ttop + 18, ttop + 51, ramp_at(st.wall, 4));
    img.rect(tx + tw - 3, ttop + 18, 3, 34, ramp_at(st.wall, 1));
    img.rect(tx + 8, ttop + 24, 16, 20, pal::INK_SOFT);
    img.ellipse_with(tx as f32 + 16.0, ttop as f32 + 34.0, 6.0, 6.5, |im, x, y, nx, ny| {
        im.set(x, y, hex(pal::GOLD[if nx + ny < -0.3 { 4 } else if nx + ny > 0.5 { 1 } else { 2 }]));
    });
    for k in 0..24 {
        let half = 19 - k * 18 / 23;
        img.hline(w / 2 - half, w / 2 + half - 1, ttop + 18 - k, ramp_at(rr, 4 - (k / 6).min(3)));
        img.set(w / 2 + half - 1, ttop + 18 - k, ramp_at(rr, 1));
    }
    img.vline(w / 2, ttop - 14, ttop - 5, hex(pal::GOLD[3]));
    img.hline(w / 2 - 4, w / 2 + 4, ttop - 11, hex(pal::GOLD[3]));
}

// ---------------------------------------------------------------- furniture

/// Running-bond stonework filling (x0..=x1, y0..=y1), lit from the left.
fn stone_course(img: &mut Img, x0: i32, x1: i32, y0: i32, y1: i32, seed: u32) {
    const BW: i32 = 8;
    const BH: i32 = 5;
    for y in y0..=y1 {
        for x in x0..=x1 {
            let row = (y - y0) / BH;
            let off = if row % 2 == 0 { 0 } else { BW / 2 };
            let bx = (x - x0 + off) % BW;
            let by = (y - y0) % BH;
            let v = 0.5 + (hash2((x - x0 + off) / BW, row, seed) % 100) as f32 / 400.0 - (x - x0) as f32 / (x1 - x0 + 1) as f32 * 0.2;
            let c = if bx == 0 || by == BH - 1 {
                ramp_at(pal::STONE_WALL, 1)
            } else if by == 0 {
                shade(pal::STONE_WALL, v + 0.2)
            } else {
                shade(pal::STONE_WALL, v)
            };
            img.set(x, y, c);
        }
    }
}

/// Village well, roofed to match the local houses.
pub fn well(culture: u8) -> (Img, (i32, i32)) {
    let mut img = Img::new(48, 64);
    let roof = match Culture::of(culture) {
        Culture::HalfTimbered => pal::ROOF_RED,
        Culture::Stone => pal::ROOF_SLATE,
        Culture::Plaster => pal::ROOF_BLUE,
        Culture::Log => pal::ROOF_GREEN,
    };
    stone_course(&mut img, 6, 41, 40, 59, 91);
    // Rim and water.
    img.ellipse_with(24.0, 40.5, 17.0, 5.0, |im, x, y, nx, ny| {
        let d = (nx * nx + ny * ny).sqrt();
        let c = if d > 0.78 {
            ramp_at(pal::STONE_WALL, 4)
        } else if d > 0.65 {
            ramp_at(pal::STONE_WALL, 2)
        } else if ny < -0.2 {
            ramp_at(pal::WATER, 0)
        } else {
            ramp_at(pal::WATER, 2)
        };
        im.set(x, y, c);
    });
    img.set(20, 41, hex(pal::WATER[6]));
    img.set(21, 41, hex(pal::WATER[5]));
    // Posts, crossbar, bucket.
    img.rect(8, 14, 4, 27, ramp_at(pal::WOOD, 3));
    img.vline(8, 14, 40, ramp_at(pal::WOOD, 4));
    img.rect(36, 14, 4, 27, ramp_at(pal::WOOD, 2));
    img.vline(39, 14, 40, ramp_at(pal::WOOD, 1));
    img.rect(10, 20, 28, 3, ramp_at(pal::WOOD, 3));
    img.hline(10, 37, 20, ramp_at(pal::WOOD, 4));
    img.vline(24, 23, 32, ramp_at(pal::METAL, 2));
    img.rect(20, 32, 8, 6, ramp_at(pal::WOOD, 3));
    img.hline(20, 27, 33, ramp_at(pal::METAL, 1));
    img.hline(20, 27, 36, ramp_at(pal::METAL, 1));
    img.vline(20, 32, 37, ramp_at(pal::WOOD, 4));
    // Roof.
    for k in 0..14 {
        let y = 2 + k;
        let inset = 13 - k;
        for x in inset..48 - inset {
            let shingle = (x + if (k / 3) % 2 == 0 { 0 } else { 3 }) % 6 == 0 || k % 3 == 2;
            let v = 0.8 - k as f32 * 0.035 - if x > 24 { 0.12 } else { 0.0 } - if shingle { 0.12 } else { 0.0 };
            img.set(x, y, shade(roof, v));
        }
    }
    img.hline(0, 47, 15, ramp_at(roof, 0));
    img.outline(pal::INK);
    (img, (24, 59))
}

/// Street lamp, always drawn lit.
pub fn lamp() -> (Img, (i32, i32)) {
    let mut img = Img::new(26, 100);
    let m = pal::METAL;
    // Post with a fluted shaft and a stepped base.
    img.rect(11, 24, 5, 68, ramp_at(m, 1));
    img.vline(11, 24, 91, ramp_at(m, 3));
    img.vline(12, 24, 91, ramp_at(m, 2));
    img.vline(15, 24, 91, ramp_at(m, 0));
    img.rect(8, 88, 11, 4, ramp_at(m, 1));
    img.hline(8, 18, 88, ramp_at(m, 3));
    img.rect(6, 92, 15, 5, ramp_at(m, 1));
    img.hline(6, 20, 92, ramp_at(m, 3));
    img.rect(9, 50, 9, 3, ramp_at(m, 2));
    img.rect(9, 26, 9, 3, ramp_at(m, 2));
    // Lantern cage with glowing glass.
    img.rect(5, 5, 17, 20, ramp_at(m, 0));
    img.rect(7, 8, 13, 15, hex(pal::WINDOW_LIT[2]));
    img.rect(8, 9, 5, 9, hex(pal::WINDOW_LIT[3]));
    img.rect(9, 10, 2, 5, hex(0xfffbe8));
    img.vline(13, 8, 22, ramp_at(m, 0));
    for k in 0..5 {
        img.hline(4 + k, 22 - k, 4 - k, ramp_at(m, 2 + (k == 0) as i32));
    }
    img.rect(12, -2, 3, 3, ramp_at(m, 2));
    img.hline(4, 22, 25, ramp_at(m, 2));
    img.outline(pal::INK);
    (img, (13, 96))
}

/// The goddess: a robed statue with folded hands on a stepped pedestal.
pub fn shrine() -> (Img, (i32, i32)) {
    let mut img = Img::new(52, 104);
    let stone = pal::STONE_WALL;
    stone_course(&mut img, 4, 47, 84, 99, 17);
    img.rect(9, 74, 34, 10, ramp_at(stone, 3));
    img.hline(9, 42, 74, ramp_at(stone, 4));
    img.vline(42, 74, 83, ramp_at(stone, 1));
    let p = pal::PLASTER;
    // Robe flaring to the pedestal.
    for k in 0..44 {
        let half = 5 + k * 9 / 43;
        for x in 26 - half..26 + half {
            let fold = (x - 26 + half) % 5 == 3 && k > 10;
            let c = if x < 26 - half + 3 {
                ramp_at(p, 4)
            } else if x > 26 + half - 4 || fold {
                ramp_at(p, 2)
            } else {
                ramp_at(p, 3)
            };
            img.set(x, 30 + k, c);
        }
    }
    // Sleeves and folded hands.
    img.ellipse_with(26.0, 42.0, 9.0, 4.0, |im, x, y, nx, _| im.set(x, y, ramp_at(p, if nx < -0.2 { 4 } else { 3 })));
    img.ellipse_with(26.0, 42.0, 3.0, 2.5, |im, x, y, _, _| im.set(x, y, ramp_at(p, 4)));
    // Head with veil.
    img.ellipse_with(26.0, 21.0, 8.0, 9.0, |im, x, y, nx, ny| {
        im.set(x, y, ramp_at(p, if nx + ny < -0.4 { 4 } else if nx + ny > 0.6 { 2 } else { 3 }));
    });
    for x in 17..36 {
        let len = if (20..=32).contains(&x) { 4 } else { 18 };
        img.vline(x, 13, 13 + len, ramp_at(p, if x < 22 { 4 } else if x > 31 { 2 } else { 3 }));
    }
    // Closed eyes.
    img.hline(22, 24, 23, ramp_at(p, 1));
    img.hline(28, 30, 23, ramp_at(p, 1));
    // Halo.
    img.ellipse_with(26.0, 12.0, 14.0, 4.0, |im, x, y, nx, ny| {
        if nx * nx + ny * ny > 0.6 {
            im.set(x, y, hex(pal::GOLD[if nx < 0.0 { 4 } else { 3 }]));
        }
    });
    img.rect(25, 48, 3, 3, hex(pal::GOLD[3]));
    // Offerings: flowers and a candle.
    for (x, c) in [(12, 0xd24c40u32), (15, 0xf4a6c0), (37, 0xeec04a)] {
        img.rect(x, 70, 3, 3, hex(c));
        img.set(x + 1, 73, hex(pal::LEAF_SUMMER[4]));
    }
    img.rect(39, 66, 3, 7, hex(0xf0e6d0));
    img.set(40, 64, hex(pal::WINDOW_LIT[3]));
    img.set(40, 65, hex(pal::WINDOW_LIT[2]));
    img.outline(pal::INK);
    (img, (26, 99))
}

/// Small props by name. `variant` picks the cloth dye of tents and stalls;
/// unknown names give an empty 1x1 sprite.
pub fn small_prop(kind: &str, variant: u8) -> (Img, (i32, i32)) {
    let dye = pal::CLOTH[(variant as usize) % AWNING_CLOTHS];
    match kind {
        "barrel" => barrel(),
        "crate" => wooden_crate(),
        "haystack" => haystack(),
        "bench" => bench(),
        "fence" => fence(),
        "tent" => tent(dye),
        "campfire" => campfire(),
        "stall" => stall(dye),
        _ => (Img::new(1, 1), (0, 0)),
    }
}

fn barrel() -> (Img, (i32, i32)) {
    let mut img = Img::new(24, 30);
    for y in 3..28 {
        let bulge = if (8..22).contains(&y) { 0 } else if (5..25).contains(&y) { 1 } else { 2 };
        for x in 2 + bulge..22 - bulge {
            let hoop = y == 7 || y == 8 || y == 21 || y == 22;
            let c = if hoop {
                if x < 7 { ramp_at(pal::METAL, 2) } else { ramp_at(pal::METAL, 1) }
            } else if (x - 2) % 5 == 0 {
                ramp_at(pal::WOOD, 1)
            } else if x < 7 {
                ramp_at(pal::WOOD, 4)
            } else if x > 16 {
                ramp_at(pal::WOOD, 2)
            } else {
                ramp_at(pal::WOOD, 3)
            };
            img.set(x, y, c);
        }
    }
    img.ellipse_with(12.0, 3.5, 8.5, 3.0, |im, x, y, nx, _| im.set(x, y, if nx < -0.3 { ramp_at(pal::WOOD, 5) } else { ramp_at(pal::WOOD, 4) }));
    img.outline(pal::INK);
    (img, (12, 27))
}

fn wooden_crate() -> (Img, (i32, i32)) {
    let mut img = Img::new(28, 28);
    img.rect(2, 2, 24, 24, ramp_at(pal::WOOD, 3));
    for y in [9, 16] {
        img.hline(4, 23, y, ramp_at(pal::WOOD, 2));
    }
    img.rect(2, 2, 24, 4, ramp_at(pal::WOOD, 4));
    img.rect(2, 2, 4, 24, ramp_at(pal::WOOD, 4));
    img.rect(22, 2, 4, 24, ramp_at(pal::WOOD, 2));
    img.rect(2, 22, 24, 4, ramp_at(pal::WOOD, 2));
    for k in 0..2 {
        img.line(6 + k, 6, 21 + k, 21, ramp_at(pal::WOOD, 1));
    }
    img.outline(pal::INK);
    (img, (14, 25))
}

fn haystack() -> (Img, (i32, i32)) {
    let mut img = Img::new(40, 32);
    img.ellipse_with(20.0, 18.0, 18.0, 13.0, |im, x, y, nx, ny| {
        let streak = match hash2(x, y / 3, 77) % 4 {
            0 => 0.14,
            1 => -0.05,
            _ => 0.0,
        };
        im.set(x, y, shade(pal::ROOF_THATCH, 0.72 - nx * 0.2 - ny * 0.28 - streak));
    });
    for k in 0..6 {
        img.line(6 + k * 5, 26, 4 + k * 5, 30, ramp_at(pal::ROOF_THATCH, 2));
    }
    img.outline(pal::INK);
    (img, (20, 29))
}

fn bench() -> (Img, (i32, i32)) {
    let mut img = Img::new(40, 24);
    img.rect(2, 3, 36, 4, ramp_at(pal::WOOD, 3));
    img.hline(2, 37, 3, ramp_at(pal::WOOD, 4));
    img.rect(2, 11, 36, 4, ramp_at(pal::WOOD, 4));
    img.hline(2, 37, 11, ramp_at(pal::WOOD, 5));
    img.hline(2, 37, 15, ramp_at(pal::WOOD, 1));
    for x in [4, 33] {
        img.rect(x, 7, 3, 4, ramp_at(pal::WOOD, 2));
        img.rect(x, 15, 3, 7, ramp_at(pal::WOOD, 1));
    }
    img.outline(pal::INK);
    (img, (20, 21))
}

fn fence() -> (Img, (i32, i32)) {
    let mut img = Img::new(32, 28);
    for x in [3, 23] {
        img.rect(x, 3, 6, 23, ramp_at(pal::WOOD, 3));
        img.vline(x, 3, 25, ramp_at(pal::WOOD, 4));
        img.vline(x + 5, 3, 25, ramp_at(pal::WOOD, 1));
        img.hline(x, x + 5, 3, ramp_at(pal::WOOD, 5));
    }
    for y in [8, 16] {
        img.rect(0, y, 32, 4, ramp_at(pal::WOOD, 3));
        img.hline(0, 31, y, ramp_at(pal::WOOD, 4));
        img.hline(0, 31, y + 3, ramp_at(pal::WOOD, 1));
    }
    img.outline(pal::INK);
    (img, (16, 25))
}

fn tent(cloth: [u32; 4]) -> (Img, (i32, i32)) {
    let mut img = Img::new(72, 56);
    for k in 0..44 {
        let half = 3 + k * 31 / 43;
        for x in 36 - half..36 + half {
            let seam = (x - 36_i32).abs() % 10 == 0;
            let c = if seam { hex(cloth[1]) } else if x < 36 { hex(cloth[3]) } else { hex(cloth[2]) };
            img.set(x, 8 + k, c);
        }
    }
    // Dark open doorway.
    for k in 0..20 {
        let half = k * 9 / 19;
        img.hline(36 - half, 36 + half, 51 - (19 - k), hex(0x2a1c24));
    }
    img.vline(36, 32, 51, hex(cloth[1]));
    img.rect(35, 2, 2, 8, ramp_at(pal::WOOD, 3));
    img.outline(pal::INK);
    (img, (36, 52))
}

/// Unlit ring of stones and crossed logs; the fire's glow is a renderer light.
fn campfire() -> (Img, (i32, i32)) {
    let mut img = Img::new(32, 24);
    for (x, y) in [(3, 17), (26, 17), (8, 20), (21, 20), (14, 21)] {
        img.ellipse_with(x as f32 + 2.5, y as f32 + 1.5, 3.0, 2.0, |im, px, py, nx, ny| {
            im.set(px, py, ramp_at(pal::STONE_WALL, if nx + ny < -0.3 { 4 } else { 2 }));
        });
    }
    for k in 0..2 {
        img.line(6, 18 + k, 25, 11 + k, ramp_at(pal::WOOD, 2 + k));
        img.line(6, 11 + k, 25, 18 + k, ramp_at(pal::WOOD, 3 + k));
    }
    img.outline(pal::INK);
    (img, (16, 20))
}

/// Market stall: counter, striped awning and a row of produce.
fn stall(awn: [u32; 4]) -> (Img, (i32, i32)) {
    let mut img = Img::new(80, 68);
    img.rect(6, 40, 68, 24, ramp_at(pal::WOOD, 3));
    for y in (44..64).step_by(5) {
        img.hline(6, 73, y, ramp_at(pal::WOOD, 2));
    }
    img.rect(6, 38, 68, 4, ramp_at(pal::WOOD, 4));
    img.rect(8, 16, 4, 26, ramp_at(pal::WOOD, 2));
    img.rect(68, 16, 4, 26, ramp_at(pal::WOOD, 1));
    for y in 4..20 {
        for x in 2..78 {
            let c = if (x / 8) % 2 == 0 { hex(awn[2]) } else { CANVAS };
            let c = if y > 15 {
                mix(c, SHADOW, 0.25)
            } else if y < 6 {
                mix(c, WHITE, 0.15)
            } else {
                c
            };
            img.set(x, y, c);
        }
    }
    for x in (2..78).step_by(8) {
        img.rect(x + 2, 20, 4, 2, hex(awn[1]));
    }
    let produce = [0xd24c40u32, 0xeec04a, 0x7aa86c, 0xe0802a, 0x9a5ec4, 0xd24c40, 0xc87ac0, 0xeec04a];
    for (k, c) in produce.into_iter().enumerate() {
        img.ellipse_with(12.0 + k as f32 * 8.0, 36.0, 3.5, 3.0, |im, x, y, nx, ny| {
            let (toward, t) = if nx + ny < -0.3 { (WHITE, 0.3) } else { (SHADOW, 0.15) };
            im.set(x, y, mix(hex(c), toward, t));
        });
    }
    img.outline(pal::INK);
    (img, (40, 64))
}
