//! Village siting and layout: plazas, buildings, farms, and the road network.

use serde::{Deserialize, Serialize};

use super::path::PathFinder;
use super::{flag, Prop, PropKind, Terrain, World};
use crate::names;
use crate::rng::Rng;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum BuildingKind {
    Home = 0,
    Inn,
    Store,
    Smithy,
    Chapel,
}

impl BuildingKind {
    pub fn size(self) -> (u8, u8) {
        match self {
            BuildingKind::Home => (5, 3),
            BuildingKind::Inn => (7, 4),
            BuildingKind::Store => (6, 3),
            BuildingKind::Smithy => (6, 3),
            BuildingKind::Chapel => (5, 4),
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            BuildingKind::Home => "Home",
            BuildingKind::Inn => "Inn",
            BuildingKind::Store => "General Store",
            BuildingKind::Smithy => "Smithy",
            BuildingKind::Chapel => "Chapel",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Building {
    pub prop: usize,
    pub kind: BuildingKind,
    /// Tile directly in front of the door, where agents enter and exit.
    pub door: (i32, i32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Village {
    pub id: usize,
    pub name: String,
    pub culture: u8,
    pub cx: i32,
    pub cy: i32,
    pub radius: i32,
    pub buildings: Vec<Building>,
    /// Farm plots as (x, y, w, h) in tiles.
    pub fields: Vec<(i32, i32, i32, i32)>,
}

fn site_score(w: &World, cx: i32, cy: i32) -> Option<f32> {
    let r = 14;
    let lv = w.level(cx, cy);
    if w.terrain(cx, cy) != Terrain::Grass || lv == 3 {
        return None;
    }
    let mut good = 0;
    let mut total = 0;
    let mut water_near = false;
    let mut forest = 0.0;
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy > r * r {
                continue;
            }
            let (x, y) = (cx + dx, cy + dy);
            if !w.in_bounds(x, y) {
                return None;
            }
            total += 1;
            let t = w.terrain(x, y);
            if t == Terrain::Grass && w.level(x, y) == lv && !w.is_face(x, y) && !w.is_face(x, y + 1) {
                good += 1;
            }
            forest += w.forest[w.idx(x, y)];
        }
    }
    for dy in -16..=16 {
        for dx in -16..=16 {
            if w.terrain(cx + dx, cy + dy).is_water() {
                water_near = true;
            }
        }
    }
    let flat = good as f32 / total as f32;
    if flat < 0.8 {
        return None;
    }
    Some(flat + if water_near { 0.2 } else { 0.0 } - forest / total as f32 * 0.3)
}

/// Cost function for laying roads. Roads may climb cliffs (becoming stairs)
/// and cross rivers (becoming bridges), but both are expensive.
fn road_step(w: &World, ax: i32, ay: i32, bx: i32, by: i32) -> Option<f32> {
    if !w.in_bounds(bx, by) || w.blocker[w.idx(bx, by)] != 0 {
        return None;
    }
    let tb = w.terrain(bx, by);
    if tb == Terrain::Deep {
        return None;
    }
    let la = w.level(ax, ay);
    let lb = w.level(bx, by);
    let face_a = w.is_face(ax, ay);
    let face_b = w.is_face(bx, by);
    let mut cost = 1.0;
    if la == lb {
        if ay == by && (face_a || face_b) {
            return None;
        }
        if face_b {
            cost += 5.0;
        }
    } else if lb + 1 == la && bx == ax && by == ay + 1 && w.face_row(bx, by) == 1 && !face_a {
        // Stepping down onto the top row of a face.
        cost += 1.0;
    } else if la + 1 == lb && bx == ax && by == ay - 1 && w.face_row(ax, ay) == 1 && !face_b {
        cost += 1.0;
    } else {
        return None;
    }
    // A staircase needs its whole run free of water and buildings.
    if face_b && w.face_row(bx, by) == 1 && (w.is_face(bx, by + 1) && w.face_row(bx, by + 1) != 2 || w.terrain(bx, by + 1).is_water()) {
        return None;
    }
    if tb.is_water() {
        // Only bridge narrow rivers, never over a waterfall.
        if !w.has(bx, by, flag::RIVER) || face_b {
            return None;
        }
        cost += 10.0;
    }
    if face_b && w.terrain(bx, by).is_water() {
        return None;
    }
    let f = w.flags(bx, by);
    if f & (flag::ROAD | flag::PLAZA) != 0 {
        cost *= 0.3;
    }
    if f & flag::TILLED != 0 {
        cost += 20.0;
    }
    if tb == Terrain::Rock || tb == Terrain::Snow {
        cost += 1.5;
    }
    cost += w.forest[w.idx(bx, by)] * 1.2;
    Some(cost)
}

fn lay_road(w: &mut World, pf: &mut PathFinder, a: (i32, i32), b: (i32, i32)) -> bool {
    let path = {
        let wr: &World = w;
        pf.search(a, b, 120_000, false, |ax, ay, bx, by| road_step(wr, ax, ay, bx, by))
    };
    let Some(path) = path else { return false };
    for &(x, y) in &path {
        let i = w.idx(x, y);
        if w.flags[i] & flag::PLAZA != 0 {
            continue;
        }
        if Terrain::from_u8(w.terrain[i]).is_water() {
            w.flags[i] |= flag::BRIDGE;
        } else if w.is_face(x, y) {
            w.flags[i] |= flag::STAIRS | flag::ROAD;
        } else {
            w.terrain[i] = Terrain::Dirt as u8;
            w.flags[i] |= flag::ROAD;
        }
    }
    true
}

pub fn generate(w: &mut World, rng: &mut Rng) {
    // 1. Site selection
    let mut candidates: Vec<(f32, i32, i32)> = Vec::new();
    for cy in (14..w.h - 14).step_by(3) {
        for cx in (14..w.w - 14).step_by(3) {
            if let Some(s) = site_score(w, cx, cy) {
                candidates.push((s + rng.f32() * 0.08, cx, cy));
            }
        }
    }
    candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let mut sites: Vec<(i32, i32)> = Vec::new();
    for min_d in [46, 38, 30] {
        for &(_, x, y) in &candidates {
            if sites.len() >= 5 {
                break;
            }
            if sites.iter().all(|&(sx, sy)| (sx - x).pow(2) + (sy - y).pow(2) >= min_d * min_d) {
                sites.push((x, y));
            }
        }
        if sites.len() >= 3 {
            break;
        }
    }

    let mut pf = PathFinder::new(w.w, w.h);
    let mut used_cultures: Vec<usize> = Vec::new();
    for (vid, &(cx, cy)) in sites.iter().enumerate() {
        // Culture follows geography, but neighbours shouldn't all match.
        let lv = w.level(cx, cy);
        let mut culture = if lv >= 2 {
            1
        } else if w.forest[w.idx(cx, cy)] > 0.35 {
            3
        } else if (0..16).any(|d| w.terrain(cx, cy + d).is_water() || w.terrain(cx - d, cy).is_water()) {
            2
        } else {
            0
        };
        if used_cultures.iter().filter(|&&c| c == culture).count() >= 2 {
            culture = (culture + 1 + vid) % names::CULTURES;
        }
        used_cultures.push(culture);
        let name = names::town(rng, culture);
        let mut v = Village { id: vid, name, culture: culture as u8, cx, cy, radius: 11, buildings: Vec::new(), fields: Vec::new() };
        layout(w, rng, &mut pf, &mut v);
        w.villages.push(v);
    }

    // 2. Roads: chain each village to its nearest already-connected one,
    // plus one extra link to form a loop.
    let n = w.villages.len();
    let mut connected = vec![0usize];
    let mut edges: Vec<(usize, usize)> = Vec::new();
    while connected.len() < n {
        let mut best = (i32::MAX, 0, 0);
        for &a in &connected {
            for b in 0..n {
                if connected.contains(&b) {
                    continue;
                }
                let (va, vb) = (&w.villages[a], &w.villages[b]);
                let d = (va.cx - vb.cx).pow(2) + (va.cy - vb.cy).pow(2);
                if d < best.0 {
                    best = (d, a, b);
                }
            }
        }
        connected.push(best.2);
        edges.push((best.1, best.2));
    }
    if n >= 4 {
        edges.push((n - 1, 1));
    }
    for (a, b) in edges {
        let pa = (w.villages[a].cx, w.villages[a].cy + 3);
        let pb = (w.villages[b].cx, w.villages[b].cy + 3);
        lay_road(w, &mut pf, pa, pb);
    }

    // 3. Spawn: just south of the first village's plaza.
    if let Some(v) = w.villages.first() {
        let (cx, cy) = (v.cx, v.cy);
        'find: for r in 3..12 {
            for dy in -r..=r {
                for dx in -r..=r {
                    let (x, y) = (cx + dx, cy + dy + 2);
                    if w.walkable(x, y) && w.level(x, y) == w.level(cx, cy) {
                        w.spawn = (x, y);
                        break 'find;
                    }
                }
            }
        }
    }
}

fn area_free(w: &World, x0: i32, y0: i32, fw: i32, fh: i32, lv: u8) -> bool {
    // Footprint grows north from row y0; require a 1-tile margin.
    for y in (y0 - fh - 2)..=(y0 + 1) {
        for x in (x0 - 1)..=(x0 + fw) {
            if !w.free_for_prop(x, y) || w.level(x, y) != lv || w.terrain(x, y) != Terrain::Grass {
                return false;
            }
        }
    }
    true
}

fn mark_no_props(w: &mut World, x0: i32, y0: i32, fw: i32, fh: i32) {
    for y in (y0 - fh - 2)..=(y0 + 1) {
        for x in (x0 - 1)..=(x0 + fw) {
            w.set_flag(x, y, flag::NO_PROPS, true);
        }
    }
}

fn layout(w: &mut World, rng: &mut Rng, pf: &mut PathFinder, v: &mut Village) {
    let (cx, cy) = (v.cx, v.cy);
    let lv = w.level(cx, cy);
    // Plaza
    for dy in -3..=3i32 {
        for dx in -3..=3i32 {
            let d2 = dx * dx + dy * dy;
            if d2 <= 10 {
                let (x, y) = (cx + dx, cy + dy);
                if w.level(x, y) == lv && !w.terrain(x, y).is_water() {
                    let i = w.idx(x, y);
                    w.terrain[i] = Terrain::Dirt as u8;
                    w.flags[i] |= flag::PLAZA;
                }
            }
        }
    }
    let mk = |kind, x, y, variant, fw, fh| Prop { kind, x, y, variant, state: 0, hp: 1, fw, fh, alive: true };
    w.add_prop(mk(PropKind::Well, cx, cy, v.culture, 1, 1));
    for (dx, dy) in [(-3, -3), (3, -3), (-3, 3), (3, 3)] {
        let (x, y) = (cx + dx, cy + dy);
        if w.walkable(x, y) {
            w.add_prop(mk(PropKind::Lamp, x, y, 0, 1, 1));
        }
    }

    // Buildings
    let mut kinds = vec![BuildingKind::Inn, BuildingKind::Store, BuildingKind::Smithy, BuildingKind::Chapel];
    let homes = rng.range(6, 10);
    for _ in 0..homes {
        kinds.push(BuildingKind::Home);
    }
    for kind in kinds {
        let (fw, fh) = kind.size();
        let (fw, fh) = (fw as i32, fh as i32);
        let mut placed = false;
        for attempt in 0..400 {
            let ring = 7.0 + (attempt as f32 / 25.0).floor() * 1.2 + rng.f32() * 2.5;
            let ang = rng.f32() * std::f32::consts::TAU;
            let x0 = cx + (ang.cos() * ring) as i32 - fw / 2;
            let y0 = cy + (ang.sin() * ring * 0.85) as i32 + fh / 2;
            if !area_free(w, x0, y0, fw, fh, lv) {
                continue;
            }
            let variant = v.culture * 4 + rng.below(4) as u8;
            let id = w.add_prop(mk(PropKind::House, x0, y0, variant, fw as u8, fh as u8));
            w.props[id].state = kind as u8;
            mark_no_props(w, x0, y0, fw, fh);
            let door = (x0 + fw / 2, y0 + 1);
            v.buildings.push(Building { prop: id, kind, door });
            v.radius = v.radius.max(((x0 - cx).abs().max((y0 - cy).abs())) + fw);
            placed = true;
            break;
        }
        if !placed && kind != BuildingKind::Home {
            // Essential buildings get a second chance with a smaller ring
            // later; for now just skip.
        }
    }
    // Paths from doors to the plaza.
    let doors: Vec<(i32, i32)> = v.buildings.iter().map(|b| b.door).collect();
    for d in doors {
        lay_road(w, pf, d, (cx, cy + 2));
    }

    // Farm plots
    let plots = rng.range(2, 4);
    let mut made = 0;
    for _ in 0..200 {
        if made >= plots {
            break;
        }
        let (pw, ph) = (rng.range(5, 8), rng.range(3, 5));
        let ang = rng.f32() * std::f32::consts::TAU;
        let dist = v.radius as f32 + 2.0 + rng.f32() * 4.0;
        let x0 = cx + (ang.cos() * dist) as i32 - pw / 2;
        let y0 = cy + (ang.sin() * dist) as i32 - ph / 2;
        let ok = (y0 - 1..y0 + ph + 1).all(|y| (x0 - 1..x0 + pw + 1).all(|x| w.free_for_prop(x, y) && w.level(x, y) == lv));
        if !ok {
            continue;
        }
        let crop = rng.below(6) as u8;
        for y in y0..y0 + ph {
            for x in x0..x0 + pw {
                w.set_flag(x, y, flag::TILLED, true);
                if (x - x0) % 1 == 0 && y % 1 == 0 {
                    let id = w.add_prop(mk(PropKind::Crop, x, y, crop, 1, 1));
                    w.props[id].state = rng.range(0, 4) as u8;
                }
            }
        }
        for y in y0 - 1..y0 + ph + 1 {
            for x in x0 - 1..x0 + pw + 1 {
                w.set_flag(x, y, flag::NO_PROPS, true);
            }
        }
        v.fields.push((x0, y0, pw, ph));
        made += 1;
    }

    // Decoration near buildings.
    let decor = [PropKind::Barrel, PropKind::Crate, PropKind::Haystack, PropKind::Bench, PropKind::Flowers];
    let builds: Vec<(i32, i32, i32)> = v
        .buildings
        .iter()
        .map(|b| {
            let p = &w.props[b.prop];
            (p.x, p.y, p.fw as i32)
        })
        .collect();
    for (bx, by, bw) in builds {
        for _ in 0..rng.range(1, 4) {
            let side = if rng.chance(0.5) { bx - 1 } else { bx + bw };
            let y = by - rng.range(0, 2);
            let k = *rng.pick(&decor);
            if w.walkable(side, y) && w.flags(side, y) & (flag::ROAD | flag::PLAZA) == 0 {
                w.add_prop(mk(k, side, y, rng.below(4) as u8, 1, 1));
            }
        }
    }
    // A shrine to the goddess at every village edge.
    for _ in 0..40 {
        let ang = rng.f32() * std::f32::consts::TAU;
        let x = cx + (ang.cos() * 4.5) as i32;
        let y = cy + (ang.sin() * 4.5) as i32;
        if w.free_for_prop(x, y) || (w.walkable(x, y) && w.flags(x, y) & flag::PLAZA == 0 && w.blocker[w.idx(x, y)] == 0 && !w.has(x, y, flag::ROAD)) {
            w.add_prop(mk(PropKind::Shrine, x, y, 0, 1, 1));
            break;
        }
    }
}
