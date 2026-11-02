//! Contact sheets of trait pairings that can clash, for reviewing the lock
//! table in `people/locks.rs`. Every cell is rendered through `frame`, so
//! a locked pairing shows what `Look::normalized` turned it into.
//! cargo run --release --example combos -- <hats|ears|held|swim|glasses> [out.png]
//! DIR=0|1|2 picks the view (default front); ROWS=3,4 and COLS=2,5 keep
//! only those rows and columns; SCALE=4 enlarges (default 2); LINT=1 also
//! lints every cell.
use sim::art::canvas::Img;
use sim::art::people::{self, held, outfit, Look};
use sim::palette::hex;

fn env_list(name: &str) -> Option<Vec<usize>> {
    std::env::var(name).ok().map(|v| v.split(',').filter_map(|s| s.trim().parse().ok()).collect())
}

fn main() {
    let mode = std::env::args().nth(1).unwrap_or("hats".into());
    let out = std::env::args().nth(2).unwrap_or(format!("../.context/review/combos-{mode}.png"));
    let dir: u8 = std::env::var("DIR").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let base = people::cast()[1].1;
    let hats = || (0..people::HATS).collect::<Vec<_>>();
    // Rows of looks; heads only unless whole bodies matter.
    let (rows, ch): (Vec<Vec<Look>>, i32) = match mode.as_str() {
        "hats" => ((0..people::HAIR_STYLES).map(|s| hats().into_iter().map(|hat| Look { hair_style: s, hat, held: 0, outfit: 0, ..base }).collect()).collect(), 64),
        "ears" => ([(people::RACE_ELF, 0), (people::RACE_BEAST, 0), (people::RACE_BEAST, 1), (people::RACE_BEAST, 2), (people::RACE_BEAST, 3), (people::RACE_DEMON, 0), (people::RACE_DEMON, 1)]
            .into_iter()
            .map(|(race, beast)| hats().into_iter().map(|hat| Look { race, beast, hair_style: 5, hat, held: 0, outfit: 0, ..base }).collect())
            .collect(), 64),
        "glasses" => ([0u8, 1, 5, 13].into_iter().map(|s| hats().into_iter().map(|hat| Look { hair_style: s, hat, glasses: true, held: 0, outfit: 0, ..base }).collect()).collect(), 64),
        "held" => ((0..outfit::COUNT).map(|o| (0..held::COUNT).map(|h| Look { outfit: o, held: h, hair_style: 5, hat: 0, ..base }).collect()).collect(), people::FH),
        "swim" => ([(outfit::SWIM, true), (outfit::SWIM, false), (outfit::MICRO, true), (outfit::MICRO, false)]
            .into_iter()
            .map(|(o, female)| hats().into_iter().map(|hat| Look { outfit: o, female, hat, held: 0, hair_style: if female { 2 } else { 0 }, ..base }).collect())
            .collect(), people::FH),
        _ => panic!("unknown mode {mode}"),
    };
    let keep = |list: &Option<Vec<usize>>, i: usize| list.as_ref().is_none_or(|l| l.contains(&i));
    let (only_rows, only_cols) = (env_list("ROWS"), env_list("COLS"));
    let rows: Vec<Vec<Look>> = rows
        .into_iter()
        .enumerate()
        .filter(|(r, _)| keep(&only_rows, *r))
        .map(|(_, row)| row.into_iter().enumerate().filter(|(c, _)| keep(&only_cols, *c)).map(|(_, l)| l).collect())
        .collect();
    let cw = people::FW;
    let cols = rows[0].len() as i32;
    let mut img = Img::new(cw * cols, ch * rows.len() as i32);
    for y in 0..img.h {
        for x in 0..img.w {
            img.set(x, y, if (x / cw + y / ch) % 2 == 0 { hex(0x6a9a5a) } else { hex(0x5a8a4e) });
        }
    }
    let lint = std::env::var("LINT").is_ok();
    for (r, row) in rows.iter().enumerate() {
        for (c, look) in row.iter().enumerate() {
            if lint {
                for i in people::lint(look).into_iter().filter(|i| !people::ADVISORY.contains(&i.kind)) {
                    println!("row {r} col {c}: {} d{} f{} {}", i.kind, i.dir, i.frame, i.detail);
                }
            }
            let mut cell = Img::new(cw, ch);
            cell.blit(&people::frame(look, dir, 0), 0, 0, false);
            img.blit(&cell, c as i32 * cw, r as i32 * ch, false);
        }
    }
    let s: usize = std::env::var("SCALE").ok().and_then(|v| v.parse().ok()).unwrap_or(2);
    let (ow, oh) = (img.w as usize * s, img.h as usize * s);
    let mut o = vec![0u8; ow * oh * 4];
    for yy in 0..oh {
        for xx in 0..ow {
            let si = ((yy / s) * img.w as usize + xx / s) * 4;
            o[(yy * ow + xx) * 4..(yy * ow + xx) * 4 + 4].copy_from_slice(&img.px[si..si + 4]);
        }
    }
    let f = std::fs::File::create(&out).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), ow as u32, oh as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.write_header().unwrap().write_image_data(&o).unwrap();
}
