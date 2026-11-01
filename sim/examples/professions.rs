//! Every profession's signature look (woman and man), idle front + side.
use sim::art::canvas::Img;
use sim::art::people::{self, Look};
use sim::game::villager::Job;
use sim::palette::hex;
use sim::rng::Rng;
fn main() {
    let out = std::env::args().nth(1).unwrap_or("professions.png".into());
    let jobs = Job::WORKING;
    let cols = 7;
    let (cw, ch) = (people::FW * 2, people::FH);
    let rows = (jobs.len() * 2).div_ceil(cols) as i32;
    let mut img = Img::new(cw * cols as i32, ch * rows);
    for y in 0..img.h { for x in 0..img.w { img.set(x, y, if ((x / cw) + (y / ch)) % 2 == 0 { hex(0x6a9a5a) } else { hex(0x5f8f50) }); } }
    let mut rng = Rng::new(3);
    let mut i = 0;
    for job in jobs {
        for female in [true, false] {
            let mut l = Look::random(&mut rng, female, 1);
            l.hair_style = if female { 1 + (i % 7) as u8 } else { [0, 4, 7][i % 3] };
            job.dress(&mut l, &mut rng);
            let (x, y) = ((i % cols) as i32 * cw, (i / cols) as i32 * ch);
            img.blit(&people::frame(&l, 0, 0), x, y, false);
            img.blit(&people::frame(&l, 2, 0), x + people::FW, y, false);
            i += 1;
        }
    }
    let s = 2usize;
    let (ow, oh) = (img.w as usize * s, img.h as usize * s);
    let mut o = vec![0u8; ow * oh * 4];
    for yy in 0..oh { for xx in 0..ow { let si = ((yy / s) * img.w as usize + xx / s) * 4; o[(yy * ow + xx) * 4..(yy * ow + xx) * 4 + 4].copy_from_slice(&img.px[si..si + 4]); } }
    let f = std::fs::File::create(&out).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), ow as u32, oh as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.write_header().unwrap().write_image_data(&o).unwrap();
}
