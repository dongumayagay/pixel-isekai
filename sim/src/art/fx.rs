//! Emote bubbles and small effect sprites.

use super::canvas::Img;
use crate::palette::{self as pal, hex, Rgb};

/// Icon bitmaps indexed by emote id - 1. Each letter is a colour from
/// `icon_colour`; anything else is transparent.
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

/// Lowercase is the base tone, uppercase its highlight.
fn icon_colour(ch: char) -> Option<Rgb> {
    let v = match ch {
        'r' => 0xc8303a,
        'R' => 0xf05a5a,
        'k' => 0x3a3048,
        'b' => 0x4a8ad8,
        'B' => 0xa8d8ff,
        'y' => pal::GOLD[2],
        'Y' => pal::GOLD[4],
        'o' => 0xb86a34,
        'O' => 0xe09a54,
        'w' => 0xf8f0e0,
        _ => return None,
    };
    Some(hex(v))
}

/// Speech bubble holding emote `n` (1-based; out-of-range ids clamp to the
/// last icon). The pivot is the tip of the bubble's tail.
pub fn emote(n: usize) -> (Img, (i32, i32)) {
    let mut img = Img::new(20, 20);
    let paper = hex(0xfdf8ec);
    let rim = hex(0xd8cfc0);
    img.ellipse_with(10.0, 8.5, 8.5, 7.5, |im, x, y, nx, ny| {
        im.set(x, y, if nx + ny > 0.9 { rim } else { paper });
    });
    for (x, y) in [(9, 16), (10, 16), (10, 17)] {
        img.set(x, y, paper);
    }
    let rows = ICONS[(n - 1).min(ICONS.len() - 1)];
    let ox = 10 - rows[0].len() as i32 / 2;
    let oy = 9 - rows.len() as i32 / 2 - 1;
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            if let Some(c) = icon_colour(ch) {
                img.set(ox + x as i32, oy + y as i32, c);
            }
        }
    }
    img.outline_solid(pal::INK);
    (img, (10, 19))
}
