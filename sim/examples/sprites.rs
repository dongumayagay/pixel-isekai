//! Contact sheet of generated sprites for art review.
//! cargo run --release --example sprites -- out.png [scale]

use sim::art::canvas::Img;
use sim::art::{flora, people};
use sim::palette::hex;
use sim::rng::Rng;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let out = a.get(1).cloned().unwrap_or("sprites.png".into());
    let scale: usize = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(4);
    let mut sheet = Img::new(480, 300);
    // Grass-coloured backdrop so outlines read as they will in game.
    sheet.rect(0, 0, 480, 300, hex(0x427f44));
    let mut x = 4;
    let mut y = 4;
    let mut row_h = 0;
    let mut put = |sheet: &mut Img, img: &Img| {
        if x + img.w > 476 {
            x = 4;
            y += row_h + 4;
            row_h = 0;
        }
        sheet.blit(img, x, y, false);
        x += img.w + 3;
        row_h = row_h.max(img.h);
    };
    for season in 0..4u8 {
        for v in 0..2u8 {
            put(&mut sheet, &flora::deciduous(0, v, season).0);
        }
        put(&mut sheet, &flora::deciduous(1, 0, season).0);
        put(&mut sheet, &flora::deciduous(2, 0, season).0);
        put(&mut sheet, &flora::pine(0, season).0);
    }
    for v in 0..2 {
        put(&mut sheet, &flora::bush(v, 1, v == 1).0);
    }
    put(&mut sheet, &flora::rock(0, 0, None, 1).0);
    put(&mut sheet, &flora::rock(1, 1, None, 1).0);
    put(&mut sheet, &flora::rock(1, 2, Some(sim::palette::GOLD), 1).0);
    put(&mut sheet, &flora::stump(1).0);
    put(&mut sheet, &flora::tall_grass(0, 1).0);
    put(&mut sheet, &flora::reeds(0).0);
    put(&mut sheet, &flora::mushroom(0).0);
    put(&mut sheet, &flora::lily(0).0);
    put(&mut sheet, &flora::flower_patch(2).0);
    for k in 0..6 {
        put(&mut sheet, &flora::crop(k, 3).0);
    }
    let mut rng = Rng::new(7);
    for i in 0..10 {
        let mut look = people::Look::random(&mut rng, i % 2 == 0, if i == 9 { 0 } else if i == 8 { 2 } else { 1 });
        look.hat = [0, 1, 2, 3, 4, 5, 6, 7, 0, 0][i];
        look.outfit = [0, 1, 3, 4, 5, 2, 2, 1, 0, 0][i];
        let frames = people::sheet(&look);
        for f in [0usize, 2, 4, 8, 16, 18, 6, 24] {
            put(&mut sheet, &frames[f]);
        }
    }
    let s = scale;
    let (w, h) = (sheet.w as usize * s, sheet.h as usize * s);
    let mut o = vec![0u8; w * h * 4];
    for yy in 0..h {
        for xx in 0..w {
            let si = ((yy / s) * sheet.w as usize + xx / s) * 4;
            o[(yy * w + xx) * 4..(yy * w + xx) * 4 + 4].copy_from_slice(&sheet.px[si..si + 4]);
        }
    }
    let f = std::fs::File::create(&out).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), w as u32, h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.write_header().unwrap().write_image_data(&o).unwrap();
}
