//! Fingerprint of every non-character sprite the art generators produce
//! (flora, buildings, props, emotes) plus an atlas packed from them. Run it
//! before and after a refactor of `art/`: identical hashes mean identical
//! pixels, pivots and house fx metadata.
//! cargo run --release --example art_hash

use sim::art::atlas::Atlas;
use sim::art::canvas::Img;
use sim::art::{buildings, flora, fx};
use sim::palette as pal;
use sim::world::villages::BuildingKind;

/// FNV-1a, 64-bit: tiny, dependency-free and order sensitive.
struct Fnv(u64);

impl Fnv {
    fn new() -> Fnv {
        Fnv(0xcbf2_9ce4_8422_2325)
    }
    fn bytes(&mut self, b: &[u8]) {
        for &x in b {
            self.0 = (self.0 ^ x as u64).wrapping_mul(0x0100_0000_01b3);
        }
    }
    fn int(&mut self, v: i64) {
        self.bytes(&v.to_le_bytes());
    }
    fn sprite(&mut self, (img, pivot): &(Img, (i32, i32))) {
        self.int(img.w as i64);
        self.int(img.h as i64);
        self.int(pivot.0 as i64);
        self.int(pivot.1 as i64);
        self.bytes(&img.px);
    }
}

fn flora_sprites() -> Vec<(Img, (i32, i32))> {
    let mut out = Vec::new();
    let ores: [Option<pal::Ramp>; 3] = [None, Some(pal::GOLD), Some(&[0x3a1a5a, 0x6a3aa0, 0xa070e0, 0xe0c0ff])];
    for s in 0..4u8 {
        for v in 0..4u8 {
            for kind in 0..3u8 {
                out.push(flora::deciduous(kind, v, s));
            }
            out.push(flora::pine(v, s));
            out.push(flora::bush(v, s, false));
            out.push(flora::bush(v, s, true));
            out.push(flora::tall_grass(v, s));
            for size in 0..2u8 {
                for ore in ores {
                    out.push(flora::rock(size, v, ore, s));
                }
            }
        }
        out.push(flora::stump(s));
    }
    for v in 0..6u8 {
        out.push(flora::reeds(v));
        out.push(flora::mushroom(v));
        out.push(flora::lily(v));
        out.push(flora::flower_patch(v));
    }
    for kind in 0..6u8 {
        for stage in 0..5u8 {
            out.push(flora::crop(kind, stage));
        }
    }
    out
}

fn prop_sprites() -> Vec<(Img, (i32, i32))> {
    let mut out = Vec::new();
    for c in 0..4u8 {
        out.push(buildings::well(c));
    }
    out.push(buildings::lamp());
    out.push(buildings::shrine());
    for kind in ["barrel", "crate", "haystack", "bench", "fence", "campfire", "tent", "stall", "unknown"] {
        for v in 0..12u8 {
            out.push(buildings::small_prop(kind, v));
        }
    }
    for n in 1..=12 {
        out.push(fx::emote(n));
    }
    out
}

fn main() {
    let mut total = Fnv::new();
    let mut atlas = Atlas::new(4);
    let mut packed = 0;

    for (label, sprites) in [("flora", flora_sprites()), ("props", prop_sprites())] {
        let mut h = Fnv::new();
        for s in &sprites {
            h.sprite(s);
            if atlas.try_add(0, &s.0, s.1).is_none() {
                atlas.add(1, &format!("{label}_{packed}"), &s.0, s.1);
            }
            packed += 1;
        }
        println!("{label:<10} {:>5} sprites  {:016x}", sprites.len(), h.0);
        total.int(h.0 as i64);
    }

    let kinds = [BuildingKind::Home, BuildingKind::Inn, BuildingKind::Store, BuildingKind::Smithy, BuildingKind::Chapel];
    let mut h = Fnv::new();
    let mut n = 0;
    for kind in kinds {
        let mut sizes = vec![kind.size()];
        sizes.extend([(4, 3), (5, 4), (7, 4), (6, 5)]);
        for (fw, fh) in sizes {
            for culture in 0..4u8 {
                for seed in 0..24u32 {
                    for lit in [false, true] {
                        let (img, pivot, hfx) = buildings::house(kind, culture, seed.wrapping_mul(7919) ^ 0xa5, fw as i32, fh as i32, lit);
                        h.sprite(&(img, pivot));
                        h.bytes(serde_json::to_string(&hfx).unwrap().as_bytes());
                        n += 1;
                    }
                }
            }
        }
    }
    println!("{:<10} {:>5} sprites  {:016x}", "houses", n, h.0);
    total.int(h.0 as i64);

    // Atlas packing and the canvas helpers not exercised above.
    let mut h = Fnv::new();
    for page in &atlas.pages {
        h.bytes(&page.px);
    }
    h.bytes(serde_json::to_string(&atlas.sprites).unwrap().as_bytes());
    h.bytes(format!("{:?}", atlas.dirty).as_bytes());
    let mut tree = flora::deciduous(0, 0, 1).0.flipped();
    tree.clear_px(40, 60);
    tree.outline_solid(pal::INK);
    h.sprite(&(tree, (0, 0)));
    println!("{:<10} {:>5} sprites  {:016x}", "atlas", atlas.sprites.len(), h.0);
    total.int(h.0 as i64);

    println!("TOTAL      {:016x}", total.0);
}
