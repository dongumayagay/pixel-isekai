//! Composite scene render: terrain + y-sorted props, shadows and villagers.
//! cargo run --release --example scene -- <seed> <season> <out.png> [x0 y0 w h]

use sim::art::canvas::Img;
use sim::art::{buildings, flora, people};
use sim::palette::{self as pal, hex};
use sim::rng::Rng;
use sim::world::bake::{Baker, CHUNK_PX};
use sim::world::villages::BuildingKind;
use sim::world::{worldgen, PropKind, CHUNK, CHUNKS_X, CHUNKS_Y, TILE};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let seed: u64 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(2026);
    let season: u8 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let out = a.get(3).cloned().unwrap_or("scene.png".into());
    let w = worldgen::generate(seed);
    let v = &w.villages[0];
    let p = |i: usize, d: i32| a.get(i).and_then(|s| s.parse().ok()).unwrap_or(d);
    let (x0, y0, tw, th) = (p(4, v.cx - 10), p(5, v.cy - 7), p(6, 20), p(7, 13));
    let (pw, ph) = (tw * TILE, th * TILE);
    let mut img = Img::new(pw, ph);
    let baker = Baker::new(seed);
    let mut buf = vec![0u8; (CHUNK_PX * CHUNK_PX * 4) as usize];
    for cy in 0..CHUNKS_Y {
        for cx in 0..CHUNKS_X {
            let (cx0, cy0) = (cx * CHUNK, cy * CHUNK);
            if cx0 + CHUNK <= x0 || cy0 + CHUNK <= y0 || cx0 >= x0 + tw || cy0 >= y0 + th {
                continue;
            }
            baker.bake_chunk(&w, cx, cy, season, &mut buf);
            for ly in 0..CHUNK_PX {
                for lx in 0..CHUNK_PX {
                    let gx = cx0 * TILE + lx - x0 * TILE;
                    let gy = cy0 * TILE + ly - y0 * TILE;
                    let s = ((ly * CHUNK_PX + lx) * 4) as usize;
                    img.set(gx, gy, [buf[s], buf[s + 1], buf[s + 2]]);
                }
            }
        }
    }
    // Collect drawables: (sort_y, x, y, sprite, pivot)
    let mut items: Vec<(i32, i32, i32, Img, (i32, i32))> = Vec::new();
    for (i, pr) in w.props.iter().enumerate() {
        if pr.x < x0 - 4 || pr.y < y0 - 2 || pr.x > x0 + tw + 4 || pr.y > y0 + th + 6 {
            continue;
        }
        let (spr, piv) = match pr.kind {
            PropKind::Oak => flora::deciduous(0, pr.variant % 3, season),
            PropKind::FruitTree => flora::deciduous(1, pr.variant % 2, season),
            PropKind::Birch => flora::deciduous(2, pr.variant % 2, season),
            PropKind::Pine => flora::pine(pr.variant % 2, season),
            PropKind::Bush => flora::bush(pr.variant % 2, season, false),
            PropKind::BerryBush => flora::bush(pr.variant % 2, season, true),
            PropKind::Rock => flora::rock(0, pr.variant, None, season),
            PropKind::Boulder => flora::rock(1, pr.variant, None, season),
            PropKind::OreRock => flora::rock(1, pr.variant, Some(pal::GOLD), season),
            PropKind::TallGrass => flora::tall_grass(pr.variant, season),
            PropKind::Reeds => flora::reeds(pr.variant),
            PropKind::Mushroom => flora::mushroom(pr.variant),
            PropKind::Lily => flora::lily(pr.variant),
            PropKind::Flowers => flora::flower_patch(pr.variant),
            PropKind::Crop => flora::crop(pr.variant % 6, pr.state.min(3)),
            PropKind::Well => buildings::well(pr.variant),
            PropKind::Lamp => buildings::lamp(),
            PropKind::Shrine => buildings::shrine(),
            PropKind::Barrel => buildings::small_prop("barrel", pr.variant),
            PropKind::Crate => buildings::small_prop("crate", pr.variant),
            PropKind::Haystack => buildings::small_prop("haystack", pr.variant),
            PropKind::Bench => buildings::small_prop("bench", pr.variant),
            PropKind::House => {
                let kind = match pr.state { 1 => BuildingKind::Inn, 2 => BuildingKind::Store, 3 => BuildingKind::Smithy, 4 => BuildingKind::Chapel, _ => BuildingKind::Home };
                let (im, piv, _) = buildings::house(kind, pr.variant / 4, i as u32 * 7919, pr.fw as i32, pr.fh as i32, false);
                let sx = (pr.x - x0) * TILE;
                let sy = (pr.y - y0 + 1) * TILE;
                items.push((sy, sx, sy, im, piv));
                continue;
            }
            _ => continue,
        };
        let sx = (pr.x - x0) * TILE + TILE / 2;
        let sy = (pr.y - y0) * TILE + TILE * 3 / 4;
        items.push((sy, sx, sy, spr, piv));
    }
    // Some villagers around the plaza.
    let mut rng = Rng::new(seed ^ 0xfeed);
    for k in 0..8 {
        let mut look = people::Look::random(&mut rng, k % 2 == 0, if k == 7 { 0 } else { 1 });
        look.hat = [1, 0, 2, 0, 5, 7, 0, 0][k];
        let frames = people::sheet(&look);
        let f = [0usize, 2, 16, 8, 18, 0, 4, 1][k];
        let ang = k as f32 * 0.8;
        let sx = (v.cx - x0) * TILE + 16 + (ang.cos() * 68.0) as i32;
        let sy = (v.cy - y0) * TILE + 16 + (ang.sin() * 52.0) as i32 + 12;
        let mut fr = frames[f].clone();
        if k % 3 == 1 {
            fr = fr.flipped();
        }
        items.push((sy, sx, sy, fr, people::PIVOT));
    }
    items.sort_by_key(|i| i.0);
    for (_, x, y, spr, piv) in &items {
        // Soft contact shadow.
        let rx = (spr.w as f32 * 0.32).clamp(3.0, 18.0);
        let ry = (rx * 0.38).max(1.5);
        let (cx, cy) = (*x as f32, *y as f32 - 1.0);
        for yy in (cy - ry) as i32..=(cy + ry) as i32 {
            for xx in (cx - rx) as i32..=(cx + rx) as i32 {
                let nx = (xx as f32 + 0.5 - cx) / rx;
                let ny = (yy as f32 + 0.5 - cy) / ry;
                if nx * nx + ny * ny <= 1.0 {
                    if let Some(c) = img.get(xx, yy) {
                        img.set(xx, yy, pal::mix(c, hex(0x1c1830), 0.35));
                    }
                }
            }
        }
        img.blit(spr, x - piv.0, y - piv.1, false);
    }
    // Warm afternoon grade + vignette.
    for yy in 0..ph {
        for xx in 0..pw {
            if let Some(c) = img.get(xx, yy) {
                let mut c2 = pal::mix(c, hex(0xffe0a8), 0.06);
                let dx = xx as f32 / pw as f32 - 0.5;
                let dy = yy as f32 / ph as f32 - 0.5;
                let vig = ((dx * dx + dy * dy).sqrt() - 0.35).max(0.0) * 0.8;
                c2 = pal::mix(c2, hex(0x14102a), vig);
                img.set(xx, yy, c2);
            }
        }
    }
    let s = 2usize;
    let (ow, oh) = (pw as usize * s, ph as usize * s);
    let mut o = vec![0u8; ow * oh * 4];
    for yy in 0..oh {
        for xx in 0..ow {
            let si = ((yy / s) * pw as usize + xx / s) * 4;
            o[(yy * ow + xx) * 4..(yy * ow + xx) * 4 + 4].copy_from_slice(&img.px[si..si + 4]);
        }
    }
    let f = std::fs::File::create(&out).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), ow as u32, oh as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.write_header().unwrap().write_image_data(&o).unwrap();
}
