//! Body types side by side (front, side, back) in a fitted tunic, for
//! anatomy review. cargo run --release --example bodies -- out.png [outfit]
use sim::art::canvas::Img;
use sim::art::people::{self, held, Look};
use sim::palette::hex;

fn main() {
    let mut args = std::env::args().skip(1);
    let out = args.next().unwrap_or("bodies.png".into());
    let outfit: u8 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let base = Look { skin: 1, hair: 3, hair_style: 2, primary: 1, secondary: 0, outfit, hat: 0, accent: 1, female: true, age: 1, eyes: 0, mature: true, race: 0, beast: 0, held: held::NONE, glasses: false, figure: 2, build: 1, height: 1, facial: 0, cut: 0 };
    let mut looks = vec![];
    for figure in 1..=4 {
        looks.push(Look { figure, ..base });
    }
    for build in [0u8, 2, 3] {
        looks.push(Look { build, ..base });
    }
    for build in 0..4u8 {
        looks.push(Look { female: false, build, hair_style: 0, ..base });
    }
    let views = [(0u8, 0u16), (2, 0), (1, 0)];
    let (cw, ch) = (people::FW, people::FH);
    let cols = 6;
    let rows = (looks.len() + 1) / 2;
    let mut img = Img::new(cw * cols, ch * rows as i32);
    for y in 0..img.h {
        for x in 0..img.w {
            img.set(x, y, if (x / (cw * 3) + y / ch) % 2 == 0 { hex(0x6a9a5a) } else { hex(0x5a8a4e) });
        }
    }
    for (i, look) in looks.iter().enumerate() {
        for (k, &(d, f)) in views.iter().enumerate() {
            let fr = people::frame(look, d, f);
            img.blit(&fr, ((i % 2) * 3 + k) as i32 * cw, (i / 2) as i32 * ch, false);
        }
    }
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
