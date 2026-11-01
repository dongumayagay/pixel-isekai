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
    // Recessed frame: dark inner reveal, lit outer trim.
    img.rect(x - 3, y - 3, w + 6, h + 6, ramp_at(trim, 1));
    img.hline(x - 3, x + w + 2, y - 3, ramp_at(trim, 3));
    img.vline(x - 3, y - 3, y + h + 2, ramp_at(trim, 2));
    img.rect(x - 1, y - 1, w + 2, h + 2, ramp_at(trim, 0));
    let glass = if lit { pal::WINDOW_LIT } else { pal::WINDOW_DARK };
    for yy in 0..h {
        for xx in 0..w {
            let c = if lit {
                let d = ((xx * 2 - w + 1).abs() as f32 / w as f32 + (yy * 2 - h + 1).abs() as f32 / h as f32) * 0.5;
                shade(glass, 0.98 - d * 0.85)
            } else if (xx + (h - yy)) % 11 < 2 && xx < w * 2 / 3 {
                // Sky reflection streaks.
                ramp_at(pal::WINDOW_DARK, 2)
            } else {
                ramp_at(glass, if yy < h / 3 { 1 } else { 0 })
            };
            img.set(x + xx, y + yy, c);
        }
    }
    if !lit {
        // Curtains tied back.
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
    if shutters {
        for (sx, light) in [(x - 11, true), (x + w + 4, false)] {
            for yy in -3..h + 3 {
                for xx in 0..7 {
                    let slat = (yy + 3) % 5 == 4;
                    let c = if xx == 0 || xx == 6 { ramp_at(trim, 1) } else if slat { ramp_at(trim, 1) } else if light && xx == 1 { ramp_at(trim, 4) } else { ramp_at(trim, 3) };
                    img.set(sx + xx, y + yy, c);
                }
            }
        }
    }
    if let Some(f) = flowers {
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
                img.set(x - 3 + k, y + h + 2, hex(f[1]));
                img.set(x - 3 + k, y + h + 1, hex(f[2]));
                img.set(x - 2 + k, y + h + 2, hex(f[0]));
            }
        }
    }
}

fn door(img: &mut Img, x: i32, y: i32, w: i32, h: i32, trim: Ramp, arch: bool) {
    // Frame and lintel.
    img.rect(x - 4, y - 5, w + 8, h + 5, ramp_at(trim, 1));
    img.hline(x - 4, x + w + 3, y - 5, ramp_at(trim, 3));
    img.vline(x - 4, y - 5, y + h - 1, ramp_at(trim, 2));
    img.rect(x - 1, y - 1, w + 2, h + 1, ramp_at(trim, 0));
    for yy in 0..h {
        for xx in 0..w {
            if arch {
                let r = w as f32 / 2.0;
                let dx = xx as f32 + 0.5 - r;
                let dy = r - yy as f32 - 0.5;
                if dy > 0.0 && dx * dx + dy * dy > r * r {
                    img.set(x + xx, y + yy, ramp_at(trim, 0));
                    continue;
                }
            }
            let plank = xx % 6;
            let c = match plank {
                5 => ramp_at(pal::WOOD, 1),
                0 => ramp_at(pal::WOOD, 4),
                _ => ramp_at(pal::WOOD, 3),
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
            img.set(x + xx, y, crate::palette::mix(c, hex(0x24203c), 0.5));
        }
    }
}

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
            img.rect(x + 6, y + 5, 8, 9, g);
            img.vline(x + 6, y + 5, y + 13, g2);
            img.rect(x + 14, y + 7, 3, 5, g3);
            img.rect(x + 15, y + 8, 1, 3, ramp_at(pal::WOOD, 3));
            img.rect(x + 5, y + 3, 10, 3, hex(0xf8f0e0));
        }
        BuildingKind::Store => {
            img.rect(x + 5, y + 7, 12, 8, g);
            img.vline(x + 5, y + 7, y + 14, g2);
            img.hline(x + 8, x + 13, y + 4, g3);
            img.hline(x + 7, x + 14, y + 5, g3);
            img.hline(x + 6, x + 15, y + 6, g3);
            img.rect(x + 10, y + 9, 2, 3, hex(0xfff4c0));
        }
        BuildingKind::Smithy => {
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

/// Shingle / thatch / slate surface colour for a roof pixel. `u` runs along
/// the courses, `t` from the ridge (0) to the eave (1).
fn roof_px(rr: Ramp, kind: u8, u: i32, row_px: i32, t: f32, seed: u32, lit: f32) -> crate::palette::Rgb {
    let course = match kind {
        2 => 5,
        1 => 6,
        _ => 8,
    };
    let row = row_px.div_euclid(course);
    let ry = row_px.rem_euclid(course);
    let tile_w = match kind {
        2 => 8,
        1 => 0,
        _ => 12,
    };
    let base = 0.66 - t * 0.2 + lit;
    let v = if kind == 1 {
        // Thatch: fibrous streaks, ragged course edges.
        let streak = hash2(u, row_px / 3, seed) % 5;
        let mut v = base + match streak {
            0 => -0.16,
            1 => 0.07,
            2 => -0.05,
            _ => 0.0,
        };
        if ry == course - 1 && hash2(u, row, seed) % 3 != 0 {
            v -= 0.18;
        }
        v
    } else {
        let off = if row % 2 == 0 { 0 } else { tile_w / 2 };
        let tx = (u + off).rem_euclid(tile_w);
        let tile = hash2((u + off).div_euclid(tile_w), row, seed ^ 0x5);
        let mut v = base + (tile % 100) as f32 / 100.0 * 0.1 - 0.05;
        if ry == course - 1 || ry == course - 2 && kind != 2 {
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
    };
    shade(rr, v)
}

/// Generate a building sprite. Returns the image, its pivot (bottom-left of
/// the footprint, in sprite pixels) and fx metadata.
pub fn house(kind: BuildingKind, culture: u8, seed: u32, fw: i32, fh: i32, lit: bool) -> (Img, (i32, i32), HouseFx) {
    let mut r = Rng::new(seed as u64 ^ 0x40053);
    let st = style(culture, &mut r);
    let over = 8;
    let w = fw * 32 + over * 2;
    let wall_h = match kind {
        BuildingKind::Inn => 140,
        BuildingKind::Chapel => 148,
        _ => 124,
    };
    let front_gable = kind != BuildingKind::Inn && match culture % 4 {
        0 => r.chance(0.5),
        1 => r.chance(0.35),
        2 => r.chance(0.2),
        _ => r.chance(0.65),
    };
    let roof_kind: u8 = if st.roof == pal::ROOF_THATCH { 1 } else if st.roof == pal::ROOF_SLATE { 2 } else { 0 };
    let depth = (fh * 32) as f32 * 0.55; // footprint depth seen in projection
    let rise = 32.0 + if kind == BuildingKind::Chapel { 16.0 } else { 0.0 };
    let gable_h = ((fw * 32) as f32 * 0.38) as i32;
    let roof_h = if front_gable { depth as i32 + gable_h } else { (depth + rise) as i32 };
    let extra_top = if kind == BuildingKind::Chapel { 46 } else { 22 };
    let h = wall_h + roof_h + extra_top;
    let mut img = Img::new(w, h);
    let base_y = h - 1;
    let wall_top = base_y - wall_h + 1;
    let wx0 = over;
    let wx1 = w - over - 1;
    let mut fx = HouseFx::default();
    let pivot = (over, h);

    // ---- Walls ----
    for y in wall_top..=base_y {
        for x in wx0..=wx1 {
            let t = (y - wall_top) as f32 / wall_h as f32;
            // Light falls from the upper left across the facade.
            let lx = (x - wx0) as f32 / (wx1 - wx0) as f32 * -0.06;
            let c;
            if st.stone_wall {
                let row = (y - wall_top) / 11;
                let off = if row % 2 == 0 { 0 } else { 9 };
                let bx = (x - wx0 + off) % 18;
                let by = (y - wall_top) % 11;
                let hsh = hash2((x - wx0 + off) / 18, row, seed);
                let v = 0.52 + (hsh % 100) as f32 / 100.0 * 0.2 - t * 0.08 + lx;
                c = if bx == 0 || by == 0 {
                    ramp_at(st.wall, 0)
                } else if by == 1 || bx == 1 {
                    shade(st.wall, v + 0.16)
                } else if by == 10 || bx == 17 {
                    shade(st.wall, v - 0.12)
                } else if hash2(x, y, seed) % 17 == 0 {
                    shade(st.wall, v - 0.07)
                } else {
                    shade(st.wall, v)
                };
            } else if st.logs {
                let row = (y - wall_top) % 10;
                c = ramp_at(st.wall, [0, 2, 3, 4, 4, 4, 3, 3, 2, 1][row as usize]);
            } else {
                let grain = hash2(x / 2, y, seed) % 23 == 0;
                let v = 0.76 - t * 0.12 + lx + if grain { -0.05 } else { 0.0 };
                let mut cc = shade(st.wall, v);
                if culture % 4 == 2 && y >= base_y - 30 {
                    let row = (base_y - y) % 7;
                    cc = match row {
                        0 => ramp_at(pal::ROOF_BLUE, 1),
                        1 => ramp_at(pal::ROOF_BLUE, 4),
                        _ => ramp_at(pal::ROOF_BLUE, 3),
                    };
                }
                c = cc;
            }
            img.set(x, y, c);
        }
    }
    if st.logs {
        // Round log ends at the corners.
        for y in (wall_top..base_y - 10).step_by(10) {
            for cx in [wx0 + 2, wx1 - 2] {
                img.ellipse_with(cx as f32 + 0.5, y as f32 + 5.0, 4.5, 4.5, |im, x, yy, nx, ny| {
                    let ring = ((nx * nx + ny * ny).sqrt() * 3.5) as i32 % 2 == 0;
                    im.set(x, yy, if ring { hex(0xc89a64) } else { hex(0xa87a48) });
                });
            }
        }
    }
    if st.timber {
        let tc = ramp_at(st.trim, 1);
        let tc2 = ramp_at(st.trim, 2);
        for yy in [wall_top + 3, base_y - 16] {
            img.rect(wx0, yy, wx1 - wx0 + 1, 4, tc);
            img.hline(wx0, wx1, yy, tc2);
        }
        let mut x = wx0;
        let mut bay = 0;
        while x <= wx1 - 3 {
            img.rect(x, wall_top, 4, wall_h, tc);
            img.vline(x, wall_top, base_y, tc2);
            if bay % 2 == 1 && x + 34 < wx1 {
                for k in 0..3 {
                    img.line(x + 4 + k, base_y - 17, x + 30 + k, wall_top + 7, tc);
                }
            }
            x += 34;
            bay += 1;
        }
        img.rect(wx1 - 3, wall_top, 4, wall_h, tc);
    }
    // Corner posts.
    img.rect(wx0, wall_top, 3, wall_h, ramp_at(st.trim, 1));
    img.vline(wx0, wall_top, base_y, ramp_at(st.trim, 2));
    img.rect(wx1 - 2, wall_top, 3, wall_h, ramp_at(st.trim, 0));
    // Stone foundation.
    for x in wx0..=wx1 {
        for y in base_y - 11..=base_y {
            let row = (base_y - y) / 6;
            let bx = (x - wx0 + row * 7) % 14;
            let by = (base_y - y) % 6;
            let v = 0.45 + (hash2((x - wx0 + row * 7) / 14, row, seed) % 100) as f32 / 300.0;
            let c = if bx == 0 || by == 5 || y == base_y - 11 { ramp_at(pal::STONE_WALL, 0) } else if by == 4 { shade(pal::STONE_WALL, v + 0.18) } else { shade(pal::STONE_WALL, v) };
            img.set(x, y, c);
        }
    }
    img.hline(wx0, wx1, base_y - 12, ramp_at(pal::STONE_WALL, 4));

    // ---- Door ----
    let (dw, dh) = if kind == BuildingKind::Inn || kind == BuildingKind::Chapel { (40, 80) } else { (34, 74) };
    let dx = w / 2 - dw / 2;
    let dy = base_y - dh;
    door(&mut img, dx, dy, dw, dh, st.trim, kind == BuildingKind::Chapel || culture % 4 == 3);
    img.rect(dx - 6, base_y - 2, dw + 12, 3, ramp_at(pal::STONE_WALL, 3));
    img.hline(dx - 6, dx + dw + 5, base_y - 2, ramp_at(pal::STONE_WALL, 4));
    fx.door = ((dx + dw / 2 - pivot.0) as i16, 0);
    // Lantern beside the door.
    let lx = dx + dw + 6;
    if lx + 9 < wx1 - 4 {
        img.rect(lx, dy + 4, 8, 2, ramp_at(pal::METAL, 1));
        img.rect(lx + 3, dy + 6, 8, 11, ramp_at(pal::METAL, 0));
        img.rect(lx + 4, dy + 7, 6, 9, if lit { hex(pal::WINDOW_LIT[3]) } else { hex(pal::WINDOW_LIT[1]) });
        img.rect(lx + 5, dy + 8, 2, 4, if lit { hex(0xffffff) } else { hex(pal::WINDOW_LIT[2]) });
        img.hline(lx + 2, lx + 11, dy + 5, ramp_at(pal::METAL, 2));
        fx.windows.push(((lx + 7 - pivot.0) as i16, (dy + 11 - pivot.1) as i16));
    }

    // ---- Windows ----
    let flowers = if r.chance(0.65) { Some(pal::FLOWERS[r.below(pal::FLOWERS.len())]) } else { None };
    let shutters = culture % 4 == 0 || culture % 4 == 2 && r.chance(0.6);
    let (ww, wh) = (24, 30);
    let win_y = base_y - 12 - 32 - wh;
    let need = ww + if shutters { 26 } else { 12 };
    let mut slots: Vec<i32> = Vec::new();
    let left_space = dx - 4 - wx0;
    let right_space = wx1 - (dx + dw + 4);
    if left_space >= need {
        slots.push(wx0 + left_space / 2 - ww / 2 + 2);
    }
    if right_space >= need {
        slots.push(dx + dw + 4 + right_space / 2 - ww / 2);
    }
    for &sx in &slots {
        window(&mut img, sx, win_y, ww, wh, lit, st.trim, flowers, shutters);
        fx.windows.push(((sx + ww / 2 - pivot.0) as i16, (win_y + wh / 2 - pivot.1) as i16));
    }
    if kind == BuildingKind::Chapel {
        let (cx, cy) = (w / 2, wall_top + 18);
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
        fx.windows.push(((cx - pivot.0) as i16, (cy - pivot.1) as i16));
    }

    // ---- Sign / awning ----
    if kind != BuildingKind::Home {
        let sx = dx - 30;
        if sx > wx0 + 4 {
            sign_icon(&mut img, kind, sx, dy + 6);
        }
    }
    if kind == BuildingKind::Store {
        let aw = pal::CLOTH[r.below(pal::CLOTH.len())];
        for y in 0..10 {
            for x in dx - 8..dx + dw + 8 {
                let stripe = ((x - dx + 8) / 6) % 2 == 0;
                let c = if stripe { hex(aw[2]) } else { hex(0xede3d0) };
                img.set(x, dy - 14 + y, if y > 7 { crate::palette::mix(c, hex(0x24203c), 0.3) } else if y < 2 { crate::palette::mix(c, hex(0xffffff), 0.2) } else { c });
            }
        }
        for x in (dx - 8..dx + dw + 8).step_by(6) {
            img.rect(x + 1, dy - 4, 4, 2, hex(aw[1]));
        }
    }

    // ---- Roof ----
    let rr = st.roof;
    let eave_y = wall_top + 10; // the roof overhangs the top of the wall
    let ridge_y = eave_y - roof_h;
    let fascia = st.trim;
    if !front_gable {
        // Front slope of a side-gabled roof: courses run parallel to the eave.
        let top = ridge_y + 6;
        for y in top..=eave_y {
            let t = (y - top) as f32 / (eave_y - top) as f32;
            for x in 0..w {
                let lit_k = (0.5 - x as f32 / w as f32) * 0.08;
                img.set(x, y, roof_px(rr, roof_kind, x, y - top, t, seed, lit_k));
            }
        }
        // Back slope sliver beyond the ridge, in shadow.
        for y in ridge_y..top {
            for x in 3..w - 3 {
                img.set(x, y, shade(rr, 0.28 + (hash2(x / 6, y, seed) % 3) as f32 * 0.03));
            }
        }
        // Ridge cap with rounded segments.
        for x in 2..w - 2 {
            let seg = x % 9;
            img.set(x, top - 2, ramp_at(rr, rr.len() as i32 - 1));
            img.set(x, top - 1, if seg == 0 { ramp_at(rr, 1) } else { ramp_at(rr, rr.len() as i32 - 2) });
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
        if roof_kind == 1 {
            // Thatch hangs over the fascia in a ragged fringe.
            for x in 0..w {
                let hang = (hash2(x, 3, seed) % 4) as i32;
                for k in 0..hang {
                    img.set(x, eave_y + 1 + k, shade(rr, 0.42 - k as f32 * 0.06));
                }
            }
        }
    } else {
        // Front-gabled: the gable triangle faces us; two slopes recede back.
        let cx = w / 2;
        let apex_y = eave_y - gable_h;
        let back = depth as i32;
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
                img.set(x, y, roof_px(rr, roof_kind, u, edge_x, t, seed, lit_k));
            }
        }
        // Ridge running back from the apex.
        for y in apex_y - back..=apex_y {
            img.set(cx - 1, y, ramp_at(rr, rr.len() as i32 - 1));
            img.set(cx, y, ramp_at(rr, rr.len() as i32 - 2));
            img.set(cx + 1, y, ramp_at(rr, 1));
        }
        // Gable wall.
        let wall_tri = |x: i32, y: i32| -> bool {
            let ex = (x - cx).abs() as f32;
            y > apex_y + 2 && y <= eave_y + 1 && x > wx0 && x < wx1 && (y - apex_y) as f32 > ex / cx as f32 * gable_h as f32 + 2.0
        };
        for y in apex_y..=eave_y + 1 {
            for x in wx0..=wx1 {
                if !wall_tri(x, y) {
                    continue;
                }
                let c = if st.logs {
                    ramp_at(st.wall, [0, 2, 3, 4, 4, 3, 3, 2, 1][((y - apex_y).rem_euclid(9)) as usize])
                } else if st.stone_wall {
                    shade(st.wall, 0.55 + (hash2(x / 9, y / 6, seed) % 100) as f32 / 500.0)
                } else {
                    shade(st.wall, 0.8 - (y - apex_y) as f32 / gable_h as f32 * 0.06)
                };
                img.set(x, y, c);
            }
        }
        if st.timber || culture % 4 == 2 {
            let tc = ramp_at(st.trim, 1);
            for k in 0..3 {
                img.vline(cx - 1 + k, apex_y + 4, eave_y + 1, if k == 0 { ramp_at(st.trim, 2) } else { tc });
            }
            let mid = apex_y + gable_h * 2 / 3;
            for x in wx0..=wx1 {
                if wall_tri(x, mid) {
                    img.set(x, mid, tc);
                    img.set(x, mid + 1, tc);
                    img.set(x, mid + 2, tc);
                }
            }
            for k in 0..3 {
                img.line(cx - 3 - k, mid - 1, cx - gable_h / 2, eave_y, tc);
                img.line(cx + 3 + k, mid - 1, cx + gable_h / 2, eave_y, tc);
            }
        }
        // Gable window.
        let gy = apex_y + gable_h / 2 + 1;
        img.ellipse_with(cx as f32 + 0.5, gy as f32, 7.0, 7.0, |im, x, y, nx, ny| {
            let d = (nx * nx + ny * ny).sqrt();
            let c = if d > 0.78 { ramp_at(st.trim, 1) } else if lit { shade(pal::WINDOW_LIT, 1.0 - d) } else { ramp_at(pal::WINDOW_DARK, if ny < -0.2 { 2 } else { 0 }) };
            im.set(x, y, c);
        });
        img.vline(cx, gy - 5, gy + 5, ramp_at(st.trim, 2));
        img.hline(cx - 5, cx + 5, gy, ramp_at(st.trim, 2));
        fx.windows.push(((cx - pivot.0) as i16, (gy - pivot.1) as i16));
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
        img.rect(cx - 1, apex_y - 6, 3, 3, hex(pal::GOLD[2]));
    }
    // Shadow cast by the eave onto the wall.
    for x in wx0..=wx1 {
        for (k, a) in [(5, 0.5), (6, 0.38), (7, 0.26), (8, 0.14)] {
            let y = eave_y + k;
            if front_gable {
                let ex = (x - w / 2).abs() as f32;
                if (y - (eave_y - gable_h)) as f32 > ex / (w / 2) as f32 * gable_h as f32 + 4.0 && y < eave_y + 4 {
                    continue;
                }
            }
            if let Some(c) = img.get(x, y) {
                img.set(x, y, crate::palette::mix(c, hex(0x24203c), a));
            }
        }
    }
    // Dormer on deep side-gabled roofs.
    if !front_gable && (fh >= 4 || kind == BuildingKind::Inn) {
        let (dxw, dyw) = (w / 2 - 14, ridge_y + roof_h / 2 - 6);
        for k in 0..12 {
            img.hline(dxw - 8 + k, dxw + 35 - k, dyw - 8 - k, ramp_at(rr, 4 - (k / 4).min(3)));
        }
        img.rect(dxw - 5, dyw - 8, 38, 30, ramp_at(st.wall, 3));
        img.vline(dxw - 5, dyw - 8, dyw + 21, ramp_at(st.wall, 4));
        img.vline(dxw + 32, dyw - 8, dyw + 21, ramp_at(st.wall, 1));
        window(&mut img, dxw + 5, dyw, 18, 14, lit, st.trim, None, false);
        fx.windows.push(((dxw + 14 - pivot.0) as i16, (dyw + 7 - pivot.1) as i16));
    }
    // Chimney.
    if kind != BuildingKind::Chapel {
        let cx = if r.chance(0.5) { w / 4 - 4 } else { w * 3 / 4 - 12 };
        let ctop = if front_gable { eave_y - gable_h - depth as i32 + 4 } else { ridge_y - 14 };
        let cbot = ctop + 34;
        for y in ctop..cbot {
            for x in cx..cx + 16 {
                let row = (y - ctop) / 5;
                let bx = (x - cx + row * 4) % 8;
                let c = if y < ctop + 4 {
                    if y == ctop { ramp_at(pal::STONE_WALL, 4) } else { ramp_at(pal::STONE_WALL, 2) }
                } else if bx == 0 || (y - ctop) % 5 == 0 {
                    ramp_at(pal::STONE_WALL, 1)
                } else if x < cx + 4 {
                    ramp_at(pal::STONE_WALL, 3)
                } else if x > cx + 12 {
                    ramp_at(pal::STONE_WALL, 1)
                } else {
                    ramp_at(pal::STONE_WALL, 2)
                };
                img.set(x, y, c);
            }
        }
        img.rect(cx - 2, ctop, 20, 4, ramp_at(pal::STONE_WALL, 3));
        img.hline(cx - 2, cx + 17, ctop, ramp_at(pal::STONE_WALL, 4));
        img.rect(cx + 3, ctop + 1, 10, 2, pal::INK_SOFT);
        fx.chimney = Some(((cx + 8 - pivot.0) as i16, (ctop - pivot.1) as i16));
    }
    // Chapel bell tower with a spire.
    if kind == BuildingKind::Chapel {
        let (tx, tw) = (w / 2 - 16, 32);
        let ttop = ridge_y - 30;
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

pub fn shrine() -> (Img, (i32, i32)) {
    // The goddess: a robed statue with folded hands on a stepped pedestal.
    let mut img = Img::new(52, 104);
    let stone = pal::STONE_WALL;
    stone_course(&mut img, 4, 47, 84, 99, 8, 5, 17);
    img.rect(9, 74, 34, 10, ramp_at(stone, 3));
    img.hline(9, 42, 74, ramp_at(stone, 4));
    img.vline(42, 74, 83, ramp_at(stone, 1));
    let p = pal::PLASTER;
    // Robe flaring to the pedestal.
    for k in 0..44 {
        let half = 5 + k * 9 / 43;
        for x in 26 - half..26 + half {
            let fold = (x - 26 + half) % 5 == 3 && k > 10;
            let c = if x < 26 - half + 3 { ramp_at(p, 4) } else if x > 26 + half - 4 || fold { ramp_at(p, 2) } else { ramp_at(p, 3) };
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
        let len = if x < 20 || x > 32 { 18 } else { 4 };
        img.vline(x, 13, 13 + len, ramp_at(p, if x < 22 { 4 } else if x > 31 { 2 } else { 3 }));
    }
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
