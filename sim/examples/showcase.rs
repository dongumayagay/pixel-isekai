//! Character showcase: named archetypes, each in several poses.
//! cargo run --release --example showcase -- out.png

use sim::art::canvas::Img;
use sim::art::people::{self, held, outfit, Look};
use sim::palette::{hex, mix};

fn main() {
    let out = std::env::args().nth(1).unwrap_or("showcase.png".into());
    let base = Look { skin: 0, hair: 2, hair_style: 5, top: 0, bottom: 0, outfit: 0, hat: 0, hat_color: 1, female: true, age: 1, beard: false, eyes: 0, mature: true, race: 0, beast: 0, held: 0, glasses: false };
    let cast = [
        // Guild receptionist
        Look { hair: 8, hair_style: 6, outfit: outfit::GUILD, hat: 9, hat_color: 1, eyes: 0, held: held::BOOK, ..base },
        // Barmaid
        Look { hair: 3, hair_style: 2, outfit: outfit::DIRNDL, top: 1, bottom: 2, hat: 10, eyes: 1, skin: 4, held: held::MUG, ..base },
        // Shopkeeper
        Look { female: false, mature: true, hair: 1, hair_style: 0, outfit: outfit::APRON, top: 9, bottom: 3, hat: 5, hat_color: 6, glasses: true, eyes: 2, skin: 1, held: held::BASKET, ..base },
        // Dwarf blacksmith
        Look { female: false, race: people::RACE_DWARF, hair: 4, hair_style: 0, outfit: outfit::SMITH, bottom: 8, beard: true, hat: 2, hat_color: 1, eyes: 2, skin: 1, held: held::HAMMER, ..base },
        // Elf mage
        Look { race: people::RACE_ELF, hair: 5, hair_style: 1, outfit: outfit::ROBE, top: 5, hat: 6, hat_color: 5, eyes: 3, skin: 4, held: held::STAFF, ..base },
        // Fox beastfolk adventurer
        Look { race: people::RACE_BEAST, beast: 1, hair: 4, hair_style: 7, outfit: outfit::CLOAK, top: 3, bottom: 8, hat_color: 2, eyes: 5, held: held::SWORD, ..base },
        // Demon
        Look { race: people::RACE_DEMON, skin: 0, hair: 0, hair_style: 1, outfit: outfit::DRESS, top: 8, eyes: 4, ..base },
    ];
    let frames: [(u8, u16); 5] = [(0, 0), (0, 2), (2, 0), (2, 4), (1, 3)];
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
    let s = 3usize;
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
