//! Every facial hair style, front and side, for review.
use sim::art::canvas::Img;
use sim::art::people::{self, Look, FACIAL};
use sim::palette::hex;
fn main() {
    let out = std::env::args().nth(1).unwrap_or("facial.png".into());
    let base = Look { hair: 1, hair_style: 0, beard: false, ..people::cast()[0].1 };
    let n = FACIAL.len() as i32;
    let (cw, ch, y0) = (people::FW, 60, 14);
    let mut img = Img::new(cw * n, ch * 2);
    for y in 0..img.h { for x in 0..img.w { img.set(x, y, if (x / cw) % 2 == 0 { hex(0x6a9a5a) } else { hex(0x5f8f50) }); } }
    for k in 0..n {
        let l = Look { facial: k as u8, ..base };
        for (r, d) in [0u8, 2].into_iter().enumerate() {
            let mut part = Img::new(cw, ch);
            part.blit(&people::frame(&l, d, 0), 0, -y0, false);
            img.blit(&part, k * cw, r as i32 * ch, false);
        }
    }
    let s = 4usize;
    let (ow, oh) = (img.w as usize * s, img.h as usize * s);
    let mut o = vec![0u8; ow * oh * 4];
    for yy in 0..oh { for xx in 0..ow { let si = ((yy / s) * img.w as usize + xx / s) * 4; o[(yy * ow + xx) * 4..(yy * ow + xx) * 4 + 4].copy_from_slice(&img.px[si..si + 4]); } }
    let f = std::fs::File::create(&out).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), ow as u32, oh as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.write_header().unwrap().write_image_data(&o).unwrap();
}
