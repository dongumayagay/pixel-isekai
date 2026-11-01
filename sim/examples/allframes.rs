//! Every frame of one cast member in every direction, for spotting
//! animation-only bugs. cargo run --release --example allframes -- out.png [cast]
use sim::art::canvas::Img;
use sim::art::people;
use sim::palette::hex;

fn main() {
    let mut args = std::env::args().skip(1);
    let out = args.next().unwrap_or("allframes.png".into());
    let idx: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    let look = people::cast()[idx].1;
    let n = people::FRAMES_PER_DIR as i32;
    let (cw, ch) = (people::FW, people::FH);
    let mut img = Img::new(cw * n, ch * 3);
    for y in 0..img.h {
        for x in 0..img.w {
            img.set(x, y, if (x / cw + y / ch) % 2 == 0 { hex(0x6a9a5a) } else { hex(0x5a8a4e) });
        }
    }
    for d in 0..3u8 {
        for f in 0..n {
            let fr = people::frame(&look, d, f as u16);
            img.blit(&fr, f * cw, d as i32 * ch, false);
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
    let f = std::fs::File::create(&out).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), ow as u32, oh as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.write_header().unwrap().write_image_data(&o).unwrap();
}
