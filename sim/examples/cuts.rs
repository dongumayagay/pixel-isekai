//! Outfits at each cut (modest, bold, daring), front views plus the daring
//! profile, for review. cargo run --release --example cuts -- out.png
use sim::art::canvas::Img;
use sim::art::people::{self, outfit, Look};
use sim::palette::hex;

fn main() {
    let out = std::env::args().nth(1).unwrap_or("cuts.png".into());
    let base = Look { figure: 3, hair_style: 2, hat: 0, held: 0, ..people::cast()[1].1 };
    let outfits = [outfit::PEASANT, outfit::INNKEEPER, outfit::NOBLE, outfit::MAGE, outfit::GUILD, outfit::ADVENTURER, outfit::DEMON, outfit::MAID, outfit::HERO, outfit::PRIEST];
    let cols = [(0u8, 0u8), (1, 0), (2, 0), (2, 2)];
    let (cw, ch) = (people::FW, 84);
    let mut img = Img::new(cw * cols.len() as i32 * 2, ch * (outfits.len() as i32 + 1) / 2);
    for y in 0..img.h {
        for x in 0..img.w {
            img.set(x, y, if (x / cw + y / ch) % 2 == 0 { hex(0x6a9a5a) } else { hex(0x5a8a4e) });
        }
    }
    for (i, &o) in outfits.iter().enumerate() {
        let (p, s, a) = outfit::defaults(o, true);
        for (k, &(cut, d)) in cols.iter().enumerate() {
            let look = Look { outfit: o, primary: p, secondary: s, accent: a, cut, ..base };
            let mut part = Img::new(cw, ch);
            part.blit(&people::frame(&look, d, 0), 0, -26, false);
            img.blit(&part, ((i % 2) * cols.len() + k) as i32 * cw, (i / 2) as i32 * ch, false);
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
    let f = std::fs::File::create(&out).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), ow as u32, oh as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.write_header().unwrap().write_image_data(&o).unwrap();
}
