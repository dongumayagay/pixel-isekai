//! Parametric anime-style characters (28×48 frames). Each person is
//! assembled from body, outfit, hair and hat layers, shaded with a top-left
//! light, then given a selective outline. Big expressive eyes with
//! highlights and an "angel ring" hair shine give the anime look.
//!
//! Sheet layout: 3 directions (down, up, side-facing-right) × 8 poses
//! (idle, breathe, walk ×4, work ×2), then one blink frame.

use serde::{Deserialize, Serialize};

use super::canvas::Img;
use crate::palette::{self as pal, hex, Rgb};
use crate::rng::Rng;

pub const FW: i32 = 28;
pub const FH: i32 = 48;
pub const FRAMES_PER_DIR: u16 = 8;
pub const SHEET_FRAMES: u16 = FRAMES_PER_DIR * 3 + 1;
pub const PIVOT: (i32, i32) = (14, 46);

pub const DIR_DOWN: u8 = 0;
pub const DIR_UP: u8 = 1;
pub const DIR_SIDE: u8 = 2;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Look {
    pub skin: u8,
    pub hair: u8,
    pub hair_style: u8,
    pub top: u8,
    pub bottom: u8,
    /// 0 tunic, 1 dress, 2 robe, 3 apron, 4 armor, 5 cloak, 6 vest
    pub outfit: u8,
    /// 0 none, 1 straw hat, 2 bandana, 3 helmet, 4 hood, 5 cap, 6 witch hat, 7 flower
    pub hat: u8,
    pub hat_color: u8,
    pub female: bool,
    /// 0 child, 1 adult, 2 elder
    pub age: u8,
    pub beard: bool,
    pub eyes: u8,
    /// Adult body (18+). Gates figure shaping and its secondary motion, so
    /// child and teen sprites never get them.
    #[serde(default)]
    pub mature: bool,
}

pub const HAIR_STYLES: u8 = 8;

impl Look {
    pub fn random(rng: &mut Rng, female: bool, age: u8) -> Look {
        let hair = if rng.chance(0.12) { 6 + rng.below(3) as u8 } else { rng.below(5) as u8 };
        let styles_f: &[u8] = &[1, 2, 3, 5, 6, 7, 1, 5];
        let styles_m: &[u8] = &[0, 4, 7, 0, 5];
        Look {
            skin: rng.below(pal::SKIN.len()) as u8,
            hair: if age == 2 { 5 } else { hair },
            hair_style: if female { *rng.pick(styles_f) } else { *rng.pick(styles_m) },
            top: rng.below(pal::CLOTH.len()) as u8,
            bottom: rng.below(pal::CLOTH.len()) as u8,
            outfit: if female && rng.chance(0.45) { 1 } else if rng.chance(0.2) { 6 } else { 0 },
            hat: 0,
            hat_color: rng.below(pal::CLOTH.len()) as u8,
            female,
            age,
            beard: !female && age >= 1 && rng.chance(if age == 2 { 0.6 } else { 0.2 }),
            eyes: rng.below(EYE_COLORS.len()) as u8,
            mature: age >= 1,
        }
    }
}

fn ramp4(r: &[u32; 4]) -> [Rgb; 4] {
    [hex(r[0]), hex(r[1]), hex(r[2]), hex(r[3])]
}

/// Iris ramps: dark, mid, light.
const EYE_COLORS: [[u32; 3]; 6] = [
    [0x1e2a5a, 0x3a5aaa, 0x7aa0e0], // blue
    [0x1a3a2a, 0x2e7a4a, 0x6ac08a], // green
    [0x3a2414, 0x6a4424, 0xa87a44], // brown
    [0x3a1a4a, 0x7a3aa0, 0xb07ad8], // violet
    [0x4a1a1a, 0xa0302a, 0xe07050], // red
    [0x3a3010, 0x9a7a1a, 0xe0c050], // amber
];

struct P<'a> {
    img: &'a mut Img,
}

impl P<'_> {
    #[inline]
    fn set(&mut self, x: i32, y: i32, c: Rgb) {
        self.img.set(x, y, c);
    }
    /// Rect shaded with a top-left light.
    fn part(&mut self, x: i32, y: i32, w: i32, h: i32, r: &[Rgb; 4]) {
        for yy in y..y + h {
            for xx in x..x + w {
                let c = if xx == x + w - 1 || (yy == y + h - 1 && h > 3) {
                    r[1]
                } else if xx == x && w > 2 {
                    r[3]
                } else {
                    r[2]
                };
                self.set(xx, yy, c);
            }
        }
    }
    fn fill(&mut self, x: i32, y: i32, w: i32, h: i32, c: Rgb) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, c);
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Pose {
    bob: i32,
    /// Secondary motion that lags the body: hair, skirts, figure.
    lag: i32,
    leg_l: i32,
    leg_r: i32,
    stride: i32,
    arm: i32,
    work: u8,
    blink: bool,
}

fn pose_for(frame: u16) -> Pose {
    let base = Pose { bob: 0, lag: 0, leg_l: 0, stride: 0, leg_r: 0, arm: 0, work: 0, blink: false };
    match frame {
        0 => base,
        1 => Pose { bob: 1, ..base },
        2 => Pose { leg_l: 2, stride: 3, arm: 2, lag: 1, ..base },
        3 => Pose { bob: -1, stride: 0, lag: 0, ..base },
        4 => Pose { leg_r: 2, stride: -3, arm: -2, lag: 1, ..base },
        5 => Pose { bob: -1, lag: -1, ..base },
        6 => Pose { work: 1, lag: -1, ..base },
        7 => Pose { work: 2, bob: 1, lag: 1, ..base },
        _ => Pose { blink: true, ..base },
    }
}

struct Body {
    skin: [Rgb; 4],
    hair: [Rgb; 4],
    top: [Rgb; 4],
    bottom: [Rgb; 4],
    hatc: [Rgb; 4],
    metal: [Rgb; 4],
    shoe: [Rgb; 4],
}

pub fn frame(look: &Look, dir: u8, frame_idx: u16) -> Img {
    let mut img = Img::new(FW, FH);
    let pose = pose_for(frame_idx);
    let b = Body {
        skin: ramp4(&pal::SKIN[look.skin as usize % pal::SKIN.len()]),
        hair: ramp4(&pal::HAIR[look.hair as usize % pal::HAIR.len()]),
        top: ramp4(&pal::CLOTH[look.top as usize % pal::CLOTH.len()]),
        bottom: ramp4(&pal::CLOTH[look.bottom as usize % pal::CLOTH.len()]),
        hatc: ramp4(&pal::CLOTH[look.hat_color as usize % pal::CLOTH.len()]),
        metal: ramp4(&[pal::METAL[1], pal::METAL[2], pal::METAL[3], pal::METAL[4]]),
        shoe: [hex(0x1e1620), hex(0x3a2828), hex(0x5a3e34), hex(0x7a5644)],
    };
    let mut p = P { img: &mut img };

    let child = look.age == 0;
    let (torso_h, leg_h) = if child { (8, 6) } else if look.age == 2 { (11, 9) } else { (11, 10) };
    let feet_y = 45;
    let leg_y = feet_y - leg_h;
    let lift = if pose.bob > 0 { 1 } else { 0 };
    let torso_y = leg_y - torso_h + pose.bob.min(0) + lift;
    let head_y = torso_y - 18;
    let side = dir == DIR_SIDE;

    // ---- Back layers ----
    back_hair(&mut p, look, dir, head_y, torso_y, &b.hair, pose.lag);
    if look.outfit == 5 && dir != DIR_UP {
        let (x, w) = if side { (7, 8) } else { (7, 14) };
        p.part(x, torso_y + 1, w, torso_h + leg_h - 2, &b.hatc);
    }

    // ---- Legs & shoes ----
    let pants = if look.outfit == 4 { b.metal } else { b.bottom };
    if side {
        let front = 13 + pose.stride;
        let back = 13 - pose.stride;
        let dark = [pants[0], pants[0], pants[1], pants[1]];
        p.part(back, leg_y, 4, leg_h, &dark);
        p.fill(back - 1, feet_y, 6, 2, b.shoe[1]);
        p.part(front, leg_y, 4, leg_h, &pants);
        p.fill(front - 1, feet_y, 6, 1, b.shoe[2]);
        p.fill(front - 1, feet_y + 1, 6, 1, b.shoe[0]);
        p.set(front + 4, feet_y, b.shoe[3]);
    } else {
        for (lx, up, light) in [(10, pose.leg_l, true), (14, pose.leg_r, false)] {
            let h = leg_h - up;
            for yy in 0..h {
                for xx in 0..4 {
                    let c = if xx == 0 && light { pants[3] } else if xx == 3 || (!light && xx == 0) { pants[1] } else { pants[2] };
                    p.set(lx + xx, leg_y + yy, c);
                }
            }
            let sx = if light { lx - 1 } else { lx };
            p.fill(sx, feet_y - up, 5, 1, b.shoe[2]);
            p.fill(sx, feet_y + 1 - up, 5, 1, b.shoe[1]);
            p.set(if light { sx + 1 } else { sx + 1 }, feet_y - up, b.shoe[3]);
        }
    }

    // ---- Torso ----
    let (tx, tw) = if side { (10, 8) } else { (9, 10) };
    let shirt = if look.outfit == 4 { b.metal } else { b.top };
    for yy in 0..torso_h {
        let inset = if yy == 0 { 1 } else { 0 };
        for xx in inset..tw - inset {
            let c = if xx == tw - 1 - inset || xx == tw - 2 - inset && yy > 2 {
                shirt[1]
            } else if xx == inset || xx == inset + 1 && yy < 3 {
                shirt[3]
            } else {
                shirt[2]
            };
            p.set(tx + xx, torso_y + yy, c);
        }
    }
    // Folds
    if !side && torso_h > 8 {
        p.set(tx + 3, torso_y + 4, shirt[1]);
        p.set(tx + 4, torso_y + 5, shirt[1]);
    }
    // Collar
    if dir == DIR_DOWN && look.outfit != 4 {
        p.set(13, torso_y, b.skin[2]);
        p.set(14, torso_y, b.skin[1]);
        p.set(13, torso_y + 1, b.skin[1]);
        p.set(12, torso_y, shirt[3]);
        p.set(15, torso_y, shirt[3]);
    }
    outfit_layer(&mut p, look, dir, tx, tw, torso_y, torso_h, leg_y, leg_h, feet_y, &b, pose.lag);
    if look.female && look.mature && look.outfit != 4 && look.age != 0 {
        figure(&mut p, look, dir, pose, tx, tw, torso_y, &b);
    }

    // ---- Arms ----
    arms(&mut p, look, dir, pose, tx, tw, torso_y, torso_h, &b);

    // ---- Neck & head ----
    p.fill(12, torso_y - 2, 4, 2, b.skin[1]);
    p.set(12, torso_y - 2, b.skin[2]);
    head(&mut p, dir, head_y, &b.skin);
    face(&mut p, look, dir, pose, head_y, &b);
    front_hair(&mut p, look, dir, head_y, &b.hair);
    hat_layer(&mut p, look, dir, head_y, &b.hatc, &b.metal);

    img.outline(pal::INK);
    img
}

const HEAD_INSET: [i32; 18] = [5, 3, 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 2, 3, 5];

fn head(p: &mut P, dir: u8, hy: i32, skin: &[Rgb; 4]) {
    let hx0 = if dir == DIR_SIDE { 7 } else { 6 };
    let hw = 16;
    for (k, &inset) in HEAD_INSET.iter().enumerate() {
        let k = k as i32;
        // Side view: jaw recedes on the back side.
        let (l, r) = if dir == DIR_SIDE { (inset.min(2), inset) } else { (inset, inset) };
        for x in hx0 + l..hx0 + hw - r {
            let c = if x >= hx0 + hw - r - 2 || k >= 15 {
                skin[1]
            } else if x <= hx0 + l + 1 && k > 2 {
                skin[3]
            } else {
                skin[2]
            };
            p.set(x, hy + k, c);
        }
    }
    if dir == DIR_SIDE {
        // Nose and ear.
        p.set(hx0 + hw, hy + 12, skin[2]);
        p.set(hx0 + hw, hy + 13, skin[1]);
        p.fill(hx0 + 5, hy + 10, 2, 3, skin[1]);
        p.set(hx0 + 5, hy + 10, skin[2]);
    }
}

fn eye(p: &mut P, x: i32, y: i32, iris: &[u32; 3], outer: i32, blink: bool) {
    let ink = pal::INK;
    if blink {
        p.fill(x, y + 2, 3, 1, ink);
        p.set(x + outer, y + 1, ink);
        return;
    }
    // Lashes
    p.fill(x, y, 3, 1, ink);
    p.set(if outer < 0 { x - 1 } else { x + 3 }, y, ink);
    p.set(if outer < 0 { x - 1 } else { x + 3 }, y - 1, ink);
    // Iris: dark at the top, lighter toward the bottom, white highlight.
    p.set(x, y + 1, hex(iris[0]));
    p.set(x + 1, y + 1, hex(0xffffff));
    p.set(x + 2, y + 1, hex(iris[0]));
    p.set(x, y + 2, hex(iris[0]));
    p.set(x + 1, y + 2, hex(iris[1]));
    p.set(x + 2, y + 2, hex(iris[1]));
    p.set(x, y + 3, hex(iris[1]));
    p.set(x + 1, y + 3, hex(iris[2]));
    p.set(x + 2, y + 3, hex(iris[2]));
    // Second, tiny highlight.
    p.set(x + 2, y + 3, hex(0xe8f0ff));
}

fn face(p: &mut P, look: &Look, dir: u8, pose: Pose, hy: i32, b: &Body) {
    let iris = &EYE_COLORS[look.eyes as usize % EYE_COLORS.len()];
    let ey = hy + 9;
    let blush = hex(0xf09a9a);
    match dir {
        DIR_DOWN => {
            eye(p, 9, ey, iris, -1, pose.blink);
            eye(p, 16, ey, iris, 1, pose.blink);
            // Brows
            let brow = b.hair[0];
            p.fill(9, ey - 2, 3, 1, brow);
            p.fill(16, ey - 2, 3, 1, brow);
            // Nose, blush, mouth
            p.set(14, ey + 4, b.skin[1]);
            p.fill(8, ey + 5, 2, 1, blush);
            p.fill(18, ey + 5, 2, 1, blush);
            p.fill(13, ey + 6, 2, 1, hex(0x9a4a4a));
            if look.beard {
                for yy in 0..4 {
                    let w = 8 - yy;
                    p.fill(14 - w / 2, ey + 5 + yy, w, 1, b.hair[1 + (yy == 0) as usize]);
                }
                p.fill(13, ey + 6, 2, 1, hex(0x6a3434));
            }
            if look.age == 2 {
                p.set(8, ey + 1, b.skin[1]);
                p.set(19, ey + 1, b.skin[1]);
            }
        }
        DIR_SIDE => {
            let ex = 18;
            if pose.blink {
                p.fill(ex, ey + 2, 3, 1, pal::INK);
            } else {
                p.fill(ex, ey, 3, 1, pal::INK);
                p.set(ex + 2, ey - 1, pal::INK);
                p.set(ex, ey + 1, hex(0xffffff));
                p.set(ex + 1, ey + 1, hex(iris[0]));
                p.set(ex, ey + 2, hex(iris[1]));
                p.set(ex + 1, ey + 2, hex(iris[0]));
                p.set(ex, ey + 3, hex(iris[2]));
                p.set(ex + 1, ey + 3, hex(iris[1]));
            }
            p.fill(ex, ey - 2, 3, 1, b.hair[0]);
            p.fill(ex - 1, ey + 5, 2, 1, blush);
            p.set(21, ey + 6, hex(0x9a4a4a));
            if look.beard {
                p.fill(17, ey + 5, 6, 3, b.hair[1]);
                p.fill(18, ey + 8, 4, 1, b.hair[1]);
            }
        }
        _ => {}
    }
}

/// Hair that sits behind the body: long hair, ponytails, twin tails.
fn back_hair(p: &mut P, look: &Look, dir: u8, hy: i32, ty: i32, hair: &[Rgb; 4], lag: i32) {
    let side = dir == DIR_SIDE;
    match look.hair_style {
        1 | 6 if dir != DIR_UP => {
            let len = if look.hair_style == 6 { 22 } else { 18 };
            let (x, w) = if side { (6, 9) } else { (6, 16) };
            for yy in 0..len {
                for xx in 0..w {
                    let tip = yy > len - 3 && (xx + yy) % 3 == 0;
                    if tip {
                        continue;
                    }
                    let c = if xx < 2 { hair[2] } else if xx > w - 3 { hair[0] } else { hair[1] };
                    // The ends trail behind the body's bounce.
                    let sway = if yy > len * 2 / 3 { lag } else { 0 };
                    p.set(x + xx, hy + 8 + yy + sway.max(0), c);
                }
            }
        }
        2 if side => {
            // Ponytail swinging behind.
            for yy in 0..14 {
                let x = 4 - (yy / 5) - if yy > 6 { lag } else { 0 };
                p.fill(x, hy + 7 + yy, 4, 1, if yy < 3 { hair[3] } else { hair[1 + (yy % 4 == 0) as usize] });
            }
        }
        6 => {
            if dir == DIR_UP {
                return;
            }
            // Twin tails either side (overrides long hair when hair_style 6 & front).
            let _ = ty;
        }
        _ => {}
    }
    if look.hair_style == 6 && dir == DIR_DOWN {
        for (x, s) in [(2, 1), (22, -1)] {
            for yy in 0..18 {
                let xx = x + if yy > 9 { s } else { 0 } + if yy > 12 { lag * s } else { 0 };
                p.fill(xx, hy + 6 + yy, 4, 1, if yy < 2 { hair[3] } else if s > 0 { hair[2] } else { hair[1] });
            }
        }
    }
}

fn front_hair(p: &mut P, look: &Look, dir: u8, hy: i32, hair: &[Rgb; 4]) {
    let style = look.hair_style;
    let side = dir == DIR_SIDE;
    let hx0 = if side { 7 } else { 6 };
    if dir == DIR_UP {
        let len = match style {
            1 => 26,
            6 => 26,
            5 => 15,
            2 | 3 => 14,
            _ => 12,
        };
        for k in 0..len {
            let inset = if k < HEAD_INSET.len() as i32 { HEAD_INSET[k as usize].min(3) } else { 0 };
            let tipcut = k > len - 3 && (k % 2 == 0);
            for x in hx0 - 1 + inset..hx0 + 17 - inset {
                if tipcut && (x % 3 == 0) {
                    continue;
                }
                let strand = (x == hx0 + 4 || x == hx0 + 9 || x == hx0 + 13) && k > 4;
                let c = if k == 3 && (hx0 + 3..hx0 + 13).contains(&x) {
                    hair[3]
                } else if x >= hx0 + 14 - inset || strand {
                    hair[1]
                } else if x <= hx0 + inset {
                    hair[2]
                } else {
                    hair[2]
                };
                p.set(x, hy + k - 1, c);
            }
        }
        if style == 2 {
            p.part(12, hy + 12, 4, 12, hair);
        }
        if style == 3 {
            p.part(10, hy - 4, 8, 5, hair);
        }
        if style == 6 {
            for x in [3, 22] {
                p.part(x, hy + 4, 3, 18, hair);
            }
        }
        return;
    }
    // Crown with an angel-ring highlight.
    for k in 0..8 {
        let inset = HEAD_INSET[k as usize].min(4);
        let (l, r) = (hx0 - 1 + inset, hx0 + 16 - inset);
        for x in l..=r {
            let c = if k == 3 && x > l + 2 && x < r - 3 && x % 5 != 0 {
                hair[3]
            } else if x <= l + 1 {
                hair[2]
            } else if x >= r - 1 {
                hair[1]
            } else {
                hair[2]
            };
            p.set(x, hy + k - 1, c);
        }
    }
    if side {
        // Hair over the back of the head, fringe in front.
        let back_len = match style {
            1 | 6 => 16,
            5 => 13,
            _ => 10,
        };
        for k in 6..6 + back_len {
            for x in hx0 - 1..hx0 + 6 {
                let c = if x == hx0 - 1 { hair[1] } else if (x + k) % 6 == 0 { hair[1] } else { hair[2] };
                p.set(x, hy + k, c);
            }
        }
        // Fringe locks over the forehead.
        let fringe = [9, 8, 9, 7, 8, 9, 7, 8, 6];
        for (i, &f) in fringe.iter().enumerate() {
            let x = hx0 + 7 + i as i32;
            for k in 6..f {
                p.set(x, hy + k, if i % 3 == 0 { hair[1] } else { hair[2] });
            }
        }
        // Sideburn by the ear.
        p.fill(hx0 + 6, hy + 7, 2, 4, hair[1]);
        if style == 3 {
            p.part(hx0 + 2, hy - 4, 7, 5, hair);
            p.set(hx0 + 3, hy - 3, hair[3]);
        }
        if style == 4 {
            for k in 0..4 {
                let x = hx0 + 1 + k * 4;
                p.set(x, hy - 2, hair[3]);
                p.set(x + 1, hy - 2, hair[2]);
                p.set(x + 1, hy - 3, hair[3]);
            }
        }
        return;
    }
    // Front: fringe with pointed anime locks.
    let fringe: [i32; 18] = match style {
        5 => [10, 10, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 10, 10],
        4 => [10, 9, 8, 10, 8, 7, 9, 8, 6, 8, 9, 7, 8, 10, 8, 9, 10, 10],
        7 => [12, 12, 11, 10, 10, 9, 9, 8, 8, 7, 7, 7, 6, 7, 8, 8, 9, 10],
        3 => [9, 8, 8, 7, 7, 8, 7, 6, 7, 7, 6, 7, 8, 7, 7, 8, 8, 9],
        _ => [10, 9, 8, 9, 7, 8, 9, 7, 8, 9, 7, 8, 9, 7, 8, 9, 9, 10],
    };
    for (i, &f) in fringe.iter().enumerate() {
        let x = hx0 - 1 + i as i32;
        for k in 6..f {
            let c = if i < 4 { hair[2] } else if i > 13 { hair[1] } else if k == f - 1 { hair[1] } else { hair[2] };
            p.set(x, hy + k, c);
        }
    }
    // Side locks framing the face.
    let lock = match style {
        1 | 6 => 18,
        5 => 15,
        2 | 3 | 7 => 13,
        _ => 11,
    };
    for k in 6..6 + lock {
        let w = if k > 4 + lock { 1 } else { 2 };
        p.fill(hx0 - 1, hy + k, w, 1, hair[2]);
        p.fill(hx0 + 17 - w, hy + k, w, 1, hair[1]);
    }
    if style == 3 {
        p.part(10, hy - 5, 8, 5, hair);
        p.set(11, hy - 4, hair[3]);
    }
    if style == 4 {
        for k in 0..5 {
            let x = hx0 + k * 4;
            p.set(x, hy - 2, hair[3]);
            p.set(x + 1, hy - 2, hair[2]);
            p.set(x + 1, hy - 3, hair[3]);
        }
    }
    if style == 2 {
        // Ponytail peeking out to the side.
        p.part(21, hy + 3, 3, 5, hair);
    }
}

#[allow(clippy::too_many_arguments)]
fn outfit_layer(p: &mut P, look: &Look, dir: u8, tx: i32, tw: i32, ty: i32, th: i32, leg_y: i32, leg_h: i32, feet_y: i32, b: &Body, lag: i32) {
    let side = dir == DIR_SIDE;
    match look.outfit {
        1 => {
            // Dress: skirt flares from the waist to the knee.
            let sy = ty + th - 4;
            let sh = leg_h - 2 + 4;
            for k in 0..sh {
                let grow = k * 2 / 5 + 1;
                // Hem flutters with the stride.
                let flutter = if k >= sh - 2 { lag } else { 0 };
                let x0 = tx - grow + if side { 1 } else { 0 } + flutter;
                let w = tw + grow * 2 - if side { 1 } else { 0 };
                for x in x0..x0 + w {
                    let fold = (x - x0) % 4 == 3 && k > 2;
                    let c = if x == x0 { b.top[3] } else if x >= x0 + w - 2 || k == sh - 1 || fold { b.top[1] } else { b.top[2] };
                    p.set(x, sy + k, c);
                }
            }
            p.fill(tx, sy, tw, 1, b.top[0]);
        }
        2 => {
            let ry = ty + 3;
            for k in 0..(feet_y - ry + 1) {
                let grow = k / 4;
                let x0 = tx - grow;
                let w = tw + grow * 2;
                for x in x0..x0 + w {
                    let c = if x == x0 { b.top[3] } else if x >= x0 + w - 2 { b.top[1] } else { b.top[2] };
                    p.set(x, ry + k, c);
                }
            }
            if !side {
                p.fill(13, ty + 1, 2, feet_y - ty - 1, b.top[1]);
                p.fill(tx, ty + th - 4, tw, 1, hex(pal::GOLD[1]));
                p.set(13, ty + 4, hex(pal::GOLD[3]));
            }
        }
        3 if dir != DIR_UP => {
            let ap = ramp4(&pal::CLOTH[7]);
            let (ax, aw) = if side { (14, 4) } else { (11, 6) };
            p.part(ax, ty + 3, aw, th + 4, &ap);
            p.fill(ax, ty + 3, aw, 1, ap[3]);
            if !side {
                p.set(11, ty + 1, ap[2]);
                p.set(16, ty + 1, ap[2]);
            }
        }
        4 => {
            if !side {
                p.part(tx - 2, ty, 3, 3, &b.metal);
                p.part(tx + tw - 1, ty, 3, 3, &b.metal);
            } else {
                p.part(tx + 2, ty, 4, 3, &b.metal);
            }
            p.fill(tx, ty + th - 3, tw, 1, hex(0x4a3020));
            p.set(tx + tw / 2, ty + th - 3, hex(pal::GOLD[3]));
            if dir == DIR_DOWN {
                p.fill(tx + 2, ty + th - 2, tw - 4, 4, b.hatc[2]);
            }
        }
        6 if dir != DIR_UP => {
            let vest = ramp4(&pal::CLOTH[3]);
            if side {
                p.part(tx + 4, ty, 4, th - 2, &vest);
            } else {
                p.part(tx, ty, 3, th - 2, &vest);
                p.part(tx + tw - 3, ty, 3, th - 2, &vest);
                p.set(tx + 3, ty + 3, hex(pal::GOLD[2]));
            }
        }
        _ => {}
    }
    if look.outfit != 2 && look.outfit != 1 && look.outfit != 4 {
        // Belt
        p.fill(tx, ty + th - 3, tw, 1, b.bottom[0]);
        if dir == DIR_DOWN {
            p.set(13, ty + th - 3, hex(pal::GOLD[2]));
            p.set(14, ty + th - 3, hex(pal::GOLD[3]));
        }
    }
    if look.outfit == 5 && dir == DIR_UP {
        p.part(7, ty, 14, th + leg_h - 2, &b.hatc);
        p.fill(7, ty, 14, 1, b.hatc[3]);
    }
    let _ = leg_y;
}

/// Adult female figure with a little secondary bounce on the walk cycle.
#[allow(clippy::too_many_arguments)]
fn figure(p: &mut P, look: &Look, dir: u8, pose: Pose, tx: i32, tw: i32, ty: i32, b: &Body) {
    let c = if look.outfit == 3 && dir != DIR_UP { ramp4(&pal::CLOTH[7]) } else { b.top };
    let y = ty + 3 + pose.lag.max(0);
    match dir {
        DIR_DOWN => {
            // Two soft curves: highlight on top, shadow beneath.
            for (cx, lit) in [(tx + 2, true), (tx + tw - 4, false)] {
                p.set(cx, y, if lit { c[3] } else { c[2] });
                p.set(cx + 1, y, c[2]);
                p.set(cx - 1, y + 1, if lit { c[3] } else { c[2] });
                p.set(cx, y + 1, c[2]);
                p.set(cx + 1, y + 1, c[2]);
                p.set(cx + 2, y + 1, if lit { c[2] } else { c[1] });
                p.set(cx, y + 2, c[1]);
                p.set(cx + 1, y + 2, c[1]);
            }
            p.set(tx + tw / 2 - 1, y + 1, c[1]);
            p.set(tx - 1, y + 1, c[2]);
            p.set(tx + tw, y + 1, c[1]);
        }
        DIR_SIDE => {
            let fx = tx + tw;
            p.set(fx, y, c[2]);
            p.set(fx, y + 1, c[2]);
            p.set(fx + 1, y + 1, c[1]);
            p.set(fx, y + 2, c[1]);
            p.set(fx - 1, y, c[3]);
        }
        _ => {
            p.set(tx - 1, y + 1, c[1]);
            p.set(tx + tw, y + 1, c[1]);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn arms(p: &mut P, look: &Look, dir: u8, pose: Pose, tx: i32, tw: i32, ty: i32, th: i32, b: &Body) {
    let sleeve = if look.outfit == 4 { b.metal } else { b.top };
    let len = th - 3;
    if dir == DIR_SIDE {
        match pose.work {
            1 => {
                p.part(14, ty - 6, 3, 7, &sleeve);
                p.fill(14, ty - 8, 3, 2, b.skin[2]);
            }
            2 => {
                p.part(16, ty + 2, 6, 3, &sleeve);
                p.fill(22, ty + 2, 2, 3, b.skin[2]);
            }
            _ => {
                let ax = 13 + pose.arm;
                p.part(ax, ty + 1, 3, len, &sleeve);
                p.fill(ax, ty + 1 + len, 3, 2, b.skin[2]);
                p.set(ax + 2, ty + 2 + len, b.skin[1]);
            }
        }
        return;
    }
    for (ax, swing, left) in [(tx - 3, pose.arm, true), (tx + tw, -pose.arm, false)] {
        match pose.work {
            1 => {
                p.part(ax, ty - 6, 3, 7, &sleeve);
                p.fill(ax, ty - 8, 3, 2, b.skin[2]);
            }
            2 => {
                let ix = if left { ax + 2 } else { ax - 2 };
                p.part(ix, ty + 2, 3, len, &sleeve);
                p.fill(ix, ty + 2 + len, 3, 2, b.skin[2]);
            }
            _ => {
                let dy = if dir == DIR_DOWN { swing.max(0) } else { (-swing).max(0) };
                let r = if left { sleeve } else { [sleeve[0], sleeve[1], sleeve[1], sleeve[2]] };
                p.part(ax, ty + 1, 3, len + dy - 1, &r);
                p.fill(ax, ty + len + dy, 3, 2, b.skin[if left { 3 } else { 2 }]);
                p.set(if left { ax + 2 } else { ax }, ty + len + dy + 1, b.skin[1]);
            }
        }
    }
}

fn hat_layer(p: &mut P, look: &Look, dir: u8, hy: i32, hc: &[Rgb; 4], metal: &[Rgb; 4]) {
    let side = dir == DIR_SIDE;
    let hx0 = if side { 7 } else { 6 };
    let (l, r) = (hx0 - 1, hx0 + 16);
    match look.hat {
        1 => {
            let straw = [hex(0x8a6a2a), hex(0xb8923e), hex(0xdcb860), hex(0xf2da8e)];
            for x in l - 3..=r + 3 {
                let c = if (x + hy) % 3 == 0 { straw[1] } else if x < l + 3 { straw[3] } else { straw[2] };
                p.set(x, hy + 4, c);
                p.set(x, hy + 5, straw[1]);
            }
            for k in 0..6 {
                let inset = if k == 0 { 2 } else { 0 };
                for x in l + 3 + inset..r - 2 - inset {
                    let c = if (x + k) % 4 == 0 { straw[1] } else if x < l + 6 { straw[3] } else { straw[2] };
                    p.set(x, hy - 2 + k, c);
                }
            }
            p.fill(l + 3, hy + 2, r - l - 5, 2, hex(0xa83a30));
            p.fill(l + 3, hy + 2, r - l - 5, 1, hex(0xd0584a));
        }
        2 => {
            p.fill(l, hy + 3, r - l + 1, 3, hc[2]);
            p.fill(l, hy + 5, r - l + 1, 1, hc[1]);
            p.fill(l, hy + 3, r - l + 1, 1, hc[3]);
            if side {
                p.fill(l - 3, hy + 5, 3, 2, hc[2]);
                p.fill(l - 2, hy + 7, 2, 3, hc[1]);
            }
        }
        3 => {
            for k in 0..9 {
                let inset = HEAD_INSET[k as usize].min(4);
                for x in l + inset..=r - inset {
                    let c = if x <= l + inset + 1 || k == 1 { metal[3] } else if x >= r - inset - 1 { metal[1] } else { metal[2] };
                    p.set(x, hy - 1 + k, c);
                }
            }
            p.fill(l, hy + 7, r - l + 1, 2, metal[1]);
            p.fill(l, hy + 7, r - l + 1, 1, metal[3]);
            if dir == DIR_DOWN {
                p.fill(13, hy + 9, 2, 5, metal[2]);
            }
            p.set(14, hy - 3, hex(0xc8403a));
            p.fill(13, hy - 2, 3, 1, hex(0xa83030));
        }
        4 => {
            for k in -1..19 {
                let inset = if k < 0 { 4 } else if (k as usize) < HEAD_INSET.len() { HEAD_INSET[k as usize].min(3) } else { 0 };
                for x in l - 1 + inset..=r + 1 - inset {
                    let face_open = match dir {
                        DIR_DOWN => k >= 6 && k <= 17 && x >= l + 3 && x <= r - 3,
                        DIR_SIDE => k >= 6 && k <= 17 && x >= l + 9,
                        _ => false,
                    };
                    if face_open {
                        continue;
                    }
                    let c = if x <= l + 1 { hc[3] } else if x >= r - 1 { hc[1] } else { hc[2] };
                    p.set(x, hy + k - 1, c);
                }
            }
        }
        5 => {
            for k in 0..5 {
                let inset = HEAD_INSET[k as usize].min(3);
                for x in l + 1 + inset..r - inset {
                    let c = if k == 0 { hc[3] } else if x >= r - inset - 2 { hc[1] } else { hc[2] };
                    p.set(x, hy - 1 + k, c);
                }
            }
            if dir == DIR_DOWN {
                p.fill(l + 2, hy + 4, r - l - 3, 2, hc[1]);
            } else if side {
                p.fill(r - 3, hy + 3, 6, 2, hc[1]);
            }
        }
        6 => {
            for x in l - 4..=r + 4 {
                p.set(x, hy + 4, if x < l { hc[3] } else { hc[2] });
                p.set(x, hy + 5, hc[1]);
            }
            for k in 0..12 {
                let half = 7 - k * 6 / 11;
                let cx = 14 + if k > 7 { (k - 7) / 2 } else { 0 };
                for x in cx - half..=cx + half {
                    let c = if x <= cx - half + 1 { hc[3] } else if x >= cx + half - 1 { hc[1] } else { hc[2] };
                    p.set(x, hy + 3 - k, c);
                }
            }
            p.fill(l + 2, hy + 2, r - l - 3, 2, hex(pal::GOLD[2]));
            p.set(14, hy + 2, hex(pal::GOLD[4]));
        }
        7 => {
            let f = pal::FLOWERS[look.hat_color as usize % pal::FLOWERS.len()];
            let (fx, fy) = if side { (10, hy + 2) } else { (19, hy + 3) };
            for (dx, dy, c) in [(-1, 0, 1), (1, 0, 1), (0, -1, 2), (0, 1, 0), (-1, -1, 2), (1, 1, 0)] {
                p.set(fx + dx, fy + dy, hex(f[c]));
            }
            p.set(fx, fy, hex(0xf6d860));
            p.set(fx + 2, fy + 2, hex(pal::LEAF_SUMMER[4]));
        }
        _ => {}
    }
}

/// Build all frames of a sheet in atlas order: for each dir (down, up,
/// side) the 8 pose frames, then the blink frame.
pub fn sheet(look: &Look) -> Vec<Img> {
    let mut out = Vec::with_capacity(SHEET_FRAMES as usize);
    for dir in [DIR_DOWN, DIR_UP, DIR_SIDE] {
        for f in 0..FRAMES_PER_DIR {
            out.push(frame(look, dir, f));
        }
    }
    out.push(frame(look, DIR_DOWN, 8));
    out
}
