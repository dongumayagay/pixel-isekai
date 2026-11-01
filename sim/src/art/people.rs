//! Procedural anime characters built from a posed skeleton.
//!
//! Each body part is a smooth primitive (ellipse head, a torso with a curved
//! width profile, tapered capsule limbs) lit per pixel from its surface
//! normal, then quantised to 5-tone ramps. Clothing and hair are shaped
//! around the same skeleton. Walking really swings the limbs at their
//! joints, with lagging secondary motion on hair, skirts and figure.
//!
//! Frames are 48×96 (adults stand ~80px). Sheet layout: 3 directions (down,
//! up, side-facing-right) × 8 poses (idle, breathe, walk ×4, work ×2), then
//! one blink frame.

use serde::{Deserialize, Serialize};

use super::canvas::Img;
use crate::palette::{self as pal, hex, mix, Rgb};
use crate::rng::Rng;

pub const FW: i32 = 48;
pub const FH: i32 = 96;
pub const FRAMES_PER_DIR: u16 = 8;
pub const SHEET_FRAMES: u16 = FRAMES_PER_DIR * 3 + 1;
pub const PIVOT: (i32, i32) = (24, 93);

pub const DIR_DOWN: u8 = 0;
pub const DIR_UP: u8 = 1;
pub const DIR_SIDE: u8 = 2;

pub const RACE_HUMAN: u8 = 0;
pub const RACE_ELF: u8 = 1;
pub const RACE_DWARF: u8 = 2;
pub const RACE_BEAST: u8 = 3;
pub const RACE_DEMON: u8 = 4;
pub const RACE_NAMES: [&str; 5] = ["Human", "Elf", "Dwarf", "Beastfolk", "Demon"];

pub mod outfit {
    pub const TUNIC: u8 = 0;
    pub const DRESS: u8 = 1;
    pub const ROBE: u8 = 2;
    pub const APRON: u8 = 3;
    pub const ARMOR: u8 = 4;
    pub const CLOAK: u8 = 5;
    pub const VEST: u8 = 6;
    pub const DIRNDL: u8 = 7;
    pub const GUILD: u8 = 8;
    pub const SMITH: u8 = 9;
}

pub mod held {
    pub const NONE: u8 = 0;
    pub const STAFF: u8 = 1;
    pub const HAMMER: u8 = 2;
    pub const MUG: u8 = 3;
    pub const BOOK: u8 = 4;
    pub const SWORD: u8 = 5;
    pub const BASKET: u8 = 6;
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Look {
    pub skin: u8,
    pub hair: u8,
    pub hair_style: u8,
    pub top: u8,
    pub bottom: u8,
    pub outfit: u8,
    /// 0 none, 1 straw hat, 2 bandana, 3 helmet, 4 hood, 5 cap, 6 witch hat,
    /// 7 flower, 8 chef toque, 9 ribbon, 10 maid headband
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
    #[serde(default)]
    pub race: u8,
    /// Beastfolk kind: 0 cat, 1 fox, 2 wolf, 3 rabbit.
    #[serde(default)]
    pub beast: u8,
    #[serde(default)]
    pub held: u8,
    #[serde(default)]
    pub glasses: bool,
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
            // Cloth that can't be mistaken for skin: no ochre, linen or rust.
            top: *rng.pick(&[0u8, 2, 4, 5, 8, 9, 0, 2, 9]),
            bottom: *rng.pick(&[0u8, 2, 3, 4, 5, 8, 9]),
            outfit: if female && rng.chance(0.45) { outfit::DRESS } else if rng.chance(0.2) { outfit::VEST } else { outfit::TUNIC },
            hat: 0,
            hat_color: rng.below(pal::CLOTH.len()) as u8,
            female,
            age,
            beard: !female && age >= 1 && rng.chance(if age == 2 { 0.6 } else { 0.2 }),
            eyes: rng.below(EYE_COLORS.len()) as u8,
            mature: age >= 1,
            race: RACE_HUMAN,
            beast: 0,
            held: held::NONE,
            glasses: false,
        }
    }
}

type R5 = [Rgb; 5];

fn ramp4(r: &[u32; 4]) -> [Rgb; 4] {
    [hex(r[0]), hex(r[1]), hex(r[2]), hex(r[3])]
}

/// Five-tone ramp from a 4-tone one: adds a soft highlight above the top.
fn r5(r: &[Rgb; 4]) -> R5 {
    [r[0], r[1], r[2], r[3], mix(r[3], hex(0xfff8ec), 0.35)]
}

fn gold5() -> R5 {
    r5(&ramp4(&[pal::GOLD[0], pal::GOLD[1], pal::GOLD[2], pal::GOLD[3]]))
}

const EYE_COLORS: [[u32; 3]; 7] = [
    [0x1a2456, 0x3a5ab0, 0x84b0f0],
    [0x14382a, 0x2c8050, 0x78d098],
    [0x34200f, 0x6e4620, 0xb88848],
    [0x341a4a, 0x7a3aa8, 0xc088e8],
    [0x46121a, 0xa82c30, 0xf07a60],
    [0x3a2c0c, 0xa07c18, 0xf0cc58],
    [0x163a40, 0x2a8a90, 0x78d8d8],
];

// ---------------------------------------------------------------- geometry

#[derive(Clone, Copy, Default)]
struct V2 {
    x: f32,
    y: f32,
}

fn v(x: f32, y: f32) -> V2 {
    V2 { x, y }
}

impl V2 {
    fn add(self, o: V2) -> V2 {
        v(self.x + o.x, self.y + o.y)
    }
    /// Angle 0 points straight down; positive swings toward +x.
    fn polar(len: f32, ang: f32) -> V2 {
        v(ang.sin() * len, ang.cos() * len)
    }
}

/// Light direction (toward the light): top-left and toward the viewer.
const LX: f32 = -0.45;
const LY: f32 = -0.6;
const LZ: f32 = 0.66;

/// Map a surface normal to a ramp index (0..=4), dithering across band edges.
fn tone(nx: f32, ny: f32, nz: f32, x: i32, y: i32, bias: f32) -> usize {
    let d = nx * LX + ny * LY + nz * LZ;
    let t = (0.42 + d * 0.62 + bias).clamp(0.0, 0.999) * 5.0;
    let i = t as usize;
    let frac = t - i as f32;
    let b = crate::palette::bayer4(x, y);
    if frac > 0.82 && b < (frac - 0.82) * 5.5 { (i + 1).min(4) } else { i }
}

struct Canvas<'a> {
    img: &'a mut Img,
}

impl Canvas<'_> {
    fn put(&mut self, x: i32, y: i32, c: Rgb) {
        self.img.set(x, y, c);
    }

    /// Filled ellipse lit as a dome.
    fn ellipse(&mut self, c: V2, rx: f32, ry: f32, ramp: &R5, bias: f32) {
        let (x0, x1) = ((c.x - rx).floor() as i32, (c.x + rx).ceil() as i32);
        let (y0, y1) = ((c.y - ry).floor() as i32, (c.y + ry).ceil() as i32);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let nx = (x as f32 + 0.5 - c.x) / rx;
                let ny = (y as f32 + 0.5 - c.y) / ry;
                let d = nx * nx + ny * ny;
                if d <= 1.0 {
                    let nz = (1.0 - d).sqrt();
                    self.put(x, y, ramp[tone(nx, ny, nz, x, y, bias)]);
                }
            }
        }
    }

    /// Tapered capsule from a (radius ra) to b (radius rb), lit as a cylinder.
    fn capsule(&mut self, a: V2, b: V2, ra: f32, rb: f32, ramp: &R5, bias: f32) {
        let r = ra.max(rb);
        let (x0, x1) = ((a.x.min(b.x) - r).floor() as i32, (a.x.max(b.x) + r).ceil() as i32);
        let (y0, y1) = ((a.y.min(b.y) - r).floor() as i32, (a.y.max(b.y) + r).ceil() as i32);
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let len2 = (dx * dx + dy * dy).max(1e-4);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                let t = (((px - a.x) * dx + (py - a.y) * dy) / len2).clamp(0.0, 1.0);
                let (cx, cy) = (a.x + dx * t, a.y + dy * t);
                let rr = ra + (rb - ra) * t;
                let (ox, oy) = (px - cx, py - cy);
                if ox * ox + oy * oy <= rr * rr {
                    let (nx, ny) = (ox / rr, oy / rr);
                    let nz = (1.0 - (nx * nx + ny * ny)).max(0.0).sqrt();
                    self.put(x, y, ramp[tone(nx, ny * 0.6, nz, x, y, bias)]);
                }
            }
        }
    }

    /// Capsule with flattened side-to-side shading, for hair locks that
    /// should read as one mass rather than separate tubes.
    fn lock(&mut self, a: V2, b: V2, ra: f32, rb: f32, ramp: &R5, bias: f32) {
        let r = ra.max(rb);
        let (x0, x1) = ((a.x.min(b.x) - r).floor() as i32, (a.x.max(b.x) + r).ceil() as i32);
        let (y0, y1) = ((a.y.min(b.y) - r).floor() as i32, (a.y.max(b.y) + r).ceil() as i32);
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let len2 = (dx * dx + dy * dy).max(1e-4);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                let t = (((px - a.x) * dx + (py - a.y) * dy) / len2).clamp(0.0, 1.0);
                let (cx, cy) = (a.x + dx * t, a.y + dy * t);
                let rr = ra + (rb - ra) * t;
                let (ox, oy) = (px - cx, py - cy);
                if ox * ox + oy * oy <= rr * rr {
                    // Mostly lit by height along the lock, little by width.
                    let nx = ox / rr * 0.25;
                    let ny = -0.3 + t * 0.9;
                    let nz = (1.0 - nx * nx - ny * ny).max(0.0).sqrt();
                    self.put(x, y, ramp[tone(nx, ny, nz, x, y, bias)]);
                }
            }
        }
    }

    /// Region y0..y1 with half-width hw(y) around centre cx(y), lit as a
    /// vertical cylinder.
    fn profile(&mut self, y0: f32, y1: f32, cx: impl Fn(f32) -> f32, hw: impl Fn(f32) -> f32, ramp: &R5, bias: f32) {
        for y in y0.floor() as i32..=y1.ceil() as i32 {
            let fy = y as f32 + 0.5;
            if fy < y0 || fy > y1 {
                continue;
            }
            let c = cx(fy);
            let w = hw(fy);
            if w <= 0.2 {
                continue;
            }
            for x in (c - w).floor() as i32..=(c + w).ceil() as i32 {
                let nx = (x as f32 + 0.5 - c) / w;
                if nx.abs() > 1.0 {
                    continue;
                }
                let nz = (1.0 - nx * nx).sqrt();
                let ny = ((fy - y0) / (y1 - y0) - 0.5) * 0.3;
                self.put(x, y, ramp[tone(nx, ny, nz, x, y, bias)]);
            }
        }
    }

    fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Rgb) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.put(xx, yy, c);
            }
        }
    }
}

/// Smooth interpolation through (y, value) keys.
fn keyed(keys: &[(f32, f32)], y: f32) -> f32 {
    if y <= keys[0].0 {
        return keys[0].1;
    }
    for w in keys.windows(2) {
        let (a, b) = (w[0], w[1]);
        if y <= b.0 {
            let t = (y - a.0) / (b.0 - a.0);
            let t = t * t * (3.0 - 2.0 * t);
            return a.1 + (b.1 - a.1) * t;
        }
    }
    keys[keys.len() - 1].1
}

// ---------------------------------------------------------------- skeleton

#[derive(Clone, Copy)]
struct Pose {
    walk: Option<f32>,
    breathe: bool,
    work: u8,
    blink: bool,
}

fn pose_for(f: u16) -> Pose {
    let base = Pose { walk: None, breathe: false, work: 0, blink: false };
    match f {
        0 => base,
        1 => Pose { breathe: true, ..base },
        2..=5 => Pose { walk: Some((f - 2) as f32 * std::f32::consts::FRAC_PI_2 + 0.5), ..base },
        6 => Pose { work: 1, ..base },
        7 => Pose { work: 2, ..base },
        _ => Pose { blink: true, ..base },
    }
}

/// Body measurements in pixels, derived from race, age and sex.
struct Build {
    cx: f32,
    ground: f32,
    head_r: (f32, f32),
    head_c: V2,
    shoulder_y: f32,
    shoulder_w: f32,
    chest_w: f32,
    waist_y: f32,
    waist_w: f32,
    hip_y: f32,
    hip_w: f32,
    crotch_y: f32,
    thigh: f32,
    calf: f32,
    upper_arm: f32,
    forearm: f32,
    limb_r: f32,
    bust: f32,
    /// Lag applied to soft parts (hair, skirts, figure) this frame.
    lag: f32,
}

fn build(look: &Look, pose: Pose) -> Build {
    let child = look.age == 0;
    let dwarf = look.race == RACE_DWARF;
    let elf = look.race == RACE_ELF;
    let f = look.female;
    let (leg_k, torso_k, width_k) = if child {
        (0.55, 0.62, 0.82)
    } else if dwarf {
        (0.62, 0.92, 1.3)
    } else if elf {
        (1.08, 1.0, 0.92)
    } else {
        (1.0, 1.0, 1.0)
    };
    let thigh = 14.0 * leg_k;
    let calf = 13.0 * leg_k;
    let ground = 92.0;
    let (bob, lag) = match pose.walk {
        Some(ph) => (-(ph.sin().abs()) * 1.6, -ph.cos() * 1.2),
        None if pose.breathe => (0.6, 0.7),
        None if pose.work == 1 => (-0.6, -0.8),
        None if pose.work == 2 => (0.8, 1.0),
        None => (0.0, 0.0),
    };
    let crotch_y = ground - thigh - calf - 2.0 + bob;
    let torso = 19.0 * torso_k;
    let shoulder_y = crotch_y - torso;
    let head_r = if child { (12.0, 12.5) } else { (12.5, 13.5) };
    let head_c = v(24.0, shoulder_y - 3.0 - head_r.1 + if pose.breathe { 0.4 } else { 0.0 });
    let sw = if f { 7.6 } else { 9.0 } * width_k;
    Build {
        cx: 24.0,
        ground,
        head_r,
        head_c,
        shoulder_y,
        shoulder_w: sw,
        chest_w: if f { 7.6 } else { 8.8 } * width_k,
        waist_y: shoulder_y + torso * 0.55,
        waist_w: if f { 5.4 } else { 7.2 } * width_k,
        hip_y: shoulder_y + torso * 0.88,
        hip_w: if f { 9.6 } else { 7.8 } * width_k,
        crotch_y,
        thigh,
        calf,
        upper_arm: 10.0 * torso_k,
        forearm: 9.0 * torso_k,
        limb_r: if child { 2.2 } else if f { 2.8 } else { 2.7 } * width_k.max(0.9),
        bust: if f && look.mature && look.age != 0 { 3.5 } else { 0.0 },
        lag,
    }
}

struct Limbs {
    hip_l: V2,
    knee_l: V2,
    ankle_l: V2,
    hip_r: V2,
    knee_r: V2,
    ankle_r: V2,
    sh_l: V2,
    el_l: V2,
    wr_l: V2,
    sh_r: V2,
    el_r: V2,
    wr_r: V2,
}

fn limbs(b: &Build, pose: Pose, dir: u8) -> Limbs {
    let side = dir == DIR_SIDE;
    let walking = pose.walk.is_some();
    let s = pose.walk.unwrap_or(0.0).sin();
    let (hip_dx, sh_dx) = if side { (0.0, 0.0) } else { (b.hip_w * 0.5, b.shoulder_w + 0.6) };
    let hip_y = b.crotch_y - 1.5;
    let hip_l = v(b.cx - hip_dx, hip_y);
    let hip_r = v(b.cx + hip_dx, hip_y);
    let leg = |hip: V2, swing: f32, lift: f32| -> (V2, V2) {
        if side {
            let th = swing * 0.5;
            let knee = hip.add(V2::polar(b.thigh, th));
            let bend = (-swing).max(0.0) * 0.7 + lift * 0.3;
            let ankle = knee.add(V2::polar(b.calf, th - bend));
            (knee, ankle)
        } else {
            let knee = hip.add(v(0.0, b.thigh - lift * 1.2));
            let ankle = knee.add(v(0.0, b.calf - lift * 2.4));
            (knee, ankle)
        }
    };
    let (sl, sr) = if walking { (s, -s) } else { (0.0, 0.0) };
    let lift_l = if walking { s.max(0.0) * 2.0 } else { 0.0 };
    let lift_r = if walking { (-s).max(0.0) * 2.0 } else { 0.0 };
    let (knee_l, ankle_l) = leg(hip_l, sl, lift_l);
    let (knee_r, ankle_r) = leg(hip_r, sr, lift_r);
    let sh_y = b.shoulder_y + 2.2;
    let sh_l = v(b.cx - sh_dx, sh_y);
    let sh_r = v(b.cx + sh_dx, sh_y);
    let arm = |sh: V2, swing: f32, out: f32| -> (V2, V2) {
        match pose.work {
            1 => {
                let el = sh.add(v(out * 2.0 + if side { 2.0 } else { 0.0 }, -b.upper_arm * 0.9));
                let wr = el.add(v(out * 1.0 + if side { 2.0 } else { 0.0 }, -b.forearm * 0.8));
                (el, wr)
            }
            2 => {
                if side {
                    let el = sh.add(V2::polar(b.upper_arm, 1.0));
                    let wr = el.add(V2::polar(b.forearm, 1.45));
                    (el, wr)
                } else {
                    let el = sh.add(v(out * 0.5, b.upper_arm * 0.85));
                    let wr = el.add(v(-out * 2.5, b.forearm * 0.5));
                    (el, wr)
                }
            }
            _ => {
                if side {
                    let el = sh.add(V2::polar(b.upper_arm, swing * 0.45));
                    let wr = el.add(V2::polar(b.forearm, swing * 0.45 + 0.25 * swing.abs() + 0.12));
                    (el, wr)
                } else {
                    let el = sh.add(v(out * 1.2, b.upper_arm - swing.abs() * 0.6));
                    let wr = el.add(v(out * 0.6, b.forearm - swing.max(0.0) * 1.4));
                    (el, wr)
                }
            }
        }
    };
    let (el_l, wr_l) = arm(sh_l, -sl, -1.0);
    let (el_r, wr_r) = arm(sh_r, -sr, 1.0);
    Limbs { hip_l, knee_l, ankle_l, hip_r, knee_r, ankle_r, sh_l, el_l, wr_l, sh_r, el_r, wr_r }
}

// ---------------------------------------------------------------- palette

struct Colors {
    skin: R5,
    hair: R5,
    top: R5,
    bottom: R5,
    hatc: R5,
    metal: R5,
    leather: R5,
    linen: R5,
    shoe: R5,
    stocking: R5,
    navy: R5,
}

fn colors(look: &Look) -> Colors {
    let skin = if look.race == RACE_DEMON {
        ramp4(&pal::DEMON_SKIN[look.skin as usize % pal::DEMON_SKIN.len()])
    } else {
        ramp4(&pal::SKIN[look.skin as usize % pal::SKIN.len()])
    };
    Colors {
        skin: r5(&skin),
        hair: r5(&ramp4(&pal::HAIR[look.hair as usize % pal::HAIR.len()])),
        top: r5(&ramp4(&pal::CLOTH[look.top as usize % pal::CLOTH.len()])),
        bottom: r5(&ramp4(&pal::CLOTH[look.bottom as usize % pal::CLOTH.len()])),
        hatc: r5(&ramp4(&pal::CLOTH[look.hat_color as usize % pal::CLOTH.len()])),
        metal: [hex(0x2a2a3a), hex(pal::METAL[1]), hex(pal::METAL[2]), hex(pal::METAL[3]), hex(pal::METAL[4])],
        leather: [hex(0x2e1c1c), hex(0x4a2e24), hex(0x6a4430), hex(0x8c5e40), hex(0xae7c54)],
        linen: [hex(0x7a6c6a), hex(0xb0a49c), hex(0xd8cec2), hex(0xf2eadc), hex(0xfffaf2)],
        shoe: [hex(0x1a1218), hex(0x30222a), hex(0x4a3430), hex(0x664a3e), hex(0x846250)],
        stocking: [hex(0x1e1828), hex(0x2c2638), hex(0x3e384c), hex(0x524c62), hex(0x6a647a)],
        navy: [hex(0x141830), hex(0x1e2848), hex(0x2c3a66), hex(0x3e5088), hex(0x5a6aa4)],
    }
}

// ---------------------------------------------------------------- drawing

pub fn frame(look: &Look, dir: u8, frame_idx: u16) -> Img {
    let mut img = Img::new(FW, FH);
    let pose = pose_for(frame_idx);
    let b = build(look, pose);
    let l = limbs(&b, pose, dir);
    let c = colors(look);
    let mut cv = Canvas { img: &mut img };
    let side = dir == DIR_SIDE;
    let back = dir == DIR_UP;

    tail(&mut cv, look, dir, &b, &c);
    if look.outfit == outfit::CLOAK && !back {
        cloak_back(&mut cv, &b, &c, side);
    }
    back_hair(&mut cv, look, dir, &b, &c);

    let left = (l.hip_l, l.knee_l, l.ankle_l);
    let right = (l.hip_r, l.knee_r, l.ankle_r);
    if side {
        // Far limbs first, in shadow.
        arm(&mut cv, look, &b, &c, (l.sh_l, l.el_l, l.wr_l), true, dir, pose);
        leg(&mut cv, look, &b, &c, left, true, dir);
        leg(&mut cv, look, &b, &c, right, false, dir);
    } else {
        leg(&mut cv, look, &b, &c, left, false, dir);
        leg(&mut cv, look, &b, &c, right, false, dir);
    }
    torso(&mut cv, look, dir, &b, &c);
    if side {
        arm(&mut cv, look, &b, &c, (l.sh_r, l.el_r, l.wr_r), false, dir, pose);
    } else {
        arm(&mut cv, look, &b, &c, (l.sh_l, l.el_l, l.wr_l), false, dir, pose);
        arm(&mut cv, look, &b, &c, (l.sh_r, l.el_r, l.wr_r), false, dir, pose);
    }
    head(&mut cv, look, dir, &b, &c);
    if !back {
        face(&mut cv, look, dir, &b, &c, pose);
    }
    hair_front(&mut cv, look, dir, &b, &c);
    ears_horns(&mut cv, look, dir, &b, &c);
    hat(&mut cv, look, dir, &b, &c);
    drop(cv);
    outline(&mut img);
    img
}

/// Coloured selective outline: each edge pixel takes a deep, hue-matched
/// shade of the neighbour it borders (lighter on the lit side).
fn outline(img: &mut Img) {
    let src = img.clone();
    for y in 0..img.h {
        for x in 0..img.w {
            if src.alpha(x, y) != 0 {
                continue;
            }
            let mut found: Option<(Rgb, bool)> = None;
            for (dx, dy, lit) in [(0, 1, true), (1, 0, true), (0, -1, false), (-1, 0, false)] {
                if let Some(c) = src.get(x + dx, y + dy) {
                    found = Some((c, lit));
                    break;
                }
            }
            if let Some((c, lit)) = found {
                img.set(x, y, mix(c, pal::INK, if lit { 0.58 } else { 0.74 }));
            }
        }
    }
}

fn pants_ramp(look: &Look, c: &Colors) -> R5 {
    match look.outfit {
        outfit::ARMOR => c.metal,
        outfit::DIRNDL | outfit::GUILD => c.stocking,
        outfit::DRESS | outfit::ROBE => c.skin,
        _ => c.bottom,
    }
}

fn leg(cv: &mut Canvas, look: &Look, b: &Build, c: &Colors, (hip, knee, ankle): (V2, V2, V2), far: bool, dir: u8) {
    let r = b.limb_r;
    let ramp = pants_ramp(look, c);
    let bias = if far { -0.18 } else { 0.0 };
    let thick = if look.female && look.mature && look.age != 0 { 0.9 } else { 0.0 };
    cv.capsule(hip, knee, r + 1.3 + thick, r + 0.2, &ramp, bias);
    cv.capsule(knee, ankle, r + 0.2, r - 0.7, &ramp, bias);
    let fwd = if dir == DIR_SIDE { 2.2 } else { 0.0 };
    let boots = matches!(look.outfit, outfit::ARMOR | outfit::CLOAK | outfit::SMITH);
    if boots {
        cv.capsule(ankle.add(v(0.0, -5.0)), ankle, r + 0.3, r + 0.3, &c.leather, bias);
    }
    let foot = ankle.add(v(fwd, 1.4));
    let shoe = if boots { c.leather } else { c.shoe };
    if dir == DIR_SIDE {
        cv.ellipse(foot, 3.6, 1.9, &shoe, bias);
    } else {
        cv.ellipse(foot, 2.6, 2.0, &shoe, bias);
    }
}

#[allow(clippy::too_many_arguments)]
fn arm(cv: &mut Canvas, look: &Look, b: &Build, c: &Colors, (sh, el, wr): (V2, V2, V2), far: bool, dir: u8, pose: Pose) {
    let r = b.limb_r * 0.85;
    let sleeve = match look.outfit {
        outfit::ARMOR => c.metal,
        outfit::DIRNDL | outfit::GUILD => c.linen,
        outfit::SMITH => c.skin,
        _ => c.top,
    };
    let bias = if far { -0.2 } else { 0.0 };
    let long_sleeve = !matches!(look.outfit, outfit::SMITH | outfit::DIRNDL);
    cv.capsule(sh, el, r + 0.7, r + 0.2, &sleeve, bias);
    if long_sleeve {
        cv.capsule(el, wr, r + 0.2, r, &sleeve, bias);
    } else {
        cv.capsule(el, wr, r + 0.1, r - 0.3, &c.skin, bias);
    }
    if matches!(look.outfit, outfit::DIRNDL | outfit::GUILD) {
        cv.ellipse(sh.add(v(0.0, 2.0)), r + 2.0, r + 1.6, &c.linen, bias);
    }
    if look.outfit == outfit::ARMOR {
        cv.ellipse(sh.add(v(0.0, 0.5)), r + 2.4, r + 1.6, &c.metal, bias + 0.05);
    }
    cv.ellipse(wr.add(v(0.0, 1.6)), r + 0.3, r + 0.6, &c.skin, bias);
    let holding = !far && (dir == DIR_SIDE || wr.x > b.cx);
    if holding && look.held != held::NONE {
        held_item(cv, look, wr.add(v(0.0, 1.6)), dir, pose.work == 1, c);
    }
}

fn torso(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
    let side = dir == DIR_SIDE;
    let back = dir == DIR_UP;
    let shirt = match look.outfit {
        outfit::ARMOR => c.metal,
        outfit::DIRNDL | outfit::GUILD => c.linen,
        _ => c.top,
    };
    let (sy, cy) = (b.shoulder_y, b.crotch_y);
    let keys: Vec<(f32, f32)> = if side {
        vec![(sy, 4.0), (sy + 2.5, 5.2), (b.waist_y, 4.4), (b.hip_y, 5.4), (cy + 1.0, 5.0)]
    } else {
        vec![(sy, b.shoulder_w - 2.4), (sy + 2.5, b.shoulder_w), (sy + 6.0, b.chest_w), (b.waist_y, b.waist_w), (b.hip_y, b.hip_w), (cy + 1.0, b.hip_w - 0.6)]
    };
    let waist_y = b.waist_y;
    let cx0 = b.cx;
    let lean = move |y: f32| -> f32 {
        if side {
            cx0 + 0.5 + if y < waist_y { 0.6 } else { -0.5 }
        } else {
            cx0
        }
    };
    let width = |y: f32| keyed(&keys, y);
    cv.capsule(v(b.cx, sy - 4.0), v(b.cx, sy + 1.0), 2.4, 2.6, &c.skin, -0.1);
    let lower = match look.outfit {
        outfit::DRESS | outfit::ROBE => c.skin,
        outfit::DIRNDL | outfit::GUILD => c.stocking,
        _ => pants_ramp(look, c),
    };
    cv.profile(b.waist_y, cy + 1.5, lean, width, &lower, 0.0);
    cv.profile(sy, b.waist_y + 3.5, lean, width, &shirt, 0.0);
    if b.bust > 0.0 && look.outfit != outfit::ARMOR {
        let by = sy + 7.0 + b.lag * 1.1;
        if side {
            cv.ellipse(v(b.cx + 4.8, by), b.bust + 0.5, b.bust + 0.3, &shirt, 0.06);
        } else if !back {
            for dx in [-3.6, 3.6] {
                cv.ellipse(v(b.cx + dx, by), b.bust + 0.7, b.bust + 0.2, &shirt, 0.02);
            }
            // Cleft shadow between.
            for y in (by - 1.0) as i32..(by + b.bust) as i32 {
                cv.put(b.cx as i32 - 1, y, shirt[1]);
            }
        }
    }
    let cx = b.cx;
    match look.outfit {
        outfit::DRESS => {
            let top = b.waist_y + 1.0;
            let len = b.thigh + 4.0;
            let sway = b.lag;
            cv.profile(top, top + len, |y| cx + (y - top) / len * sway * 0.8, |y| b.waist_w + 1.0 + (y - top) * 0.42, &c.top, -0.02);
            hem_folds(cv, top, top + len, cx, b.waist_w + 1.0, 0.42, &c.top, sway);
        }
        outfit::DIRNDL | outfit::GUILD => {
            let bod = if look.outfit == outfit::DIRNDL { c.top } else { c.navy };
            cv.profile(sy + 4.5, b.waist_y + 3.0, lean, |y| width(y) - 0.4, &bod, 0.0);
            if b.bust > 0.0 && !back {
                let by = sy + 7.0 + b.lag * 1.1;
                if side {
                    cv.ellipse(v(b.cx + 4.8, by - 0.8), b.bust + 0.2, b.bust - 0.5, &c.linen, 0.1);
                } else {
                    for dx in [-3.6, 3.6] {
                        cv.ellipse(v(b.cx + dx, by - 1.0), b.bust + 0.4, b.bust - 0.6, &c.linen, 0.08);
                    }
                    for y in (by - 2.0) as i32..(by + b.bust - 1.0) as i32 {
                        cv.put(b.cx as i32 - 1, y, c.linen[1]);
                    }
                }
            }
            let skirt = if look.outfit == outfit::DIRNDL { c.bottom } else { c.navy };
            let top = b.waist_y + 2.0;
            let len = b.thigh + 2.0;
            let sway = b.lag;
            cv.profile(top, top + len, |y| cx + (y - top) / len * sway * 0.8, |y| b.waist_w + 1.6 + (y - top) * 0.5, &skirt, 0.0);
            hem_folds(cv, top, top + len, cx, b.waist_w + 1.6, 0.5, &skirt, sway);
            if !back {
                if look.outfit == outfit::DIRNDL {
                    let aw = if side { 2.4 } else { 4.0 };
                    let ax = if side { cx + 3.0 } else { cx };
                    cv.profile(top - 0.5, top + len - 2.0, |_| ax, |y| aw + (y - top) * 0.12, &c.linen, 0.05);
                    if !side {
                        for k in 0..3 {
                            cv.put(cx as i32 - 1 + k % 2, (sy + 8.0) as i32 + k * 2, hex(0xfff2dc));
                        }
                    }
                } else {
                    let red = [hex(0x5a1420), hex(0x8a2232), hex(0xb83a4a), hex(0xe0606e), hex(0xf8909a)];
                    let ty = sy + 2.0;
                    if side {
                        cv.ellipse(v(cx + 4.5, ty + 1.0), 1.6, 1.4, &red, 0.0);
                    } else {
                        cv.ellipse(v(cx - 2.2, ty + 0.8), 2.0, 1.5, &red, 0.0);
                        cv.ellipse(v(cx + 2.2, ty + 0.8), 2.0, 1.5, &red, -0.05);
                        cv.ellipse(v(cx, ty + 1.0), 1.0, 1.0, &red, -0.1);
                        cv.capsule(v(cx - 0.8, ty + 2.0), v(cx - 1.6, ty + 5.5), 0.8, 0.7, &red, 0.0);
                        cv.capsule(v(cx + 0.8, ty + 2.0), v(cx + 1.6, ty + 5.5), 0.8, 0.7, &red, -0.05);
                        for k in 0..3 {
                            cv.put(cx as i32, (sy + 10.0) as i32 + k * 3, hex(pal::GOLD[3]));
                        }
                        cv.ellipse(v(cx + b.chest_w - 2.5, sy + 9.0), 1.5, 1.5, &gold5(), 0.0);
                    }
                }
            }
        }
        outfit::ROBE => {
            let top = b.waist_y - 2.0;
            let bottom = b.ground - 1.0;
            let len = bottom - top;
            let sway = b.lag;
            cv.profile(sy, top + 2.0, lean, |y| width(y) + 0.6, &c.top, 0.0);
            cv.profile(top, bottom, |y| cx + (y - top) / len * sway * 0.7, |y| b.waist_w + 1.4 + (y - top) * 0.22, &c.top, 0.0);
            hem_folds(cv, top, bottom, cx, b.waist_w + 1.4, 0.22, &c.top, sway);
            if !side && !back {
                let gold = gold5();
                for y in (sy + 1.0) as i32..bottom as i32 {
                    cv.put(cx as i32, y, gold[1]);
                }
                for x in (cx - b.waist_w) as i32..=(cx + b.waist_w) as i32 {
                    cv.put(x, b.waist_y as i32, gold[2]);
                }
                cv.ellipse(v(cx, b.waist_y), 1.2, 1.2, &gold, 0.1);
                cv.profile(sy - 0.5, sy + 6.0, |_| cx, |y| b.shoulder_w + 1.6 - (y - sy) * 0.1, &c.top, 0.08);
            }
        }
        outfit::APRON | outfit::SMITH if !back => {
            let ap = if look.outfit == outfit::SMITH { c.leather } else { c.linen };
            let (ax, aw) = if side { (cx + 3.2, 2.2) } else { (cx, b.chest_w - 2.4) };
            cv.profile(sy + 4.0, b.hip_y + 6.0, |_| ax, |_| aw, &ap, 0.06);
            if !side {
                for k in 0..4 {
                    cv.put((cx - aw + 1.0) as i32, (sy + k as f32) as i32, ap[2]);
                    cv.put((cx + aw - 1.0) as i32, (sy + k as f32) as i32, ap[1]);
                }
                cv.rect((cx - 2.0) as i32, (b.waist_y + 3.0) as i32, 4, 3, ap[1]);
                cv.rect((cx - 2.0) as i32, (b.waist_y + 3.0) as i32, 4, 1, ap[0]);
            }
        }
        outfit::ARMOR => {
            if !back && !side {
                cv.profile(sy + 4.0, b.hip_y + 5.0, |_| cx, |_| 3.2, &c.hatc, 0.0);
                cv.ellipse(v(cx, sy + 8.0), 1.4, 1.4, &gold5(), 0.0);
            }
            let y = b.waist_y + 1.0;
            for x in (cx - b.waist_w - 1.0) as i32..=(cx + b.waist_w + 1.0) as i32 {
                cv.put(x, y as i32, c.leather[1]);
                cv.put(x, y as i32 + 1, c.leather[0]);
            }
        }
        outfit::VEST if !back => {
            let w = b.chest_w * 0.45;
            if side {
                cv.profile(sy + 0.5, b.hip_y, |_| cx + 1.5, |_| 3.2, &c.leather, 0.0);
            } else {
                cv.profile(sy + 0.5, b.hip_y, |_| cx - b.chest_w + w, |_| w, &c.leather, 0.0);
                cv.profile(sy + 0.5, b.hip_y, |_| cx + b.chest_w - w, |_| w, &c.leather, 0.0);
            }
        }
        _ => {}
    }
    if matches!(look.outfit, outfit::TUNIC | outfit::VEST | outfit::APRON | outfit::CLOAK | outfit::SMITH) {
        let y = b.waist_y + 2.5;
        let w = width(y) + 0.3;
        let x0 = (lean(y) - w).round() as i32;
        let x1 = (lean(y) + w).round() as i32;
        for x in x0..=x1 {
            cv.put(x, y as i32, c.leather[2]);
            cv.put(x, y as i32 + 1, c.leather[0]);
        }
        if dir == DIR_DOWN {
            cv.rect(cx as i32 - 1, y as i32, 2, 2, hex(pal::GOLD[3]));
        }
    }
    if look.outfit == outfit::CLOAK && back {
        cloak_back(cv, b, c, false);
    }
    if dir == DIR_DOWN && !matches!(look.outfit, outfit::ARMOR | outfit::ROBE | outfit::GUILD) {
        cv.rect(cx as i32 - 1, sy as i32, 2, 1, c.skin[2]);
        cv.rect(cx as i32 - 1, sy as i32 + 1, 2, 1, c.skin[1]);
    }
    if look.beard && !back && look.race == RACE_DWARF {
        let (bx, w) = if side { (cx + 5.0, 3.6) } else { (cx, 6.0) };
        cv.profile(sy - 3.0, sy + 9.0, |_| bx, |y| w * (1.0 - ((y - (sy - 3.0)) / 12.0).powf(2.0) * 0.7), &c.hair, 0.05);
    }
}

#[allow(clippy::too_many_arguments)]
fn hem_folds(cv: &mut Canvas, top: f32, bottom: f32, cx: f32, w0: f32, k: f32, ramp: &R5, sway: f32) {
    let len = bottom - top;
    for i in [-2.0f32, -0.7, 0.7, 2.0] {
        for y in (top + len * 0.35) as i32..bottom as i32 {
            let t = (y as f32 - top) / len;
            let w = w0 + (y as f32 - top) * k;
            let x = cx + t * sway * 0.8 + i / 2.6 * w;
            cv.put(x.round() as i32, y, ramp[1]);
        }
    }
}

fn cloak_back(cv: &mut Canvas, b: &Build, c: &Colors, side: bool) {
    let top = b.shoulder_y - 0.5;
    let bottom = b.ground - 6.0;
    let cx = if side { b.cx - 3.0 } else { b.cx };
    let lag = b.lag;
    let sw = b.shoulder_w;
    cv.profile(top, bottom, |y| cx - (y - top) / (bottom - top) * lag * 1.5, |y| if side { 4.5 + (y - top) * 0.12 } else { sw + 1.5 + (y - top) * 0.18 }, &c.hatc, -0.1);
}

fn held_item(cv: &mut Canvas, look: &Look, hand: V2, dir: u8, raised: bool, c: &Colors) {
    let wood = [hex(0x2e1c1c), hex(0x4a2e24), hex(0x6b412f), hex(0x8c5a38), hex(0xab7645)];
    let (x, y) = (hand.x, hand.y);
    match look.held {
        held::STAFF => {
            let top = if raised { y - 18.0 } else { y - 34.0 };
            cv.capsule(v(x, top), v(x, y + 12.0), 1.1, 1.1, &wood, 0.0);
            let gem = [hex(0x2a1050), hex(0x4a2490), hex(0x7a48d0), hex(0xb090f8), hex(0xf0e6ff)];
            cv.capsule(v(x - 2.5, top + 1.0), v(x, top - 2.0), 1.0, 0.9, &wood, 0.0);
            cv.capsule(v(x + 2.5, top + 1.0), v(x, top - 2.0), 1.0, 0.9, &wood, 0.0);
            cv.ellipse(v(x, top - 3.0), 2.6, 2.8, &gem, 0.1);
        }
        held::HAMMER => {
            let (hx, hy) = if raised { (x, y - 12.0) } else { (x + 0.5, y + 1.0) };
            cv.capsule(v(hx, hy), v(hx, hy + 11.0), 1.0, 1.0, &wood, 0.0);
            cv.capsule(v(hx - 3.5, hy - 1.0), v(hx + 3.5, hy - 1.0), 2.2, 2.2, &c.metal, 0.05);
        }
        held::MUG if dir != DIR_UP => {
            let mug = [hex(0x5a3418), hex(0x8a5a2a), hex(0xb07a40), hex(0xd8a060), hex(0xf0c080)];
            cv.capsule(v(x, y - 2.0), v(x, y + 3.0), 2.6, 2.6, &mug, 0.0);
            cv.ellipse(v(x, y - 3.0), 2.8, 1.6, &[hex(0xb0a490), hex(0xc8bca8), hex(0xe8dcc8), hex(0xfff6e0), hex(0xffffff)], 0.1);
            cv.capsule(v(x + 3.0, y - 1.0), v(x + 3.0, y + 2.0), 0.8, 0.8, &mug, -0.1);
        }
        held::BOOK if dir != DIR_UP => {
            let cover = [hex(0x3a1420), hex(0x5a2030), hex(0x7a2e40), hex(0x9a4458), hex(0xba6070)];
            cv.capsule(v(x - 3.0, y), v(x + 3.0, y), 2.6, 2.6, &cover, 0.0);
            cv.rect((x - 2.0) as i32, (y - 2.0) as i32, 5, 3, hex(0xf0e6d0));
            cv.put(x as i32, (y - 1.0) as i32, hex(0xc8bca8));
        }
        held::SWORD => {
            let (top, bot) = if raised { (y - 18.0, y + 1.0) } else { (y + 1.0, y + 18.0) };
            cv.capsule(v(x, top), v(x, bot), 1.0, 0.8, &c.metal, 0.1);
            let gy = if raised { bot } else { top + 1.0 };
            cv.capsule(v(x - 3.0, gy), v(x + 3.0, gy), 0.9, 0.9, &gold5(), 0.0);
        }
        held::BASKET if dir != DIR_UP => {
            let wick = [hex(0x5a3a1c), hex(0x8a6a3a), hex(0xa8844e), hex(0xc8a464), hex(0xe0c080)];
            cv.capsule(v(x - 3.0, y + 2.0), v(x + 3.0, y + 2.0), 2.6, 2.6, &wick, 0.0);
            for (dx, col) in [(-1.5, 0xd24c40u32), (0.5, 0xeec04a), (2.0, 0x7aa86c)] {
                let cc = hex(col);
                cv.ellipse(v(x + dx, y - 0.5), 1.3, 1.2, &[mix(cc, pal::INK, 0.5), mix(cc, pal::INK, 0.25), cc, mix(cc, hex(0xffffff), 0.3), mix(cc, hex(0xffffff), 0.5)], 0.0);
            }
        }
        _ => {}
    }
}

fn head(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
    let (rx, ry) = b.head_r;
    let hc = b.head_c;
    let side = dir == DIR_SIDE;
    cv.ellipse(hc, rx, ry, &c.skin, 0.06);
    let jaw = if side { v(hc.x + 2.2, hc.y + ry * 0.45) } else { v(hc.x, hc.y + ry * 0.45) };
    cv.ellipse(jaw, rx * if side { 0.62 } else { 0.72 }, ry * 0.55, &c.skin, 0.04);
    if side {
        cv.put((hc.x + rx + 0.5) as i32, (hc.y + 2.0) as i32, c.skin[3]);
        cv.put((hc.x + rx + 0.5) as i32, (hc.y + 3.0) as i32, c.skin[2]);
        if look.race != RACE_ELF {
            cv.ellipse(v(hc.x - 1.0, hc.y + 1.5), 1.8, 2.6, &c.skin, -0.05);
        }
    }
}

fn eye(cv: &mut Canvas, x: i32, y: i32, iris: &[u32; 3], right: bool, blink: bool) {
    let ink = pal::INK;
    let mirror = |xx: i32| if right { 5 - xx } else { xx };
    if blink {
        // Closed: a soft downward curve with a lash tip.
        for (xx, yy) in [(0, 4), (1, 5), (2, 5), (3, 5), (4, 5), (5, 4)] {
            cv.put(x + mirror(xx), y + yy, ink);
        }
        cv.put(x + mirror(-1), y + 3, ink);
        return;
    }
    let (d, m, l) = (Some(hex(iris[0])), Some(hex(iris[1])), Some(hex(iris[2])));
    let k = Some(ink);
    let w = Some(hex(0xffffff));
    let s = Some(hex(0xf2eef4));
    let lo = Some(hex(0x6a4a56));
    // 6x8 anime eye: heavy upper lash, sclera on the inner side, three-tone
    // iris and two catchlights.
    let rows: [[Option<Rgb>; 6]; 8] = [
        [None, None, k, k, k, None],
        [None, k, k, k, k, k],
        [k, k, d, d, d, k],
        [s, d, w, w, d, d],
        [s, d, w, m, m, d],
        [s, m, m, m, l, m],
        [None, m, l, l, w, m],
        [None, None, lo, lo, lo, None],
    ];
    for (yy, row) in rows.iter().enumerate() {
        for (xx, cc) in row.iter().enumerate() {
            if let Some(cc) = cc {
                cv.put(x + mirror(xx as i32), y + yy as i32, *cc);
            }
        }
    }
    // Lash flicks at the outer corner.
    cv.put(x + mirror(6), y + 1, ink);
    cv.put(x + mirror(6), y, ink);
    cv.put(x + mirror(7), y - 1, ink);
}

fn face(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors, pose: Pose) {
    let iris = if look.race == RACE_DEMON && look.eyes < 4 { &EYE_COLORS[4 + (look.eyes % 2) as usize] } else { &EYE_COLORS[look.eyes as usize % EYE_COLORS.len()] };
    let hc = b.head_c;
    let (cx, cy) = (hc.x.round() as i32, hc.y.round() as i32);
    let blush = mix(c.skin[2], hex(0xf07a88), 0.55);
    let brow = mix(c.hair[0], pal::INK, 0.2);
    let lip = hex(0xb04a58);
    if dir == DIR_DOWN {
        let ey = cy - 1;
        eye(cv, cx - 10, ey, iris, false, pose.blink);
        eye(cv, cx + 4, ey, iris, true, pose.blink);
        // Arched brows.
        for (xx, yy) in [(0, 1), (1, 0), (2, 0), (3, 0), (4, 1)] {
            cv.put(cx - 10 + xx, ey - 3 + yy, brow);
            cv.put(cx + 9 - xx, ey - 3 + yy, brow);
        }
        // Nose: a single soft shadow pixel and highlight.
        cv.put(cx, ey + 9, c.skin[1]);
        cv.put(cx - 1, ey + 8, c.skin[4]);
        // Blush hatching.
        for k in 0..4 {
            cv.put(cx - 11 + k, ey + 9 + (k % 2), blush);
            cv.put(cx + 7 + k, ey + 9 + (k % 2), blush);
        }
        // Mouth: small smile with a lip tint.
        cv.put(cx - 2, ey + 11, mix(c.skin[1], hex(0x7a3440), 0.5));
        cv.put(cx - 1, ey + 12, hex(0x8a3a48));
        cv.put(cx, ey + 12, hex(0x8a3a48));
        cv.put(cx + 1, ey + 11, mix(c.skin[1], hex(0x7a3440), 0.5));
        let _ = lip;
        if look.glasses {
            let g = hex(0x4a3a30);
            for gx in [cx - 11, cx + 3] {
                for k in 0..9 {
                    cv.put(gx + k, ey - 1, g);
                    cv.put(gx + k, ey + 9, g);
                }
                for k in 0..11 {
                    cv.put(gx, ey - 1 + k, g);
                    cv.put(gx + 8, ey - 1 + k, g);
                }
            }
            for k in -2..=2 {
                cv.put(cx + k, ey + 2, g);
            }
        }
        if look.beard && look.race != RACE_DWARF {
            cv.ellipse(v(hc.x, hc.y + 10.5), 7.0, 3.5, &c.hair, 0.0);
            cv.put(cx - 1, ey + 12, hex(0x6a3434));
            cv.put(cx, ey + 12, hex(0x6a3434));
        }
        if look.beard && look.race == RACE_DWARF {
            cv.ellipse(v(hc.x - 3.0, hc.y + 9.0), 3.6, 1.8, &c.hair, 0.1);
            cv.ellipse(v(hc.x + 3.0, hc.y + 9.0), 3.6, 1.8, &c.hair, 0.0);
        }
        if look.age == 2 {
            cv.put(cx - 11, ey + 8, c.skin[1]);
            cv.put(cx + 10, ey + 8, c.skin[1]);
        }
    } else if dir == DIR_SIDE {
        let ex = cx + 5;
        let ey = cy - 1;
        if pose.blink {
            for k in 0..5 {
                cv.put(ex + k, ey + 5, pal::INK);
            }
        } else {
            for k in 0..5 {
                cv.put(ex + k, ey, pal::INK);
                cv.put(ex + k, ey + 1, pal::INK);
            }
            cv.put(ex + 5, ey, pal::INK);
            cv.put(ex + 5, ey - 1, pal::INK);
            for (xx, yy, col) in [
                (0, 2, 0xffffffu32), (1, 2, iris[0]), (2, 2, iris[0]), (3, 2, iris[0]),
                (0, 3, 0xffffff), (1, 3, 0xffffff), (2, 3, iris[1]), (3, 3, iris[0]),
                (0, 4, 0xf2eef4), (1, 4, iris[1]), (2, 4, iris[1]), (3, 4, iris[1]),
                (1, 5, iris[2]), (2, 5, 0xffffff), (3, 5, iris[2]),
                (1, 6, 0x6a4a56), (2, 6, 0x6a4a56),
            ] {
                cv.put(ex + xx, ey + yy, hex(col));
            }
        }
        for k in 0..4 {
            cv.put(ex + k, ey - 3 + (k == 0) as i32, brow);
        }
        for k in 0..3 {
            cv.put(ex - 2 + k, ey + 9 + (k % 2), blush);
        }
        cv.put(cx + 9, ey + 11, hex(0x7a3440));
        cv.put(cx + 9, ey + 12, lip);
        if look.beard {
            cv.ellipse(v(hc.x + 6.0, hc.y + 9.5), 4.5, 3.5, &c.hair, 0.0);
        }
        if look.glasses {
            let g = hex(0x4a3a30);
            for k in 0..7 {
                cv.put(ex - 1 + k, ey - 1, g);
                cv.put(ex - 1 + k, ey + 8, g);
            }
            for k in 0..9 {
                cv.put(ex - 10 + k, ey + 1, g);
            }
        }
    }
}

// ---------------------------------------------------------------- hair

fn back_hair(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
    let h = &c.hair;
    let hc = b.head_c;
    let (rx, _) = b.head_r;
    let side = dir == DIR_SIDE;
    let sway = b.lag;
    if dir == DIR_UP {
        return;
    }
    match look.hair_style {
        1 | 6 => {
            let len = if look.hair_style == 6 { 40.0 } else { 34.0 };
            let top = hc.y;
            let w0 = if side { rx * 0.7 } else { rx + 1.2 };
            let cx = if side { hc.x - 3.5 } else { hc.x };
            cv.profile(
                top,
                top + len,
                |y| cx + ((y - top) / len).powi(2) * sway * 2.0,
                |y| {
                    let t = (y - top) / len;
                    w0 * (1.0 - t * t * 0.55) + if t > 0.85 { -(t - 0.85) * 18.0 } else { 0.0 }
                },
                h,
                -0.22,
            );
        }
        2 if side => {
            let base = v(hc.x - rx + 1.0, hc.y - 2.0);
            let tip = base.add(v(-4.0 - sway * 2.0, 18.0));
            cv.capsule(base, base.add(v(-3.0, 8.0)), 3.2, 3.0, h, -0.05);
            cv.capsule(base.add(v(-3.0, 8.0)), tip, 3.0, 1.0, h, -0.1);
        }
        2 => {
            let base = v(hc.x + rx - 2.0, hc.y + 1.0);
            cv.capsule(base, base.add(v(1.0 + sway, 13.0)), 2.6, 1.2, h, -0.15);
        }
        _ => {}
    }
}

fn hair_front(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
    let h = &c.hair;
    let hc = b.head_c;
    let (rx, ry) = b.head_r;
    let style = look.hair_style;
    let shine = mix(h[4], hex(0xffffff), 0.3);
    let side = dir == DIR_SIDE;
    let sway = b.lag;
    if dir == DIR_UP {
        cv.ellipse(v(hc.x, hc.y - 0.5), rx + 1.3, ry + 1.2, h, 0.0);
        let len = match style {
            1 => 34.0,
            6 => 40.0,
            5 => 12.0,
            2 | 3 => 9.0,
            _ => 6.0,
        };
        if len > 6.0 {
            let top = hc.y;
            cv.profile(
                top,
                top + len,
                |y| hc.x + ((y - top) / len).powi(2) * sway * 2.0,
                |y| {
                    let t = (y - top) / len;
                    (rx + 1.2) * (1.0 - t * t * 0.5) + if t > 0.85 { -(t - 0.85) * 18.0 } else { 0.0 }
                },
                h,
                -0.05,
            );
        }
        for sx in [-5, 0, 5] {
            for y in (hc.y as i32 + 2)..(hc.y + len.max(8.0)) as i32 - 2 {
                if cv.img.alpha(hc.x as i32 + sx, y) != 0 {
                    cv.put(hc.x as i32 + sx, y, h[1]);
                }
            }
        }
        for x in (hc.x - rx * 0.6) as i32..(hc.x + rx * 0.5) as i32 {
            let y = (hc.y - ry * 0.6 + ((x as f32 - hc.x) / rx).powi(2) * 3.0) as i32;
            if x % 4 != 0 {
                cv.put(x, y, shine);
            }
        }
        match style {
            2 => {
                let base = v(hc.x, hc.y + 2.0);
                cv.capsule(base, base.add(v(sway, 18.0)), 3.2, 1.2, h, 0.0);
                cv.rect(hc.x as i32 - 2, hc.y as i32 + 1, 4, 2, hex(0xc83a4a));
            }
            3 => cv.ellipse(v(hc.x, hc.y - ry - 1.0), 4.5, 4.0, h, 0.05),
            6 => {
                for dx in [-1.0, 1.0] {
                    let base = v(hc.x + dx * (rx + 1.0), hc.y - 2.0);
                    cv.capsule(base, base.add(v(dx * 2.0 + sway, 22.0)), 3.0, 1.2, h, -0.05);
                }
            }
            _ => {}
        }
        return;
    }
    // Skull cap: the hair mass over the top of the head.
    let cap_c = v(hc.x - if side { 1.0 } else { 0.0 }, hc.y - 1.6);
    let (cap_rx, cap_ry) = (rx + 1.4, ry + 1.0);
    let hairline = |x: f32| -> f32 {
        let nx = (x - hc.x) / rx;
        hc.y - ry * 0.32 + nx * nx * ry * 0.5
    };
    let (x0, x1) = ((cap_c.x - cap_rx).floor() as i32, (cap_c.x + cap_rx).ceil() as i32);
    for y in (cap_c.y - cap_ry).floor() as i32..=(hc.y + ry * 0.3) as i32 {
        for x in x0..=x1 {
            let nx = (x as f32 + 0.5 - cap_c.x) / cap_rx;
            let ny = (y as f32 + 0.5 - cap_c.y) / cap_ry;
            let d = nx * nx + ny * ny;
            if d > 1.0 {
                continue;
            }
            let limit = if side && x as f32 <= hc.x + 1.0 { hc.y + ry * 0.35 } else { hairline(x as f32) };
            if (y as f32 + 0.5) > limit {
                continue;
            }
            let nz = (1.0 - d).sqrt();
            cv.put(x, y, h[tone(nx, ny, nz, x, y, 0.02)]);
        }
    }
    for x in (cap_c.x - cap_rx * 0.55) as i32..(cap_c.x + cap_rx * 0.35) as i32 {
        let t = (x as f32 - cap_c.x) / cap_rx;
        let y = (cap_c.y - cap_ry * 0.55 + t * t * 4.0) as i32;
        if (x + y) % 5 != 0 {
            cv.put(x, y, shine);
            if t < 0.0 {
                cv.put(x, y + 1, h[4]);
            }
        }
    }
    if side {
        let back_len = match style {
            1 | 6 => 6.0,
            5 => 9.0,
            _ => 5.0,
        };
        cv.ellipse(v(hc.x - 3.5, hc.y + 1.0), rx * 0.62, ry * 0.75 + back_len * 0.4, h, -0.04);
        for (k, dx) in [3.0f32, 5.5, 8.0].iter().enumerate() {
            let base = v(hc.x + dx - 2.0, hc.y - ry * 0.7);
            cv.capsule(base, base.add(v(2.0 + k as f32, 7.5 - k as f32 * 1.2)), 2.4, 0.6, h, 0.0);
        }
        if style == 3 {
            cv.ellipse(v(hc.x - 3.0, hc.y - ry - 1.0), 4.5, 4.0, h, 0.05);
        }
        if style == 4 {
            for k in 0..4 {
                let base = v(hc.x - 6.0 + k as f32 * 3.5, hc.y - ry);
                cv.capsule(base, base.add(v(-2.0, -4.0)), 2.0, 0.5, h, 0.08);
            }
        }
        if matches!(style, 1 | 5 | 6 | 7) {
            cv.capsule(v(hc.x + 1.0, hc.y - 2.0), v(hc.x + 0.5 + sway * 0.5, hc.y + ry * 0.9), 2.4, 1.0, h, -0.06);
        }
        return;
    }
    // Front bangs: pointed locks hanging from the hairline.
    let locks: &[(f32, f32, f32)] = match style {
        5 => &[(-7.5, 10.0, 0.0), (-4.5, 10.0, 0.0), (-1.5, 10.0, 0.0), (1.5, 10.0, 0.0), (4.5, 10.0, 0.0), (7.5, 10.0, 0.0)],
        4 => &[(-7.5, 8.0, -1.5), (-4.0, 9.5, -0.5), (-0.5, 7.0, 0.5), (3.0, 9.5, 1.0), (6.5, 8.0, 1.5)],
        7 => &[(-7.0, 12.0, 1.5), (-3.5, 11.0, 2.0), (0.0, 10.0, 2.2), (3.5, 8.0, 2.2), (7.0, 6.0, 2.0)],
        3 => &[(-6.5, 7.5, -0.5), (-2.5, 8.5, 0.0), (1.5, 8.0, 0.0), (5.5, 7.5, 0.5)],
        6 => &[(-7.0, 10.5, 0.0), (-3.5, 9.0, -0.5), (0.0, 7.5, 0.0), (3.5, 9.0, 0.5), (7.0, 10.5, 0.0)],
        _ => &[(-7.5, 10.5, -0.5), (-4.0, 9.0, 0.5), (-0.5, 10.0, -0.3), (3.0, 8.5, 0.6), (6.5, 10.0, 0.3)],
    };
    let top_y = hc.y - ry * 0.8;
    let k = rx / 10.5;
    // Bangs stop at the brows so the eyes stay clear.
    let kl = k * 0.78;
    for &(dx, len, lean) in locks {
        let a = v(hc.x + dx * k, top_y);
        let bb = v(hc.x + dx * k + lean, top_y + len * kl);
        cv.lock(a, bb, 2.9, 0.5, h, 0.06);
    }
    // Fine locks between the main ones, and strand lines down each lock.
    for w in locks.windows(2) {
        let (dx, len, lean) = ((w[0].0 + w[1].0) * 0.5, (w[0].1 + w[1].1) * 0.4, (w[0].2 + w[1].2) * 0.5);
        let a = v(hc.x + dx * k, top_y + 1.0);
        cv.lock(a, a.add(v(lean, len * kl)), 1.6, 0.4, h, 0.0);
    }
    // A soft strand line down the longer locks only.
    let strand = mix(h[1], h[2], 0.5);
    for (i, &(dx, len, lean)) in locks.iter().enumerate() {
        if i % 2 == 1 || len < 9.0 {
            continue;
        }
        let l = len * kl;
        for t in 3..l as i32 - 2 {
            let tt = t as f32 / l;
            let x = hc.x + dx * k + lean * tt + 0.8;
            cv.put(x.round() as i32, (top_y + t as f32) as i32, strand);
        }
    }
    // Soft shadow on the forehead just under the bangs.
    for x in (hc.x - rx) as i32..=(hc.x + rx) as i32 {
        for y in (hc.y - ry * 0.4) as i32..(hc.y + 3.0) as i32 {
            let here = cv.img.get(x, y);
            if here == Some(c.skin[2]) || here == Some(c.skin[3]) || here == Some(c.skin[4]) {
                if cv.img.get(x, y - 1).map_or(false, |p| h.contains(&p)) {
                    cv.put(x, y, c.skin[1]);
                }
            }
        }
    }
    let lock_len = match style {
        1 | 6 => 22.0,
        5 => 15.0,
        2 | 3 | 7 => 13.0,
        _ => 11.0,
    };
    for s in [-1.0f32, 1.0] {
        let a = v(hc.x + s * (rx - 0.5), hc.y - ry * 0.4);
        let bb = v(hc.x + s * (rx + 0.5) + sway * 0.4, hc.y - ry * 0.4 + lock_len);
        cv.lock(a, bb, 2.6, 0.8, h, if s > 0.0 { -0.1 } else { 0.04 });
    }
    match style {
        3 => cv.ellipse(v(hc.x + 1.0, hc.y - ry - 1.5), 4.5, 4.0, h, 0.05),
        4 => {
            for k in 0..5 {
                let base = v(hc.x - 8.0 + k as f32 * 4.0, hc.y - ry + 0.5);
                cv.capsule(base, base.add(v(if k < 2 { -2.0 } else { 2.0 }, -4.0)), 2.0, 0.5, h, 0.08);
            }
        }
        6 => {
            for s in [-1.0f32, 1.0] {
                let base = v(hc.x + s * (rx + 1.5), hc.y - 3.0);
                cv.capsule(base, base.add(v(s * 3.0 + sway, 22.0)), 3.2, 1.2, h, if s > 0.0 { -0.1 } else { 0.0 });
                cv.ellipse(base.add(v(0.0, -1.0)), 2.0, 1.6, &r5(&ramp4(&pal::CLOTH[1])), 0.1);
            }
        }
        _ => {}
    }
}

fn ears_horns(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
    let hc = b.head_c;
    let (rx, ry) = b.head_r;
    match look.race {
        RACE_ELF => {
            let sides: &[f32] = if dir == DIR_SIDE { &[-0.2] } else { &[-1.0, 1.0] };
            for &s in sides {
                let base = if dir == DIR_SIDE { v(hc.x - 1.0, hc.y + 1.5) } else { v(hc.x + s * (rx - 0.5), hc.y + 1.5) };
                let tip = base.add(v(if dir == DIR_SIDE { -6.0 } else { s * 6.5 }, -5.0));
                cv.capsule(base, tip, 1.8, 0.5, &c.skin, if s > 0.0 { -0.1 } else { 0.05 });
            }
        }
        RACE_BEAST => {
            let inner = [hex(0xa8606a), hex(0xd08890), hex(0xf0a8b0), hex(0xf8c8cc), hex(0xffe0e4)];
            let sides: &[f32] = if dir == DIR_SIDE { &[0.0] } else { &[-1.0, 1.0] };
            for &s in sides {
                let base = v(hc.x + s * rx * 0.55 - if dir == DIR_SIDE { 2.0 } else { 0.0 }, hc.y - ry * 0.85);
                let (len, w, tilt) = match look.beast {
                    3 => (12.0, 2.4, 0.5),
                    1 => (8.0, 3.6, 1.6),
                    2 => (7.0, 3.0, 1.8),
                    _ => (6.0, 3.0, 1.2),
                };
                let tip = base.add(v(s * tilt, -len));
                cv.capsule(base, tip, w, 0.6, &c.hair, if s > 0.0 { -0.08 } else { 0.06 });
                if dir == DIR_DOWN {
                    cv.capsule(base.add(v(0.0, -1.0)), tip.add(v(-s * 0.3, 2.5)), w * 0.45, 0.3, &inner, 0.0);
                }
                if look.beast == 1 {
                    cv.put(tip.x.round() as i32, tip.y.round() as i32, hex(0xf8f0e8));
                }
            }
        }
        RACE_DEMON => {
            let horn = [hex(0x1e1622), hex(0x3a2e40), hex(0x5a4a62), hex(0x857590), hex(0xb0a0ba)];
            let sides: &[f32] = if dir == DIR_SIDE { &[1.0] } else { &[-1.0, 1.0] };
            for &s in sides {
                let base = v(hc.x + s * rx * 0.55, hc.y - ry * 0.75);
                let mid = base.add(v(s * 3.0, -3.0));
                let tip = mid.add(v(s * 1.5, -4.5));
                cv.capsule(base, mid, 2.2, 1.6, &horn, 0.0);
                cv.capsule(mid, tip, 1.6, 0.5, &horn, 0.05);
            }
        }
        _ => {}
    }
}

fn tail(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
    let sway = b.lag;
    let base = v(b.cx + if dir == DIR_SIDE { -4.5 } else { 0.0 }, b.hip_y - 1.0);
    match look.race {
        RACE_BEAST => {
            if dir == DIR_DOWN && look.beast == 3 {
                return;
            }
            let dirx = if dir == DIR_SIDE { -1.0 } else { 0.6 };
            let white = [hex(0xc8bcb0), hex(0xe0d8ce), hex(0xf0ece6), hex(0xfaf8f4), hex(0xffffff)];
            match look.beast {
                3 => cv.ellipse(base.add(v(dirx * 2.0, 1.0)), 3.0, 2.8, &white, 0.0),
                1 => {
                    let mid = base.add(v(dirx * 7.0, 4.0 + sway));
                    let tip = mid.add(v(dirx * 5.0, -5.0 + sway));
                    cv.capsule(base, mid, 2.2, 4.4, &c.hair, -0.05);
                    cv.capsule(mid, tip, 4.4, 2.0, &c.hair, 0.0);
                    cv.ellipse(tip, 2.6, 2.4, &white, 0.0);
                }
                2 => {
                    let tip = base.add(v(dirx * 9.0, 6.0 + sway));
                    cv.capsule(base, tip, 2.0, 3.2, &c.hair, -0.05);
                }
                _ => {
                    let mid = base.add(v(dirx * 6.0, 2.0));
                    let tip = mid.add(v(dirx * 2.0, -8.0 + sway));
                    cv.capsule(base, mid, 1.5, 1.5, &c.hair, -0.05);
                    cv.capsule(mid, tip, 1.5, 1.2, &c.hair, 0.0);
                }
            }
        }
        RACE_DEMON => {
            if dir == DIR_DOWN {
                return;
            }
            let dirx = if dir == DIR_SIDE { -1.0 } else { 0.5 };
            let mid = base.add(v(dirx * 7.0, 6.0 + sway));
            let tip = mid.add(v(dirx * 3.0, -4.0));
            cv.capsule(base, mid, 1.2, 1.0, &c.skin, -0.1);
            cv.capsule(mid, tip, 1.0, 0.8, &c.skin, -0.1);
            let s = [c.skin[0], c.skin[0], c.skin[1], c.skin[2], c.skin[3]];
            cv.ellipse(tip.add(v(dirx, -1.0)), 2.0, 2.4, &s, 0.0);
        }
        _ => {}
    }
}

fn hat(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
    let hc = b.head_c;
    let (rx, ry) = b.head_r;
    let side = dir == DIR_SIDE;
    let top = hc.y - ry;
    let hcol = &c.hatc;
    match look.hat {
        1 => {
            let straw = [hex(0x6a4c1e), hex(0x9a7630), hex(0xc8a24c), hex(0xe6c870), hex(0xf8e4a0)];
            cv.profile(top + 3.0, top + 5.5, |_| hc.x, |_| rx + 6.0, &straw, 0.05);
            cv.ellipse(v(hc.x, top + 1.0), rx * 0.75, 5.0, &straw, 0.08);
            for x in (hc.x - rx * 0.75) as i32..=(hc.x + rx * 0.75) as i32 {
                cv.put(x, (top + 3.0) as i32, hex(0xa83a30));
                cv.put(x, (top + 4.0) as i32, hex(0x8a2a26));
            }
        }
        2 => {
            cv.profile(top + 2.5, top + 6.5, |_| hc.x, |_| rx + 1.4, hcol, 0.04);
            if side || dir == DIR_UP {
                let knot = v(hc.x - rx, top + 5.0);
                cv.capsule(knot, knot.add(v(-2.0, 6.0)), 1.6, 1.0, hcol, -0.05);
                cv.capsule(knot, knot.add(v(-3.5, 4.0)), 1.6, 1.0, hcol, -0.1);
            }
        }
        3 => {
            cv.ellipse(v(hc.x, top + 4.0), rx + 1.6, ry * 0.6, &c.metal, 0.06);
            cv.profile(top + 6.0, top + 8.5, |_| hc.x, |_| rx + 2.0, &c.metal, 0.0);
            if dir == DIR_DOWN {
                cv.capsule(v(hc.x, top + 7.0), v(hc.x, top + 13.0), 1.0, 0.9, &c.metal, 0.0);
            }
            let plume = [hex(0x5a1418), hex(0x8a2228), hex(0xb83a3a), hex(0xe05a50), hex(0xf88a78)];
            cv.capsule(v(hc.x, top - 1.0), v(hc.x - if side { 4.0 } else { 0.0 }, top - 6.0), 1.8, 1.2, &plume, 0.0);
        }
        4 => {
            let (x0, x1) = ((hc.x - rx - 2.5) as i32, (hc.x + rx + 2.5) as i32);
            for y in (top - 2.0) as i32..(hc.y + ry + 4.0) as i32 {
                for x in x0..=x1 {
                    let nx = (x as f32 + 0.5 - hc.x) / (rx + 2.5);
                    let ny = (y as f32 + 0.5 - hc.y) / (ry + 2.6);
                    let d = nx * nx + ny * ny;
                    if d > 1.0 {
                        continue;
                    }
                    let face_open = match dir {
                        DIR_DOWN => ((x as f32 - hc.x) / (rx - 1.5)).powi(2) + ((y as f32 - hc.y - 2.0) / (ry - 1.0)).powi(2) < 1.0 && (y as f32) > hc.y - ry * 0.55,
                        DIR_SIDE => (x as f32) > hc.x - 1.0 && (y as f32) > hc.y - ry * 0.55,
                        _ => false,
                    };
                    if face_open {
                        continue;
                    }
                    let nz = (1.0 - d).sqrt();
                    cv.put(x, y, hcol[tone(nx, ny, nz, x, y, 0.0)]);
                }
            }
        }
        5 => {
            cv.ellipse(v(hc.x, top + 2.5), rx * 0.92, 4.0, hcol, 0.06);
            if dir == DIR_DOWN {
                cv.profile(top + 4.5, top + 6.5, |_| hc.x, |_| rx * 0.8, hcol, -0.1);
            } else if side {
                cv.capsule(v(hc.x + 2.0, top + 5.0), v(hc.x + rx + 3.0, top + 5.0), 1.2, 1.0, hcol, -0.1);
            }
        }
        6 => {
            cv.profile(top + 3.5, top + 5.5, |_| hc.x, |_| rx + 7.0, hcol, 0.0);
            let tip = v(hc.x + if side { -6.0 } else { 4.0 }, top - 15.0);
            cv.capsule(v(hc.x, top + 2.0), v(hc.x + 1.0, top - 6.0), rx * 0.8, rx * 0.5, hcol, 0.04);
            cv.capsule(v(hc.x + 1.0, top - 6.0), tip, rx * 0.5, 0.6, hcol, 0.06);
            cv.profile(top + 1.5, top + 3.5, |_| hc.x, |_| rx * 0.82, &gold5(), 0.0);
            cv.ellipse(v(hc.x, top + 2.5), 1.4, 1.4, &[hex(0x2a1050), hex(0x3a1a6a), hex(0x6a3ac0), hex(0xa070f0), hex(0xe8d8ff)], 0.1);
        }
        7 => {
            let f = pal::FLOWERS[look.hat_color as usize % pal::FLOWERS.len()];
            let pet = [mix(hex(f[0]), pal::INK, 0.3), hex(f[0]), hex(f[1]), hex(f[2]), mix(hex(f[2]), hex(0xffffff), 0.4)];
            let fc = v(hc.x + if side { -3.0 } else { rx * 0.6 }, top + 3.0);
            for (dx, dy) in [(-1.8, 0.0), (1.8, 0.0), (0.0, -1.8), (0.0, 1.8)] {
                cv.ellipse(fc.add(v(dx, dy)), 1.4, 1.4, &pet, 0.0);
            }
            cv.put(fc.x as i32, fc.y as i32, hex(0xf6d860));
        }
        8 => {
            let w = [hex(0x9a9290), hex(0xc8c0bc), hex(0xe8e2de), hex(0xf8f6f4), hex(0xffffff)];
            cv.profile(top + 1.5, top + 5.5, |_| hc.x, |_| rx * 0.82, &w, 0.0);
            cv.ellipse(v(hc.x, top - 4.0), rx * 0.95, 6.5, &w, 0.08);
        }
        9 => {
            let bc = v(hc.x + if side { -4.0 } else { rx * 0.55 }, top + 1.5);
            cv.ellipse(bc.add(v(-2.6, 0.0)), 2.4, 2.0, hcol, 0.05);
            cv.ellipse(bc.add(v(2.6, 0.0)), 2.4, 2.0, hcol, -0.05);
            cv.ellipse(bc, 1.0, 1.0, hcol, -0.2);
            cv.capsule(bc, bc.add(v(-1.5, 4.0)), 0.9, 0.7, hcol, -0.1);
            cv.capsule(bc, bc.add(v(1.5, 4.0)), 0.9, 0.7, hcol, -0.15);
        }
        10 => {
            let w = [hex(0xb8b0ae), hex(0xd8d2d0), hex(0xf0ecea), hex(0xfcfaf8), hex(0xffffff)];
            cv.profile(top + 1.0, top + 3.6, |_| hc.x, |_| rx * 0.85, &w, 0.0);
            for k in 0..7 {
                let x = hc.x - rx * 0.75 + k as f32 * rx * 0.25;
                cv.ellipse(v(x, top + 0.5), 1.4, 1.2, &w, 0.05);
            }
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
