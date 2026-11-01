//! Procedural world generation: an island with plateaus, rivers, beaches,
//! forests, mountains and villages joined by roads.

use std::collections::VecDeque;

use super::villages;
use super::{flag, Prop, PropKind, Terrain, World, WORLD_H, WORLD_W};
use crate::noise::Noise;
use crate::rng::{hash2f, Rng};

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn quantile(values: &[f32], q: f32) -> f32 {
    let mut v: Vec<f32> = values.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if v.is_empty() {
        return 0.0;
    }
    v[((v.len() - 1) as f32 * q) as usize]
}

const N4: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const N8: [(i32, i32); 8] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, 1), (1, -1), (-1, -1)];

pub fn generate(seed: u64) -> World {
    let mut w = World::empty(seed);
    let mut rng = Rng::new(seed);
    let n_height = Noise::new(rng.next_u64());
    let n_ridge = Noise::new(rng.next_u64());
    let n_moist = Noise::new(rng.next_u64());
    let n_forest = Noise::new(rng.next_u64());
    let n_misc = Noise::new(rng.next_u64());

    heights(&mut w, &n_height, &n_ridge, &n_misc);
    let sea = quantile(&w.height, 0.34);
    levels(&mut w, sea);
    water(&mut w, sea);
    let mut river_rng = rng.fork(11);
    rivers(&mut w, &mut river_rng, &n_misc);
    enforce_slopes(&mut w);
    surfaces(&mut w, &n_misc);
    fields(&mut w, &n_moist, &n_forest);

    let mut vrng = rng.fork(23);
    villages::generate(&mut w, &mut vrng);

    let mut prng = rng.fork(37);
    scatter_props(&mut w, &mut prng, &n_misc);
    w
}

fn heights(w: &mut World, n_h: &Noise, n_r: &Noise, n_m: &Noise) {
    for y in 0..WORLD_H {
        for x in 0..WORLD_W {
            let nx = x as f32 / WORLD_W as f32;
            let ny = y as f32 / WORLD_H as f32;
            let base = n_h.warped(nx * 2.4, ny * 2.4, 0.85, 5) * 0.5 + 0.5;
            let mask = smoothstep(0.42, 0.78, n_r.fbm(nx * 1.4 + 3.0, ny * 1.4 - 2.0, 2, 2.0, 0.5) * 0.5 + 0.5);
            let ridge = n_r.ridged(nx * 3.2 + 11.0, ny * 3.2 - 4.0, 4);
            let dx = nx - 0.5;
            let dy = ny - 0.5;
            let d = (dx * dx + dy * dy).sqrt() * 2.0;
            let coast = n_m.fbm(nx * 5.0, ny * 5.0, 3, 2.0, 0.5) * 0.16;
            let fall = smoothstep(0.5, 1.02, d + coast);
            let h = base * 0.62 + ridge * mask * 0.42 - fall * 0.95;
            let i = w.idx(x, y);
            w.height[i] = h;
        }
    }
}

fn levels(w: &mut World, sea: f32) {
    let land: Vec<f32> = w.height.iter().copied().filter(|h| *h >= sea).collect();
    let t1 = quantile(&land, 0.22);
    let t2 = quantile(&land, 0.66);
    let t3 = quantile(&land, 0.9);
    for i in 0..w.height.len() {
        let h = w.height[i];
        w.level[i] = if h < sea {
            0
        } else if h < t1 {
            0
        } else if h < t2 {
            1
        } else if h < t3 {
            2
        } else {
            3
        };
    }
    // Morphological opening per level removes thin slivers and one-tile
    // pillars, so every plateau has a readable face.
    for lv in (1..=3u8).rev() {
        let mask: Vec<bool> = w.level.iter().map(|&l| l >= lv).collect();
        let eroded: Vec<bool> = (0..w.level.len())
            .map(|i| {
                let x = i as i32 % w.w;
                let y = i as i32 / w.w;
                mask[i] && N8.iter().all(|(dx, dy)| {
                    let (nx, ny) = (x + dx, y + dy);
                    !w.in_bounds(nx, ny) || mask[w.idx(nx, ny)]
                })
            })
            .collect();
        for i in 0..w.level.len() {
            let x = i as i32 % w.w;
            let y = i as i32 / w.w;
            let keep = eroded[i] || N8.iter().any(|(dx, dy)| {
                let (nx, ny) = (x + dx, y + dy);
                w.in_bounds(nx, ny) && eroded[w.idx(nx, ny)]
            });
            if mask[i] && !keep {
                w.level[i] = lv - 1;
            }
        }
    }
    remove_small_regions(w, 30);
}

fn remove_small_regions(w: &mut World, min: usize) {
    let n = w.level.len();
    let mut seen = vec![false; n];
    let mut q = VecDeque::new();
    for start in 0..n {
        if seen[start] {
            continue;
        }
        let lv = w.level[start];
        let mut region = vec![start];
        seen[start] = true;
        q.push_back(start);
        let mut border_levels = [0usize; 4];
        while let Some(i) = q.pop_front() {
            let x = i as i32 % w.w;
            let y = i as i32 / w.w;
            for (dx, dy) in N4 {
                let (nx, ny) = (x + dx, y + dy);
                if !w.in_bounds(nx, ny) {
                    continue;
                }
                let j = w.idx(nx, ny);
                if w.level[j] == lv {
                    if !seen[j] {
                        seen[j] = true;
                        region.push(j);
                        q.push_back(j);
                    }
                } else {
                    border_levels[w.level[j].min(3) as usize] += 1;
                }
            }
        }
        if region.len() < min {
            let best = (0..4).max_by_key(|&l| border_levels[l]).unwrap_or(0) as u8;
            for i in region {
                w.level[i] = best;
            }
        }
    }
}

/// Neighbouring tiles may differ by at most one level, so every drop is a
/// single readable cliff.
fn enforce_slopes(w: &mut World) {
    for _ in 0..6 {
        let mut changed = false;
        for y in 0..w.h {
            for x in 0..w.w {
                let i = w.idx(x, y);
                let mut lo = 255u8;
                for (dx, dy) in N8 {
                    let (nx, ny) = (x + dx, y + dy);
                    if w.in_bounds(nx, ny) {
                        lo = lo.min(w.level[w.idx(nx, ny)]);
                    }
                }
                if w.level[i] > lo + 1 {
                    w.level[i] = lo + 1;
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
}

fn water(w: &mut World, sea: f32) {
    let n = w.terrain.len();
    // Distance from land, for deep vs shallow.
    let mut dist = vec![u16::MAX; n];
    let mut q = VecDeque::new();
    for i in 0..n {
        if w.height[i] >= sea {
            dist[i] = 0;
            q.push_back(i);
        }
    }
    while let Some(i) = q.pop_front() {
        let x = i as i32 % w.w;
        let y = i as i32 / w.w;
        for (dx, dy) in N4 {
            let (nx, ny) = (x + dx, y + dy);
            if !w.in_bounds(nx, ny) {
                continue;
            }
            let j = w.idx(nx, ny);
            if dist[j] == u16::MAX {
                dist[j] = dist[i] + 1;
                q.push_back(j);
            }
        }
    }
    for i in 0..n {
        if w.height[i] < sea {
            w.terrain[i] = if dist[i] <= 3 { Terrain::Shallow } else { Terrain::Deep } as u8;
            w.level[i] = 0;
        }
    }
}

fn rivers(w: &mut World, rng: &mut Rng, n: &Noise) {
    let mut sources: Vec<(i32, i32)> = Vec::new();
    for _ in 0..4000 {
        if sources.len() >= 6 {
            break;
        }
        let x = rng.range(20, w.w - 20);
        let y = rng.range(16, w.h / 2);
        if w.level(x, y) < 2 || w.terrain(x, y).is_water() {
            continue;
        }
        if sources.iter().any(|&(sx, sy)| (sx - x).abs() + (sy - y).abs() < 40) {
            continue;
        }
        sources.push((x, y));
    }
    for (sx, sy) in sources {
        let mut path = Vec::new();
        let (mut x, mut y) = (sx, sy);
        let mut visited = std::collections::HashSet::new();
        let mut ok = false;
        for _ in 0..500 {
            visited.insert((x, y));
            path.push((x, y));
            if w.terrain(x, y).is_water() && !w.has(x, y, flag::RIVER) {
                ok = true;
                break;
            }
            // Flow to the lowest unvisited neighbour, with a southward bias so
            // rivers tend to cross cliffs as waterfalls facing the camera.
            let mut best: Option<((i32, i32), f32)> = None;
            for (dx, dy) in N4 {
                let (nx, ny) = (x + dx, y + dy);
                if !w.in_bounds(nx, ny) || visited.contains(&(nx, ny)) {
                    continue;
                }
                let jitter = n.get(nx as f32 * 0.21, ny as f32 * 0.21) * 0.025;
                let bias = if dy == 1 { -0.012 } else if dy == -1 { 0.01 } else { 0.0 };
                let h = w.height[w.idx(nx, ny)] + jitter + bias + w.level(nx, ny) as f32 * 0.2;
                if best.map_or(true, |(_, bh)| h < bh) {
                    best = Some(((nx, ny), h));
                }
            }
            match best {
                Some(((nx, ny), _)) => {
                    x = nx;
                    y = ny;
                }
                None => break,
            }
        }
        if !ok || path.len() < 18 {
            continue;
        }
        let mut lv = w.level(path[0].0, path[0].1);
        for (k, &(x, y)) in path.iter().enumerate() {
            if w.terrain(x, y).is_water() && !w.has(x, y, flag::RIVER) {
                break;
            }
            lv = lv.min(w.level(x, y));
            let i = w.idx(x, y);
            w.level[i] = lv;
            w.terrain[i] = Terrain::Shallow as u8;
            w.flags[i] |= flag::RIVER;
            // Widen downstream.
            if k > path.len() / 3 {
                let (px, _py) = if k > 0 { path[k - 1] } else { (x, y) };
                let (ox, oy) = if px != x { (0, 1) } else { (1, 0) };
                let (wx, wy) = (x + ox, y + oy);
                if w.in_bounds(wx, wy) && !w.terrain(wx, wy).is_water() && w.level(wx, wy) >= lv {
                    let j = w.idx(wx, wy);
                    w.level[j] = lv;
                    w.terrain[j] = Terrain::Shallow as u8;
                    w.flags[j] |= flag::RIVER;
                }
            }
        }
    }
}

fn surfaces(w: &mut World, n: &Noise) {
    // Distance to any water, for beaches.
    let len = w.terrain.len();
    let mut dist = vec![u16::MAX; len];
    let mut q = VecDeque::new();
    for i in 0..len {
        if Terrain::from_u8(w.terrain[i]).is_water() && w.flags[i] & flag::RIVER == 0 {
            dist[i] = 0;
            q.push_back(i);
        }
    }
    while let Some(i) = q.pop_front() {
        let x = i as i32 % w.w;
        let y = i as i32 / w.w;
        for (dx, dy) in N4 {
            let (nx, ny) = (x + dx, y + dy);
            if !w.in_bounds(nx, ny) {
                continue;
            }
            let j = w.idx(nx, ny);
            if dist[j] == u16::MAX {
                dist[j] = dist[i] + 1;
                q.push_back(j);
            }
        }
    }
    let land: Vec<f32> = (0..len).filter(|&i| w.level[i] == 3).map(|i| w.height[i]).collect();
    let snow_line = quantile(&land, 0.55);
    for y in 0..w.h {
        for x in 0..w.w {
            let i = w.idx(x, y);
            let t = Terrain::from_u8(w.terrain[i]);
            if t.is_water() {
                continue;
            }
            let lv = w.level[i];
            let nv = n.fbm(x as f32 * 0.09, y as f32 * 0.09, 3, 2.0, 0.5);
            let beach = 2.0 + nv * 2.5;
            w.terrain[i] = if lv == 0 && (dist[i] as f32) <= beach {
                Terrain::Sand
            } else if lv == 3 && w.height[i] > snow_line + nv * 0.03 {
                Terrain::Snow
            } else if lv == 3 || (lv == 2 && nv > 0.42) {
                Terrain::Rock
            } else {
                Terrain::Grass
            } as u8;
        }
    }
}

fn fields(w: &mut World, n_m: &Noise, n_f: &Noise) {
    for y in 0..w.h {
        for x in 0..w.w {
            let i = w.idx(x, y);
            let nx = x as f32 / w.w as f32;
            let ny = y as f32 / w.h as f32;
            let m = n_m.fbm(nx * 4.0, ny * 4.0, 4, 2.0, 0.5) * 0.5 + 0.5;
            w.moist[i] = m;
            let f = n_f.warped(nx * 6.0, ny * 6.0, 0.6, 4) * 0.5 + 0.5;
            w.forest[i] = smoothstep(0.45, 0.72, f * 0.75 + m * 0.35);
        }
    }
}

fn scatter_props(w: &mut World, rng: &mut Rng, n: &Noise) {
    let seed = w.seed as u32;
    let mut taken = vec![false; (w.w * w.h) as usize];
    for y in 1..w.h - 1 {
        for x in 1..w.w - 1 {
            let i = w.idx(x, y);
            if !w.free_for_prop(x, y) || taken[i] {
                continue;
            }
            let t = w.terrain(x, y);
            let forest = w.forest[i];
            let lv = w.level[i];
            let r = hash2f(x, y, seed ^ 0x51);
            // Keep space near villages open; they get their own decoration.
            let near_village = w.villages.iter().any(|v| {
                let dx = (v.cx - x) as f32;
                let dy = (v.cy - y) as f32;
                dx * dx + dy * dy < (v.radius as f32 + 4.0).powi(2)
            });
            let near_road = (-1..=1).any(|dy| (-1..=1).any(|dx| w.has(x + dx, y + dy, flag::ROAD | flag::PLAZA)));
            let kind = match t {
                Terrain::Grass => {
                    let tree_p = if near_village { 0.0 } else { forest * 0.42 + 0.012 };
                    if !near_road && r < tree_p {
                        let pine_p = if lv >= 2 { 0.75 } else { 0.12 };
                        let birch_p = w.moist[i] * 0.25;
                        let rr = hash2f(x, y, seed ^ 0x77);
                        Some(if rr < pine_p { PropKind::Pine } else if rr < pine_p + birch_p { PropKind::Birch } else { PropKind::Oak })
                    } else if r < tree_p + 0.03 + forest * 0.04 {
                        Some(if hash2f(x, y, seed ^ 3) < 0.2 { PropKind::BerryBush } else { PropKind::Bush })
                    } else if r < tree_p + 0.085 {
                        Some(PropKind::TallGrass)
                    } else if r < tree_p + 0.092 {
                        Some(PropKind::Rock)
                    } else if forest > 0.5 && r < tree_p + 0.1 {
                        Some(PropKind::Mushroom)
                    } else {
                        None
                    }
                }
                Terrain::Rock => {
                    if r < 0.05 {
                        Some(PropKind::OreRock)
                    } else if r < 0.13 {
                        Some(PropKind::Rock)
                    } else if r < 0.17 {
                        Some(PropKind::Boulder)
                    } else if r < 0.2 && lv < 3 {
                        Some(PropKind::Pine)
                    } else {
                        None
                    }
                }
                Terrain::Snow => (r < 0.06).then_some(PropKind::Pine).or((r < 0.1).then_some(PropKind::Rock)),
                Terrain::Sand => (r < 0.02).then_some(PropKind::Rock).or((r < 0.035).then_some(PropKind::Reeds)),
                _ => None,
            };
            let Some(kind) = kind else { continue };
            // Trees need breathing room so canopies don't become a wall.
            if kind.is_tree() {
                if (-1..=1).any(|dy| (-1..=1).any(|dx| taken[w.idx(x + dx, y + dy)])) {
                    continue;
                }
            }
            taken[i] = true;
            let variant = (hash2f(x, y, seed ^ 0x99) * 8.0) as u8;
            let hp = match kind {
                k if k.is_tree() => 5,
                PropKind::OreRock | PropKind::Boulder => 6,
                PropKind::Rock => 3,
                _ => 1,
            };
            w.add_prop(Prop { kind, x, y, variant, state: 0, hp, fw: 1, fh: 1, alive: true });
        }
    }
    // Reeds and lilies along river banks and shallows.
    for y in 1..w.h - 1 {
        for x in 1..w.w - 1 {
            let t = w.terrain(x, y);
            let r = hash2f(x, y, seed ^ 0x1234);
            if t == Terrain::Shallow && w.flags(x, y) & (flag::RIVER | flag::BRIDGE) == 0 && r < 0.03 {
                let near_land = N4.iter().any(|(dx, dy)| !w.terrain(x + dx, y + dy).is_water());
                if near_land {
                    let v = (rng.f32() * 4.0) as u8;
                    w.add_prop(Prop { kind: PropKind::Lily, x, y, variant: v, state: 0, hp: 1, fw: 1, fh: 1, alive: true });
                }
            }
        }
    }
    let _ = n;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_world() {
        let a = generate(12345);
        let b = generate(12345);
        assert_eq!(a.checksum(), b.checksum());
        let c = generate(54321);
        assert_ne!(a.checksum(), c.checksum());
    }

    #[test]
    fn world_is_sane() {
        for seed in [1u64, 2, 3, 99, 2026] {
            let w = generate(seed);
            let n = w.terrain.len() as f32;
            let water = w.terrain.iter().filter(|&&t| Terrain::from_u8(t).is_water()).count() as f32;
            assert!(water / n > 0.2 && water / n < 0.6, "seed {seed}: water {}", water / n);
            assert!(w.villages.len() >= 3, "seed {seed}: villages {}", w.villages.len());
            // Slopes are at most one level.
            for y in 0..w.h - 1 {
                for x in 0..w.w - 1 {
                    let l = w.level(x, y) as i32;
                    assert!((l - w.level(x + 1, y) as i32).abs() <= 1);
                    assert!((l - w.level(x, y + 1) as i32).abs() <= 1);
                }
            }
            let (sx, sy) = w.spawn;
            assert!(w.walkable(sx, sy), "seed {seed}: spawn not walkable");
        }
    }

    #[test]
    fn villages_connected_by_road() {
        let w = generate(777);
        let mut pf = super::super::path::PathFinder::new(w.w, w.h);
        let a = &w.villages[0];
        for b in &w.villages[1..] {
            let p = pf.walk(&w, (a.cx, a.cy + 3), (b.cx, b.cy + 3), 200_000);
            assert!(p.is_some(), "no walk path from {} to {}", a.name, b.name);
        }
    }
}
