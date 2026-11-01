//! Procedural anime characters, constructed the way pixel artists build
//! hand-drawn sprites at this size (64×112 frames, adults ~87px, about three
//! heads tall):
//!
//! - Front and back views are symmetric about the frame's centre line and lit
//!   from the front, so every curve steps identically on both sides.
//! - The face is a hand-tuned silhouette table (straight cheeks, soft U jaw),
//!   not an ellipse. Eyes are authored stamps with a heavy two-row lash line
//!   and a dark-to-light iris.
//! - Hair is clumped ribbons: pointed bangs from a tip list, strand lines
//!   running from the crown into each notch, a broken highlight ring, and side
//!   locks framing the face.
//! - Every body part draws under its own id. A contour pass then puts a 1px
//!   dark line on the back part wherever a front part overlaps it (arm over
//!   torso, bangs over forehead, thigh against thigh, chin over neck). A
//!   coloured outline follows.
//! - Shading is flat bands from 5-tone ramps, with no dithering noise.
//!
//! Sheet layout: 3 directions (down, up, side-facing-right) × 8 poses (idle,
//! breathe, walk ×4, work ×2), then one blink frame.

use serde::{Deserialize, Serialize};

use super::canvas::Img;
use crate::palette::{self as pal, hex, mix, Rgb};
use crate::rng::Rng;

pub const FW: i32 = 64;
pub const FH: i32 = 112;
pub const FRAMES_PER_DIR: u16 = 8;
pub const SHEET_FRAMES: u16 = FRAMES_PER_DIR * 3 + 1;
pub const PIVOT: (i32, i32) = (32, 108);

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
    [0x1e2a6a, 0x3a64c8, 0x8ab8ff],
    [0x14382a, 0x2c8050, 0x78d098],
    [0x3a220f, 0x7a4c20, 0xc89450],
    [0x341a5a, 0x7a3ab8, 0xc490f0],
    [0x4a121e, 0xb02c36, 0xf88068],
    [0x3a2c0c, 0xa88018, 0xf8d460],
    [0x163a44, 0x2a8a98, 0x7ae0e0],
];

/// Colour the contour pass pulls overlapped edges toward.
const CONTOUR: Rgb = hex(0x2a1430);

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

/// Light comes from the front and above, so symmetric shapes shade
/// symmetrically.
const LY: f32 = -0.55;
const LZ: f32 = 0.83;

/// Map a surface normal to a ramp index (0..=4) in clean bands: most of a
/// form sits on the base tone, edges step down, a small cap steps up.
fn tone(ny: f32, nz: f32, bias: f32) -> usize {
    let d = ny * LY + nz * LZ;
    ((0.30 + d * 0.52 + bias).clamp(0.0, 0.999) * 5.0) as usize
}

struct Canvas<'a> {
    img: &'a mut Img,
    /// Owning part per pixel (0 = none). Later parts have higher ids.
    ids: Vec<u8>,
    cur: u8,
}

impl Canvas<'_> {
    /// Start a new part: its edges get contour lines where it overlaps
    /// anything drawn before.
    fn begin(&mut self) {
        self.cur = self.cur.saturating_add(1);
    }

    /// Detail pixel: recolours without changing which part owns it.
    fn put(&mut self, x: i32, y: i32, c: Rgb) {
        self.img.set(x, y, c);
    }

    /// Body pixel owned by the current part.
    fn fill(&mut self, x: i32, y: i32, c: Rgb) {
        if self.img.inside(x, y) {
            self.img.set(x, y, c);
            self.ids[(y * self.img.w + x) as usize] = self.cur;
        }
    }

    fn get(&self, x: i32, y: i32) -> Option<Rgb> {
        self.img.get(x, y)
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
                    self.fill(x, y, ramp[tone(ny, nz, bias)]);
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
                    // Shade as a cylinder (perpendicular to the axis only), so
                    // rounded ends don't draw rings where limbs join.
                    let along = (ox * dx + oy * dy) / len2;
                    let (qx, qy) = (ox - along * dx, oy - along * dy);
                    let (nx, ny) = (qx / rr, qy / rr);
                    let nz = (1.0 - (nx * nx + ny * ny)).max(0.0).sqrt();
                    self.fill(x, y, ramp[tone(ny * 0.5, nz, bias)]);
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
                let ny = ((fy - y0) / (y1 - y0) - 0.5) * 0.4;
                self.fill(x, y, ramp[tone(ny, nz, bias)]);
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

    /// Recolour a pixel only where something is already drawn.
    fn tint(&mut self, x: i32, y: i32, c: Rgb) {
        if self.img.alpha(x, y) != 0 {
            self.img.set(x, y, c);
        }
    }

    /// Pixel line (DDA) drawn as detail.
    fn line(&mut self, a: V2, b: V2, mut col: impl FnMut(f32) -> Option<Rgb>) {
        let n = (b.x - a.x).abs().max((b.y - a.y).abs()).ceil().max(1.0) as i32;
        for i in 0..=n {
            let t = i as f32 / n as f32;
            let (x, y) = (a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t);
            if let Some(c) = col(t) {
                self.tint(x.floor() as i32, y.floor() as i32, c);
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

/// Draw an authored pixel stamp; `mirror` reflects it across the frame's
/// centre line so paired features match exactly.
fn stamp(cv: &mut Canvas, x0: i32, y0: i32, rows: &[&str], mirror: bool, pal: &impl Fn(char) -> Option<Rgb>) {
    for (r, row) in rows.iter().enumerate() {
        for (i, ch) in row.chars().enumerate() {
            if let Some(c) = pal(ch) {
                let x = x0 + i as i32;
                cv.put(if mirror { FW - 1 - x } else { x }, y0 + r as i32, c);
            }
        }
    }
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
    /// Skull radii used by hats and ears.
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
    hip_joint_y: f32,
    thigh: f32,
    calf: f32,
    thigh_r: f32,
    calf_r: f32,
    upper_arm: f32,
    forearm: f32,
    arm_r: f32,
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
        (0.62, 0.66, 0.84)
    } else if dwarf {
        (0.66, 0.92, 1.22)
    } else if elf {
        (1.1, 1.02, 0.94)
    } else {
        (1.0, 1.0, 1.0)
    };
    // Long legs: about half the standing height for adults.
    let thigh = 15.0 * leg_k;
    let calf = 14.5 * leg_k;
    let ground = 108.0;
    let (bob, lag) = match pose.walk {
        Some(ph) => (-(ph.sin().abs()) * 1.5, -ph.cos() * 1.4),
        None if pose.breathe => (0.6, 0.8),
        None if pose.work == 1 => (-0.6, -0.9),
        None if pose.work == 2 => (0.8, 1.1),
        None => (0.0, 0.0),
    };
    let hip_joint_y = ground - 3.0 - calf - thigh + bob;
    let crotch_y = hip_joint_y + 2.0;
    let torso = if f { 28.0 } else { 30.0 } * torso_k;
    let shoulder_y = crotch_y - torso;
    let head_c = v(32.0, shoulder_y - 17.0 + if pose.breathe { 0.5 } else { 0.0 });
    let fig = f && look.mature && !child;
    // Hourglass for women (waist about 0.6 of the hips), V-taper for men.
    let (sw, chest, waist, hip) = if f { (9.0, 9.2, 6.6, 11.2) } else { (12.0, 11.5, 9.0, 9.8) };
    Build {
        cx: 32.0,
        ground,
        head_r: (13.0, 14.0),
        head_c,
        shoulder_y,
        shoulder_w: sw * width_k,
        chest_w: chest * width_k,
        waist_y: shoulder_y + torso * 0.58,
        waist_w: waist * width_k,
        hip_y: shoulder_y + torso * 0.86,
        hip_w: if fig { hip } else if f { hip - 1.5 } else { hip } * width_k,
        crotch_y,
        hip_joint_y,
        thigh,
        calf,
        thigh_r: if fig { 5.6 } else if child { 4.4 } else { 5.0 } * width_k.max(0.9),
        calf_r: if child { 3.2 } else if f { 3.7 } else { 4.0 } * width_k.max(0.9),
        upper_arm: 12.0 * torso_k,
        forearm: 11.0 * torso_k,
        arm_r: if child { 2.6 } else if f { 2.7 } else { 3.4 } * width_k.max(0.9),
        bust: if fig { if look.age == 2 { 4.8 } else { 5.8 } } else { 0.0 },
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
    let hip_dx = if side { 0.0 } else { b.thigh_r };
    let hip_l = v(b.cx - hip_dx, b.hip_joint_y);
    let hip_r = v(b.cx + hip_dx, b.hip_joint_y);
    let leg = |hip: V2, sgn: f32, swing: f32, lift: f32| -> (V2, V2) {
        if side {
            let th = swing * 0.5;
            let knee = hip.add(V2::polar(b.thigh, th));
            let bend = (-swing).max(0.0) * 0.7 + lift * 0.25;
            let ankle = knee.add(V2::polar(b.calf, th - bend));
            (knee, ankle)
        } else {
            // Knees drawn slightly together: the soft anime stance.
            let knee = hip.add(v(-sgn * 1.2, b.thigh - lift * 1.3));
            let ankle = knee.add(v(-sgn * 0.3, b.calf - lift * 2.6));
            (knee, ankle)
        }
    };
    let (sl, sr) = if walking { (s, -s) } else { (0.0, 0.0) };
    let lift_l = if walking { s.max(0.0) * 2.0 } else { 0.0 };
    let lift_r = if walking { (-s).max(0.0) * 2.0 } else { 0.0 };
    let (knee_l, ankle_l) = leg(hip_l, -1.0, sl, lift_l);
    let (knee_r, ankle_r) = leg(hip_r, 1.0, sr, lift_r);
    let sh_dx = if side { 0.0 } else { b.shoulder_w - 0.6 };
    let sh_y = b.shoulder_y + 3.0;
    let sh_l = v(b.cx - sh_dx, sh_y);
    let sh_r = v(b.cx + sh_dx, sh_y);
    let arm = |sh: V2, swing: f32, out: f32| -> (V2, V2) {
        match pose.work {
            1 => {
                let el = sh.add(v(out * 2.5 + if side { 2.5 } else { 0.0 }, -b.upper_arm * 0.85));
                let wr = el.add(v(out * 1.0 + if side { 2.5 } else { 0.0 }, -b.forearm * 0.8));
                (el, wr)
            }
            2 => {
                if side {
                    let el = sh.add(V2::polar(b.upper_arm, 1.0));
                    let wr = el.add(V2::polar(b.forearm, 1.45));
                    (el, wr)
                } else {
                    let el = sh.add(v(out * 1.5, b.upper_arm * 0.85));
                    let wr = el.add(v(-out * 3.5, b.forearm * 0.55));
                    (el, wr)
                }
            }
            _ => {
                if side {
                    let el = sh.add(V2::polar(b.upper_arm, swing * 0.45));
                    let wr = el.add(V2::polar(b.forearm, swing * 0.45 + 0.25 * swing.abs() + 0.12));
                    (el, wr)
                } else {
                    // A relaxed A-pose: elbows and hands clear the hips.
                    let el = sh.add(v(out * 4.2, b.upper_arm - 0.6 - swing.abs() * 0.8));
                    let wr = el.add(v(out * 2.6, b.forearm - swing.max(0.0) * 1.8));
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
    let mut cv = Canvas { img: &mut img, ids: vec![0; (FW * FH) as usize], cur: 0 };
    let side = dir == DIR_SIDE;
    let back = dir == DIR_UP;

    tail(&mut cv, look, dir, &b, &c);
    if look.outfit == outfit::CLOAK && !back {
        cloak_back(&mut cv, &b, &c, side);
    }
    if !back {
        back_hair(&mut cv, look, dir, &b, &c);
    }

    let left = (l.hip_l, l.knee_l, l.ankle_l);
    let right = (l.hip_r, l.knee_r, l.ankle_r);
    if side {
        // Far limbs first, in shadow.
        arm(&mut cv, look, &b, &c, (l.sh_l, l.el_l, l.wr_l), true, dir, pose);
        leg(&mut cv, look, &b, &c, left, true, dir);
        leg(&mut cv, look, &b, &c, right, false, dir);
    } else {
        // Arms hang just behind the body: the torso, bust and hips overlap
        // them, which also tucks the shoulder joint under the deltoid.
        arm(&mut cv, look, &b, &c, (l.sh_l, l.el_l, l.wr_l), false, dir, pose);
        arm(&mut cv, look, &b, &c, (l.sh_r, l.el_r, l.wr_r), false, dir, pose);
        leg(&mut cv, look, &b, &c, left, false, dir);
        leg(&mut cv, look, &b, &c, right, false, dir);
    }
    torso(&mut cv, look, dir, &b, &c);
    if side {
        arm(&mut cv, look, &b, &c, (l.sh_r, l.el_r, l.wr_r), false, dir, pose);
    }
    head(&mut cv, look, dir, &b, &c);
    if !back {
        face(&mut cv, look, dir, &b, &c, pose);
    }
    hair_front(&mut cv, look, dir, &b, &c);
    ears_horns(&mut cv, look, dir, &b, &c);
    hat(&mut cv, look, dir, &b, &c);
    let ids = std::mem::take(&mut cv.ids);
    drop(cv);
    contours(&mut img, &ids);
    outline(&mut img);
    img
}

/// Inner contour lines: a pixel of a part that borders a later (in-front)
/// part darkens, so overlaps read as drawn lines rather than merged blobs.
/// Lines fall beside and below the front part only (light from above), so a
/// form's top edge blends into what is behind it.
fn contours(img: &mut Img, ids: &[u8]) {
    let src = img.clone();
    let w = img.w;
    for y in 0..img.h {
        for x in 0..w {
            let a = ids[(y * w + x) as usize];
            if a == 0 {
                continue;
            }
            let Some(c) = src.get(x, y) else { continue };
            let covered = [(1, 0), (-1, 0), (0, -1)].iter().any(|&(dx, dy)| {
                let (nx, ny) = (x + dx, y + dy);
                src.inside(nx, ny) && src.alpha(nx, ny) != 0 && ids[(ny * w + nx) as usize] > a
            });
            if covered {
                img.set(x, y, mix(c, CONTOUR, 0.5));
            }
        }
    }
}

/// Coloured outline: each edge pixel takes a deep, hue-matched shade of the
/// neighbour it borders.
fn outline(img: &mut Img) {
    let src = img.clone();
    for y in 0..img.h {
        for x in 0..img.w {
            if src.alpha(x, y) != 0 {
                continue;
            }
            let mut found: Option<Rgb> = None;
            for (dx, dy) in [(0, 1), (1, 0), (-1, 0), (0, -1)] {
                if let Some(c) = src.get(x + dx, y + dy) {
                    found = Some(c);
                    break;
                }
            }
            if let Some(c) = found {
                img.set(x, y, mix(c, pal::INK, 0.72));
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
    let ramp = pants_ramp(look, c);
    let bias = if far { -0.16 } else { 0.0 };
    let (tr, cr) = (b.thigh_r, b.calf_r);
    let ankle_r = cr * 0.62;
    cv.begin();
    // Thigh: full through the upper half, then narrowing into the knee.
    let mid = v(hip.x + (knee.x - hip.x) * 0.5, hip.y + (knee.y - hip.y) * 0.5);
    cv.capsule(hip, mid, tr, tr * 0.88, &ramp, bias);
    cv.capsule(mid, knee, tr * 0.88, cr * 0.95, &ramp, bias);
    // Calf: swells just below the knee, then tapers to the ankle.
    let calf_mid = v(knee.x + (ankle.x - knee.x) * 0.3, knee.y + (ankle.y - knee.y) * 0.3);
    cv.capsule(knee, calf_mid, cr * 0.95, cr, &ramp, bias);
    cv.capsule(calf_mid, ankle, cr, ankle_r, &ramp, bias);
    let side = dir == DIR_SIDE;
    let boots = matches!(look.outfit, outfit::ARMOR | outfit::CLOAK | outfit::SMITH);
    if boots {
        cv.begin();
        cv.capsule(ankle.add(v(0.0, -6.5)), ankle, cr * 0.9, ankle_r + 0.6, &c.leather, bias);
    }
    cv.begin();
    let shoe = if boots { c.leather } else { c.shoe };
    if side {
        cv.ellipse(ankle.add(v(2.4, 1.6)), 4.4, 2.2, &shoe, bias);
    } else {
        cv.ellipse(ankle.add(v(0.0, 1.8)), 3.2, 2.4, &shoe, bias);
    }
}

#[allow(clippy::too_many_arguments)]
fn arm(cv: &mut Canvas, look: &Look, b: &Build, c: &Colors, (sh, el, wr): (V2, V2, V2), far: bool, dir: u8, pose: Pose) {
    let r = b.arm_r;
    let sleeve = match look.outfit {
        outfit::ARMOR => c.metal,
        outfit::DIRNDL | outfit::GUILD => c.linen,
        outfit::SMITH => c.skin,
        _ => c.top,
    };
    let bias = if far { -0.18 } else { 0.0 };
    let long_sleeve = !matches!(look.outfit, outfit::SMITH | outfit::DIRNDL);
    cv.begin();
    cv.capsule(sh, el, r, r * 0.88, &sleeve, bias);
    if long_sleeve {
        cv.capsule(el, wr, r * 0.88, r * 0.8, &sleeve, bias);
    } else {
        cv.begin();
        cv.capsule(el, wr, r * 0.86, r * 0.72, &c.skin, bias);
    }
    if matches!(look.outfit, outfit::DIRNDL | outfit::GUILD) {
        cv.begin();
        cv.ellipse(sh.add(v(0.0, 2.2)), r + 2.0, r + 1.8, &c.linen, bias);
    }
    if look.outfit == outfit::ARMOR {
        cv.begin();
        cv.ellipse(sh.add(v(0.0, 0.5)), r + 2.6, r + 1.8, &c.metal, bias + 0.05);
    }
    // Hands are drawn generously: small hands read as stumps at this size.
    let dirv = v(wr.x - el.x, wr.y - el.y);
    let len = (dirv.x * dirv.x + dirv.y * dirv.y).sqrt().max(0.01);
    let hand = wr.add(v(dirv.x / len * 2.4, dirv.y / len * 2.4));
    cv.begin();
    cv.ellipse(hand, r + 0.3, r + 0.7, &c.skin, bias - 0.06);
    if dir != DIR_UP && !far {
        // Finger creases on the lower half of the fist.
        let hy = (hand.y + 0.5).floor() as i32;
        for dx in [-1.0f32, 1.0] {
            let hx = (hand.x + dx * 1.2).floor() as i32;
            cv.put(hx, hy, c.skin[1]);
            cv.put(hx, hy + 1, c.skin[1]);
        }
    }
    let holding = !far && (dir == DIR_SIDE || wr.x > b.cx);
    if holding && look.held != held::NONE {
        cv.begin();
        held_item(cv, look, hand, dir, pose.work == 1, c);
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
    let fig = b.bust > 0.0;
    let keys: Vec<(f32, f32)> = if side {
        let chest = if look.female { 5.6 } else { 6.6 };
        vec![(sy - 0.5, 4.6), (sy + 3.0, chest), (b.waist_y, chest - 1.0), (b.hip_y, chest + if fig { 0.6 } else { 0.0 }), (cy + 1.0, chest - 0.2)]
    } else {
        vec![
            (sy - 0.5, if look.female { 5.0 } else { 6.5 }),
            (sy + 2.0, b.shoulder_w),
            (sy + 7.0, b.chest_w),
            (sy + 12.0, b.chest_w - 1.0),
            (b.waist_y, b.waist_w),
            (b.hip_y, b.hip_w),
            (cy + 1.0, b.hip_w - 0.4),
        ]
    };
    let waist_y = b.waist_y;
    let cx0 = b.cx;
    let lean = move |y: f32| -> f32 {
        if side {
            cx0 + 0.5 + if y < waist_y { 0.6 } else { -0.4 }
        } else {
            cx0
        }
    };
    let width = |y: f32| keyed(&keys, y);
    // Neck: its own part so the chin and collar line it.
    cv.begin();
    let nw = if look.female { 3.4 } else { 4.2 };
    cv.profile(sy - 6.0, sy + 1.5, |_| b.cx + if side { 0.5 } else { 0.0 }, |_| nw, &c.skin, -0.06);
    let lower = match look.outfit {
        outfit::DRESS | outfit::ROBE => c.skin,
        outfit::DIRNDL | outfit::GUILD => c.stocking,
        _ => pants_ramp(look, c),
    };
    cv.begin();
    // Pelvis: hip curve, then a V into the crotch so the thighs carry the
    // silhouette instead of a flat box edge.
    let pelvis: Vec<(f32, f32)> = if side {
        keys.clone()
    } else {
        vec![(b.waist_y, b.waist_w), (b.hip_y - 1.0, b.hip_w), (cy - 2.0, b.hip_w - 1.2), (cy + 1.5, 2.5)]
    };
    // Same cloth as the thighs: drawn without an id so no seam is traced
    // where it meets them; only the inner-leg line runs down from the crotch.
    let part = cv.cur;
    cv.cur = 0;
    cv.profile(b.waist_y, cy + 1.5, lean, |y| keyed(&pelvis, y), &lower, 0.0);
    cv.cur = part;
    if fig && back && !matches!(look.outfit, outfit::DRESS | outfit::ROBE | outfit::DIRNDL | outfit::GUILD) {
        // Seat: two soft rounds with a centre seam from the contour pass.
        for s in [-1.0f32, 1.0] {
            cv.begin();
            cv.ellipse(v(b.cx + s * b.hip_w * 0.48, b.hip_y + 0.5), b.hip_w * 0.55, 5.5, &lower, 0.02);
        }
    }
    if fig && side && !matches!(look.outfit, outfit::DRESS | outfit::ROBE) {
        cv.ellipse(v(b.cx - 2.8, b.hip_y), 5.6, 5.4, &lower, -0.02);
    }
    cv.begin();
    cv.profile(sy, b.waist_y + 3.5, lean, width, &shirt, 0.0);
    let by = sy + 9.0 + b.lag * 1.2;
    let cx = b.cx;
    // Neckline: a little skin under the collarbone for open-necked tops.
    if !back && !side && matches!(look.outfit, outfit::DRESS | outfit::DIRNDL | outfit::SMITH | outfit::APRON) {
        for y in sy as i32..(sy + if fig { 5.0 } else { 3.0 }) as i32 {
            let half = (4.5 - (y as f32 - sy) * 0.6).max(1.0);
            for x in (cx - half) as i32..(cx + half) as i32 {
                cv.put(x, y, if y == sy as i32 { c.skin[3] } else { c.skin[2] });
            }
        }
    }
    if fig && look.outfit != outfit::ARMOR {
        let bust_ramp = if matches!(look.outfit, outfit::DIRNDL | outfit::GUILD) { c.linen } else { shirt };
        if side {
            bust_side(cv, b, by, &bust_ramp, |y| lean(y) + width(y));
        } else {
            bust(cv, b, dir, by, &bust_ramp);
        }
    }
    match look.outfit {
        outfit::DRESS => {
            let top = b.waist_y + 1.0;
            let len = b.thigh + 4.0;
            let sway = b.lag;
            cv.begin();
            cv.profile(top, top + len, |y| cx + (y - top) / len * sway * 0.8, |y| b.waist_w + 1.2 + (y - top) * 0.45, &c.top, -0.02);
            hem_folds(cv, top, top + len, cx, b.waist_w + 1.2, 0.45, &c.top, sway);
            sash(cv, b, &c.linen, lean, width);
        }
        outfit::DIRNDL | outfit::GUILD => {
            let bod = if look.outfit == outfit::DIRNDL { c.top } else { c.navy };
            cv.begin();
            // Bodice under the bust: a corset from the ribs to the waist.
            let bt = if fig { by + b.bust * 0.55 } else { sy + 6.0 };
            cv.profile(bt, b.waist_y + 3.0, lean, |y| width(y) - 0.3, &bod, 0.0);
            if !fig && !back {
                cv.profile(sy + 4.0, bt + 0.5, lean, |y| width(y) - 1.6, &bod, 0.0);
            }
            if !back && !side {
                // Lacing.
                for k in 0..3 {
                    let y = bt as i32 + 2 + k * 3;
                    cv.put(cx as i32 - 2, y, c.linen[3]);
                    cv.put(cx as i32 + 1, y, c.linen[3]);
                    cv.put(cx as i32 - 1, y + 1, c.linen[2]);
                    cv.put(cx as i32, y + 1, c.linen[2]);
                }
            }
            let skirt = if look.outfit == outfit::DIRNDL { c.bottom } else { c.navy };
            let top = b.waist_y + 2.0;
            let len = b.thigh + 2.5;
            let sway = b.lag;
            cv.begin();
            cv.profile(top, top + len, |y| cx + (y - top) / len * sway * 0.8, |y| b.waist_w + 1.8 + (y - top) * 0.52, &skirt, 0.0);
            hem_folds(cv, top, top + len, cx, b.waist_w + 1.8, 0.52, &skirt, sway);
            if !back {
                if look.outfit == outfit::DIRNDL {
                    let aw = if side { 3.0 } else { 5.0 };
                    let ax = if side { cx + 3.5 } else { cx };
                    cv.begin();
                    cv.profile(top - 0.5, top + len - 2.5, |_| ax, |y| aw + (y - top) * 0.12, &c.linen, 0.05);
                } else {
                    let red = [hex(0x5a1420), hex(0x8a2232), hex(0xb83a4a), hex(0xe0606e), hex(0xf8909a)];
                    let ty = sy + 2.0;
                    cv.begin();
                    if side {
                        cv.ellipse(v(cx + 5.0, ty + 1.0), 2.0, 1.7, &red, 0.0);
                    } else {
                        cv.ellipse(v(cx - 2.6, ty + 1.0), 2.4, 1.8, &red, 0.0);
                        cv.ellipse(v(cx + 2.6, ty + 1.0), 2.4, 1.8, &red, 0.0);
                        cv.ellipse(v(cx, ty + 1.2), 1.2, 1.2, &red, -0.1);
                        cv.capsule(v(cx - 1.0, ty + 2.4), v(cx - 2.0, ty + 7.0), 1.0, 0.8, &red, 0.0);
                        cv.capsule(v(cx + 1.0, ty + 2.4), v(cx + 2.0, ty + 7.0), 1.0, 0.8, &red, 0.0);
                        cv.ellipse(v(cx + b.chest_w - 3.0, b.waist_y - 3.0), 1.6, 1.6, &gold5(), 0.0);
                    }
                }
            }
        }
        outfit::ROBE => {
            let top = b.waist_y - 2.0;
            let bottom = b.ground - 1.5;
            let len = bottom - top;
            let sway = b.lag;
            cv.begin();
            cv.profile(top, bottom, |y| cx + (y - top) / len * sway * 0.7, |y| b.waist_w + 1.6 + (y - top) * 0.24, &c.top, 0.0);
            hem_folds(cv, top, bottom, cx, b.waist_w + 1.6, 0.24, &c.top, sway);
            if !side && !back {
                let gold = gold5();
                for y in (sy + 1.0) as i32..bottom as i32 {
                    cv.put(cx as i32 - 1, y, gold[2]);
                    cv.put(cx as i32, y, gold[1]);
                }
                sash(cv, b, &gold, lean, width);
                // Mantle over the shoulders.
                cv.begin();
                cv.profile(sy - 0.5, sy + 7.0, |_| cx, |y| b.shoulder_w + 2.0 - (y - sy) * 0.15, &c.top, 0.08);
                cv.ellipse(v(cx, sy + 3.0), 1.6, 1.6, &gold, 0.1);
            }
        }
        outfit::APRON | outfit::SMITH if !back => {
            let ap = if look.outfit == outfit::SMITH { c.leather } else { c.linen };
            let (ax, aw) = if side { (cx + 3.5, 2.6) } else { (cx, b.chest_w - 3.0) };
            cv.begin();
            let at = if fig && !side { by - 1.0 } else { sy + 5.0 };
            cv.profile(at, b.hip_y + 8.0, |_| ax, |_| aw, &ap, 0.06);
            if !side {
                // Straps up to the shoulders.
                for k in 0..(at - sy) as i32 {
                    cv.put((cx - aw + 1.0) as i32, sy as i32 + k, ap[2]);
                    cv.put((cx + aw - 2.0) as i32, sy as i32 + k, ap[2]);
                }
                cv.rect((cx - 2.5) as i32, (b.waist_y + 4.0) as i32, 5, 4, ap[2]);
                cv.rect((cx - 2.5) as i32, (b.waist_y + 4.0) as i32, 5, 1, ap[1]);
            }
        }
        outfit::ARMOR => {
            if !back && !side {
                cv.begin();
                cv.profile(sy + 5.0, b.hip_y + 6.0, |_| cx, |_| 3.6, &c.hatc, 0.0);
                cv.ellipse(v(cx, sy + 9.0), 1.6, 1.6, &gold5(), 0.0);
            }
            belt(cv, b, &c.leather, lean, width, dir == DIR_DOWN);
        }
        outfit::VEST if !back => {
            let w = b.chest_w * 0.42;
            cv.begin();
            if side {
                cv.profile(sy + 0.5, b.hip_y, |_| cx + 2.0, |_| 3.6, &c.leather, 0.0);
            } else {
                cv.profile(sy + 0.5, b.hip_y, |_| cx - b.chest_w + w, |_| w, &c.leather, 0.0);
                cv.begin();
                cv.profile(sy + 0.5, b.hip_y, |_| cx + b.chest_w - w, |_| w, &c.leather, 0.0);
            }
        }
        _ => {}
    }
    if matches!(look.outfit, outfit::TUNIC | outfit::VEST | outfit::APRON | outfit::CLOAK | outfit::SMITH) {
        belt(cv, b, &c.leather, lean, width, dir == DIR_DOWN);
    }
    if look.outfit == outfit::CLOAK && back {
        cloak_back(cv, b, c, false);
    }
    if look.beard && !back && look.race == RACE_DWARF {
        let (bx, w) = if side { (cx + 6.0, 4.5) } else { (cx, 7.5) };
        cv.begin();
        cv.profile(sy - 4.0, sy + 11.0, |_| bx, |y| w * (1.0 - ((y - (sy - 4.0)) / 15.0).powf(2.0) * 0.7), &c.hair, 0.05);
    }
}

/// Side-view bust grown out of the chest line, as part of the same garment:
/// a gentle slope from the collarbone to the fullest point, then a round
/// underside curving back to the ribs, shaded darker underneath.
fn bust_side(cv: &mut Canvas, b: &Build, by: f32, ramp: &R5, front: impl Fn(f32) -> f32) {
    let r = b.bust;
    let top = by - r * 1.7;
    let apex = by + r * 0.1;
    let bottom = by + r * 0.95;
    let depth = r * 1.0;
    for y in top.floor() as i32..=bottom.ceil() as i32 {
        let fy = y as f32 + 0.5;
        if fy < top || fy > bottom {
            continue;
        }
        let (d, under) = if fy < apex {
            let t = (fy - top) / (apex - top);
            (depth * (t * (2.0 - t)).sqrt().min(1.0) * t.powf(0.35), 0.0)
        } else {
            let u = (fy - apex) / (bottom - apex);
            (depth * (1.0 - u * u).max(0.0).sqrt(), u)
        };
        let f = front(fy);
        let x0 = f.floor() as i32 - 1;
        let x1 = (f + d).floor() as i32;
        for x in x0..=x1 {
            let rim = x == x1;
            let i = if under > 0.75 {
                1
            } else if under > 0.35 || (rim && under > 0.0) {
                2
            } else if rim && fy > apex - 3.0 {
                4
            } else {
                3
            };
            cv.fill(x, y, ramp[i]);
        }
    }
}

/// The bust as two rounded forms, each its own part, so the contour pass
/// draws the cleavage and the line beneath. `by` already carries the lag.
fn bust(cv: &mut Canvas, b: &Build, dir: u8, by: f32, ramp: &R5) {
    let r = b.bust;
    match dir {
        DIR_SIDE => {
            cv.begin();
            cv.ellipse(v(b.cx + 4.6, by), r * 0.85, r * 0.92, ramp, 0.0);
        }
        DIR_DOWN => {
            for s in [-1.0f32, 1.0] {
                cv.begin();
                cv.ellipse(v(b.cx + s * r * 0.92, by), r, r * 0.9, ramp, -0.03);
            }
        }
        _ => {}
    }
}

fn belt(cv: &mut Canvas, b: &Build, leather: &R5, lean: impl Fn(f32) -> f32, width: impl Fn(f32) -> f32, buckle: bool) {
    let y = b.waist_y + 2.5;
    let w = width(y) + 0.4;
    let c = lean(y);
    cv.begin();
    cv.profile(y - 1.0, y + 1.0, |_| c, |_| w, leather, 0.05);
    if buckle {
        let g = gold5();
        cv.rect(c as i32 - 1, y as i32 - 1, 3, 2, g[3]);
        cv.put(c as i32, y as i32 - 1, g[1]);
    }
}

fn sash(cv: &mut Canvas, b: &Build, ramp: &R5, lean: impl Fn(f32) -> f32, width: impl Fn(f32) -> f32) {
    let y = b.waist_y + 1.0;
    let w = width(y) + 0.3;
    let c = lean(y);
    cv.begin();
    cv.profile(y - 1.0, y + 1.5, |_| c, |_| w, ramp, 0.05);
}

#[allow(clippy::too_many_arguments)]
fn hem_folds(cv: &mut Canvas, top: f32, bottom: f32, cx: f32, w0: f32, k: f32, ramp: &R5, sway: f32) {
    let len = bottom - top;
    for i in [-2.0f32, -0.7, 0.7, 2.0] {
        for y in (top + len * 0.4) as i32..bottom as i32 - 1 {
            let t = (y as f32 - top) / len;
            let w = w0 + (y as f32 - top) * k;
            let x = cx + t * sway * 0.8 + i / 2.6 * w;
            cv.tint(x.floor() as i32, y, ramp[2]);
        }
    }
}

fn cloak_back(cv: &mut Canvas, b: &Build, c: &Colors, side: bool) {
    let top = b.shoulder_y - 0.5;
    let bottom = b.ground - 7.0;
    let cx = if side { b.cx - 3.5 } else { b.cx };
    let lag = b.lag;
    let sw = b.shoulder_w;
    cv.begin();
    cv.profile(top, bottom, |y| cx - (y - top) / (bottom - top) * lag * 1.5, |y| if side { 5.5 + (y - top) * 0.14 } else { sw + 2.0 + (y - top) * 0.2 }, &c.hatc, -0.1);
}

fn held_item(cv: &mut Canvas, look: &Look, hand: V2, dir: u8, raised: bool, c: &Colors) {
    let wood = [hex(0x2e1c1c), hex(0x4a2e24), hex(0x6b412f), hex(0x8c5a38), hex(0xab7645)];
    let (x, y) = (hand.x, hand.y);
    match look.held {
        held::STAFF => {
            let top = if raised { y - 22.0 } else { y - 40.0 };
            cv.capsule(v(x, top), v(x, y + 14.0), 1.3, 1.3, &wood, 0.0);
            let gem = [hex(0x2a1050), hex(0x4a2490), hex(0x7a48d0), hex(0xb090f8), hex(0xf0e6ff)];
            cv.capsule(v(x - 3.0, top + 1.0), v(x, top - 2.5), 1.2, 1.0, &wood, 0.0);
            cv.capsule(v(x + 3.0, top + 1.0), v(x, top - 2.5), 1.2, 1.0, &wood, 0.0);
            cv.ellipse(v(x, top - 3.5), 3.0, 3.2, &gem, 0.1);
        }
        held::HAMMER => {
            let (hx, hy) = if raised { (x, y - 14.0) } else { (x + 0.5, y + 1.0) };
            cv.capsule(v(hx, hy), v(hx, hy + 13.0), 1.2, 1.2, &wood, 0.0);
            cv.capsule(v(hx - 4.0, hy - 1.0), v(hx + 4.0, hy - 1.0), 2.6, 2.6, &c.metal, 0.05);
        }
        held::MUG if dir != DIR_UP => {
            let mug = [hex(0x5a3418), hex(0x8a5a2a), hex(0xb07a40), hex(0xd8a060), hex(0xf0c080)];
            cv.capsule(v(x, y - 2.5), v(x, y + 3.5), 3.0, 3.0, &mug, 0.0);
            cv.ellipse(v(x, y - 3.5), 3.2, 1.8, &[hex(0xb0a490), hex(0xc8bca8), hex(0xe8dcc8), hex(0xfff6e0), hex(0xffffff)], 0.1);
            cv.capsule(v(x + 3.5, y - 1.0), v(x + 3.5, y + 2.5), 0.9, 0.9, &mug, -0.1);
        }
        held::BOOK if dir != DIR_UP => {
            let cover = [hex(0x3a1420), hex(0x5a2030), hex(0x7a2e40), hex(0x9a4458), hex(0xba6070)];
            cv.capsule(v(x - 3.5, y), v(x + 3.5, y), 3.0, 3.0, &cover, 0.0);
            cv.rect((x - 2.5) as i32, (y - 2.0) as i32, 6, 3, hex(0xf0e6d0));
            cv.put(x as i32, (y - 1.0) as i32, hex(0xc8bca8));
        }
        held::SWORD => {
            let (top, bot) = if raised { (y - 22.0, y + 1.0) } else { (y + 1.0, y + 22.0) };
            cv.capsule(v(x, top), v(x, bot), 1.2, 1.0, &c.metal, 0.1);
            let gy = if raised { bot } else { top + 1.0 };
            cv.capsule(v(x - 3.5, gy), v(x + 3.5, gy), 1.1, 1.1, &gold5(), 0.0);
        }
        held::BASKET if dir != DIR_UP => {
            let wick = [hex(0x5a3a1c), hex(0x8a6a3a), hex(0xa8844e), hex(0xc8a464), hex(0xe0c080)];
            cv.capsule(v(x - 3.5, y + 2.5), v(x + 3.5, y + 2.5), 3.0, 3.0, &wick, 0.0);
            for (dx, col) in [(-1.8, 0xd24c40u32), (0.6, 0xeec04a), (2.4, 0x7aa86c)] {
                let cc = hex(col);
                cv.ellipse(v(x + dx, y - 0.5), 1.5, 1.4, &[mix(cc, pal::INK, 0.5), mix(cc, pal::INK, 0.25), cc, mix(cc, hex(0xffffff), 0.3), mix(cc, hex(0xffffff), 0.5)], 0.0);
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------- head

/// Front-view face half-width for row `dy` relative to the head centre
/// (-13..=14). Straight cheeks, then a soft U jaw stepping 1,1,2,2 so the
/// curve reads clean at 1x. Above the eyes it is covered by hair anyway.
fn face_hw(look: &Look, dy: i32) -> f32 {
    const SKULL: [f32; 13] = [6.0, 8.0, 9.0, 10.0, 11.0, 11.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0];
    const FEMALE: [f32; 15] = [12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 11.0, 10.0, 9.0, 7.0, 5.0];
    const MALE: [f32; 15] = [12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 11.0, 11.0, 10.0, 7.0];
    const CHILD: [f32; 15] = [12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 11.0, 10.0, 8.0, 6.0, 0.0];
    if !(-13..=14).contains(&dy) {
        return 0.0;
    }
    let extra = if look.race == RACE_DWARF && !look.female { 1.0 } else { 0.0 };
    if dy < 0 {
        return SKULL[(dy + 13) as usize] + extra;
    }
    let t = if look.age == 0 {
        &CHILD
    } else if look.female {
        &FEMALE
    } else {
        &MALE
    };
    let w = t[dy as usize];
    if w > 0.0 { w + extra } else { 0.0 }
}

/// Side-view head rows (left, right) offsets from x=32 for dy -13..=14:
/// round skull at the back, forehead, a one-pixel nose, lips and chin.
const SIDE_HEAD: [(i32, i32); 28] = [
    (-6, 5),
    (-9, 8),
    (-10, 9),
    (-11, 10),
    (-12, 11),
    (-12, 11),
    (-13, 12),
    (-13, 12),
    (-13, 12),
    (-13, 12),
    (-13, 12),
    (-13, 12),
    (-13, 12),
    (-13, 12),
    (-13, 12),
    (-13, 12),
    (-13, 12),
    (-13, 12),
    (-13, 12),
    (-13, 12),
    (-13, 12),
    (-12, 12),
    (-12, 12),
    (-12, 13),
    (-11, 12),
    (-10, 11),
    (-8, 11),
    (-3, 9),
];

fn head(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
    let hc = b.head_c;
    let (cx, cy) = (32, hc.y.round() as i32);
    cv.begin();
    match dir {
        DIR_SIDE => {
            // Ear behind the cheek.
            if look.race != RACE_ELF {
                cv.ellipse(v(30.5, hc.y + 4.0), 2.2, 3.0, &c.skin, -0.05);
            }
            for dy in -13..=14 {
                let (l, r) = SIDE_HEAD[(dy + 13) as usize];
                for x in cx + l..cx + r {
                    let edge = x == cx + r - 1 || (dy >= 9 && x < cx + l + 2);
                    cv.fill(x, cy + dy, c.skin[if edge { 2 } else { 3 }]);
                }
            }
        }
        _ => {
            for dy in -13..=14 {
                let hw = face_hw(look, dy);
                if hw <= 0.0 {
                    continue;
                }
                let w = hw as i32;
                for x in cx - w..cx + w {
                    let u = (x as f32 + 0.5 - 32.0).abs() / hw;
                    let shade = u > 0.84 || (dy >= 10 && u > 0.55);
                    cv.fill(x, cy + dy, c.skin[if shade { 2 } else { 3 }]);
                }
            }
        }
    }
}

/// Female eye, left side (column 0 is the outer corner). Two-row lash bar
/// tilting down toward the nose, a 4-wide iris dark at the top and light at
/// the bottom, a big catchlight, sclera at both corners.
const EYE_F: [&str; 8] = [
    "aKKKKa.",
    "KKKKKKa",
    "abDDDKK",
    ".WWDDDS",
    ".WDDMMS",
    ".SMMLMS",
    "..MLLW.",
    "...bb..",
];
const EYE_M: [&str; 6] = [
    ".aKKKKa",
    "aKKKKKK",
    ".SDDDKK",
    ".SWDMS.",
    "..MLM..",
    "...bb..",
];
const EYE_CLOSED: [&str; 8] = [
    ".......",
    ".......",
    ".......",
    ".......",
    "K......",
    ".KKKKKa",
    "..bbb..",
    ".......",
];
/// Side-view eye facing right (column 0 is the back corner).
const EYE_SIDE: [&str; 8] = [
    "aKKKa",
    "KKKKK",
    ".bDDK",
    ".WDDS",
    ".WDMS",
    ".MMLS",
    "..LM.",
    "..b..",
];
const EYE_SIDE_CLOSED: [&str; 8] = [".....", ".....", ".....", ".....", ".....", "KKKKa", ".bb..", "....."];

fn face(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors, pose: Pose) {
    let iris = if look.race == RACE_DEMON && look.eyes < 4 { &EYE_COLORS[4 + (look.eyes % 2) as usize] } else { &EYE_COLORS[look.eyes as usize % EYE_COLORS.len()] };
    let hc = b.head_c;
    let cy = hc.y.round() as i32;
    let lash = mix(hex(0x160c1a), c.hair[0], 0.3);
    let eye_pal = |ch: char| -> Option<Rgb> {
        match ch {
            'K' => Some(lash),
            'a' => Some(mix(lash, c.skin[2], 0.5)),
            'b' => Some(c.skin[1]),
            'D' => Some(hex(iris[0])),
            'M' => Some(hex(iris[1])),
            'L' => Some(hex(iris[2])),
            'W' => Some(hex(0xffffff)),
            'S' => Some(hex(0xeeeaf4)),
            _ => None,
        }
    };
    let blush = mix(c.skin[3], hex(0xf2707e), 0.38);
    let lip = mix(c.skin[1], hex(0xa83448), 0.45);
    let soft = look.female || look.age == 0;
    if dir == DIR_DOWN {
        let ey = cy + 3;
        let rows: &[&str] = if pose.blink {
            &EYE_CLOSED
        } else if soft {
            &EYE_F
        } else {
            &EYE_M
        };
        let ex = 23;
        let ey = if !pose.blink && !soft { ey + 1 } else { ey };
        stamp(cv, ex, ey, rows, false, &eye_pal);
        stamp(cv, ex, ey, rows, true, &eye_pal);
        // Nose: one soft shadow pixel, light from the left.
        cv.put(32, cy + 11, c.skin[2]);
        // Mouth, on the second-to-last row of the face.
        if soft {
            cv.put(31, cy + 13, lip);
            cv.put(32, cy + 13, lip);
        } else {
            cv.put(30, cy + 13, c.skin[1]);
            cv.put(31, cy + 13, lip);
            cv.put(32, cy + 13, lip);
            cv.put(33, cy + 13, c.skin[1]);
        }
        if soft {
            for x in [22, 23, 24] {
                cv.put(x, cy + 11, blush);
                cv.put(FW - 1 - x, cy + 11, blush);
            }
        }
        if look.age == 2 {
            cv.put(22, cy + 11, c.skin[2]);
            cv.put(FW - 1 - 22, cy + 11, c.skin[2]);
        }
        if look.glasses {
            // Thin half-rim frames: lower rim, short sides, bridge, a glint.
            let g = mix(c.skin[0], hex(0x3a2a3a), 0.4);
            let bottom = ey + if soft { 8 } else { 6 };
            for side in [false, true] {
                let px = |x: i32| if side { FW - 1 - x } else { x };
                for x in 22..30 {
                    cv.put(px(x), bottom, g);
                }
                for y in bottom - 3..bottom {
                    cv.put(px(21), y, g);
                    cv.put(px(30), y, g);
                }
                cv.put(px(28), bottom - 1, hex(0xffffff));
            }
            for x in 31..33 {
                cv.put(x, ey + 3, g);
            }
        }
        if look.beard && look.race != RACE_DWARF {
            for dy in 10..=15 {
                let hw = face_hw(look, dy.min(14)).max(5.0) as i32 + 1;
                for x in 32 - hw..32 + hw {
                    let mouth = dy == 13 && (30..34).contains(&x);
                    if !mouth && (dy >= 12 || (x - 31).abs() > 6) {
                        cv.put(x, cy + dy, c.hair[if dy == 15 { 1 } else { 2 }]);
                    }
                }
            }
        }
        if look.beard && look.race == RACE_DWARF {
            cv.begin();
            cv.ellipse(v(28.5, hc.y + 12.0), 4.0, 2.2, &c.hair, 0.1);
            cv.ellipse(v(35.5, hc.y + 12.0), 4.0, 2.2, &c.hair, 0.1);
        }
    } else if dir == DIR_SIDE {
        let ex = 37;
        let ey = cy + 3;
        stamp(cv, ex, ey, if pose.blink { &EYE_SIDE_CLOSED } else { &EYE_SIDE }, false, &eye_pal);
        if soft {
            for k in 0..3 {
                cv.put(ex - 1 + k, cy + 11, blush);
            }
        }
        cv.put(32 + 9, cy + 13, lip);
        if look.beard {
            cv.begin();
            cv.ellipse(v(32.0 + 6.0, hc.y + 12.0), 5.0, 3.6, &c.hair, 0.0);
        }
        if look.glasses {
            let g = hex(0x4a3a30);
            for k in 0..7 {
                cv.put(ex - 1 + k, ey - 1, g);
                cv.put(ex - 1 + k, ey + 7, g);
            }
            for k in 0..9 {
                cv.put(ex - 10 + k, ey + 2, g);
            }
        }
    }
}

// ---------------------------------------------------------------- hair

/// How a hairstyle is cut.
struct Cut {
    /// Bang tips: (x offset from centre, tip y offset from head centre,
    /// slope of the clump's sides). Lower slope = blunter cut.
    tips: &'static [(f32, f32, f32)],
    /// Side locks framing the face: length below the head centre, width.
    lock: (f32, f32),
    /// Length of the back mass below the head centre (front view behind the
    /// body); 0 = short.
    back: f32,
    /// How far the hair reaches below the head centre at the nape.
    nape: f32,
}

fn cut(style: u8) -> Cut {
    match style {
        // Short, slightly messy (masculine).
        0 => Cut { tips: &[(-11.0, -1.0, 1.6), (-7.5, 0.0, 1.8), (-3.5, 1.0, 1.8), (0.5, 0.0, 1.8), (4.5, 1.0, 1.8), (8.5, -0.5, 1.6), (12.0, -2.0, 1.4)], lock: (4.0, 3.0), back: 0.0, nape: 8.0 },
        // Long straight, hime-style.
        1 => Cut { tips: &[(-10.5, 4.5, 1.5), (-6.5, 0.5, 1.7), (-2.0, 4.0, 1.7), (2.0, 4.0, 1.7), (6.5, 0.5, 1.7), (10.5, 4.5, 1.5)], lock: (24.0, 4.0), back: 46.0, nape: 12.0 },
        // Ponytail.
        2 => Cut { tips: &[(-10.5, 4.0, 1.5), (-6.5, 1.5, 1.6), (-2.0, 3.5, 1.7), (2.5, 2.5, 1.7), (6.5, 1.0, 1.6), (10.5, 3.5, 1.5)], lock: (12.0, 3.5), back: 0.0, nape: 9.0 },
        // Bun.
        3 => Cut { tips: &[(-10.5, 3.5, 1.5), (-6.5, 1.0, 1.8), (-2.5, 3.0, 1.8), (2.5, 3.0, 1.8), (6.5, 1.0, 1.8), (10.5, 3.5, 1.5)], lock: (9.0, 3.0), back: 0.0, nape: 8.0 },
        // Spiky hero.
        4 => Cut { tips: &[(-11.0, 0.5, 1.3), (-7.0, 1.5, 1.5), (-3.0, 2.0, 1.6), (1.0, 1.0, 1.6), (5.0, 2.0, 1.5), (9.0, 0.5, 1.4), (12.0, -1.0, 1.3)], lock: (6.0, 3.5), back: 0.0, nape: 9.0 },
        // Bob with a blunt fringe.
        5 => Cut { tips: &[(-10.0, 2.0, 0.6), (-6.5, 1.5, 0.6), (-2.0, 2.0, 0.6), (2.0, 2.0, 0.6), (6.5, 1.5, 0.6), (10.0, 2.0, 0.6)], lock: (13.0, 5.5), back: 12.0, nape: 13.0 },
        // Twin tails.
        6 => Cut { tips: &[(-10.5, 4.5, 1.5), (-6.5, 0.5, 1.7), (-2.0, 4.0, 1.7), (2.0, 4.0, 1.7), (6.5, 0.5, 1.7), (10.5, 4.5, 1.5)], lock: (11.0, 3.5), back: 0.0, nape: 9.0 },
        // Side-swept.
        _ => Cut { tips: &[(-10.0, 4.0, 1.2), (-6.0, 3.5, 1.2), (-2.0, 2.5, 1.3), (2.0, 1.0, 1.4), (6.0, -0.5, 1.5), (10.0, -1.0, 1.5)], lock: (14.0, 4.0), back: 10.0, nape: 11.0 },
    }
}

/// Bottom edge of the fringe at pixel column x: each clump hangs as a
/// downward V from its tip, and the fringe is their union.
fn fringe_bottom(tips: &[(f32, f32, f32)], cy: f32, x: i32) -> f32 {
    let px = x as f32 + 0.5;
    tips.iter().map(|&(tx, ty, s)| cy + ty - s * (px - (32.0 + tx)).abs()).fold(f32::MIN, f32::max)
}

/// Hair mass radii (front view).
fn mass_r(look: &Look) -> (f32, f32) {
    if look.hair_style == 0 || look.hair_style == 4 { (14.0, 14.5) } else { (14.5, 15.0) }
}

fn back_hair(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
    let h = &c.hair;
    let hc = b.head_c;
    let side = dir == DIR_SIDE;
    let sway = b.lag;
    let k = cut(look.hair_style);
    cv.begin();
    if k.back > 0.0 {
        let top = hc.y - 2.0;
        let len = k.back;
        let (rx, _) = mass_r(look);
        let w0 = if side { 7.5 } else { rx };
        let cx = if side { 32.0 - 6.5 } else { 32.0 };
        let end = top + len;
        hair_sheet(cv, h, top, end, false, |y| cx + ((y - top) / len).powi(2) * sway * 2.2, |y| {
            let t = (y - top) / len;
            w0 + t * 1.2 - if t > 0.88 { (t - 0.88) * 30.0 } else { 0.0 }
        });
    }
    // Ponytail and twin tails hang behind in front and side views.
    match look.hair_style {
        2 if side => {
            let base = v(32.0 - 13.0, hc.y - 6.0);
            tail_lock(cv, h, base, base.add(v(-5.0 - sway * 2.0, 26.0)), 4.2);
        }
        2 => {
            let base = v(32.0 + 11.0, hc.y + 2.0);
            tail_lock(cv, h, base, base.add(v(2.0 + sway, 18.0)), 3.2);
        }
        6 => {
            let sides: &[f32] = if side { &[-1.0] } else { &[-1.0, 1.0] };
            for &s in sides {
                let base = v(32.0 + s * 14.5 + if side { 4.0 } else { 0.0 }, hc.y - 4.0);
                tail_lock(cv, h, base, base.add(v(s * 3.0 + sway, 30.0)), 4.0);
            }
        }
        _ => {}
    }
}

/// A falling sheet of hair: 5px ribbon clumps, each ending in a point, split
/// by strand lines in their lower part. `lit` sheets (seen from behind) sit
/// a band brighter than ones hanging behind the body.
#[allow(clippy::too_many_arguments)]
fn hair_sheet(cv: &mut Canvas, h: &R5, top: f32, end: f32, lit: bool, cx: impl Fn(f32) -> f32, hw: impl Fn(f32) -> f32) {
    let len = (end - top).max(1.0);
    let base = if lit { 3 } else { 2 };
    for y in top.floor() as i32..=end.ceil() as i32 {
        let fy = y as f32 + 0.5;
        let t = (fy - top) / len;
        let c = cx(fy);
        let w = hw(fy);
        for x in (c - w).floor() as i32..=(c + w).ceil() as i32 {
            let px = x as f32 + 0.5;
            let u = (px - c) / w.max(0.5);
            if u.abs() > 1.0 {
                continue;
            }
            let q = (px - c + 62.5) / 5.0;
            let uc = q - q.floor();
            if fy > end - (1.0 - (uc - 0.5).abs() * 2.0) * 3.5 - 0.5 && fy > end - 4.0 {
                // Between clump points.
                if fy > end - (1.0 - (uc - 0.5).abs() * 2.0) * 3.5 {
                    continue;
                }
            }
            let strand = uc < 0.2 && t > 0.3;
            let edge = u.abs() > 0.84;
            let lower = t > 0.72;
            let tone = if strand || edge { base - 1 } else if lower && uc > 0.6 { base - 1 } else { base };
            cv.fill(x, y, h[tone]);
        }
    }
}

/// A tapered lock (ponytail, twin tail) with flat banded shading.
fn tail_lock(cv: &mut Canvas, h: &R5, a: V2, b: V2, r: f32) {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len2 = (dx * dx + dy * dy).max(1e-4);
    let (x0, x1) = ((a.x.min(b.x) - r - 1.0) as i32, (a.x.max(b.x) + r + 1.0) as i32);
    let (y0, y1) = ((a.y.min(b.y) - r - 1.0) as i32, (a.y.max(b.y) + r + 1.0) as i32);
    for y in y0..=y1 {
        for x in x0..=x1 {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let t = (((px - a.x) * dx + (py - a.y) * dy) / len2).clamp(0.0, 1.0);
            // Full near the tie, swelling, then tapering to a point.
            let rr = r * (0.75 + (t * 3.0).min(1.0) * 0.35) * (1.0 - t.powf(1.6) * 0.92);
            let (ox, oy) = (px - (a.x + dx * t), py - (a.y + dy * t));
            let d2 = ox * ox + oy * oy;
            if d2 > rr * rr {
                continue;
            }
            let u = ox / rr.max(0.5);
            let tone = if u < -0.55 { 2 } else if u > 0.6 || t > 0.8 { 2 } else { 3 };
            let tone = if u.abs() < 0.15 && t > 0.2 && t < 0.85 { 2 } else { tone };
            cv.fill(x, y, h[tone]);
        }
    }
}

fn hair_front(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
    let h = &c.hair;
    let hc = b.head_c;
    let style = look.hair_style;
    let k = cut(style);
    let sway = b.lag;
    let (rx, ry) = mass_r(look);
    let shine = h[4];
    cv.begin();
    match dir {
        DIR_UP => {
            // Back of the head: the whole cap, nape, and the long sheet over
            // the back.
            let mc = v(32.0, hc.y - 1.0);
            let nape = hc.y + k.nape;
            for y in (mc.y - ry).floor() as i32..=nape.ceil() as i32 {
                for x in (32.0 - rx - 1.0) as i32..=(32.0 + rx + 1.0) as i32 {
                    let nx = (x as f32 + 0.5 - mc.x) / rx;
                    let ny = (y as f32 + 0.5 - mc.y) / ry;
                    let inside = if ny < 0.0 { nx * nx + ny * ny <= 1.0 } else { nx.abs() <= 1.0 - (ny * 0.25).max(0.0) * 0.4 };
                    if !inside || y as f32 + 0.5 > nape - (nx.abs() * 3.0) {
                        continue;
                    }
                    cv.fill(x, y, h[if nx.abs() > 0.86 || ny > 0.7 { 2 } else { 3 }]);
                }
            }
            if k.back > 0.0 {
                let top = hc.y + 2.0;
                let len = k.back - 2.0;
                hair_sheet(cv, h, top, top + len, true, |y| 32.0 + ((y - top) / len).powi(2) * sway * 2.2, |y| {
                    let t = (y - top) / len;
                    rx * (1.0 - t * 0.12) - if t > 0.9 { (t - 0.9) * 25.0 } else { 0.0 }
                });
            }
            // Strand lines from the crown.
            let crown = v(32.0, hc.y - 11.0);
            for sx in [-9.0f32, -4.5, 0.0, 4.5, 9.0] {
                let end = v(32.0 + sx * 1.15, (nape - 2.0).max(hc.y + 4.0) + if k.back > 0.0 { k.back - 6.0 } else { 0.0 });
                let start = v(crown.x + sx * 0.4, crown.y + 2.0);
                cv.line(start, end, |t| if t < 0.12 { None } else { Some(h[2]) });
            }
            highlight_ring(cv, h, shine, v(32.0, hc.y - 1.0), rx, ry, &[]);
            match style {
                2 => {
                    let base = v(32.0, hc.y + 1.0);
                    cv.begin();
                    tail_lock(cv, h, base, base.add(v(sway, 24.0)), 4.2);
                    cv.rect(30, hc.y as i32, 4, 2, hex(0xc83a4a));
                }
                3 => {
                    cv.begin();
                    cv.ellipse(v(32.0, hc.y - ry - 0.5), 5.5, 5.0, h, 0.05);
                }
                6 => {
                    for s in [-1.0f32, 1.0] {
                        let base = v(32.0 + s * (rx + 0.5), hc.y - 4.0);
                        cv.begin();
                        tail_lock(cv, h, base, base.add(v(s * 3.0 + sway, 30.0)), 4.0);
                        cv.ellipse(base.add(v(0.0, -1.0)), 2.4, 2.0, &r5(&ramp4(&pal::CLOTH[1])), 0.1);
                    }
                }
                _ => {}
            }
        }
        DIR_SIDE => {
            let mc = v(31.0, hc.y - 1.0);
            let cy = hc.y;
            let front_tips: &[(f32, f32, f32)] = match style {
                0 | 4 => &[(5.0, 0.0, 1.6), (8.5, 1.0, 1.5), (12.0, -1.0, 1.3)],
                5 => &[(5.0, 1.5, 0.7), (9.0, 1.5, 0.7), (12.5, 1.0, 0.7)],
                7 => &[(5.0, 3.5, 1.2), (9.0, 2.5, 1.3), (12.5, 0.0, 1.4)],
                _ => &[(5.0, 2.0, 1.5), (9.0, 0.5, 1.6), (12.5, 1.5, 1.4)],
            };
            let nape = cy + k.nape;
            for y in (mc.y - ry).floor() as i32..=nape.ceil() as i32 {
                for x in (mc.x - rx - 1.0) as i32..=(mc.x + rx + 1.0) as i32 {
                    let nx = (x as f32 + 0.5 - mc.x) / rx;
                    let ny = (y as f32 + 0.5 - mc.y) / ry;
                    if nx * nx + ny.min(0.0).powi(2) > 1.0 || (ny > 0.0 && nx * nx + ny * ny > 1.0 && x > 30) {
                        continue;
                    }
                    let fy = y as f32 + 0.5;
                    if x >= 35 {
                        if fy > fringe_bottom(front_tips, cy, x) {
                            continue;
                        }
                    } else if x >= 28 {
                        // Over the ear the hair lifts to the temple.
                        if fy > cy - 2.0 + (35 - x) as f32 * 0.2 && fy > cy - 1.0 {
                            continue;
                        }
                    } else if fy > nape - (28 - x).max(0) as f32 * -0.2 {
                        continue;
                    }
                    let rim = nx * nx + ny * ny > 0.8 && ny > -0.2;
                    cv.fill(x, y, h[if rim { 2 } else { 3 }]);
                }
            }
            // Side lock in front of the ear.
            let (ll, lw) = k.lock;
            if ll > 5.0 {
                lock_strip(cv, h, 31.5, cy - 3.0, cy + ll * 0.85, lw * 0.9, sway, 1.0);
            }
            for &(tx, _, _) in front_tips.iter().skip(1) {
                let nb = fringe_bottom(front_tips, cy, (32.0 + tx - 2.0) as i32);
                cv.line(v(32.0 + tx * 0.5, cy - 11.0), v(32.0 + tx - 2.0, nb), |t| if t < 0.1 { None } else if t > 0.8 { Some(h[1]) } else { Some(h[2]) });
            }
            for dx in [-8.0f32, -3.0] {
                cv.line(v(31.0 + dx * 0.4, cy - 12.0), v(31.0 + dx * 1.3, nape - 2.0), |t| if t < 0.15 { None } else { Some(h[2]) });
            }
            highlight_ring(cv, h, shine, mc, rx, ry, &[]);
            match style {
                3 => {
                    cv.begin();
                    cv.ellipse(v(28.0, cy - ry - 0.5), 5.5, 5.0, h, 0.05);
                }
                4 => spikes(cv, h, hc.y, ry, true),
                _ => {}
            }
        }
        _ => {
            let cy = hc.y;
            let mc = v(32.0, cy - 1.0);
            let tips = k.tips;
            let face_w = 12.0;
            for y in (mc.y - ry).floor() as i32..=(cy + 4.0).ceil() as i32 {
                for x in (32.0 - rx - 1.0) as i32..=(32.0 + rx + 1.0) as i32 {
                    let nx = (x as f32 + 0.5 - mc.x) / rx;
                    let ny = (y as f32 + 0.5 - mc.y) / ry;
                    if nx * nx + ny * ny > 1.0 {
                        continue;
                    }
                    let fy = y as f32 + 0.5;
                    let dxa = (x as f32 + 0.5 - 32.0).abs();
                    let limit = if dxa < face_w + 0.5 { fringe_bottom(tips, cy, x) } else { cy + 3.0 };
                    if fy > limit {
                        continue;
                    }
                    let temple = dxa > face_w - 1.0 && fy > cy - 4.0;
                    cv.fill(x, y, h[if temple { 2 } else { 3 }]);
                }
            }
            // Strand lines from the crown into each notch between clumps.
            let mut xs: Vec<f32> = tips.iter().map(|t| t.0).collect();
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let mut notches = Vec::new();
            for w in xs.windows(2) {
                let nx = (w[0] + w[1]) * 0.5;
                let px = (32.0 + nx).floor() as i32;
                let nb = fringe_bottom(tips, cy, px);
                notches.push(px);
                let start = v(32.0 + nx * 0.35, cy - 12.0);
                let end = v(px as f32 + 0.5, nb - 0.5);
                cv.line(start, end, |t| if t < 0.4 { None } else if t > 0.8 { Some(h[1]) } else { Some(h[2]) });
            }
            // Dark underside right at each clump tip edge.
            for x in (32.0 - face_w) as i32..(32.0 + face_w) as i32 {
                let fb = fringe_bottom(tips, cy, x);
                let y = (fb - 0.5).floor() as i32;
                if cv.get(x, y).is_some() && fb > cy - 3.0 {
                    cv.put(x, y, h[2]);
                }
            }
            highlight_ring(cv, h, shine, mc, rx, ry, &notches);
            // Side locks framing the face.
            let (ll, lw) = k.lock;
            for s in [-1.0f32, 1.0] {
                let cxl = 32.0 + s * (rx - lw * 0.5);
                lock_strip(cv, h, cxl, cy - 4.0, cy + ll, lw, sway, s);
            }
            match style {
                3 => {
                    cv.begin();
                    cv.ellipse(v(32.0, cy - ry - 1.0), 5.5, 5.0, h, 0.05);
                }
                4 => spikes(cv, h, cy, ry, false),
                6 => {
                    for s in [-1.0f32, 1.0] {
                        let base = v(32.0 + s * (rx + 0.5), cy - 4.0);
                        cv.begin();
                        cv.ellipse(base.add(v(0.0, -1.0)), 2.4, 2.0, &r5(&ramp4(&pal::CLOTH[1])), 0.1);
                    }
                }
                _ => {}
            }
        }
    }
}

/// A side lock: straight strip with a pointed end, a strand line down the
/// middle and a darker face-side edge. `s` is -1 for the left lock.
#[allow(clippy::too_many_arguments)]
fn lock_strip(cv: &mut Canvas, h: &R5, cx: f32, top: f32, end: f32, w: f32, sway: f32, s: f32) {
    let len = (end - top).max(1.0);
    for y in top.floor() as i32..=end.ceil() as i32 {
        let fy = y as f32 + 0.5;
        if fy < top || fy > end {
            continue;
        }
        let t = (fy - top) / len;
        let ww = if t > 0.6 { w * (1.0 - (t - 0.6) / 0.4 * 0.9) } else { w };
        // The point swings outward with the lag.
        let c = cx + t * t * sway * 0.6 + if t > 0.6 { s * (t - 0.6) * 2.0 } else { 0.0 };
        for x in (c - ww * 0.5 - 1.0) as i32..=(c + ww * 0.5 + 1.0) as i32 {
            let px = x as f32 + 0.5;
            let u = (px - c) / (ww * 0.5).max(0.3);
            if u.abs() > 1.0 {
                continue;
            }
            let inner = u * -s > 0.45;
            let strand = u.abs() < 0.25 && t > 0.2 && t < 0.8 && w >= 3.5;
            cv.fill(x, y, h[if inner || strand { 2 } else { 3 }]);
        }
    }
}

/// The anime "angel ring": a broken arc of highlight across the crown.
fn highlight_ring(cv: &mut Canvas, h: &R5, shine: Rgb, mc: V2, rx: f32, ry: f32, skip: &[i32]) {
    for x in (mc.x - rx * 0.62) as i32..(mc.x + rx * 0.62) as i32 {
        if skip.contains(&x) || skip.contains(&(x + 1)) {
            continue;
        }
        let nx = (x as f32 + 0.5 - mc.x) / rx;
        let y = (mc.y - ry * 0.62 + nx * nx * ry * 0.45).floor() as i32;
        if cv.get(x, y) == Some(h[3]) {
            cv.put(x, y, shine);
            if nx.abs() < 0.4 && cv.get(x, y + 1) == Some(h[3]) {
                cv.put(x, y + 1, mix(shine, h[3], 0.5));
            }
        }
    }
}

fn spikes(cv: &mut Canvas, h: &R5, cy: f32, ry: f32, side: bool) {
    cv.begin();
    let n = if side { 4 } else { 5 };
    for k in 0..n {
        let x = if side { 22.0 + k as f32 * 4.0 } else { 22.0 + k as f32 * 5.0 };
        let base = v(x, cy - ry + 2.0);
        let lean = if side { -3.0 } else if k < 2 { -2.0 } else if k > 2 { 2.0 } else { 0.0 };
        let tip = base.add(v(lean, -5.5));
        tail_lock(cv, h, base, tip, 2.6);
    }
}

fn ears_horns(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
    let hc = b.head_c;
    let (rx, ry) = b.head_r;
    match look.race {
        RACE_ELF => {
            let sides: &[f32] = if dir == DIR_SIDE { &[-0.2] } else { &[-1.0, 1.0] };
            for &s in sides {
                let base = if dir == DIR_SIDE { v(30.0, hc.y + 3.0) } else { v(32.0 + s * (rx - 1.0), hc.y + 3.0) };
                let tip = base.add(v(if dir == DIR_SIDE { -7.5 } else { s * 8.0 }, -6.0));
                cv.begin();
                cv.capsule(base, tip, 2.2, 0.5, &c.skin, 0.0);
            }
        }
        RACE_BEAST => {
            let inner = if look.beast == 1 || look.beast == 2 {
                [hex(0xb09a88), hex(0xd8c4b0), hex(0xf0e2d0), hex(0xfaf2e6), hex(0xfffaf2)]
            } else {
                [hex(0xa8606a), hex(0xd08890), hex(0xf0a8b0), hex(0xf8c8cc), hex(0xffe0e4)]
            };
            let sides: &[f32] = if dir == DIR_SIDE { &[0.0] } else { &[-1.0, 1.0] };
            for &s in sides {
                let base = v(32.0 + s * rx * 0.6 - if dir == DIR_SIDE { 3.0 } else { 0.0 }, hc.y - ry * 0.8);
                let (len, w, tilt) = match look.beast {
                    3 => (15.0, 2.9, 0.6),
                    1 => (10.5, 5.0, 2.2),
                    2 => (9.0, 3.8, 2.2),
                    _ => (7.5, 3.8, 1.5),
                };
                let tip = base.add(v(s * tilt, -len));
                cv.begin();
                cv.capsule(base, tip, w, 0.7, &c.hair, 0.0);
                if dir == DIR_DOWN {
                    cv.capsule(base.add(v(0.0, -1.5)), tip.add(v(-s * 0.4, 3.0)), w * 0.45, 0.3, &inner, 0.0);
                }
                if look.beast == 1 {
                    cv.put(tip.x.floor() as i32, tip.y.floor() as i32, hex(0xf8f0e8));
                }
            }
        }
        RACE_DEMON => {
            let horn = [hex(0x1e1622), hex(0x3a2e40), hex(0x5a4a62), hex(0x857590), hex(0xb0a0ba)];
            let sides: &[f32] = if dir == DIR_SIDE { &[1.0] } else { &[-1.0, 1.0] };
            for &s in sides {
                let base = v(32.0 + s * rx * 0.55, hc.y - ry * 0.75);
                let mid = base.add(v(s * 3.5, -3.5));
                let tip = mid.add(v(s * 2.0, -5.5));
                cv.begin();
                cv.capsule(base, mid, 2.6, 1.9, &horn, 0.0);
                cv.capsule(mid, tip, 1.9, 0.5, &horn, 0.05);
            }
        }
        _ => {}
    }
}

fn tail(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
    let sway = b.lag;
    let base = v(b.cx + if dir == DIR_SIDE { -5.5 } else { 0.0 }, b.hip_y - 1.0);
    cv.begin();
    match look.race {
        RACE_BEAST => {
            if dir == DIR_DOWN && look.beast == 3 {
                return;
            }
            let dirx = if dir == DIR_SIDE { -1.0 } else { 0.6 };
            let white = [hex(0xc8bcb0), hex(0xe0d8ce), hex(0xf0ece6), hex(0xfaf8f4), hex(0xffffff)];
            match look.beast {
                3 => cv.ellipse(base.add(v(dirx * 2.5, 1.0)), 3.6, 3.4, &white, 0.0),
                1 => {
                    let mid = base.add(v(dirx * 8.5, 5.0 + sway));
                    let tip = mid.add(v(dirx * 6.0, -6.0 + sway));
                    cv.capsule(base, mid, 2.6, 5.2, &c.hair, -0.05);
                    cv.capsule(mid, tip, 5.2, 2.4, &c.hair, 0.0);
                    cv.ellipse(tip, 3.0, 2.8, &white, 0.0);
                }
                2 => {
                    let tip = base.add(v(dirx * 11.0, 7.0 + sway));
                    cv.capsule(base, tip, 2.4, 3.8, &c.hair, -0.05);
                }
                _ => {
                    let mid = base.add(v(dirx * 7.0, 2.5));
                    let tip = mid.add(v(dirx * 2.5, -10.0 + sway));
                    cv.capsule(base, mid, 1.8, 1.8, &c.hair, -0.05);
                    cv.capsule(mid, tip, 1.8, 1.4, &c.hair, 0.0);
                }
            }
        }
        RACE_DEMON => {
            if dir == DIR_DOWN {
                return;
            }
            let dirx = if dir == DIR_SIDE { -1.0 } else { 0.5 };
            let mid = base.add(v(dirx * 8.5, 7.0 + sway));
            let tip = mid.add(v(dirx * 3.5, -5.0));
            cv.capsule(base, mid, 1.4, 1.2, &c.skin, -0.1);
            cv.capsule(mid, tip, 1.2, 1.0, &c.skin, -0.1);
            let s = [c.skin[0], c.skin[0], c.skin[1], c.skin[2], c.skin[3]];
            cv.ellipse(tip.add(v(dirx, -1.0)), 2.4, 2.8, &s, 0.0);
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
    cv.begin();
    match look.hat {
        1 => {
            let straw = [hex(0x6a4c1e), hex(0x9a7630), hex(0xc8a24c), hex(0xe6c870), hex(0xf8e4a0)];
            cv.profile(top + 3.0, top + 6.0, |_| hc.x, |_| rx + 7.0, &straw, 0.05);
            cv.begin();
            cv.ellipse(v(hc.x, top + 1.0), rx * 0.78, 6.0, &straw, 0.08);
            for x in (hc.x - rx * 0.78) as i32..(hc.x + rx * 0.78) as i32 {
                cv.put(x, (top + 3.0) as i32, hex(0xa83a30));
                cv.put(x, (top + 4.0) as i32, hex(0x8a2a26));
            }
        }
        2 => {
            // Cloth tied over the crown, following the skull.
            let cc = v(hc.x - if side { 1.0 } else { 0.0 }, hc.y - 1.0);
            for y in (top - 3.0) as i32..(top + 7.0) as i32 {
                for x in (hc.x - rx - 2.0) as i32..=(hc.x + rx + 2.0) as i32 {
                    let nx = (x as f32 + 0.5 - cc.x) / (rx + 1.6);
                    let ny = (y as f32 + 0.5 - cc.y) / (ry + 1.5);
                    let d = nx * nx + ny * ny;
                    if d <= 1.0 {
                        cv.fill(x, y, hcol[tone(ny, (1.0 - d).sqrt(), 0.02)]);
                    }
                }
            }
            for x in (hc.x - rx) as i32..(hc.x + rx) as i32 {
                cv.tint(x, (top + 6.0) as i32, hcol[1]);
            }
            if side || dir == DIR_UP {
                let knot = v(hc.x - rx, top + 6.0);
                cv.capsule(knot, knot.add(v(-2.5, 7.0)), 1.9, 1.2, hcol, -0.05);
                cv.capsule(knot, knot.add(v(-4.0, 5.0)), 1.9, 1.2, hcol, -0.1);
            }
        }
        3 => {
            cv.ellipse(v(hc.x, top + 5.0), rx + 2.0, ry * 0.62, &c.metal, 0.06);
            cv.profile(top + 7.0, top + 10.0, |_| hc.x, |_| rx + 2.4, &c.metal, 0.0);
            if dir == DIR_DOWN {
                cv.capsule(v(hc.x, top + 8.0), v(hc.x, top + 15.0), 1.2, 1.0, &c.metal, 0.0);
            }
            let plume = [hex(0x5a1418), hex(0x8a2228), hex(0xb83a3a), hex(0xe05a50), hex(0xf88a78)];
            cv.begin();
            cv.capsule(v(hc.x, top - 1.0), v(hc.x - if side { 5.0 } else { 0.0 }, top - 7.0), 2.2, 1.4, &plume, 0.0);
        }
        4 => {
            let (x0, x1) = ((hc.x - rx - 3.0) as i32, (hc.x + rx + 3.0) as i32);
            for y in (top - 2.0) as i32..(hc.y + ry + 4.0) as i32 {
                for x in x0..=x1 {
                    let nx = (x as f32 + 0.5 - hc.x) / (rx + 3.0);
                    let ny = (y as f32 + 0.5 - hc.y) / (ry + 3.0);
                    let d = nx * nx + ny * ny;
                    if d > 1.0 {
                        continue;
                    }
                    let dy = y - hc.y.round() as i32;
                    let face_open = match dir {
                        DIR_DOWN => dy > -6 && (x as f32 + 0.5 - 32.0).abs() < face_hw(look, dy.min(14)) - 0.5,
                        DIR_SIDE => x > 33 && dy > -6,
                        _ => false,
                    };
                    if face_open {
                        continue;
                    }
                    let nz = (1.0 - d).sqrt();
                    cv.fill(x, y, hcol[tone(ny, nz, 0.0)]);
                }
            }
        }
        5 => {
            // A round crown sitting on the head, then a peak.
            let (x0, x1) = ((hc.x - rx - 2.0) as i32, (hc.x + rx + 2.0) as i32);
            let cc = v(hc.x - if side { 1.0 } else { 0.0 }, hc.y - 1.0);
            for y in (top - 3.0) as i32..(top + 9.0) as i32 {
                for x in x0..=x1 {
                    let nx = (x as f32 + 0.5 - cc.x) / (rx + 1.4);
                    let ny = (y as f32 + 0.5 - cc.y) / (ry + 1.4);
                    let d = nx * nx + ny * ny;
                    if d <= 1.0 {
                        cv.fill(x, y, hcol[tone(ny, (1.0 - d).sqrt(), 0.04)]);
                    }
                }
            }
            for x in (hc.x - 2.0) as i32..(hc.x + 2.0) as i32 {
                cv.put(x, (top - 2.0) as i32, hcol[1]);
            }
            cv.begin();
            if dir == DIR_DOWN {
                cv.profile(top + 8.0, top + 10.5, |_| hc.x, |y| rx * 0.9 - (y - top - 8.0) * 0.6, hcol, -0.12);
            } else if side {
                cv.capsule(v(hc.x + 6.0, top + 9.0), v(hc.x + rx + 4.5, top + 9.5), 1.6, 1.2, hcol, -0.1);
            }
        }
        6 => {
            cv.profile(top + 4.0, top + 6.5, |_| hc.x, |_| rx + 8.0, hcol, 0.0);
            let tip = v(hc.x + if side { -7.0 } else { 5.0 }, top - 18.0);
            cv.begin();
            cv.capsule(v(hc.x, top + 2.5), v(hc.x + 1.0, top - 7.0), rx * 0.8, rx * 0.5, hcol, 0.04);
            cv.capsule(v(hc.x + 1.0, top - 7.0), tip, rx * 0.5, 0.7, hcol, 0.06);
            cv.begin();
            cv.profile(top + 2.0, top + 4.5, |_| hc.x, |_| rx * 0.82, &gold5(), 0.0);
            cv.ellipse(v(hc.x, top + 3.0), 1.7, 1.7, &[hex(0x2a1050), hex(0x3a1a6a), hex(0x6a3ac0), hex(0xa070f0), hex(0xe8d8ff)], 0.1);
        }
        7 => {
            let f = pal::FLOWERS[look.hat_color as usize % pal::FLOWERS.len()];
            let pet = [mix(hex(f[0]), pal::INK, 0.3), hex(f[0]), hex(f[1]), hex(f[2]), mix(hex(f[2]), hex(0xffffff), 0.4)];
            let fc = v(hc.x + if side { -3.5 } else { rx * 0.62 }, top + 4.0);
            for (dx, dy) in [(-2.1, 0.0), (2.1, 0.0), (0.0, -2.1), (0.0, 2.1)] {
                cv.ellipse(fc.add(v(dx, dy)), 1.7, 1.7, &pet, 0.0);
            }
            cv.put(fc.x as i32, fc.y as i32, hex(0xf6d860));
        }
        8 => {
            let w = [hex(0x9a9290), hex(0xc8c0bc), hex(0xe8e2de), hex(0xf8f6f4), hex(0xffffff)];
            cv.profile(top + 2.0, top + 6.5, |_| hc.x, |_| rx * 0.85, &w, 0.0);
            cv.begin();
            cv.ellipse(v(hc.x, top - 4.5), rx * 0.98, 7.5, &w, 0.08);
        }
        9 => {
            let bc = v(hc.x + if side { -5.0 } else { rx * 0.58 }, top + 2.0);
            cv.ellipse(bc.add(v(-3.0, 0.0)), 2.9, 2.4, hcol, 0.05);
            cv.ellipse(bc.add(v(3.0, 0.0)), 2.9, 2.4, hcol, -0.05);
            cv.begin();
            cv.ellipse(bc, 1.2, 1.2, hcol, -0.2);
            cv.capsule(bc, bc.add(v(-1.8, 5.0)), 1.1, 0.8, hcol, -0.1);
            cv.capsule(bc, bc.add(v(1.8, 5.0)), 1.1, 0.8, hcol, -0.15);
        }
        10 => {
            let w = [hex(0xb8b0ae), hex(0xd8d2d0), hex(0xf0ecea), hex(0xfcfaf8), hex(0xffffff)];
            cv.profile(top + 1.5, top + 4.5, |_| hc.x, |_| rx * 0.88, &w, 0.0);
            for k in 0..7 {
                let x = hc.x - rx * 0.78 + k as f32 * rx * 0.26;
                cv.ellipse(v(x, top + 1.0), 1.7, 1.4, &w, 0.05);
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
