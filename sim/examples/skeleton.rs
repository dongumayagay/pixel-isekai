//! The core cast in three views with the skeleton's joints drawn over them,
//! for proportion review. Row 1 adds a height ladder: petite, average and
//! tall for a woman and a man.
//! cargo run --release --example skeleton -- out.png   (SCALE=4 enlarges)
use sim::art::canvas::Img;
use sim::art::people::{self, held, Look};
use sim::palette::hex;

fn main() {
    let out = std::env::args().nth(1).unwrap_or("skeleton.png".into());
    let mut cast: Vec<(String, Look)> = Vec::new();
    let base = Look { skin: 1, hair: 3, hair_style: 2, primary: 1, secondary: 0, outfit: 0, hat: 0, accent: 1, female: true, age: 1, eyes: 0, mature: true, race: 0, beast: 0, held: held::NONE, glasses: false, figure: 2, build: 1, height: 1, facial: 0, cut: 0 };
    for female in [true, false] {
        for height in 0..3u8 {
            cast.push((format!("h{height}{}", if female { 'f' } else { 'm' }), Look { female, height, hair_style: if female { 2 } else { 0 }, ..base }));
        }
    }
    // Figure extremes at both ends of the height range.
    for (height, figure) in [(0u8, 0u8), (0, 4), (2, 0), (2, 4)] {
        cast.push((format!("h{height}f{figure}"), Look { height, figure, ..base }));
    }
    cast.extend(people::cast().into_iter().map(|(n, l)| (n.to_string(), l)));
    let views = [0u8, 2, 1];
    let (cw, ch) = (people::FW, people::FH);
    let per_row = 6;
    let rows = cast.len().div_ceil(per_row);
    let mut img = Img::new(cw * (views.len() * per_row) as i32, ch * rows as i32);
    for y in 0..img.h {
        for x in 0..img.w {
            img.set(x, y, if (x / (cw * views.len() as i32) + y / ch) % 2 == 0 { hex(0x6a9a5a) } else { hex(0x5a8a4e) });
        }
    }
    for (i, (_, look)) in cast.iter().enumerate() {
        for (k, &d) in views.iter().enumerate() {
            let ox = ((i % per_row) * views.len() + k) as i32 * cw;
            let oy = (i / per_row) as i32 * ch;
            img.blit(&people::frame(look, d, 0), ox, oy, false);
            for (name, x, y) in people::skeleton(look, d, 0) {
                let col = match name {
                    "head" | "crown" | "eye" | "mouth" => hex(0xff2040),
                    "neck_top" | "neck" | "chest" | "bust" | "waist" | "hip" | "crotch" => hex(0xffe020),
                    n if n.starts_with("sh") || n.starts_with("el") || n.starts_with("wr") => hex(0x20c0ff),
                    _ => hex(0xff80ff),
                };
                let (px, py) = (ox + x.floor() as i32, oy + y.floor() as i32);
                if px >= 0 && py >= 0 && px < img.w && py < img.h {
                    img.set(px, py, col);
                }
            }
        }
    }
    println!("{} figures -> {out}", cast.len());
    let s: usize = std::env::var("SCALE").ok().and_then(|v| v.parse().ok()).unwrap_or(3);
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
