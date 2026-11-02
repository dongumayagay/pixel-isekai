//! Fast loop for the core roster: lint every frame of every character and
//! write one sheet (front, side, back at idle and mid-stride).
//! cargo run --release --example roster -- [out.png] [name filter, e.g. Dwarf,Rogue]
//! SCALE=4 enlarges the sheet (default 2).
use sim::art::canvas::Img;
use sim::art::people;
use sim::palette::hex;

fn main() {
    let out = std::env::args().nth(1).unwrap_or("../.context/review/roster.png".into());
    let only: Vec<String> = std::env::args().nth(2).map(|v| v.split(',').map(str::to_owned).collect()).unwrap_or_default();
    let cast: Vec<_> = people::cast().into_iter().filter(|(n, _)| only.is_empty() || only.iter().any(|o| n.contains(o.as_str()))).collect();
    let views = [(0u8, 0u16), (2, 0), (1, 0), (0, 5), (2, 5), (1, 5)];
    let (cw, ch) = (people::FW, people::FH);
    let per_row = cast.len().min(2);
    let rows = cast.len().div_ceil(per_row);
    let mut img = Img::new(cw * (views.len() * per_row) as i32, ch * rows as i32);
    for y in 0..img.h {
        for x in 0..img.w {
            img.set(x, y, if (x / (cw * views.len() as i32) + y / ch) % 2 == 0 { hex(0x6a9a5a) } else { hex(0x5a8a4e) });
        }
    }
    let mut problems = 0;
    for (i, (name, look)) in cast.iter().enumerate() {
        for issue in people::lint(look).into_iter().filter(|i| !people::ADVISORY.contains(&i.kind)) {
            problems += 1;
            println!("{name}: {} d{} f{} {}", issue.kind, issue.dir, issue.frame, issue.detail);
        }
        for (k, &(d, f)) in views.iter().enumerate() {
            img.blit(&people::frame(look, d, f), ((i % per_row) * views.len() + k) as i32 * cw, (i / per_row) as i32 * ch, false);
        }
    }
    println!("{} characters, {problems} defects -> {out}", cast.len());
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
