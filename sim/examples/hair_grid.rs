//! Every hairstyle x (down, side, up) x (female, male) at 4x, heads only,
//! for checking that no cut goes bald from the side or behind.
//! cargo run --release --example hair_grid -- out.png
use sim::art::canvas::Img;
use sim::art::people::{self, Look};
use sim::palette::hex;

fn main() {
    let out = std::env::args().nth(1).unwrap_or("hair_grid.png".into());
    let base = people::cast()[1].1;
    let f: u16 = std::env::var("FRAME").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let views = [0u8, 2, 1];
    let (cw, ch) = (people::FW, 72);
    // STYLES=13,1,8 renders only those styles (in that order).
    let styles: Vec<i32> = match std::env::var("STYLES") {
        Ok(v) => v.split(',').filter_map(|t| t.trim().parse().ok()).collect(),
        Err(_) => (0..people::HAIR_STYLES as i32).collect(),
    };
    let n = styles.len() as i32;
    let mut img = Img::new(cw * 6, ch * n);
    for y in 0..img.h {
        for x in 0..img.w {
            img.set(x, y, if (x / (cw * 3) + y / ch) % 2 == 0 { hex(0x6a9a5a) } else { hex(0x5a8a4e) });
        }
    }
    for (row, &style) in styles.iter().enumerate() {
        let row = row as i32;
        for (si, female) in [true, false].into_iter().enumerate() {
            let look = Look { hair_style: style as u8, female, hat: 0, held: 0, outfit: 0, hair: [1, 3, 0, 4, 2][style as usize % 5], ..base };
            for (k, &d) in views.iter().enumerate() {
                let fr = people::frame(&look, d, f);
                let mut head = Img::new(cw, ch);
                head.blit(&fr, 0, -6, false);
                img.blit(&head, (si as i32 * 3 + k as i32) * cw, row * ch, false);
            }
        }
    }
    let s = 4usize;
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
