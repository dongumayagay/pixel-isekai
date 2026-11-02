//! Render the n-th random look (seeded like calib) in front/side/back.
use sim::art::canvas::Img;
use sim::art::people::{self, Look};
use sim::rng::Rng;
fn main() {
    let mut a = std::env::args().skip(1);
    let out = a.next().unwrap();
    let n: usize = a.next().unwrap().parse().unwrap();
    let seed: u64 = a.next().and_then(|v| v.parse().ok()).unwrap_or(3);
    let mut rng = Rng::new(seed);
    let mut look = Look::random_any(&mut rng);
    for _ in 0..n { look = Look::random_any(&mut rng); }
    println!("{look:?}");
    let mut img = Img::new(people::FW * 3, people::FH);
    for (k, d) in [0u8, 2, 1].into_iter().enumerate() {
        img.blit(&people::frame(&look, d, 0), k as i32 * people::FW, 0, false);
    }
    let (w, h, s) = (img.w as usize, img.h as usize, 3usize);
    let mut o = vec![0u8; w * h * s * s * 4];
    for y in 0..h * s { for x in 0..w * s { let si = ((y / s) * w + x / s) * 4; let di = (y * w * s + x) * 4; o[di..di + 4].copy_from_slice(&img.px[si..si + 4]); if o[di+3]==0 { o[di..di+4].copy_from_slice(&[106,154,90,255]); } } }
    let f = std::fs::File::create(&out).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), (w * s) as u32, (h * s) as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.write_header().unwrap().write_image_data(&o).unwrap();
}
