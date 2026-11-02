//! Visual QA loop: lint many random characters, summarise defects, and
//! write contact sheets (random sample + failures) for eyeballing.
//! cargo run --release --example qa -- [count] [seed] [outdir]
use std::collections::BTreeMap;

use sim::art::canvas::Img;
use sim::art::people::{self, Look};
use sim::palette::{hex, mix};
use sim::rng::Rng;

fn sheet(looks: &[Look], frames: &[(u8, u16)], out: &str) {
    let cols = 12usize.min(looks.len().max(1));
    let (cw, ch) = (people::FW, people::FH);
    let per = frames.len() as i32;
    let rows = looks.len().div_ceil(cols);
    let (w, h) = (cols as i32 * cw * per, rows as i32 * ch);
    let mut img = Img::new(w.max(1), h.max(1));
    for y in 0..img.h {
        for x in 0..img.w {
            let cell = ((x / (cw * per)) + (y / ch)) % 2 == 0;
            img.set(x, y, if cell { hex(0x6a9a5a) } else { mix(hex(0x6a9a5a), hex(0x3f6a48), 0.4) });
        }
    }
    for (i, look) in looks.iter().enumerate() {
        for (k, &(d, f)) in frames.iter().enumerate() {
            let fr = people::frame(look, d, f);
            img.blit(&fr, (i % cols) as i32 * cw * per + k as i32 * cw, (i / cols) as i32 * ch, false);
        }
    }
    let s = 2usize;
    let (ow, oh) = (img.w as usize * s, img.h as usize * s);
    let mut o = vec![0u8; ow * oh * 4];
    for yy in 0..oh {
        for xx in 0..ow {
            let si = ((yy / s) * img.w as usize + xx / s) * 4;
            o[(yy * ow + xx) * 4..(yy * ow + xx) * 4 + 4].copy_from_slice(&img.px[si..si + 4]);
        }
    }
    let f = std::fs::File::create(out).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), ow as u32, oh as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.write_header().unwrap().write_image_data(&o).unwrap();
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let n: usize = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(400);
    let seed: u64 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let dir = a.get(3).cloned().unwrap_or(".context/shots".into());
    let mut rng = Rng::new(seed);
    let t = std::time::Instant::now();
    let looks: Vec<Look> = (0..n).map(|_| Look::random_any(&mut rng)).collect();
    let mut by_kind: BTreeMap<&str, usize> = BTreeMap::new();
    let mut failing = Vec::new();
    let mut shown: Vec<(&str, u8, u8)> = Vec::new();
    for (i, l) in looks.iter().enumerate() {
        let issues = people::lint(l);
        for is in &issues {
            *by_kind.entry(is.kind).or_default() += 1;
        }
        for is in &issues {
            if is.kind != "palette" && !shown.contains(&(is.kind, l.race, l.outfit)) && shown.len() < 30 {
                shown.push((is.kind, l.race, l.outfit));
                println!("#{i} race {} style {} outfit {} hat {} held {} age {} f {}: {} d{} f{} {}", l.race, l.hair_style, l.outfit, l.hat, l.held, l.age, l.female, is.kind, is.dir, is.frame, is.detail);
            }
        }
        if issues.iter().any(|i| !people::ADVISORY.contains(&i.kind)) {
            failing.push(*l);
        }
    }
    // Mirror-diff of the first asymmetric front view, differing pixels in red.
    let asym = looks.iter().find_map(|l| people::lint(l).into_iter().find(|i| i.kind == "asymmetric").map(|i| (*l, i)));
    if let Some((l, issue)) = asym {
        let l = &l;
        let fr = people::frame(l, issue.dir, issue.frame);
        let mut d = fr.clone();
        for y in 0..fr.h {
            for x in 0..fr.w {
                if fr.get(x, y) != fr.get(fr.w - 1 - x, y) {
                    d.set(x, y, hex(0xff0000));
                }
            }
        }
        let mut both = Img::new(fr.w * 2, fr.h);
        both.blit(&fr, 0, 0, false);
        both.blit(&d, fr.w, 0, false);
        let s = 6usize;
        let (ow, oh) = (both.w as usize * s, both.h as usize * s);
        let mut o = vec![0u8; ow * oh * 4];
        for yy in 0..oh {
            for xx in 0..ow {
                let si = ((yy / s) * both.w as usize + xx / s) * 4;
                o[(yy * ow + xx) * 4..(yy * ow + xx) * 4 + 4].copy_from_slice(&both.px[si..si + 4]);
            }
        }
        let f = std::fs::File::create(format!("{dir}/qa_asym.png")).unwrap();
        let mut enc = png::Encoder::new(std::io::BufWriter::new(f), ow as u32, oh as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.write_header().unwrap().write_image_data(&o).unwrap();
        println!("asym sample d{} f{}: {:?}", issue.dir, issue.frame, l);
        for y in 0..fr.h {
            for x in 0..fr.w / 2 {
                let (a, b) = (fr.get(x, y), fr.get(fr.w - 1 - x, y));
                if a != b {
                    println!("  ({x},{y}) {:?} vs ({},{y}) {:?}", a, fr.w - 1 - x, b);
                }
            }
        }
    }
    let mut pc: Vec<usize> = Vec::new();
    for l in looks.iter().take(60) {
        for is in people::lint(l) {
            if is.kind == "palette" {
                pc.push(is.detail.split(' ').next().unwrap().parse().unwrap());
            }
        }
    }
    pc.sort();
    if !pc.is_empty() {
        println!("palette overflow: median {} max {}", pc[pc.len() / 2], pc[pc.len() - 1]);
    }
    println!("{n} looks, {} failing, {:?}, {:.0} ms", failing.len(), by_kind, t.elapsed().as_secs_f64() * 1000.0);
    let views = [(0u8, 0u16), (2, 0), (1, 0)];
    sheet(&looks[..n.min(36)], &views, &format!("{dir}/qa_sample.png"));
    if !failing.is_empty() {
        sheet(&failing[..failing.len().min(24)], &views, &format!("{dir}/qa_fail.png"));
    }
}
