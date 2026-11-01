//! Every outfit in its default colours, women and men, front/side/back.
//! cargo run --release --example outfits -- out.png [first] [count]
use sim::art::canvas::Img;
use sim::art::people::{self, outfit, Look};
use sim::palette::hex;

fn main() {
    let mut args = std::env::args().skip(1);
    let out = args.next().unwrap_or("outfits.png".into());
    let first: u8 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let count: u8 = args.next().and_then(|s| s.parse().ok()).unwrap_or(outfit::COUNT);
    let base = people::cast()[1].1;
    let views = [(0u8, 0u16), (2, 0), (1, 0)];
    let (cw, ch) = (people::FW, people::FH);
    let n = count.min(outfit::COUNT - first) as i32;
    let mut img = Img::new(cw * 6, ch * n);
    for y in 0..img.h {
        for x in 0..img.w {
            img.set(x, y, if (x / (cw * 3) + y / ch) % 2 == 0 { hex(0x6a9a5a) } else { hex(0x5a8a4e) });
        }
    }
    for row in 0..n {
        let o = first + row as u8;
        for (si, female) in [true, false].into_iter().enumerate() {
            let (p, s, a) = outfit::defaults(o, female);
            let look = Look { outfit: o, female, primary: p, secondary: s, accent: a, hat: 0, held: 0, hair_style: if female { 2 } else { 13 }, ..base };
            for (k, &(d, f)) in views.iter().enumerate() {
                let fr = people::frame(&look, d, f);
                img.blit(&fr, (si as i32 * 3 + k as i32) * cw, row * ch, false);
            }
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
