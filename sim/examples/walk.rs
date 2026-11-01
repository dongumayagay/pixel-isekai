//! Walk cycle strip (front and side) for bust/secondary-motion review.
//! cargo run --release --example walk -- out.png [cast index]
use sim::art::canvas::Img;
use sim::art::people::{self, WALK_FIRST, WALK_FRAMES};
use sim::palette::hex;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let out = a.get(1).cloned().unwrap_or("walk.png".into());
    let idx: usize = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(7);
    let look = people::cast()[idx].1;
    let (y0, h) = (40, 50);
    let n = WALK_FRAMES as i32;
    let mut img = Img::new(people::FW * n, h * 2);
    for y in 0..img.h {
        for x in 0..img.w {
            img.set(x, y, if (x / people::FW) % 2 == 0 { hex(0x6a9a5a) } else { hex(0x5f8f50) });
        }
    }
    for (row, dir) in [0u8, 2].into_iter().enumerate() {
        for k in 0..n {
            let fr = people::frame(&look, dir, WALK_FIRST + k as u16);
            let mut part = Img::new(people::FW, h);
            part.blit(&fr, 0, -y0, false);
            img.blit(&part, k * people::FW, row as i32 * h, false);
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
