//! Every hairstyle in profile, heads only, big, for review.
//! cargo run --release --example sideheads -- out.png
use sim::art::canvas::Img;
use sim::art::people::{self, Look};
use sim::palette::hex;

fn main() {
    let out = std::env::args().nth(1).unwrap_or("sideheads.png".into());
    let base = people::cast()[1].1;
    let (cw, ch) = (40, 44);
    let n = people::HAIR_STYLES as i32;
    let mut img = Img::new(cw * n, ch * 2);
    for y in 0..img.h {
        for x in 0..img.w {
            img.set(x, y, if (x / cw + y / ch) % 2 == 0 { hex(0x6a9a5a) } else { hex(0x5a8a4e) });
        }
    }
    for style in 0..n {
        for (row, female) in [true, false].into_iter().enumerate() {
            let look = Look { hair_style: style as u8, female, hat: 0, held: 0, outfit: 0, hair: [1, 3, 0, 4][style as usize % 4], ..base };
            let fr = people::frame(&look, 2, 0);
            let mut head = Img::new(cw, ch);
            head.blit(&fr, -12, -14, false);
            img.blit(&head, style * cw, row as i32 * ch, false);
        }
    }
    let s = 5usize;
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
