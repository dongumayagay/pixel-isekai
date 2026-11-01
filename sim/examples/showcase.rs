//! Character showcase: named archetypes, each in several poses.
//! cargo run --release --example showcase -- out.png

use sim::art::canvas::Img;
use sim::art::people::{self, Look};
use sim::palette::{hex, mix};

fn main() {
    let out = std::env::args().nth(1).unwrap_or("showcase.png".into());
    let cast: Vec<Look> = people::cast().into_iter().map(|(_, l)| l).collect();
    let only: Option<usize> = std::env::args().nth(2).and_then(|s| s.parse().ok());
    let cast: Vec<Look> = match only { Some(i) => vec![cast[i]], None => cast };
    let frames: [(u8, u16); 6] = [(0, 0), (0, 5), (2, 0), (2, 6), (1, 0), (1, 7)];
    let (cw, ch) = (people::FW + 6, people::FH + 6);
    let w = cw * frames.len() as i32 + 8;
    let h = ch * cast.len() as i32 + 8;
    let mut sheet = Img::new(w, h);
    for y in 0..h {
        for x in 0..w {
            // Soft grass-to-sky backdrop so outlines read like in game.
            let t = y as f32 / h as f32;
            sheet.set(x, y, mix(hex(0x6a9a5a), hex(0x3f6a48), t));
        }
    }
    for (r, look) in cast.iter().enumerate() {
        for (c, &(dir, f)) in frames.iter().enumerate() {
            let img = people::frame(look, dir, f);
            sheet.blit(&img, 4 + c as i32 * cw + 3, 4 + r as i32 * ch + 3, false);
        }
    }
    let s = 2usize;
    let (ow, oh) = (w as usize * s, h as usize * s);
    let mut o = vec![0u8; ow * oh * 4];
    for yy in 0..oh {
        for xx in 0..ow {
            let si = ((yy / s) * w as usize + xx / s) * 4;
            o[(yy * ow + xx) * 4..(yy * ow + xx) * 4 + 4].copy_from_slice(&sheet.px[si..si + 4]);
        }
    }
    let f = std::fs::File::create(&out).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), ow as u32, oh as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.write_header().unwrap().write_image_data(&o).unwrap();
}
