//! Review sheets: every cast member, every frame of one direction.
//! cargo run --release --example review -- outdir
use sim::art::canvas::Img;
use sim::art::people;
use sim::palette::hex;

fn main() {
    let dir = std::env::args().nth(1).unwrap_or(".".into());
    let cast = people::cast();
    let n = people::FRAMES_PER_DIR as i32;
    let (cw, ch) = (people::FW, people::FH);
    for (d, name) in [(0u8, "down"), (2, "side"), (1, "up")] {
        let mut img = Img::new(cw * n, ch * cast.len() as i32);
        for y in 0..img.h {
            for x in 0..img.w {
                img.set(x, y, if (x / cw + y / ch) % 2 == 0 { hex(0x6a9a5a) } else { hex(0x5a8a4e) });
            }
        }
        for (r, (_, look)) in cast.iter().enumerate() {
            for f in 0..n {
                img.blit(&people::frame(look, d, f as u16), f * cw, r as i32 * ch, false);
            }
        }
        let s = 3usize;
        let (ow, oh) = (img.w as usize * s, img.h as usize * s);
        let mut o = vec![0u8; ow * oh * 4];
        for yy in 0..oh {
            for xx in 0..ow {
                let si = ((yy / s) * img.w as usize + xx / s) * 4;
                o[(yy * ow + xx) * 4..(yy * ow + xx) * 4 + 4].copy_from_slice(&img.px[si..si + 4]);
            }
        }
        let f = std::fs::File::create(format!("{dir}/cast_{name}.png")).unwrap();
        let mut enc = png::Encoder::new(std::io::BufWriter::new(f), ow as u32, oh as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.write_header().unwrap().write_image_data(&o).unwrap();
    }
}
