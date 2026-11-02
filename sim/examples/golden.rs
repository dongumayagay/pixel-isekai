// Golden-output harness for behaviour-preserving refactors. Hashes every
// observable output of worldgen, the sprite atlas, chunk baking and a long
// scripted play session for three seeds. Too slow (minutes) for `cargo test`;
// run it before and after a change and diff the output:
//
//     cargo run --release --example golden > before.txt
//     cargo run --release --example golden bake > before_bake.txt  (all chunks, all seasons)
use sim::game::Game;
use sim::world::{CHUNKS_X, CHUNKS_Y};

fn fnv(h: &mut u64, b: &[u8]) {
    for &x in b {
        *h ^= x as u64;
        *h = h.wrapping_mul(0x100_0000_01b3);
    }
}
fn hf(v: &[f32]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325;
    for x in v {
        fnv(&mut h, &x.to_bits().to_le_bytes());
    }
    h
}
fn hb(v: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325;
    fnv(&mut h, v);
    h
}
fn houses(v: Vec<f32>) -> u64 {
    // HashMap order: sort the 4-float records first.
    let mut recs: Vec<Vec<u32>> = v.chunks(4).map(|c| c.iter().map(|x| x.to_bits()).collect()).collect();
    recs.sort();
    let flat: Vec<f32> = recs.concat().into_iter().map(f32::from_bits).collect();
    hf(&flat)
}
fn hs(s: &str) -> u64 {
    hb(s.as_bytes())
}

fn villagers_digest(g: &Game) -> u64 {
    let mut s = String::new();
    for v in &g.villagers {
        s += &format!(
            "{} {} {} {} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}\n",
            v.id, v.first, v.family, v.female, v.race, v.age.to_bits(), serde_json::to_string(&v.look).unwrap(),
            v.traits, v.needs, v.job, v.village, v.home, v.actor, v.goal, v.phase, v.timer.to_bits(), v.target,
            v.work_prop, v.partner, v.dating, v.parents, v.children, v.opinions, v.player_opinion.to_bits(),
            (v.emote, v.emote_t.to_bits(), v.inside, v.pregnant.to_bits(), v.gold, v.think_cd.to_bits(), v.alive),
            (&v.memories, v.talked_today)
        );
    }
    hs(&s)
}

fn main() {
    let full_bake = std::env::args().any(|a| a == "bake");
    for seed in [2026u64, 99, 7] {
        let mut g = Game::new(seed);
        println!("seed {seed} world {:x} worldjson {:x}", g.world.checksum(), hs(&serde_json::to_string(&g.world).unwrap()));
        let mut ah = 0xcbf2_9ce4_8422_2325u64;
        for p in &g.sprites.atlas.pages {
            fnv(&mut ah, &p.px);
        }
        for s in &g.sprites.atlas.sprites {
            fnv(&mut ah, format!("{:?}", (s.page, s.x, s.y, s.w, s.h, s.px, s.py)).as_bytes());
        }
        println!("  atlas {ah:x}");
        println!("  props {:x} lights {:x} houses {:x}", hf(&g.static_props()), hf(&g.static_lights()), houses(g.house_info()));
        g.speed = 6.0;
        let mut acc = 0xcbf2_9ce4_8422_2325u64;
        for t in 0..60_000u32 {
            g.tick(0.1);
            if t % 97 == 0 {
                fnv(&mut acc, &hf(&g.snapshot()).to_le_bytes());
                fnv(&mut acc, g.take_ui().as_bytes());
            }
            if t % 523 == 0 {
                // Talk to someone.
                let alive: Vec<_> = g.villagers.iter().filter(|v| v.alive && !v.actor.hidden).map(|v| (v.actor.x, v.actor.y)).collect();
                if !alive.is_empty() {
                    let (x, y) = alive[(t as usize / 523) % alive.len()];
                    g.command(&format!(r#"{{"t":"teleport","x":{},"y":{}}}"#, x + 20.0, y));
                    g.command(&format!(r#"{{"t":"tap","x":{},"y":{}}}"#, x, y - 20.0));
                }
            }
            if t % 811 == 0 {
                // Harvest the k-th harvestable prop.
                let hp: Vec<_> = g.world.props.iter().filter(|p| p.alive && (p.kind.is_tree() || matches!(p.kind, sim::world::PropKind::Rock | sim::world::PropKind::Crop | sim::world::PropKind::BerryBush | sim::world::PropKind::Mushroom | sim::world::PropKind::OreRock | sim::world::PropKind::Stump))).map(|p| (p.x, p.y)).collect();
                let (px, py) = hp[(t as usize * 7) % hp.len()];
                for k in 0..6 {
                    let (tx, ty) = (px * 32 + 16, py * 32 + 16);
                    if k == 0 {
                        g.command(&format!(r#"{{"t":"teleport","x":{},"y":{}}}"#, tx, ty + 32 + 6));
                    }
                    g.command(&format!(r#"{{"t":"tap","x":{},"y":{}}}"#, tx, ty));
                    g.tick(0.1);
                }
            }
            if t % 1999 == 0 {
                g.command(&format!(r#"{{"t":"weather","w":{},"i":0.7}}"#, t % 6));
                g.command(r#"{"t":"dir","x":1,"y":0.3}"#);
            }
            if t % 1999 == 40 {
                g.command(r#"{"t":"dir","x":0,"y":0}"#);
                let w = g.world.w as f32 * 16.0;
                g.command(&format!(r#"{{"t":"tap","x":{},"y":{}}}"#, w + (t % 300) as f32, w - 40.0));
            }
            if t % 7001 == 0 {
                g.command(r#"{"t":"skip","hours":3}"#);
            }
            if t % 2500 == 0 {
                let ids: Vec<u32> = g.villagers.iter().map(|v| v.id).collect();
                for id in ids.iter().step_by(5) {
                    fnv(&mut acc, g.inspect(*id).as_bytes());
                }
                fnv(&mut acc, g.status().as_bytes());
                fnv(&mut acc, &[g.take_changed_props() as u8]);
                fnv(&mut acc, &hf(&g.static_props()).to_le_bytes());
                fnv(&mut acc, &hf(&g.static_lights()).to_le_bytes());
            }
        }
        println!("  run {acc:x}");
        println!("  villagers {:x} n {}", villagers_digest(&g), g.villagers.len());
        println!("  rng {:?} clock {:?}", g.rng, g.clock);
        println!("  chronicle {:x} list {:x} status {:x}", hs(&g.chronicle_json()), hs(&g.villager_list()), hs(&g.status()));
        println!("  player {:?} inv {:?} gold {} energy {}", g.player.actor, g.player.inventory, g.player.gold, g.player.energy);
        println!("  stocks {:?}", g.stocks);
        println!("  minimap {:x} worldjson {:x}", hb(&g.minimap()), hs(&serde_json::to_string(&g.world).unwrap()));
        println!("  atlas sprites {}", g.sprites.atlas.sprites.len());
        let mut bh = 0xcbf2_9ce4_8422_2325u64;
        let step = if full_bake { 1 } else { 3 };
        for cy in (0..CHUNKS_Y).step_by(step) {
            for cx in (0..CHUNKS_X).step_by(step) {
                fnv(&mut bh, &g.bake_chunk(cx, cy));
            }
        }
        println!("  bake {bh:x}");
        if full_bake {
            for s in 0..4u32 {
                let mut bh = 0xcbf2_9ce4_8422_2325u64;
                for cy in 0..CHUNKS_Y {
                    for cx in 0..CHUNKS_X {
                        let mut buf = vec![0u8; (sim::world::bake::CHUNK_PX * sim::world::bake::CHUNK_PX * 4) as usize];
                        g.baker.bake_chunk(&g.world, cx, cy, s as u8, &mut buf);
                        fnv(&mut bh, &buf);
                    }
                }
                println!("  bake season {s} {bh:x}");
            }
        }
    }
}
