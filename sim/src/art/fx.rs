//! Emote bubbles and small effect sprites.

use super::canvas::Img;
use crate::palette::{self as pal, hex};

const ICONS: [&[&str]; 11] = [
    // 1 heart
    &[".rr.rr.", "rRRrRRr", "rRRRRRr", ".rRRRr.", "..rRr..", "...r..."],
    // 2 chat
    &["......", "......", "k.k.k.", "......"],
    // 3 music
    &["...kkk", "...k.k", "...k.k", ".kkk.k", "kkkkkk", "kk.kk."],
    // 4 zzz
    &["kkkk...", "..k....", ".k.kkk.", "kkkk.k.", "...k...", "...kkk."],
    // 5 anger
    &["r.r.r.", ".rrr..", "rr.rr.", ".rrr..", "r.r.r."],
    // 6 sweat
    &["..b..", ".bb..", "bBbb.", "bBBb.", ".bb.."],
    // 7 exclaim
    &["rr", "rr", "rr", "rr", "..", "rr"],
    // 8 question
    &[".kkk.", "k...k", "...k.", "..k..", ".....", "..k.."],
    // 9 sparkle
    &["...y...", "...y...", ".yyYyy.", "yyYYYyy", ".yyYyy.", "...y...", "...y..."],
    // 10 food
    &["..ooo", ".oOOo", ".oOOo", "w.oo.", ".w..."],
    // 11 sad
    &["k...k", ".....", ".kkk.", "k...k", "...b.", "...b."],
];

pub fn emote(n: usize) -> (Img, (i32, i32)) {
    let mut img = Img::new(20, 20);
    let paper = hex(0xfdf8ec);
    let rim = hex(0xd8cfc0);
    // Bubble with a tail.
    img.ellipse_with(10.0, 8.5, 8.5, 7.5, |im, x, y, nx, ny| {
        let c = if nx + ny > 0.9 { rim } else { paper };
        im.set(x, y, c);
    });
    img.set(9, 16, paper);
    img.set(10, 16, paper);
    img.set(10, 17, paper);
    let rows = ICONS[(n - 1).min(ICONS.len() - 1)];
    let h = rows.len() as i32;
    let w = rows[0].len() as i32;
    let ox = 10 - w / 2;
    let oy = 9 - h / 2 - 1;
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            let c = match ch {
                'r' => Some(hex(0xc8303a)),
                'R' => Some(hex(0xf05a5a)),
                'k' => Some(hex(0x3a3048)),
                'b' => Some(hex(0x4a8ad8)),
                'B' => Some(hex(0xa8d8ff)),
                'y' => Some(hex(pal::GOLD[2])),
                'Y' => Some(hex(pal::GOLD[4])),
                'o' => Some(hex(0xb86a34)),
                'O' => Some(hex(0xe09a54)),
                'w' => Some(hex(0xf8f0e0)),
                _ => None,
            };
            if let Some(c) = c {
                img.set(ox + x as i32, oy + y as i32, c);
            }
        }
    }
    img.outline_solid(pal::INK);
    (img, (10, 19))
}
