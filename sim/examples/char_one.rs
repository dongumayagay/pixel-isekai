//! Render one character frame set to a raw RGBA dump for comparison tooling.
//! cargo run --release --example char_one -- out.png
use sim::art::people::{self, held, outfit, Look};
fn main() {
    let out = std::env::args().nth(1).unwrap_or("one.png".into());
    let look = Look { skin: 4, hair: 7, hair_style: 1, top: 1, bottom: 0, outfit: outfit::TUNIC, hat: 0, hat_color: 1, female: true, age: 1, beard: false, eyes: 0, mature: true, race: 0, beast: 0, held: held::NONE, glasses: false, figure: 3, build: 2, height: 1 };
    let frames: Vec<_> = [(0u8, 0u16), (0, 2), (2, 0), (2, 2), (1, 0)].iter().map(|&(d, f)| people::frame(&look, d, f)).collect();
    let (w, h) = (people::FW * frames.len() as i32, people::FH);
    let mut o = vec![0u8; (w * h * 4) as usize];
    for (i, f) in frames.iter().enumerate() {
        for y in 0..h {
            for x in 0..people::FW {
                let s = ((y * f.w + x) * 4) as usize;
                let d = ((y * w + i as i32 * people::FW + x) * 4) as usize;
                o[d..d + 4].copy_from_slice(&f.px[s..s + 4]);
            }
        }
    }
    let file = std::fs::File::create(&out).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w as u32, h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.write_header().unwrap().write_image_data(&o).unwrap();
}
