//! Buildings and village furniture. Every house is generated from its own
//! seed, so no two are identical, while each village keeps a consistent
//! architectural culture.

use super::canvas::Img;
use super::flora::shade;
use crate::palette::{self as pal, hex, ramp_at, Ramp, Rgb};
use crate::rng::{hash2, Rng};
use crate::world::villages::BuildingKind;

/// Visual metadata the renderer needs for lights and smoke, in pixels
/// relative to the sprite pivot (bottom-left of the footprint).
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct HouseFx {
    pub windows: Vec<(i16, i16)>,
    pub chimney: Option<(i16, i16)>,
    pub door: (i16, i16),
}

struct Style {
    wall: Ramp,
    roof: Ramp,
    trim: Ramp,
    timber: bool,
    stone_wall: bool,
    logs: bool,
}

fn style(culture: u8, r: &mut Rng) -> Style {
    match culture % 4 {
        0 => Style {
            wall: pal::PLASTER,
            roof: *r.pick(&[pal::ROOF_RED, pal::ROOF_THATCH, pal::ROOF_RED]),
            trim: pal::TIMBER,
            timber: true,
            stone_wall: false,
            logs: false,
        },
        1 => Style {
            wall: pal::STONE_WALL,
            roof: *r.pick(&[pal::ROOF_SLATE, pal::ROOF_SLATE, pal::ROOF_RED]),
            trim: pal::TIMBER,
            timber: false,
            stone_wall: true,
            logs: false,
        },
        2 => Style {
            wall: pal::PLASTER,
            roof: *r.pick(&[pal::ROOF_BLUE, pal::ROOF_BLUE, pal::ROOF_THATCH]),
            trim: pal::WOOD,
            timber: false,
            stone_wall: false,
            logs: false,
        },
        _ => Style {
            wall: pal::WOOD,
            roof: *r.pick(&[pal::ROOF_GREEN, pal::ROOF_THATCH, pal::ROOF_GREEN]),
            trim: pal::TIMBER,
            timber: false,
            stone_wall: false,
            logs: true,
        },
    }
}

fn window(img: &mut Img, x: i32, y: i32, w: i32, h: i32, lit: bool, trim: Ramp, flowers: Option<[u32; 3]>, shutters: bool) {
    // Frame
    img.rect(x - 2, y - 2, w + 4, h + 4, ramp_at(trim, 1));
    img.hline(x - 2, x + w + 1, y - 2, ramp_at(trim, 2));
    let glass = if lit { pal::WINDOW_LIT } else { pal::WINDOW_DARK };
    for yy in 0..h {
        for xx in 0..w {
            let c = if lit {
                let d = ((xx * 2 - w + 1).abs() + (yy * 2 - h + 1).abs()) as f32 / (w + h) as f32;
                shade(glass, 0.98 - d * 0.9)
            } else if (xx + yy) % 9 == 0 && xx + yy < w {
                ramp_at(pal::WINDOW_DARK, 2)
            } else {
                ramp_at(glass, if yy < h / 3 { 1 } else { 0 })
            };
            img.set(x + xx, y + yy, c);
        }
    }
    // Curtains peeking in when dark
    if !lit {
        img.vline(x, y, y + h - 1, hex(0x8a4a4a));
        img.vline(x + w - 1, y, y + h - 1, hex(0x7a3a3a));
    }
    // Mullions
    img.vline(x + w / 2, y, y + h - 1, ramp_at(trim, 2));
    img.hline(x, x + w - 1, y + h / 2, ramp_at(trim, 2));
    // Sill
    img.hline(x - 3, x + w + 2, y + h + 2, ramp_at(trim, 3));
    img.hline(x - 3, x + w + 2, y + h + 3, ramp_at(trim, 1));
    if shutters {
        for (sx, light) in [(x - 8, true), (x + w + 3, false)] {
            for yy in -2..h + 2 {
                for xx in 0..5 {
                    let c = if yy % 4 == 0 { ramp_at(trim, 1) } else if light && xx == 0 { ramp_at(trim, 3) } else { ramp_at(trim, 2) };
                    img.set(sx + xx, y + yy, c);
                }
            }
        }
    }
    if let Some(f) = flowers {
        img.rect(x - 2, y + h + 4, w + 4, 4, hex(0x6b412f));
        img.hline(x - 2, x + w + 1, y + h + 4, hex(0x8c5a38));
        for k in 0..w + 4 {
            let leaf = if k % 2 == 0 { hex(pal::LEAF_SUMMER[4]) } else { hex(pal::LEAF_SUMMER[3]) };
            img.set(x - 2 + k, y + h + 3, leaf);
            if k % 3 == 1 {
                img.set(x - 2 + k, y + h + 2, hex(f[1]));
                img.set(x - 2 + k, y + h + 1, hex(f[2]));
            }
        }
    }
}

fn door(img: &mut Img, x: i32, y: i32, w: i32, h: i32, trim: Ramp, arch: bool) {
    img.rect(x - 2, y - 2, w + 4, h + 2, ramp_at(trim, 0));
    img.hline(x - 2, x + w + 1, y - 2, ramp_at(trim, 2));
    for yy in 0..h {
        for xx in 0..w {
            if arch && yy < 3 && (xx < 3 - yy || xx >= w - 3 + yy) {
                img.set(x + xx, y + yy, ramp_at(trim, 0));
                continue;
            }
            let plank = xx % 5 == 4;
            let c = if plank { ramp_at(pal::WOOD, 1) } else if xx % 5 == 0 { ramp_at(pal::WOOD, 4) } else { ramp_at(pal::WOOD, 3) };
            img.set(x + xx, y + yy, c);
        }
    }
    for by in [y + h / 4, y + h * 3 / 4] {
        img.hline(x, x + w - 1, by, ramp_at(pal::METAL, 1));
        img.hline(x, x + w - 1, by + 1, ramp_at(pal::METAL, 0));
        img.set(x + 1, by, ramp_at(pal::METAL, 3));
    }
    img.rect(x + w - 4, y + h / 2, 2, 2, hex(pal::GOLD[3]));
    img.set(x + w - 4, y + h / 2, hex(pal::GOLD[4]));
}

fn sign_icon(img: &mut Img, kind: BuildingKind, x: i32, y: i32) {
    // Bracket and hanging board with an emblem.
    img.hline(x - 2, x + 17, y - 4, ramp_at(pal::METAL, 1));
    img.hline(x - 2, x + 17, y - 3, ramp_at(pal::METAL, 0));
    img.vline(x + 1, y - 3, y - 1, ramp_at(pal::METAL, 2));
    img.vline(x + 14, y - 3, y - 1, ramp_at(pal::METAL, 2));
    img.rect(x - 1, y, 18, 14, ramp_at(pal::WOOD, 1));
    img.rect(x, y + 1, 16, 12, ramp_at(pal::WOOD, 3));
    img.hline(x, x + 15, y + 1, ramp_at(pal::WOOD, 4));
    let g = hex(pal::GOLD[2]);
    let g2 = hex(pal::GOLD[3]);
    let g3 = hex(pal::GOLD[1]);
    match kind {
        BuildingKind::Inn => {
            img.rect(x + 4, y + 4, 6, 7, g);
            img.vline(x + 4, y + 4, y + 10, g2);
            img.rect(x + 10, y + 5, 2, 4, g3);
            img.hline(x + 4, x + 9, y + 3, hex(0xf8f0e0));
            img.hline(x + 3, x + 10, y + 2, hex(0xf8f0e0));
        }
        BuildingKind::Store => {
            img.rect(x + 4, y + 5, 8, 6, g);
            img.vline(x + 4, y + 5, y + 10, g2);
            img.hline(x + 6, x + 9, y + 3, g3);
            img.hline(x + 5, x + 10, y + 4, g3);
            img.set(x + 8, y + 7, hex(0xfff4c0));
        }
        BuildingKind::Smithy => {
            img.rect(x + 2, y + 4, 12, 3, ramp_at(pal::METAL, 3));
            img.hline(x + 2, x + 13, y + 4, ramp_at(pal::METAL, 4));
            img.rect(x + 6, y + 7, 4, 3, ramp_at(pal::METAL, 2));
            img.rect(x + 4, y + 10, 8, 2, ramp_at(pal::METAL, 1));
        }
        BuildingKind::Chapel => {
            img.vline(x + 8, y + 2, y + 11, g2);
            img.vline(x + 7, y + 2, y + 11, g);
            img.hline(x + 4, x + 11, y + 5, g2);
            img.hline(x + 4, x + 11, y + 6, g);
        }
        BuildingKind::Home => {}
    }
}

/// Generate a building sprite. Returns the image, its pivot (bottom-left of
/// the footprint, in sprite pixels) and fx metadata.
pub fn house(kind: BuildingKind, culture: u8, seed: u32, fw: i32, fh: i32, lit: bool) -> (Img, (i32, i32), HouseFx) {
    let mut r = Rng::new(seed as u64 ^ 0x40053);
    let st = style(culture, &mut r);
    let over = 6;
    let w = fw * 32 + over * 2;
    let wall_h = match kind {
        BuildingKind::Inn => 70,
        BuildingKind::Chapel => 76,
        _ => 58,
    };
    let front_gable = kind != BuildingKind::Inn && match culture % 4 { 0 => r.chance(0.55), 1 => r.chance(0.4), 2 => r.chance(0.2), _ => r.chance(0.7) };
    let roof_h = fh * 32 - if front_gable { 4 } else { 26 } + if kind == BuildingKind::Chapel { 14 } else { 0 };
    let extra_top = if kind == BuildingKind::Chapel { 40 } else { 18 };
    let h = wall_h + roof_h + extra_top;
    let mut img = Img::new(w, h);
    let base_y = h - 1;
    let wall_top = base_y - wall_h + 1;
    let wx0 = over;
    let wx1 = w - over - 1;
    let mut fx = HouseFx::default();
    let pivot = (over, h);

    // ---- Wall ----
    for y in wall_top..=base_y {
        for x in wx0..=wx1 {
            let t = (y - wall_top) as f32 / wall_h as f32;
            let c;
            if st.stone_wall {
                let row = (y - wall_top) / 9;
                let off = if row % 2 == 0 { 0 } else { 7 };
                let bx = (x - wx0 + off) % 14;
                let by = (y - wall_top) % 9;
                let hsh = hash2((x - wx0 + off) / 14, row, seed);
                let v = 0.5 + (hsh % 100) as f32 / 100.0 * 0.25 - t * 0.1;
                c = if bx == 0 || by == 0 {
                    ramp_at(st.wall, 0)
                } else if by == 1 || bx == 1 {
                    shade(st.wall, v + 0.18)
                } else if by == 8 || bx == 13 {
                    shade(st.wall, v - 0.12)
                } else if hash2(x, y, seed) % 13 == 0 {
                    shade(st.wall, v - 0.08)
                } else {
                    shade(st.wall, v)
                };
            } else if st.logs {
                let row = (y - wall_top) % 7;
                c = match row {
                    0 => ramp_at(st.wall, 0),
                    1 => ramp_at(st.wall, 2),
                    2 => ramp_at(st.wall, 4),
                    3 => ramp_at(st.wall, 4),
                    4 => ramp_at(st.wall, 3),
                    5 => ramp_at(st.wall, 3),
                    _ => ramp_at(st.wall, 2),
                };
            } else {
                let grain = hash2(x, y, seed) % 17 == 0;
                let v = 0.74 - t * 0.14 + if grain { -0.06 } else { 0.0 };
                let mut cc = shade(st.wall, v);
                if culture % 4 == 2 && y >= base_y - 20 {
                    let row = (y - base_y).rem_euclid(5);
                    cc = if row == 0 { ramp_at(pal::ROOF_BLUE, 1) } else if row == 1 { ramp_at(pal::ROOF_BLUE, 4) } else { ramp_at(pal::ROOF_BLUE, 3) };
                }
                c = cc;
            }
            img.set(x, y, c);
        }
    }
    // Log ends at corners
    if st.logs {
        for y in (wall_top..base_y - 6).step_by(7) {
            for (cx, flip) in [(wx0 + 1, false), (wx1 - 1, true)] {
                img.ellipse_with(cx as f32 + 0.5, y as f32 + 3.5, 3.0, 3.0, |im, x, yy, nx, ny| {
                    let ring = ((nx * nx + ny * ny).sqrt() * 3.0) as i32 % 2 == 0;
                    let _ = flip;
                    im.set(x, yy, if ring { hex(0xc89a64) } else { hex(0xa87a48) });
                });
            }
        }
    }
    // Timber framing
    if st.timber {
        let tc = ramp_at(st.trim, 1);
        let tc2 = ramp_at(st.trim, 2);
        for yy in [wall_top + 2, base_y - 10] {
            img.hline(wx0, wx1, yy, tc);
            img.hline(wx0, wx1, yy + 1, tc);
            img.hline(wx0, wx1, yy + 2, tc2);
        }
        let mut x = wx0;
        let mut bay = 0;
        while x <= wx1 {
            for k in 0..3 {
                img.vline(x + k, wall_top, base_y, if k == 2 { tc2 } else { tc });
            }
            if bay % 2 == 0 && x + 24 < wx1 {
                for k in 0..2 {
                    img.line(x + 3 + k, base_y - 11, x + 22 + k, wall_top + 5, tc);
                }
            }
            x += 24;
            bay += 1;
        }
    }
    // Corner posts and stone footing
    for k in 0..2 {
        img.vline(wx0 + k, wall_top, base_y, ramp_at(st.trim, k));
        img.vline(wx1 - k, wall_top, base_y, ramp_at(st.trim, 0));
    }
    for x in wx0..=wx1 {
        for y in base_y - 7..=base_y {
            let row = (base_y - y) / 4;
            let bx = (x - wx0 + row * 5) % 10;
            let v = 0.45 + (hash2((x - wx0 + row * 5) / 10, row, seed) % 100) as f32 / 300.0;
            let c = if bx == 0 || y == base_y - 7 || (base_y - y) % 4 == 3 { ramp_at(pal::STONE_WALL, 0) } else if (base_y - y) % 4 == 2 { shade(pal::STONE_WALL, v + 0.15) } else { shade(pal::STONE_WALL, v) };
            img.set(x, y, c);
        }
    }

    // ---- Door ----
    let (dw, dh) = if kind == BuildingKind::Inn || kind == BuildingKind::Chapel { (22, 34) } else { (18, 30) };
    let dx = w / 2 - dw / 2;
    let dy = base_y - dh + 1;
    door(&mut img, dx, dy, dw, dh, st.trim, kind == BuildingKind::Chapel || culture % 4 == 3);
    // Step
    img.rect(dx - 4, base_y - 1, dw + 8, 2, ramp_at(pal::STONE_WALL, 3));
    img.hline(dx - 4, dx + dw + 3, base_y - 1, ramp_at(pal::STONE_WALL, 4));
    fx.door = ((dx + dw / 2 - pivot.0) as i16, 0);
    // Lantern beside the door
    let lx = dx + dw + 4;
    if lx + 6 < wx1 && kind != BuildingKind::Home || r.chance(0.5) && lx + 6 < wx1 {
        img.hline(lx, lx + 4, dy + 2, ramp_at(pal::METAL, 1));
        img.rect(lx + 2, dy + 3, 5, 7, ramp_at(pal::METAL, 0));
        img.rect(lx + 3, dy + 4, 3, 5, if lit { hex(pal::WINDOW_LIT[3]) } else { hex(pal::WINDOW_LIT[1]) });
        fx.windows.push(((lx + 4 - pivot.0) as i16, (dy + 6 - pivot.1) as i16));
    }

    // ---- Windows ----
    let flowers = if r.chance(0.65) { Some(pal::FLOWERS[r.below(pal::FLOWERS.len())]) } else { None };
    let shutters = culture % 4 == 0 || culture % 4 == 2 && r.chance(0.6);
    let win_y = wall_top + if wall_h > 64 { 18 } else { 14 };
    let (ww, wh) = (14, 16);
    let mut slots: Vec<i32> = Vec::new();
    let left_space = dx - wx0;
    let right_space = wx1 - (dx + dw);
    let need = ww + if shutters { 18 } else { 8 };
    if left_space >= need {
        slots.push(wx0 + left_space / 2 - ww / 2);
    }
    if right_space >= need {
        slots.push(dx + dw + 1 + right_space / 2 - ww / 2);
    }
    if kind == BuildingKind::Chapel {
        let (cx, cy) = (w / 2, wall_top + 13);
        img.ellipse_with(cx as f32, cy as f32 + 0.5, 9.0, 9.0, |im, x, y, nx, ny| {
            let d = (nx * nx + ny * ny).sqrt();
            let spoke = ((ny.atan2(nx) * 4.0 / std::f32::consts::PI).fract().abs() < 0.12) && d < 0.8;
            let c = if d > 0.82 {
                ramp_at(st.trim, 1)
            } else if spoke {
                ramp_at(st.trim, 2)
            } else if lit {
                shade(pal::WINDOW_LIT, 1.0 - d * 0.7)
            } else if ((x + y) / 2) % 3 == 0 {
                hex(0x3f6496)
            } else if ((x + y) / 2) % 3 == 1 {
                hex(0x8c5a98)
            } else {
                hex(0xad4a40)
            };
            im.set(x, y, c);
        });
        fx.windows.push(((cx - pivot.0) as i16, (cy - pivot.1) as i16));
    }
    for sx in slots {
        window(&mut img, sx, win_y, ww, wh, lit, st.trim, flowers, shutters);
        fx.windows.push(((sx + ww / 2 - pivot.0) as i16, (win_y + wh / 2 - pivot.1) as i16));
    }

    // ---- Sign / awning ----
    if kind != BuildingKind::Home {
        let sx = dx - 22;
        if sx > wx0 + 4 {
            sign_icon(&mut img, kind, sx, dy + 2);
        }
    }
    if kind == BuildingKind::Store {
        let aw = pal::CLOTH[r.below(pal::CLOTH.len())];
        for y in 0..8 {
            for x in dx - 6..dx + dw + 6 {
                let stripe = ((x - dx + 6) / 5) % 2 == 0;
                let c = if stripe { hex(aw[2]) } else { hex(0xede3d0) };
                img.set(x, dy - 10 + y, if y > 5 { crate::palette::mix(c, hex(0x24203c), 0.3) } else { c });
            }
        }
        for x in (dx - 6..dx + dw + 6).step_by(5) {
            img.rect(x, dy - 2, 3, 2, hex(aw[1]));
        }
    }

    // ---- Roof ----
    let roof_bottom = wall_top + 4;
    let roof_top = roof_bottom - roof_h;
    let rr = st.roof;
    let thatch = rr == pal::ROOF_THATCH;
    let cxr = w / 2;
    // Front gable: triangle of wall rising into the roof.
    let gable_h = if front_gable { ((wx1 - wx0) as f32 * 0.42) as i32 } else { 0 };
    let apex_y = roof_bottom - gable_h;
    let in_gable = |x: i32, y: i32| front_gable && y > apex_y && y <= roof_bottom && (x - cxr).abs() * gable_h < (y - apex_y) * ((wx1 - wx0) / 2 + 1);
    for y in roof_top..=roof_bottom {
        let t = (y - roof_top) as f32 / roof_h as f32;
        let inset = if front_gable { 2 } else { ((1.0 - t) * 12.0) as i32 };
        for x in inset..w - inset {
            if in_gable(x, y) {
                continue;
            }
            let row = (y - roof_top) / 7;
            let off = if row % 2 == 0 { 0 } else { 5 };
            let sx = (x + off) % 10;
            let sy = (y - roof_top) % 7;
            let hsh = hash2((x + off) / 10, row, seed ^ 0x5);
            let mut v = if front_gable {
                // Two slopes either side of a north-south ridge.
                let d = (x - cxr) as f32 / (w as f32 / 2.0);
                (if d < 0.0 { 0.8 + d * 0.15 } else { 0.48 - d * 0.12 }) + (hsh % 100) as f32 / 100.0 * 0.08
            } else {
                0.78 - t * 0.4 + (hsh % 100) as f32 / 100.0 * 0.08
            };
            if sy == 6 {
                v -= 0.22;
            } else if sy == 5 {
                v -= 0.09;
            } else if sx == 0 {
                v -= 0.1;
            } else if sy == 0 && sx < 6 {
                v += 0.06;
            }
            if thatch {
                let streak = hash2(x, y / 3, seed) % 4;
                let base = if front_gable { if x < cxr { 0.74 } else { 0.5 } } else { 0.72 - t * 0.36 };
                v = base + match streak {
                    0 => -0.14,
                    1 => 0.06,
                    _ => 0.0,
                };
                if (y - roof_top) % 11 == 10 {
                    v -= 0.16;
                }
            }
            let mut c = if x == inset || x == w - inset - 1 { ramp_at(rr, 0) } else { shade(rr, v) };
            if (culture % 4 == 3 || thatch) && hash2(x / 3, y / 3, seed ^ 0x33) % 23 == 0 {
                c = ramp_at(pal::LEAF_SUMMER, 3);
            }
            if !front_gable && (x < inset + 3 || x > w - inset - 4) {
                c = crate::palette::mix(c, hex(0x24203c), 0.2);
            }
            img.set(x, y, c);
        }
    }
    if front_gable {
        // Ridge running back from the apex.
        for y in roof_top..apex_y {
            img.set(cxr - 1, y, ramp_at(rr, rr.len() as i32 - 1));
            img.set(cxr, y, ramp_at(rr, rr.len() as i32 - 2));
            img.set(cxr + 1, y, ramp_at(rr, 1));
        }
        img.hline(2, w - 3, roof_top, ramp_at(rr, 0));
        // Gable wall: plaster with timber or the house's own wall material.
        for y in apex_y..=roof_bottom {
            for x in wx0..=wx1 {
                if !in_gable(x, y) {
                    continue;
                }
                let tt = (y - apex_y) as f32 / gable_h as f32;
                let c = if st.logs {
                    ramp_at(st.wall, [0, 2, 4, 4, 3, 3, 2][((y - apex_y) % 7) as usize])
                } else if st.stone_wall {
                    shade(st.wall, 0.55 + (hash2(x / 7, y / 5, seed) % 100) as f32 / 500.0)
                } else {
                    shade(st.wall, 0.78 - tt * 0.08)
                };
                img.set(x, y, c);
            }
        }
        if st.timber || culture % 4 == 2 {
            let tc = ramp_at(st.trim, 1);
            img.vline(cxr, apex_y + 2, roof_bottom, tc);
            img.vline(cxr + 1, apex_y + 2, roof_bottom, ramp_at(st.trim, 2));
            let mid = apex_y + gable_h * 2 / 3;
            for x in wx0..=wx1 {
                if in_gable(x, mid) {
                    img.set(x, mid, tc);
                    img.set(x, mid + 1, tc);
                }
            }
            for k in 0..2 {
                img.line(cxr - 2 - k, mid - 1, cxr - gable_h / 2, roof_bottom - 1, tc);
                img.line(cxr + 3 + k, mid - 1, cxr + gable_h / 2, roof_bottom - 1, tc);
            }
        }
        // Gable window (round)
        let gy = apex_y + gable_h / 2 - 2;
        img.ellipse_with(cxr as f32 + 0.5, gy as f32, 5.0, 5.0, |im, x, y, nx, ny| {
            let d = (nx * nx + ny * ny).sqrt();
            let c = if d > 0.75 { ramp_at(st.trim, 1) } else if lit { shade(pal::WINDOW_LIT, 1.0 - d) } else { ramp_at(pal::WINDOW_DARK, if ny < 0.0 { 1 } else { 0 }) };
            im.set(x, y, c);
        });
        img.vline(cxr, gy - 3, gy + 3, ramp_at(st.trim, 2));
        fx.windows.push(((cxr - pivot.0) as i16, (gy - pivot.1) as i16));
        // Barge boards along the slopes, overhanging the walls.
        for k in 0..4 {
            let col = if k == 0 { ramp_at(st.trim, 3) } else if k == 3 { ramp_at(st.trim, 0) } else { ramp_at(st.trim, 2) };
            img.line(cxr, apex_y - 3 + k, 0, roof_bottom + k - 2, col);
            img.line(cxr + 1, apex_y - 3 + k, w - 1, roof_bottom + k - 2, col);
        }
        img.set(cxr, apex_y - 4, hex(pal::GOLD[2]));
    } else {
        for (k, idx) in [(0, rr.len() as i32 - 1), (1, rr.len() as i32 - 2), (2, rr.len() as i32 - 3)] {
            img.hline(12, w - 13, roof_top + k, ramp_at(rr, idx));
        }
        for x in (12..w - 13).step_by(6) {
            img.set(x, roof_top + 2, ramp_at(rr, 1));
        }
        img.hline(0, w - 1, roof_bottom, ramp_at(rr, 0));
        img.hline(0, w - 1, roof_bottom - 1, ramp_at(rr, 1));
        img.hline(0, w - 1, roof_bottom - 2, ramp_at(rr, 2));
    }
    // Shadow under the eave
    for x in wx0..=wx1 {
        for (k, a) in [(1, 0.5), (2, 0.35), (3, 0.2), (4, 0.1)] {
            if front_gable && in_gable(x, roof_bottom) {
                continue;
            }
            if let Some(c) = img.get(x, roof_bottom + k) {
                img.set(x, roof_bottom + k, crate::palette::mix(c, hex(0x24203c), a));
            }
        }
    }
    // Dormer window on big roofs
    if !front_gable && (fh >= 4 || kind == BuildingKind::Inn) {
        let (dxw, dyw) = (w / 2 - 10, roof_top + roof_h / 2 - 8);
        for k in 0..10 {
            img.hline(dxw - 6 + k, dxw + 25 - k, dyw - 6 - k, ramp_at(rr, 4 - (k / 4).min(3)));
        }
        img.rect(dxw - 4, dyw - 6, 28, 24, ramp_at(st.wall, 3));
        img.vline(dxw - 4, dyw - 6, dyw + 17, ramp_at(st.wall, 4));
        img.vline(dxw + 23, dyw - 6, dyw + 17, ramp_at(st.wall, 1));
        window(&mut img, dxw + 3, dyw, 14, 12, lit, st.trim, None, false);
        fx.windows.push(((dxw + 10 - pivot.0) as i16, (dyw + 6 - pivot.1) as i16));
    }
    // Chimney
    if kind != BuildingKind::Chapel {
        let cx = if r.chance(0.5) { w / 4 - 2 } else { w * 3 / 4 - 10 };
        let ctop = roof_top - 12 + if front_gable { 10 } else { 0 };
        for y in ctop..roof_top + 16 {
            for x in cx..cx + 12 {
                let row = (y - ctop) / 4;
                let bx = (x - cx + row * 3) % 6;
                let c = if y < ctop + 3 {
                    if y == ctop { ramp_at(pal::STONE_WALL, 4) } else { ramp_at(pal::STONE_WALL, 2) }
                } else if bx == 0 || (y - ctop) % 4 == 0 {
                    ramp_at(pal::STONE_WALL, 1)
                } else if x < cx + 3 {
                    ramp_at(pal::STONE_WALL, 3)
                } else if x > cx + 9 {
                    ramp_at(pal::STONE_WALL, 1)
                } else {
                    ramp_at(pal::STONE_WALL, 2)
                };
                img.set(x, y, c);
            }
        }
        img.rect(cx - 1, ctop, 14, 3, ramp_at(pal::STONE_WALL, 3));
        img.hline(cx + 2, cx + 9, ctop + 1, pal::INK_SOFT);
        fx.chimney = Some(((cx + 6 - pivot.0) as i16, (ctop - pivot.1) as i16));
    }
    // Chapel bell tower with a spire
    if kind == BuildingKind::Chapel {
        let (tx, tw) = (w / 2 - 12, 24);
        let ttop = roof_top - 26;
        img.rect(tx, ttop + 14, tw, 26, ramp_at(st.wall, 3));
        img.vline(tx, ttop + 14, ttop + 39, ramp_at(st.wall, 4));
        img.vline(tx + tw - 1, ttop + 14, ttop + 39, ramp_at(st.wall, 1));
        img.vline(tx + tw - 2, ttop + 14, ttop + 39, ramp_at(st.wall, 2));
        img.rect(tx + 6, ttop + 19, 12, 14, pal::INK_SOFT);
        img.ellipse_with(tx as f32 + 12.0, ttop as f32 + 27.0, 4.0, 4.5, |im, x, y, nx, ny| {
            im.set(x, y, hex(pal::GOLD[if nx + ny < -0.3 { 4 } else if nx + ny > 0.5 { 1 } else { 2 }]));
        });
        for k in 0..18 {
            let half = 14 - k * 13 / 17;
            img.hline(w / 2 - half, w / 2 + half - 1, ttop + 14 - k, ramp_at(rr, 4 - (k / 5).min(3)));
            img.set(w / 2 + half - 1, ttop + 14 - k, ramp_at(rr, 1));
        }
        img.vline(w / 2, ttop - 10, ttop - 3, hex(pal::GOLD[3]));
        img.hline(w / 2 - 3, w / 2 + 3, ttop - 8, hex(pal::GOLD[3]));
    }

    img.outline(pal::INK);
    (img, pivot, fx)
}

// ---------------------------------------------------------------- furniture

fn stone_course(img: &mut Img, x0: i32, x1: i32, y0: i32, y1: i32, bw: i32, bh: i32, seed: u32) {
    for y in y0..=y1 {
        for x in x0..=x1 {
            let row = (y - y0) / bh;
            let off = if row % 2 == 0 { 0 } else { bw / 2 };
            let bx = (x - x0 + off) % bw;
            let by = (y - y0) % bh;
            let v = 0.5 + (hash2((x - x0 + off) / bw, row, seed) % 100) as f32 / 400.0 - (x - x0) as f32 / (x1 - x0 + 1) as f32 * 0.2;
            let c = if bx == 0 || by == bh - 1 { ramp_at(pal::STONE_WALL, 1) } else if by == 0 { shade(pal::STONE_WALL, v + 0.2) } else { shade(pal::STONE_WALL, v) };
            img.set(x, y, c);
        }
    }
}

pub fn well(culture: u8) -> (Img, (i32, i32)) {
    let mut img = Img::new(48, 64);
    let roof = match culture % 4 {
        1 => pal::ROOF_SLATE,
        2 => pal::ROOF_BLUE,
        3 => pal::ROOF_GREEN,
        _ => pal::ROOF_RED,
    };
    stone_course(&mut img, 6, 41, 40, 59, 8, 5, 91);
    img.ellipse_with(24.0, 40.5, 17.0, 5.0, |im, x, y, nx, ny| {
        let d = (nx * nx + ny * ny).sqrt();
        let c = if d > 0.78 { ramp_at(pal::STONE_WALL, 4) } else if d > 0.65 { ramp_at(pal::STONE_WALL, 2) } else if ny < -0.2 { ramp_at(pal::WATER, 0) } else { ramp_at(pal::WATER, 2) };
        im.set(x, y, c);
    });
    img.set(20, 41, hex(pal::WATER[6]));
    img.set(21, 41, hex(pal::WATER[5]));
    // Posts, crossbar, bucket
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
    // Roof
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

pub fn lamp() -> (Img, (i32, i32)) {
    let mut img = Img::new(20, 60);
    img.rect(8, 16, 4, 40, ramp_at(pal::METAL, 1));
    img.vline(8, 16, 55, ramp_at(pal::METAL, 3));
    img.vline(11, 16, 55, ramp_at(pal::METAL, 0));
    img.rect(5, 54, 10, 4, ramp_at(pal::METAL, 1));
    img.hline(5, 14, 54, ramp_at(pal::METAL, 3));
    img.rect(6, 30, 8, 2, ramp_at(pal::METAL, 2));
    // Lantern
    img.rect(4, 4, 12, 13, ramp_at(pal::METAL, 0));
    img.rect(6, 6, 8, 9, hex(pal::WINDOW_LIT[2]));
    img.rect(7, 7, 4, 5, hex(pal::WINDOW_LIT[3]));
    img.vline(10, 6, 14, ramp_at(pal::METAL, 0));
    for k in 0..4 {
        img.hline(3 + k, 16 - k, 3 - k, ramp_at(pal::METAL, 2 + (k == 0) as i32));
    }
    img.rect(9, -1, 2, 2, ramp_at(pal::METAL, 2));
    img.outline(pal::INK);
    (img, (10, 57))
}

pub fn shrine() -> (Img, (i32, i32)) {
    let mut img = Img::new(36, 68);
    let stone = pal::STONE_WALL;
    stone_course(&mut img, 4, 31, 52, 63, 7, 4, 17);
    img.rect(8, 44, 20, 8, ramp_at(stone, 3));
    img.hline(8, 27, 44, ramp_at(stone, 4));
    img.vline(27, 44, 51, ramp_at(stone, 1));
    // Robe
    for k in 0..24 {
        let half = 4 + k / 3;
        for x in 18 - half..18 + half {
            let fold = (x - 18 + half) % 4 == 2 && k > 6;
            let c = if x < 18 - half + 3 { ramp_at(pal::PLASTER, 4) } else if x > 18 + half - 4 || fold { ramp_at(pal::PLASTER, 2) } else { ramp_at(pal::PLASTER, 3) };
            img.set(x, 20 + k, c);
        }
    }
    // Folded hands, head, hair, halo
    img.rect(15, 26, 6, 3, ramp_at(pal::PLASTER, 4));
    img.ellipse_with(18.0, 15.0, 6.0, 6.0, |im, x, y, nx, ny| {
        im.set(x, y, ramp_at(pal::PLASTER, if nx + ny < -0.3 { 4 } else if nx + ny > 0.6 { 2 } else { 3 }));
    });
    for x in 11..25 {
        img.vline(x, 9, 13 + if x < 13 || x > 22 { 8 } else { 0 }, ramp_at(pal::PLASTER, if x < 15 { 4 } else { 2 }));
    }
    img.set(15, 16, ramp_at(pal::PLASTER, 1));
    img.set(20, 16, ramp_at(pal::PLASTER, 1));
    img.ellipse_with(18.0, 9.0, 10.0, 3.0, |im, x, y, nx, ny| {
        if nx * nx + ny * ny > 0.55 {
            im.set(x, y, hex(pal::GOLD[if nx < 0.0 { 4 } else { 3 }]));
        }
    });
    img.rect(17, 30, 2, 2, hex(pal::GOLD[3]));
    // Offerings
    img.rect(9, 48, 3, 3, hex(0xd24c40));
    img.rect(24, 48, 3, 3, hex(0xeec04a));
    img.outline(pal::INK);
    (img, (18, 63))
}

pub fn small_prop(kind: &str, variant: u8) -> (Img, (i32, i32)) {
    match kind {
        "barrel" => {
            let mut img = Img::new(24, 30);
            for y in 3..28 {
                let bulge = if (8..22).contains(&y) { 0 } else if (5..25).contains(&y) { 1 } else { 2 };
                for x in 2 + bulge..22 - bulge {
                    let stave = (x - 2) % 5 == 0;
                    let c = if y == 7 || y == 8 || y == 21 || y == 22 {
                        if x < 7 { ramp_at(pal::METAL, 2) } else { ramp_at(pal::METAL, 1) }
                    } else if stave {
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
        "crate" => {
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
        "haystack" => {
            let mut img = Img::new(40, 32);
            img.ellipse_with(20.0, 18.0, 18.0, 13.0, |im, x, y, nx, ny| {
                let streak = hash2(x, y / 3, 77) % 4;
                let v = 0.72 - nx * 0.2 - ny * 0.28 - match streak {
                    0 => 0.14,
                    1 => -0.05,
                    _ => 0.0,
                };
                im.set(x, y, shade(pal::ROOF_THATCH, v));
            });
            for k in 0..6 {
                img.line(6 + k * 5, 26, 4 + k * 5, 30, ramp_at(pal::ROOF_THATCH, 2));
            }
            img.outline(pal::INK);
            (img, (20, 29))
        }
        "bench" => {
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
        "fence" => {
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
        "tent" => {
            let mut img = Img::new(72, 56);
            let cloth = pal::CLOTH[(variant as usize) % pal::CLOTH.len()];
            for k in 0..44 {
                let half = 3 + k * 31 / 43;
                for x in 36 - half..36 + half {
                    let seam = (x - 36_i32).abs() % 10 == 0;
                    let c = if seam { hex(cloth[1]) } else if x < 36 { hex(cloth[3]) } else { hex(cloth[2]) };
                    img.set(x, 8 + k, c);
                }
            }
            for k in 0..20 {
                let half = k * 9 / 19;
                img.hline(36 - half, 36 + half, 51 - (19 - k), hex(0x2a1c24));
            }
            img.vline(36, 32, 51, hex(cloth[1]));
            img.rect(35, 2, 2, 8, ramp_at(pal::WOOD, 3));
            img.outline(pal::INK);
            (img, (36, 52))
        }
        "campfire" => {
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
        "stall" => {
            let mut img = Img::new(80, 68);
            let awn = pal::CLOTH[(variant as usize) % pal::CLOTH.len()];
            img.rect(6, 40, 68, 24, ramp_at(pal::WOOD, 3));
            for y in (44..64).step_by(5) {
                img.hline(6, 73, y, ramp_at(pal::WOOD, 2));
            }
            img.rect(6, 38, 68, 4, ramp_at(pal::WOOD, 4));
            img.rect(8, 16, 4, 26, ramp_at(pal::WOOD, 2));
            img.rect(68, 16, 4, 26, ramp_at(pal::WOOD, 1));
            for y in 4..20 {
                for x in 2..78 {
                    let stripe = (x / 8) % 2 == 0;
                    let c = if stripe { hex(awn[2]) } else { hex(0xede3d0) };
                    img.set(x, y, if y > 15 { crate::palette::mix(c, hex(0x24203c), 0.25) } else if y < 6 { crate::palette::mix(c, hex(0xffffff), 0.15) } else { c });
                }
            }
            for x in (2..78).step_by(8) {
                img.rect(x + 2, 20, 4, 2, hex(awn[1]));
            }
            for k in 0..8 {
                let c = [0xd24c40u32, 0xeec04a, 0x7aa86c, 0xe0802a, 0x9a5ec4, 0xd24c40, 0xc87ac0, 0xeec04a][k];
                img.ellipse_with(12.0 + k as f32 * 8.0, 36.0, 3.5, 3.0, |im, x, y, nx, ny| {
                    im.set(x, y, crate::palette::mix(hex(c), if nx + ny < -0.3 { hex(0xffffff) } else { hex(0x24203c) }, if nx + ny < -0.3 { 0.3 } else { 0.15 }));
                });
            }
            img.outline(pal::INK);
            (img, (40, 64))
        }
        _ => (Img::new(1, 1), (0, 0)),
    }
}

pub fn wall_color(culture: u8) -> Rgb {
    hex(match culture % 4 {
        1 => pal::STONE_WALL[3],
        _ => pal::PLASTER[3],
    })
}
