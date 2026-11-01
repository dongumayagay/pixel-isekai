//! Back views (idle + walk) of the whole core cast, for review.
use sim::art::canvas::Img;
use sim::art::people;
use sim::palette::hex;
fn main() {
    let out = std::env::args().nth(1).unwrap_or("backs.png".into());
    let cast = people::cast();
    let cols = 6;
    let (cw, ch) = (people::FW * 2, people::FH);
    let rows = cast.len().div_ceil(cols) as i32;
    let mut img = Img::new(cw * cols as i32, ch * rows);
    for y in 0..img.h { for x in 0..img.w { img.set(x, y, if ((x / cw) + (y / ch)) % 2 == 0 { hex(0x6a9a5a) } else { hex(0x5f8f50) }); } }
    for (i, (_, l)) in cast.iter().enumerate() {
        let (cx, cy) = ((i % cols) as i32 * cw, (i / cols) as i32 * ch);
        img.blit(&people::frame(l, 1, 0), cx, cy, false);
        img.blit(&people::frame(l, 1, 6), cx + people::FW, cy, false);
    }
    let s = 2usize;
    let (ow, oh) = (img.w as usize * s, img.h as usize * s);
    let mut o = vec![0u8; ow * oh * 4];
    for yy in 0..oh { for xx in 0..ow { let si = ((yy / s) * img.w as usize + xx / s) * 4; o[(yy * ow + xx) * 4..(yy * ow + xx) * 4 + 4].copy_from_slice(&img.px[si..si + 4]); } }
    let f = std::fs::File::create(&out).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), ow as u32, oh as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.write_header().unwrap().write_image_data(&o).unwrap();
}
