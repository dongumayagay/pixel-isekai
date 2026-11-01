//! Bake a region of a world to PNG for fast art iteration:
//! cargo run --release --example preview -- <seed> <season> <x0> <y0> <w> <h> <scale> <out.png>
//! Region is in tiles. Pass "full" as x0 to render the whole map at 1/4 scale.

use sim::world::bake::{Baker, CHUNK_PX};
use sim::world::{worldgen, CHUNKS_X, CHUNKS_Y, CHUNK, TILE};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let seed: u64 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(2026);
    let season: u8 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let t0 = std::time::Instant::now();
    let w = worldgen::generate(seed);
    eprintln!("generate: {:?}, villages: {:?}", t0.elapsed(), w.villages.iter().map(|v| (&v.name, v.cx, v.cy)).collect::<Vec<_>>());
    let baker = Baker::new(seed);
    let full = a.get(3).map(|s| s == "full").unwrap_or(false);
    let (x0, y0, tw, th, scale) = if full {
        (0, 0, w.w, w.h, 1)
    } else {
        let p = |i: usize, d: i32| a.get(i).and_then(|s| s.parse().ok()).unwrap_or(d);
        let (vx, vy) = w.villages.first().map(|v| (v.cx, v.cy)).unwrap_or((96, 96));
        (p(3, vx - 12), p(4, vy - 10), p(5, 26), p(6, 18), p(7, 3))
    };
    let out = a.get(8).cloned().unwrap_or_else(|| if full { "map.png".into() } else { "region.png".into() });

    let pw = (tw * TILE) as usize;
    let ph = (th * TILE) as usize;
    let mut img = vec![0u8; pw * ph * 4];
    let mut buf = vec![0u8; (CHUNK_PX * CHUNK_PX * 4) as usize];
    let t1 = std::time::Instant::now();
    let mut baked = 0;
    for cy in 0..CHUNKS_Y {
        for cx in 0..CHUNKS_X {
            let (cx0, cy0) = (cx * CHUNK, cy * CHUNK);
            if cx0 + CHUNK <= x0 || cy0 + CHUNK <= y0 || cx0 >= x0 + tw || cy0 >= y0 + th {
                continue;
            }
            baker.bake_chunk(&w, cx, cy, season, &mut buf);
            baked += 1;
            for ly in 0..CHUNK_PX {
                for lx in 0..CHUNK_PX {
                    let gx = cx0 * TILE + lx - x0 * TILE;
                    let gy = cy0 * TILE + ly - y0 * TILE;
                    if gx < 0 || gy < 0 || gx as usize >= pw || gy as usize >= ph {
                        continue;
                    }
                    let s = ((ly * CHUNK_PX + lx) * 4) as usize;
                    let d = (gy as usize * pw + gx as usize) * 4;
                    img[d..d + 3].copy_from_slice(&buf[s..s + 3]);
                    img[d + 3] = 255;
                }
            }
        }
    }
    eprintln!("baked {baked} chunks in {:?}", t1.elapsed());
    let (img, pw, ph) = if full {
        // Downsample 4x for an overview.
        let (nw, nh) = (pw / 4, ph / 4);
        let mut o = vec![0u8; nw * nh * 4];
        for y in 0..nh {
            for x in 0..nw {
                let s = ((y * 4) * pw + x * 4) * 4;
                let d = (y * nw + x) * 4;
                o[d..d + 4].copy_from_slice(&img[s..s + 4]);
            }
        }
        (o, nw, nh)
    } else if scale > 1 {
        let s = scale as usize;
        let (nw, nh) = (pw * s, ph * s);
        let mut o = vec![0u8; nw * nh * 4];
        for y in 0..nh {
            for x in 0..nw {
                let si = ((y / s) * pw + x / s) * 4;
                let d = (y * nw + x) * 4;
                o[d..d + 4].copy_from_slice(&img[si..si + 4]);
            }
        }
        (o, nw, nh)
    } else {
        (img, pw, ph)
    };
    let f = std::fs::File::create(&out).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), pw as u32, ph as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&img).unwrap();
    eprintln!("wrote {out}");
}
