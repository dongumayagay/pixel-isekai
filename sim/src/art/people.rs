//! Procedural anime characters, constructed the way pixel artists build
//! hand-drawn sprites at this size (64×120 frames, adults ~92px, about three
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
//! Sheet layout: 3 directions (down, up, side-facing-right) × 12 poses (idle
//! breathing ×4, walk ×6, work ×2), then one blink frame.

use serde::{Deserialize, Serialize};

use super::canvas::Img;
use crate::palette::{self as pal, hex, mix, Rgb};
use crate::rng::Rng;

pub const FW: i32 = 64;
pub const FH: i32 = 120;
/// Poses per direction: idle breathing 0..=3, walk 4..=9, work 10..=11.
pub const FRAMES_PER_DIR: u16 = 12;
pub const IDLE_FRAMES: u16 = 4;
pub const WALK_FIRST: u16 = 4;
pub const WALK_FRAMES: u16 = 6;
pub const WORK_FIRST: u16 = 10;
pub const SHEET_FRAMES: u16 = FRAMES_PER_DIR * 3 + 1;
pub const PIVOT: (i32, i32) = (32, 116);

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
    pub const PEASANT: u8 = 0;
    pub const LABORER: u8 = 1;
    pub const FISHER: u8 = 2;
    pub const SMITH: u8 = 3;
    pub const COOK: u8 = 4;
    pub const MERCHANT: u8 = 5;
    pub const INNKEEPER: u8 = 6;
    pub const GUARD: u8 = 7;
    pub const KNIGHT: u8 = 8;
    pub const HERO: u8 = 9;
    pub const ADVENTURER: u8 = 10;
    pub const ROGUE: u8 = 11;
    pub const RANGER: u8 = 12;
    pub const MAGE: u8 = 13;
    pub const SCHOLAR: u8 = 14;
    pub const PRIEST: u8 = 15;
    pub const MONK: u8 = 16;
    pub const BARD: u8 = 17;
    pub const MAID: u8 = 18;
    pub const NOBLE: u8 = 19;
    pub const GUILD: u8 = 20;
    pub const DEMON: u8 = 21;
    pub const SWIM: u8 = 22;
    pub const COUNT: u8 = 23;

    pub const NAMES: [&str; COUNT as usize] = [
        "Peasant", "Labourer", "Fisher", "Smith", "Cook", "Merchant", "Innkeeper", "Guard", "Knight", "Hero", "Adventurer", "Rogue", "Ranger", "Mage", "Scholar", "Priest", "Monk", "Bard", "Maid / Butler", "Noble", "Guild clerk", "Demon lord", "Swimwear",
    ];

    /// What the primary, secondary and accent dyes colour on each outfit
    /// ("" = unused): (women, men).
    pub fn slots(o: u8, female: bool) -> [&'static str; 3] {
        match (o, female) {
            (PEASANT, true) => ["Dress", "Apron", "Kerchief"],
            (PEASANT, false) => ["Tunic", "Trousers", "Kerchief"],
            (LABORER, _) => ["Jerkin", "Shirt", "Kerchief"],
            (FISHER, true) => ["Oilskin cape", "Skirt", "Kerchief"],
            (FISHER, false) => ["Oilskin cape", "Trousers", "Kerchief"],
            (SMITH, _) => ["Apron", "Shirt", ""],
            (COOK, _) => ["Coat", "Apron", "Kerchief"],
            (MERCHANT, _) => ["Coat", "Shirt", "Sash"],
            (INNKEEPER, true) => ["Bodice", "Skirt", "Lacing"],
            (INNKEEPER, false) => ["Vest", "Trousers", "Kerchief"],
            (GUARD, _) => ["Tabard", "Trousers", "Stripe"],
            (KNIGHT, _) => ["Cape", "Skirt", "Trim"],
            (HERO, _) => ["Coat", "Shirt", "Trim"],
            (ADVENTURER, _) => ["Tunic", "Capelet", "Scarf"],
            (ROGUE, _) => ["Top", "Scarf", "Belt"],
            (RANGER, _) => ["Tunic", "Cloak", "Clasp"],
            (MAGE, true) => ["Dress", "Cape", "Sash"],
            (MAGE, false) => ["Robe", "Cape", "Sash"],
            (SCHOLAR, _) => ["Gown", "Capelet", "Clasp"],
            (PRIEST, _) => ["Robe", "Stole", "Sash"],
            (MONK, _) => ["Gi", "Trousers", "Sash"],
            (BARD, _) => ["Doublet", "Half-cape", "Clasp"],
            (MAID, true) => ["Dress", "Apron", "Ribbon"],
            (MAID, false) => ["Tailcoat", "Shirt", "Tie"],
            (NOBLE, true) => ["Gown", "Sleeves", "Jewel"],
            (NOBLE, false) => ["Frock coat", "Shirt", "Sash"],
            (GUILD, _) => ["Vest", "Blouse", "Bow"],
            (SWIM, true) => ["Bikini", "", "Ties"],
            (SWIM, false) => ["Trunks", "Waistband", ""],
            _ => ["Gown", "Lining", "Jewel"],
        }
    }

    /// The colours each outfit is best known by: (primary, secondary,
    /// accent) cloth indices.
    pub fn defaults(o: u8, female: bool) -> (u8, u8, u8) {
        const BLUE: u8 = 0;
        const RED: u8 = 1;
        const GREEN: u8 = 2;
        const BROWN: u8 = 3;
        const GREY: u8 = 4;
        const PURPLE: u8 = 5;
        const OCHRE: u8 = 6;
        const LINEN: u8 = 7;
        const CHARCOAL: u8 = 8;
        const TEAL: u8 = 9;
        const WHITE: u8 = 10;
        const BLACK: u8 = 11;
        const PINK: u8 = 12;
        const NAVY: u8 = 13;
        match o {
            PEASANT => if female { (GREEN, LINEN, RED) } else { (BROWN, CHARCOAL, RED) },
            LABORER => (BROWN, LINEN, RED),
            FISHER => (NAVY, GREY, RED),
            SMITH => (BROWN, CHARCOAL, RED),
            COOK => (WHITE, WHITE, RED),
            MERCHANT => (RED, LINEN, OCHRE),
            INNKEEPER => if female { (GREEN, BROWN, RED) } else { (BROWN, CHARCOAL, RED) },
            GUARD => (BLUE, CHARCOAL, OCHRE),
            KNIGHT => (RED, NAVY, OCHRE),
            HERO => (BLACK, GREY, TEAL),
            ADVENTURER => (GREEN, BROWN, RED),
            ROGUE => (CHARCOAL, BLACK, OCHRE),
            RANGER => (GREEN, BROWN, OCHRE),
            MAGE => (PURPLE, BLACK, OCHRE),
            SCHOLAR => (NAVY, LINEN, OCHRE),
            PRIEST => (WHITE, BLUE, OCHRE),
            MONK => (OCHRE, BROWN, RED),
            BARD => (TEAL, PURPLE, OCHRE),
            MAID => (BLACK, WHITE, RED),
            NOBLE => if female { (PINK, WHITE, OCHRE) } else { (NAVY, WHITE, RED) },
            GUILD => (NAVY, WHITE, RED),
            SWIM => if female { (PINK, WHITE, WHITE) } else { (BLUE, WHITE, WHITE) },
            _ => (BLACK, RED, PURPLE),
        }
    }
}

pub mod held {
    pub const NONE: u8 = 0;
    pub const STAFF: u8 = 1;
    pub const HAMMER: u8 = 2;
    pub const MUG: u8 = 3;
    pub const BOOK: u8 = 4;
    pub const SWORD: u8 = 5;
    pub const BASKET: u8 = 6;
    pub const AXE: u8 = 7;
    pub const PICKAXE: u8 = 8;
    pub const HOE: u8 = 9;
    pub const ROD: u8 = 10;
    pub const SPEAR: u8 = 11;
    pub const LADLE: u8 = 12;
    pub const COUNT: u8 = 13;
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Look {
    pub skin: u8,
    pub hair: u8,
    pub hair_style: u8,
    /// Outfit dyes: the main garment, the second garment (or trim), and a
    /// small accent (ribbons, sashes, gems, the hat). Which piece takes
    /// which is up to each outfit.
    #[serde(alias = "top")]
    pub primary: u8,
    #[serde(alias = "bottom")]
    pub secondary: u8,
    pub outfit: u8,
    /// 0 none, 1 straw hat, 2 bandana, 3 helmet, 4 hood, 5 cap, 6 witch hat,
    /// 7 flower, 8 chef toque, 9 ribbon, 10 maid headband, 11 miner helmet
    pub hat: u8,
    #[serde(alias = "hat_color")]
    pub accent: u8,
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
    /// Bust size for adult women: 0 flat, 1 small, 2 medium, 3 large, 4 huge.
    #[serde(default = "default_figure")]
    pub figure: u8,
    /// Body build. Women: 0 slim, 1 standard, 2 curvy, 3 thick.
    /// Men: 0 slim, 1 standard, 2 muscular, 3 heavy.
    #[serde(default = "default_build")]
    pub build: u8,
    /// 0 short, 1 medium, 2 tall.
    #[serde(default = "default_height")]
    pub height: u8,
    /// Facial hair for grown men (see [`FACIAL`]); 0 falls back to `beard`.
    #[serde(default)]
    pub facial: u8,
}

pub const FACIAL: [&str; 8] = ["None", "Stubble", "Hero mustache", "Goatee", "Short beard", "Full beard", "Sage beard", "Crescent mustache"];

impl Look {
    /// Facial hair style actually drawn: grown men only; the legacy `beard`
    /// flag means a full beard (a long one on dwarves).
    pub fn facial_style(&self) -> u8 {
        if self.female || !self.mature {
            0
        } else if self.facial > 0 {
            self.facial.min(FACIAL.len() as u8 - 1)
        } else if self.beard {
            if self.race == RACE_DWARF { 6 } else { 5 }
        } else {
            0
        }
    }
}

fn default_height() -> u8 {
    1
}

pub const HEIGHTS: [&str; 3] = ["Short", "Medium", "Tall"];

fn default_figure() -> u8 {
    2
}

fn default_build() -> u8 {
    1
}

pub const FIGURES: [&str; 5] = ["Flat", "Small", "Medium", "Large", "Huge"];
pub const BUILDS_F: [&str; 4] = ["Slim", "Standard", "Curvy", "Thick"];
pub const BUILDS_M: [&str; 4] = ["Slim", "Standard", "Muscular", "Heavy"];

pub const HAIR_STYLES: u8 = 19;
pub const HAIR_STYLE_NAMES: [&str; HAIR_STYLES as usize] = [
    "Short", "Long", "Ponytail", "Bun", "Spiky", "Bob", "Twin tails", "Side-swept", "Hime cut", "Drills", "Twin braids", "Side braid", "Very long", "Messy", "Low tie", "Slicked back", "Undercut", "Samurai", "Bald",
];
pub const HATS: u8 = 12;

impl Look {
    pub fn random(rng: &mut Rng, female: bool, age: u8) -> Look {
        let hair = if rng.chance(0.12) { 6 + rng.below(3) as u8 } else { rng.below(5) as u8 };
        let styles_f: &[u8] = &[1, 2, 3, 5, 6, 7, 8, 9, 10, 11, 12, 1, 5, 2];
        let styles_m: &[u8] = if age == 2 { &[0, 15, 18, 18, 14] } else { &[0, 4, 7, 13, 14, 15, 16, 17, 0, 13] };
        // Townsfolk clothes in their usual colours, often re-dyed.
        let o = if female { *rng.pick(&[outfit::PEASANT, outfit::PEASANT, outfit::INNKEEPER, outfit::MERCHANT, outfit::COOK]) } else { *rng.pick(&[outfit::PEASANT, outfit::LABORER, outfit::PEASANT, outfit::MERCHANT]) };
        let (p, s, a) = outfit::defaults(o, female);
        let dyes = [0u8, 1, 2, 3, 4, 5, 8, 9, 13];
        Look {
            skin: rng.below(pal::SKIN.len()) as u8,
            hair: if age == 2 { 5 } else { hair },
            hair_style: if female { *rng.pick(styles_f) } else { *rng.pick(styles_m) },
            primary: if rng.chance(0.5) { p } else { *rng.pick(&dyes) },
            secondary: s,
            outfit: o,
            hat: 0,
            accent: if rng.chance(0.6) { a } else { *rng.pick(&dyes) },
            female,
            age,
            beard: !female && age >= 1 && rng.chance(if age == 2 { 0.6 } else { 0.2 }),
            eyes: rng.below(EYE_COLORS.len()) as u8,
            mature: age >= 1,
            race: RACE_HUMAN,
            beast: 0,
            held: held::NONE,
            glasses: false,
            figure: *rng.pick(&[1u8, 2, 2, 2, 3, 3, 4]),
            build: *rng.pick(&[0u8, 1, 1, 1, 2, 2, 3]),
            height: *rng.pick(&[0u8, 1, 1, 1, 2]),
            facial: if !female && age >= 1 && rng.chance(if age == 2 { 0.7 } else { 0.4 }) { 1 + rng.below(FACIAL.len() - 1) as u8 } else { 0 },
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

pub const EYE_COLORS: [[u32; 3]; 7] = [
    [0x1e2a6a, 0x3a64c8, 0x8ab8ff],
    [0x14382a, 0x2c8050, 0x78d098],
    [0x3a220f, 0x7a4c20, 0xc89450],
    [0x341a5a, 0x7a3ab8, 0xc490f0],
    [0x4a121e, 0xb02c36, 0xf88068],
    [0x3a2c0c, 0xa88018, 0xf8d460],
    [0x163a44, 0x2a8a98, 0x7ae0e0],
];

/// Inside-test tolerance, so shapes mirrored across the centre line include
/// exactly the same boundary pixels despite float rounding.
const EPS: f32 = 1e-3;

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
    /// Ramps in use, and per pixel which ramp/step its colour came from
    /// (255 = a one-off detail colour). Contours and outlines step down the
    /// pixel's own ramp, so the palette stays fixed and hue-matched.
    ramps: Vec<R5>,
    rid: Vec<u8>,
    ridx: Vec<u8>,
    /// Mirror group per part id (0 none, 1 left, 2 right): where a left
    /// part meets a right one (legs, shoes, bust) the seam is shaded on both
    /// sides so front views stay symmetric.
    groups: Vec<u8>,
    side: u8,
    /// Optional column range a part may paint (keeps paired limbs on their
    /// own side of the centre line).
    clip: Option<(i32, i32)>,
}

impl Canvas<'_> {
    /// Start a new part: its edges get contour lines where it overlaps
    /// anything drawn before.
    fn begin(&mut self) {
        self.cur = self.cur.saturating_add(1);
        self.groups[self.cur as usize] = self.side;
    }

    fn register(&mut self, r: &R5) {
        if !self.ramps.contains(r) && self.ramps.len() < 250 {
            self.ramps.push(*r);
        }
    }

    fn mark(&mut self, i: usize, c: Rgb) {
        let found = self.ramps.iter().enumerate().find_map(|(k, r)| r.iter().position(|&q| q == c).map(|j| (k as u8, j as u8)));
        let (r, j) = found.unwrap_or((255, 0));
        self.rid[i] = r;
        self.ridx[i] = j;
    }

    /// Detail pixel: recolours without changing which part owns it.
    fn put(&mut self, x: i32, y: i32, c: Rgb) {
        if self.img.inside(x, y) {
            self.img.set(x, y, c);
            self.mark((y * self.img.w + x) as usize, c);
        }
    }

    /// Body pixel owned by the current part.
    fn fill(&mut self, x: i32, y: i32, c: Rgb) {
        if let Some((x0, x1)) = self.clip {
            if x < x0 || x > x1 {
                return;
            }
        }
        if self.img.inside(x, y) {
            self.img.set(x, y, c);
            let i = (y * self.img.w + x) as usize;
            self.ids[i] = self.cur;
            self.mark(i, c);
        }
    }

    fn get(&self, x: i32, y: i32) -> Option<Rgb> {
        self.img.get(x, y)
    }

    /// Step a pixel one shade darker if it was painted from `ramp`.
    fn darken_on(&mut self, x: i32, y: i32, ramp: &R5) {
        if !self.img.inside(x, y) {
            return;
        }
        let i = (y * self.img.w + x) as usize;
        let r = self.rid[i] as usize;
        if r < self.ramps.len() && self.ramps[r] == *ramp && self.ridx[i] > 1 {
            let c = ramp[self.ridx[i] as usize - 1];
            self.put(x, y, c);
        }
    }

    /// Whether the current part painted this pixel.
    fn owns(&self, x: i32, y: i32) -> bool {
        self.img.inside(x, y) && self.ids[(y * self.img.w + x) as usize] == self.cur
    }

    /// Copy every part from `first` to the current one across the centre
    /// line as new parts, so paired pieces (drills, braids) mirror exactly.
    fn mirror_from(&mut self, first: u8) {
        let last = self.cur;
        let w = self.img.w;
        let side = self.side;
        for id in first..=last {
            // Left/right pair: where they meet, the seam shades both sides.
            self.groups[id as usize] = 1;
            self.side = 2;
            let px: Vec<(i32, i32, Rgb)> = (0..self.img.h)
                .flat_map(|y| (0..w).map(move |x| (x, y)))
                .filter(|&(x, y)| self.ids[(y * w + x) as usize] == id)
                .filter_map(|(x, y)| self.img.get(x, y).map(|c| (x, y, c)))
                .collect();
            self.begin();
            for (x, y, c) in px {
                self.fill(FW - 1 - x, y, c);
            }
        }
        self.side = side;
    }

    /// Filled ellipse lit as a dome.
    fn ellipse(&mut self, c: V2, rx: f32, ry: f32, ramp: &R5, bias: f32) {
        self.register(ramp);
        let (x0, x1) = ((c.x - rx).floor() as i32, (c.x + rx).ceil() as i32);
        let (y0, y1) = ((c.y - ry).floor() as i32, (c.y + ry).ceil() as i32);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let nx = (x as f32 + 0.5 - c.x) / rx;
                let ny = (y as f32 + 0.5 - c.y) / ry;
                let d = nx * nx + ny * ny;
                if d <= 1.0 + EPS {
                    let nz = (1.0 - d).max(0.0).sqrt();
                    self.fill(x, y, ramp[tone(ny, nz, bias)]);
                }
            }
        }
    }

    /// Tapered capsule from a (radius ra) to b (radius rb), lit as a cylinder.
    fn capsule(&mut self, a: V2, b: V2, ra: f32, rb: f32, ramp: &R5, bias: f32) {
        self.register(ramp);
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
                if ox * ox + oy * oy <= rr * rr + EPS {
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
        self.register(ramp);
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
                if nx.abs() > 1.0 + EPS {
                    continue;
                }
                let nz = (1.0 - nx * nx).max(0.0).sqrt();
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
            self.put(x, y, c);
        }
    }

    /// Pixel line (DDA) drawn as detail.
    fn line(&mut self, a: V2, b: V2, mut col: impl FnMut(f32) -> Option<Rgb>) {
        let n = (b.x - a.x).abs().max((b.y - a.y).abs()).ceil().max(1.0) as i32;
        // Interpolate relative to the centre line on a fine grid: endpoints
        // computed as 32 +/- d carry different rounding errors, and mirrored
        // lines must land on mirrored pixels.
        let snap = |q: f32| ((q - 32.0) * 256.0).round() / 256.0;
        let (ax, bx) = (snap(a.x), snap(b.x));
        for i in 0..=n {
            let t = i as f32 / n as f32;
            let (x, y) = (32.0 + (ax + (bx - ax) * t), a.y + (b.y - a.y) * t);
            if let Some(c) = col(t) {
                // Nudge toward the centre line so mirrored lines that land on
                // exact integers round to mirrored pixels.
                let xs = x + if x < 32.0 { 1e-3 } else { -1e-3 };
                self.tint(xs.floor() as i32, y.floor() as i32, c);
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

/// Detail row centred on the frame's mirror line, so it stays symmetric.
fn hspan(cv: &mut Canvas, half: f32, y: i32, c: Rgb) {
    let x0 = (32.0 - half).round() as i32;
    for x in x0..FW - x0 {
        cv.put(x, y, c);
    }
}

/// An authored stamp painted as part of the current body part (so it joins
/// the contour pass), mirrored across the centre line when asked.
fn stamp_fill(cv: &mut Canvas, x0: i32, y0: i32, rows: &[&str], mirror: bool, pal: &impl Fn(char) -> Option<Rgb>) {
    for (r, row) in rows.iter().enumerate() {
        for (i, ch) in row.chars().enumerate() {
            if let Some(c) = pal(ch) {
                let x = x0 + i as i32;
                cv.fill(if mirror { FW - 1 - x } else { x }, y0 + r as i32, c);
            }
        }
    }
}

/// A body-part stamp flipped horizontally in place (not across the frame's
/// centre), for profile views where the piece must face forward.
fn stamp_at(cv: &mut Canvas, x0: i32, y0: i32, rows: &[&str], flip: bool, pal: &impl Fn(char) -> Option<Rgb>) {
    for (r, row) in rows.iter().enumerate() {
        let w = row.len() as i32;
        for (i, ch) in row.chars().enumerate() {
            if let Some(c) = pal(ch) {
                let x = if flip { x0 + w - 1 - i as i32 } else { x0 + i as i32 };
                cv.fill(x, y0 + r as i32, c);
            }
        }
    }
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
    /// Walk cycle angle (radians, one full stride pair = 2π).
    walk: Option<f32>,
    /// Inhale amount 0..1 for idle breathing.
    breathe: f32,
    work: u8,
    blink: bool,
    /// 0 idle, 1 walk, 2 work, and the sheet frame index.
    cycle: u8,
    frame: u16,
}

fn pose_for(f: u16) -> Pose {
    use std::f32::consts::TAU;
    let base = Pose { walk: None, breathe: 0.0, work: 0, blink: false, cycle: 0, frame: f };
    match f {
        // In for two frames, out for two: a held breath, not a twitch.
        0..=3 => Pose { breathe: if f >= 2 { 1.0 } else { 0.0 }, ..base },
        4..=9 => {
            let ph = (f - WALK_FIRST) as f32 / WALK_FRAMES as f32;
            Pose { walk: Some(TAU * ph + 0.5), cycle: 1, ..base }
        }
        10 => Pose { work: 1, cycle: 2, ..base },
        11 => Pose { work: 2, cycle: 2, ..base },
        _ => Pose { blink: true, ..base },
    }
}

/// Bust secondary motion. Researched model (2D rig physics, breast
/// biomechanics, the hand-animated reference): in a looping walk the bust is
/// a forced follower that trails the torso by one frame — its height equals
/// the torso's height a frame earlier — so with our bob (-1,-2,-1 per step)
/// the offset relative to the chest is [0, +1, -1] per step. Real walking
/// moves ~1.3px at this scale, so medium only drags, large follows the lag
/// rule, huge adds overshoot. Shape steps are whole pixels and follow the
/// lag (lagging down = stretch, pulled up = squash); the silhouette never
/// shimmers. Idle follows the breath (the chest lifts on frame 2).
/// Entries are (offset px, + = down; shape -1 squash / +1 stretch).
type Jig = (i32, i32);
const JIGGLE_WALK: [[Jig; 6]; 5] = [
    [(0, 0); 6],
    [(0, 0); 6],
    [(0, 0), (1, 0), (0, 0), (0, 0), (1, 0), (0, 0)],
    [(0, 0), (1, 1), (-1, -1), (0, 0), (1, 1), (-1, -1)],
    [(-1, -1), (1, 1), (-1, -1), (-1, -1), (1, 1), (-1, -1)],
];
const JIGGLE_IDLE: [[Jig; 4]; 5] = [
    [(0, 0); 4],
    [(0, 0); 4],
    [(0, 0), (0, 0), (1, 0), (0, 0)],
    [(-1, -1), (0, 0), (1, 1), (0, 0)],
    [(-1, -1), (0, 0), (1, 1), (0, 0)],
];

fn jiggle(figure: u8, pose: &Pose, frame: u16) -> Jig {
    jiggle_at(figure, pose, frame, 0)
}

/// The table entry `back` frames earlier in the same cycle.
fn jiggle_at(figure: u8, pose: &Pose, frame: u16, back: u16) -> Jig {
    let f = figure.min(4) as usize;
    match pose.cycle {
        1 => JIGGLE_WALK[f][((frame - WALK_FIRST + WALK_FRAMES - back) % WALK_FRAMES) as usize],
        0 => JIGGLE_IDLE[f][((frame + IDLE_FRAMES - back) % IDLE_FRAMES) as usize],
        _ => (0, 0),
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
    /// Adult woman: curvy hips, seat and fitted clothes (independent of bust).
    woman: bool,
    /// Lag applied to soft parts (hair, skirts) this frame.
    lag: f32,
    /// Bust offset this frame (whole px, down positive) and shape step
    /// (-1 squash, +1 stretch).
    jig: f32,
    jsq: f32,
    /// Highlight lag relative to the bust (whole px).
    jhl: f32,
    /// Sideways swing of hair, skirts and capes: side view only. Seen from
    /// the front or back, soft parts bounce vertically instead.
    sway: f32,
    /// Authored walk bob before the feet are planted.
    bob: f32,
}

fn build(look: &Look, pose: Pose, plant: f32) -> Build {
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
    // Height: legs carry most of it, the torso a little.
    let (hl, ht) = if child { (1.0, 1.0) } else { [(0.92, 0.96), (1.0, 1.0), (1.07, 1.03)][look.height.min(2) as usize] };
    let (leg_k, torso_k) = (leg_k * hl, torso_k * ht);
    // Long legs: just under half the standing height for adults.
    let thigh = 14.0 * leg_k;
    let calf = 13.5 * leg_k;
    let ground = 116.0;
    // Work poses keep the feet planted; only soft parts lag.
    let (bob, lag) = match pose.walk {
        Some(ph) => (-(ph.sin().abs()) * 1.5, -ph.cos() * 1.4),
        None if pose.work == 1 => (0.0, -0.9),
        None if pose.work == 2 => (0.0, 1.1),
        None => (0.0, 0.0),
    };
    // Whole-pixel motion: fractional bobbing makes every frame re-rasterise
    // differently (shimmering outlines), so the body moves in pixel steps.
    let hip_joint_y = ground - 3.0 - calf - thigh + (bob + plant).round();
    let crotch_y = hip_joint_y + 2.0;
    // Breathing lifts the chest and head; hips and feet stay put.
    let torso = if f { 30.0 } else { 31.0 } * torso_k + pose.breathe.floor();
    let shoulder_y = crotch_y - torso;
    let head_c = v(32.0, shoulder_y - 18.0);
    let (jig, jsq) = jiggle(look.figure, &pose, pose.frame);
    // The highlight trails the bust by another frame: softness without
    // moving the outline.
    let jhl = jiggle_at(look.figure, &pose, pose.frame, 1).0 - jig;
    let fig = f && look.mature && !child;
    let bd = look.build.min(3) as usize;
    // Hourglass for women (waist about 0.6 of the hips), V-taper for men;
    // each build scales shoulders, chest, waist, hips, thighs and arms.
    let (sw, chest, waist, hip, thr, arm) = if f {
        [(8.6, 8.8, 6.0, 10.2, 5.0, 2.5), (9.0, 9.2, 6.6, 11.2, 5.6, 2.7), (9.2, 9.4, 6.8, 12.4, 6.2, 2.8), (9.8, 10.2, 8.2, 13.2, 6.8, 3.1)][bd]
    } else {
        [(11.0, 10.4, 8.4, 9.2, 4.7, 3.0), (12.0, 11.5, 9.0, 9.8, 5.0, 3.4), (13.4, 13.0, 9.6, 10.2, 5.8, 4.1), (12.6, 13.0, 12.2, 11.4, 6.0, 3.7)][bd]
    };
    let bust: f32 = if fig {
        [0.0f32, 4.2, 5.4, 6.4, 7.2][look.figure.min(4) as usize]
    } else {
        0.0
    };

    Build {
        cx: 32.0,
        ground,
        head_r: (13.0, 14.0),
        head_c,
        shoulder_y,
        shoulder_w: sw * width_k,
        chest_w: chest * width_k,
        waist_y: shoulder_y + torso * 0.62,
        waist_w: waist * width_k,
        hip_y: shoulder_y + torso * 0.87,
        hip_w: if fig || !f { hip } else { hip - 1.5 } * width_k,
        crotch_y,
        hip_joint_y,
        thigh,
        calf,
        thigh_r: if child { 4.4 } else if f && !fig { thr - 0.6 } else { thr } * width_k.max(0.9),
        calf_r: if child { 3.2 } else if f { 3.7 } else { 4.0 } * width_k.max(0.9),
        // Dwarves are broad, but their arms stay in proportion to their
        // short legs: hands at mid-thigh, not the knee.
        upper_arm: 12.0 * torso_k * if dwarf { 0.86 } else { 1.0 },
        forearm: 11.0 * torso_k * if dwarf { 0.86 } else { 1.0 },
        arm_r: if child { 2.6 } else { arm } * width_k.clamp(0.9, 1.06),
        bust,
        woman: fig,
        lag,
        jig: if bust > 0.0 { jig as f32 } else { 0.0 },
        jsq: if bust > 0.0 { jsq as f32 } else { 0.0 },
        jhl: if bust > 0.0 { jhl as f32 } else { 0.0 },
        sway: 0.0,
        bob,
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
    let walk_ang = pose.walk.unwrap_or(0.0);
    let leg = |hip: V2, sgn: f32, swing: f32, lift: f32| -> (V2, V2) {
        if side {
            if !walking {
                return (hip.add(v(0.0, b.thigh)), hip.add(v(0.0, b.thigh + b.calf)));
            }
            // Gait phase per leg (0 = heel strike). Stance (60%): the planted
            // leg sweeps back. Swing (40%): the knee bends most mid-swing as
            // the leg comes forward; it never kicks up while trailing.
            use std::f32::consts::{PI, TAU};
            let p = ((walk_ang + if sgn < 0.0 { 0.0 } else { PI }) / TAU).rem_euclid(1.0);
            let th = 0.38 * (TAU * p).cos();
            let flex = if p > 0.6 { 0.75 * (PI * (p - 0.6) / 0.4).sin() } else { 0.08 * (PI * p / 0.6).sin() };
            let knee = hip.add(V2::polar(b.thigh, th));
            let ankle = knee.add(V2::polar(b.calf, th - flex));
            let _ = (swing, lift);
            (knee, ankle)
        } else {
            // Knees drawn slightly together: the soft anime stance. Feet are
            // placed from the ground up, so a dip of the hips bends the legs
            // instead of sinking the feet.
            let ankle_y = b.ground - 3.0 - lift * 3.9;
            let reach = ankle_y - hip.y + lift * 3.9;
            let knee = v(hip.x - sgn * 1.2, hip.y + reach * b.thigh / (b.thigh + b.calf) - lift * 1.3);
            let ankle = v(knee.x - sgn * 0.3, ankle_y);
            (knee, ankle)
        }
    };
    let (sl, sr) = if walking { (s, -s) } else { (0.0, 0.0) };
    let lift_l = if walking { s.max(0.0) * 2.0 } else { 0.0 };
    let lift_r = if walking { (-s).max(0.0) * 2.0 } else { 0.0 };
    let (knee_l, ankle_l) = leg(hip_l, -1.0, sl, lift_l);
    let (knee_r, ankle_r) = leg(hip_r, 1.0, sr, lift_r);
    let sh_dx = if side { 0.0 } else { b.shoulder_w - 0.6 };
    // The arm's ball sits just under the shoulder line so the deltoid reads
    // as a one-pixel rounding, not a pad poking above the slope.
    let sh_y = b.shoulder_y + 2.0 + b.arm_r * 0.6;
    let sh_l = v(b.cx - sh_dx, sh_y);
    let sh_r = v(b.cx + sh_dx, sh_y);
    let arm = |sh: V2, swing: f32, out: f32| -> (V2, V2) {
        match pose.work {
            1 => {
                // Wind-up: in profile the hands go up beside the back of the
                // head, clear of the face.
                if side {
                    let el = sh.add(v(-0.5, -b.upper_arm * 0.95));
                    let wr = el.add(v(-1.5, -b.forearm * 0.9));
                    return (el, wr);
                }
                let el = sh.add(v(out * 2.5, -b.upper_arm * 0.85));
                let wr = el.add(v(out * 1.0, -b.forearm * 0.8));
                (el, wr)
            }
            2 => {
                if side {
                    let el = sh.add(V2::polar(b.upper_arm, 1.0));
                    let wr = el.add(V2::polar(b.forearm, 1.25));
                    (el, wr)
                } else {
                    // Follow-through: arms swept down and out past the hips,
                    // the tool at the low point beside the body.
                    let el = sh.add(v(out * 3.6, b.upper_arm * 0.92));
                    let wr = el.add(v(out * 1.8, b.forearm * 0.88));
                    (el, wr)
                }
            }
            _ => {
                if side {
                    let el = sh.add(V2::polar(b.upper_arm, swing * 0.45));
                    let wr = el.add(V2::polar(b.forearm, swing * 0.45 + 0.25 * swing.abs() + 0.12));
                    (el, wr)
                } else {
                    // A relaxed A-pose: elbows and hands clear the hips. In a
                    // walk the forward arm foreshortens (hand rises and tucks
                    // in) and the trailing arm hangs straighter.
                    let el = sh.add(v(out * 4.2, b.upper_arm - 0.6 - swing.abs() * 1.2));
                    let wr = el.add(v(out * (2.6 - swing.max(0.0) * 1.2), b.forearm - swing.max(0.0) * 3.2));
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
    primary: R5,
    secondary: R5,
    accent: R5,
    white: R5,
    drab: R5,
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
        primary: r5(&ramp4(&pal::CLOTH[look.primary as usize % pal::CLOTH.len()])),
        secondary: r5(&ramp4(&pal::CLOTH[look.secondary as usize % pal::CLOTH.len()])),
        accent: r5(&ramp4(&pal::CLOTH[look.accent as usize % pal::CLOTH.len()])),
        white: r5(&ramp4(&pal::CLOTH[10])),
        drab: r5(&ramp4(&pal::CLOTH[8])),
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
    let look = &look.normalized();
    let mut img = Img::new(FW, FH);
    let pose = pose_for(frame_idx);
    // Solve the body bob so the lowest foot always rests on the ground.
    let b0 = build(look, pose, 0.0);
    let l0 = limbs(&b0, pose, dir);
    let low = l0.ankle_l.y.max(l0.ankle_r.y);
    // In profile the legs' angles set the bob; from the front or back the
    // standing leg is straight, so the hips dip a pixel at each contact and
    // the legs take it up.
    let plant = match pose.walk {
        Some(ph) if dir != DIR_SIDE => -b0.bob + (1.0 - ph.sin().abs()) * 1.2,
        Some(_) => b0.ground - 3.0 - low,
        None => 0.0,
    };
    let mut b = build(look, pose, plant);
    b.sway = if dir == DIR_SIDE { b.lag } else { 0.0 };
    let l = limbs(&b, pose, dir);
    let c = colors(look);
    let n = (FW * FH) as usize;
    let mut cv = Canvas { img: &mut img, ids: vec![0; n], cur: 0, ramps: Vec::new(), rid: vec![255; n], ridx: vec![0; n], groups: vec![0; 256], side: 0, clip: None };
    for r in [c.skin, c.hair, c.primary, c.secondary, c.accent, c.white, c.drab, c.metal, c.leather, c.linen, c.shoe, c.stocking, c.navy, gold5()] {
        cv.register(&r);
    }
    let side = dir == DIR_SIDE;
    let back = dir == DIR_UP;

    if !back {
        tail(&mut cv, look, dir, &b, &c);
    }
    let cape = garb(look).cape();
    if let (Some((_, lining)), false) = (cape, back) {
        cloak_back(&mut cv, &b, &dye(&c, lining), side, true);
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
        // Arms raised to work come up in front of the hair (drawn later).
        if pose.work != 1 {
            arm(&mut cv, look, &b, &c, (l.sh_l, l.el_l, l.wr_l), false, dir, pose);
            arm(&mut cv, look, &b, &c, (l.sh_r, l.el_r, l.wr_r), false, dir, pose);
        }
        leg(&mut cv, look, &b, &c, left, false, dir);
        leg(&mut cv, look, &b, &c, right, false, dir);
    }
    torso(&mut cv, look, dir, &b, &c);
    let late_arms = |cv: &mut Canvas| {
        arm(cv, look, &b, &c, (l.sh_l, l.el_l, l.wr_l), false, dir, pose);
        arm(cv, look, &b, &c, (l.sh_r, l.el_r, l.wr_r), false, dir, pose);
    };
    // Raised arms come up in front of the head and hair (drawn later).
    if side && pose.work != 1 {
        arm(&mut cv, look, &b, &c, (l.sh_r, l.el_r, l.wr_r), false, dir, pose);
    }
    head(&mut cv, look, dir, &b, &c);
    if !back {
        face(&mut cv, look, dir, &b, &c, pose);
    }
    elf_ears(&mut cv, look, dir, &b, &c, false);
    beast_ears(&mut cv, look, dir, &b, &c);
    hair_front(&mut cv, look, dir, &b, &c);
    if !side && pose.work == 1 {
        late_arms(&mut cv);
    } else if side && pose.work == 1 {
        arm(&mut cv, look, &b, &c, (l.sh_r, l.el_r, l.wr_r), false, dir, pose);
    }
    elf_ears(&mut cv, look, dir, &b, &c, true);
    horns(&mut cv, look, dir, &b, &c);
    // From behind a cape covers the tail.
    if back && cape.is_none() {
        tail(&mut cv, look, dir, &b, &c);
    }
    hat(&mut cv, look, dir, &b, &c);
    let ids = std::mem::take(&mut cv.ids);
    let ramps = std::mem::take(&mut cv.ramps);
    let rid = std::mem::take(&mut cv.rid);
    let ridx = std::mem::take(&mut cv.ridx);
    let groups = std::mem::take(&mut cv.groups);
    drop(cv);
    let shades = Shades { ramps: &ramps, rid: &rid, ridx: &ridx };
    contours(&mut img, &ids, &groups, &shades);
    outline(&mut img, &shades);
    close_pinholes(&mut img);
    img
}

/// Per-pixel ramp provenance, for picking contour and outline shades.
struct Shades<'a> {
    ramps: &'a [R5],
    rid: &'a [u8],
    ridx: &'a [u8],
}

impl Shades<'_> {
    /// Contour shade: `steps` down the pixel's own ramp.
    fn contour(&self, i: usize, c: Rgb, steps: u8) -> Rgb {
        match self.ramps.get(self.rid[i] as usize) {
            Some(r) if self.ridx[i] >= steps => r[(self.ridx[i] - steps) as usize],
            Some(r) => mix(r[0], CONTOUR, 0.35),
            None => mix(c, CONTOUR, 0.5),
        }
    }

    /// Outline shade: the ramp's darkest step pulled toward ink.
    fn outline(&self, i: usize, c: Rgb) -> Rgb {
        match self.ramps.get(self.rid[i] as usize) {
            Some(r) => mix(r[0], pal::INK, 0.55),
            None => mix(c, pal::INK, 0.72),
        }
    }
}

/// Close tiny enclosed gaps (slivers between outlines) with the darkest
/// neighbouring colour; they read as noise rather than negative space.
fn close_pinholes(img: &mut Img) {
    let (w, h) = (img.w, img.h);
    let mut outside = vec![false; (w * h) as usize];
    let mut stack: Vec<(i32, i32)> = Vec::new();
    for x in 0..w {
        stack.push((x, 0));
        stack.push((x, h - 1));
    }
    for y in 0..h {
        stack.push((0, y));
        stack.push((w - 1, y));
    }
    while let Some((x, y)) = stack.pop() {
        if !img.inside(x, y) || img.alpha(x, y) != 0 || outside[(y * w + x) as usize] {
            continue;
        }
        outside[(y * w + x) as usize] = true;
        stack.extend([(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]);
    }
    let mut seen = vec![false; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            if outside[i] || seen[i] || img.alpha(x, y) != 0 {
                continue;
            }
            let mut region = Vec::new();
            let mut st = vec![(x, y)];
            seen[i] = true;
            while let Some((cx, cy)) = st.pop() {
                region.push((cx, cy));
                for (nx, ny) in [(cx + 1, cy), (cx - 1, cy), (cx, cy + 1), (cx, cy - 1)] {
                    let j = (ny * w + nx) as usize;
                    if img.inside(nx, ny) && img.alpha(nx, ny) == 0 && !seen[j] {
                        seen[j] = true;
                        st.push((nx, ny));
                    }
                }
            }
            if region.len() > 8 {
                continue;
            }
            // Fill inward in layers from a snapshot, so the result doesn't
            // depend on visiting order (keeps mirrored gaps identical).
            let mut left = region;
            while !left.is_empty() {
                let snap = img.clone();
                let mut next = Vec::new();
                let mut fills = Vec::new();
                for &(px, py) in &left {
                    let dark = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                        .iter()
                        .filter_map(|&(dx, dy)| snap.get(px + dx, py + dy))
                        .min_by_key(|c| (c[0] as u32 + c[1] as u32 + c[2] as u32, *c));
                    match dark {
                        Some(c) => fills.push((px, py, c)),
                        None => next.push((px, py)),
                    }
                }
                if fills.is_empty() {
                    break;
                }
                for (px, py, c) in fills {
                    img.set(px, py, c);
                }
                left = next;
            }
        }
    }
}

/// Inner contour lines: a pixel of a part that borders a later (in-front)
/// part darkens, so overlaps read as drawn lines rather than merged blobs.
/// Lines fall beside and below the front part only (light from above), so a
/// form's top edge blends into what is behind it.
fn contours(img: &mut Img, ids: &[u8], groups: &[u8], sh: &Shades) {
    let src = img.clone();
    let w = img.w;
    for y in 0..img.h {
        for x in 0..w {
            let a = ids[(y * w + x) as usize];
            if a == 0 {
                continue;
            }
            let Some(c) = src.get(x, y) else { continue };
            let mut covered = false;
            let mut seam = false;
            for (dx, dy) in [(1, 0), (-1, 0), (0, -1)] {
                let (nx, ny) = (x + dx, y + dy);
                if !src.inside(nx, ny) || src.alpha(nx, ny) == 0 {
                    continue;
                }
                let b = ids[(ny * w + nx) as usize];
                let pair = groups[a as usize] + groups[b as usize] == 3;
                if pair {
                    seam = true;
                } else if b > a {
                    covered = true;
                }
            }
            if covered {
                img.set(x, y, sh.contour((y * w + x) as usize, c, 2));
            } else if seam {
                img.set(x, y, sh.contour((y * w + x) as usize, c, 1));
            }
        }
    }
}

/// Coloured outline: each edge pixel takes a deep, hue-matched shade of the
/// neighbour it borders.
fn outline(img: &mut Img, sh: &Shades) {
    let src = img.clone();
    for y in 0..img.h {
        for x in 0..img.w {
            if src.alpha(x, y) != 0 {
                continue;
            }
            // Below wins, then the darker of left/right (a symmetric
            // choice), then above.
            let pick = |dx: i32, dy: i32| src.get(x + dx, y + dy).map(|c| sh.outline(((y + dy) * img.w + x + dx) as usize, c));
            let key = |c: &Rgb| (c[0] as u32 + c[1] as u32 + c[2] as u32, *c);
            let side = match (pick(-1, 0), pick(1, 0)) {
                (Some(a), Some(b)) => Some(if key(&a) <= key(&b) { a } else { b }),
                (a, b) => a.or(b),
            };
            if let Some(c) = pick(0, 1).or(side).or_else(|| pick(0, -1)) {
                img.set(x, y, c);
            }
        }
    }
}

/// Where a garment piece takes its colour from: one of the look's three
/// dyes, or a fixed material.
#[derive(Clone, Copy, PartialEq)]
enum Dye {
    P,
    S,
    A,
    Leather,
    Linen,
    White,
    /// Dark work trousers.
    Drab,
    Metal,
    Stocking,
    Skin,
    Gold,
}

fn dye(c: &Colors, d: Dye) -> R5 {
    match d {
        Dye::P => c.primary,
        Dye::S => c.secondary,
        Dye::A => c.accent,
        Dye::Leather => c.leather,
        Dye::Linen => c.linen,
        Dye::White => c.white,
        Dye::Drab => c.drab,
        Dye::Metal => c.metal,
        Dye::Stocking => c.stocking,
        Dye::Skin => c.skin,
        Dye::Gold => gold5(),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Sleeve {
    Long,
    /// Rolled to the elbow, bare forearms.
    Rolled,
    Bare,
    /// Long with a puffed shoulder.
    Puff,
    /// Bell sleeves widening to the wrist (robes).
    Wide,
}

#[derive(Clone, Copy, PartialEq)]
enum Lower {
    Pants,
    /// Skirt from the waist: length as a fraction of the thigh, flare.
    Skirt(f32, f32),
    /// Floor-length robe or gown with this flare.
    Gown(f32),
}

/// One layer over the body, drawn in list order.
#[derive(Clone, Copy, PartialEq)]
enum Piece {
    Belt(Dye),
    Sash(Dye),
    /// Diagonal sash from the right shoulder to the left hip.
    DiagSash(Dye),
    /// Apron: with a bib from the chest, or from the waist; length below
    /// the hips.
    Apron(Dye, bool, f32),
    /// Tabard with a band of the second dye.
    Tabard(Dye, Dye),
    /// Open vest panels (closed when the bool is set).
    Vest(Dye, bool),
    /// Long open coat: colour, what shows down the opening, length as a
    /// fraction of the thigh (big values reach the ground), cutaway front.
    Coat(Dye, Dye, f32, bool),
    /// Laced bodice under the bust.
    Corset(Dye),
    Capelet(Dye),
    /// Full cape behind: outside, and the lining seen from the front.
    Cape(Dye, Dye),
    /// Chemise yoke showing above a bodice, with a scooped neckline.
    Yoke(Dye),
    /// A short cape over one shoulder.
    HalfCape(Dye),
    Scarf(Dye),
    Kerchief(Dye),
    Stole(Dye),
    Collar(Dye),
    Bow(Dye),
    Lacing(Dye),
    Gem(Dye),
    /// Wrap-front gi neckline.
    Wrap,
    Pouches,
    Satchel,
    Quiver,
    /// Shoulder armour on both shoulders, or only the left one; trim dye.
    Pauldrons(bool, Dye),
    /// Armoured skirt of overlapping plates.
    Faulds,
    /// Leather chest guard over the tunic.
    ChestGuard,
    /// Centre ridge of a breastplate.
    Ridge,
    /// Triangle bikini: top and tie-side bottoms, ties in the second dye.
    Bikini(Dye, Dye),
    /// Swim trunks with a waistband.
    Trunks(Dye, Dye),
}

/// A whole outfit: body garment, sleeves, legs, lower garment, footwear and
/// the layers over them.
struct Garb {
    shirt: Dye,
    sleeve: (Sleeve, Dye),
    legs: Dye,
    lower: (Lower, Dye),
    /// Boots: material and height above the ankle.
    boots: Option<(Dye, f32)>,
    /// Baggy trousers.
    wide: bool,
    /// Skin showing at an open collar.
    neckline: bool,
    barefoot: bool,
    pieces: Vec<Piece>,
}

fn garb(look: &Look) -> Garb {
    use Dye::*;
    use Piece::*;
    let f = look.female;
    let mut g = Garb { shirt: P, sleeve: (Sleeve::Long, P), legs: Drab, lower: (Lower::Pants, Drab), boots: None, wide: false, neckline: false, barefoot: false, pieces: Vec::new() };
    let skirt = |len: f32, flare: f32, d: Dye| (Lower::Skirt(len, flare), d);
    match look.outfit {
        outfit::PEASANT => {
            g.sleeve = (Sleeve::Rolled, P);
            if f {
                // Bodice dress, apron and a kerchief at the throat.
                g.lower = skirt(1.15, 0.42, P);
                g.legs = Skin;
                g.neckline = true;
                g.pieces = vec![Apron(S, false, 9.0), Kerchief(A)];
            } else {
                g.legs = S;
                g.pieces = vec![Belt(Leather), Kerchief(A)];
            }
        }
        outfit::LABORER => {
            // Sleeveless jerkin over a shirt, sturdy boots.
            g.sleeve = (Sleeve::Rolled, S);
            g.boots = Some((Leather, 7.0));
            g.pieces = vec![Lacing(Leather), Belt(Leather), Kerchief(A)];
        }
        outfit::FISHER => {
            g.shirt = Linen;
            g.sleeve = (Sleeve::Rolled, Linen);
            g.boots = Some((Leather, 10.0));
            if f {
                g.lower = skirt(0.75, 0.4, S);
                g.legs = Stocking;
            } else {
                g.legs = S;
            }
            g.pieces = vec![Capelet(P), Kerchief(A)];
        }
        outfit::SMITH => {
            g.shirt = S;
            g.sleeve = (Sleeve::Bare, Skin);
            g.boots = Some((Leather, 7.0));
            g.pieces = vec![Apron(P, true, 12.0)];
        }
        outfit::COOK => {
            g.sleeve = (Sleeve::Rolled, P);
            if f {
                g.lower = skirt(1.3, 0.4, P);
                g.legs = Stocking;
            }
            g.pieces = vec![Apron(S, true, 13.0), Kerchief(A)];
        }
        outfit::MERCHANT => {
            g.shirt = S;
            g.sleeve = (Sleeve::Long, P);
            if f {
                g.legs = Stocking;
            }
            g.boots = Some((Leather, 7.0));
            g.pieces = vec![Coat(P, S, 0.9, false), Sash(A), Satchel, Pouches];
        }
        outfit::INNKEEPER => {
            if f {
                // Laced bodice over a chemise, full skirt and apron.
                g.sleeve = (Sleeve::Puff, Linen);
                g.lower = skirt(1.05, 0.52, S);
                g.legs = Stocking;
                g.pieces = vec![Yoke(Linen), Lacing(A), Apron(White, false, 7.0)];
            } else {
                g.shirt = Linen;
                g.sleeve = (Sleeve::Rolled, Linen);
                g.legs = S;
                g.pieces = vec![Vest(P, false), Apron(White, false, 9.0), Kerchief(A)];
            }
        }
        outfit::GUARD => {
            // Heraldic tabard over mail.
            g.shirt = Metal;
            g.sleeve = (Sleeve::Long, Metal);
            g.legs = S;
            g.boots = Some((Leather, 8.0));
            g.pieces = vec![Tabard(P, A), Belt(Leather)];
        }
        outfit::KNIGHT => {
            g.shirt = Metal;
            g.sleeve = (Sleeve::Long, Metal);
            g.legs = Metal;
            g.boots = Some((Metal, 9.0));
            g.lower = skirt(0.5, 0.35, S);
            g.pieces = vec![Ridge, Cape(P, P), Faulds, Pauldrons(true, A), Belt(Leather)];
        }
        outfit::HERO => {
            // Long open coat over a light shirt, one pauldron.
            g.shirt = S;
            g.sleeve = (Sleeve::Long, P);
            g.boots = Some((Leather, 8.0));
            if f {
                g.legs = Stocking;
            }
            g.pieces = vec![Coat(P, S, 0.75, false), Belt(Leather), Pauldrons(false, A), Gem(A)];
        }
        outfit::ADVENTURER => {
            g.sleeve = (Sleeve::Rolled, P);
            g.boots = Some((Leather, if f { 13.0 } else { 9.0 }));
            if f {
                g.lower = skirt(0.5, 0.45, Leather);
                g.legs = Skin;
            }
            g.pieces = vec![ChestGuard, Belt(Leather), Pouches, Capelet(S), Scarf(A)];
        }
        outfit::ROGUE => {
            g.boots = Some((Leather, 8.0));
            if f {
                g.lower = skirt(0.45, 0.4, P);
                g.legs = Stocking;
            }
            g.pieces = vec![Belt(A), Pouches, Scarf(S)];
        }
        outfit::RANGER => {
            g.boots = Some((Leather, 11.0));
            g.pieces = vec![Cape(S, S), Belt(Leather), Quiver, Gem(A)];
        }
        outfit::MAGE => {
            g.sleeve = (Sleeve::Wide, P);
            if f {
                // Short dress with thigh-highs.
                g.lower = skirt(1.2, 0.5, P);
                g.legs = Stocking;
            } else {
                g.lower = (Lower::Gown(0.24), P);
                g.legs = Skin;
            }
            g.pieces = vec![Cape(S, S), Sash(A), Gem(A)];
        }
        outfit::SCHOLAR => {
            g.lower = (Lower::Gown(0.18), P);
            g.legs = Skin;
            g.pieces = vec![Capelet(S), Satchel, Gem(A)];
        }
        outfit::PRIEST => {
            g.sleeve = (Sleeve::Wide, P);
            g.lower = (Lower::Gown(0.22), P);
            g.legs = Skin;
            g.pieces = vec![Stole(S), Sash(A), Gem(A)];
        }
        outfit::MONK => {
            g.sleeve = (Sleeve::Rolled, P);
            g.legs = S;
            g.wide = true;
            g.neckline = true;
            g.pieces = vec![Wrap, Sash(A)];
        }
        outfit::BARD => {
            g.sleeve = (Sleeve::Puff, P);
            if f {
                g.lower = skirt(1.2, 0.45, P);
                g.legs = Stocking;
            } else {
                g.legs = Stocking;
                g.boots = Some((Leather, 9.0));
            }
            g.pieces = vec![Belt(Leather), HalfCape(S), Gem(A)];
        }
        outfit::MAID => {
            if f {
                g.sleeve = (Sleeve::Puff, P);
                g.lower = skirt(0.95, 0.55, P);
                g.legs = Stocking;
                g.pieces = vec![Apron(S, true, 6.0), Bow(A)];
            } else {
                // Butler: white shirt, black tailcoat, tie.
                g.shirt = S;
                g.legs = P;
                g.pieces = vec![Coat(P, S, 0.8, true), Bow(A)];
            }
        }
        outfit::NOBLE => {
            if f {
                // Bell gown, corset and puffed sleeves.
                g.shirt = P;
                g.sleeve = (Sleeve::Puff, S);
                g.lower = (Lower::Gown(0.42), P);
                g.legs = Skin;
                g.neckline = true;
                g.pieces = vec![Corset(P), Gem(A)];
            } else {
                g.shirt = S;
                g.legs = S;
                g.boots = Some((Leather, 11.0));
                g.pieces = vec![Coat(P, S, 0.8, false), DiagSash(A), Bow(S)];
            }
        }
        outfit::GUILD => {
            // Blouse, closed vest, bow at the collar.
            g.shirt = S;
            g.sleeve = (Sleeve::Long, S);
            if f {
                g.lower = skirt(1.35, 0.3, P);
                g.legs = Stocking;
            } else {
                g.legs = P;
            }
            g.pieces = vec![Vest(P, true), Bow(A)];
        }
        outfit::SWIM => {
            g.shirt = Skin;
            g.sleeve = (Sleeve::Bare, Skin);
            g.legs = Skin;
            g.barefoot = true;
            g.pieces = vec![if f { Bikini(P, A) } else { Trunks(P, S) }];
        }
        _ => {
            // Demon lord: floor-length gown or coat, a high collar, a cape
            // lined in the second dye.
            g.lower = (Lower::Gown(0.36), P);
            g.legs = Skin;
            g.pieces = vec![Cape(P, S), Sash(Gold), Collar(S), Gem(A)];
        }
    }
    g
}

impl Garb {
    /// The cape's outside and lining, if the outfit has one.
    fn cape(&self) -> Option<(Dye, Dye)> {
        self.pieces.iter().find_map(|p| if let Piece::Cape(o, l) = p { Some((*o, *l)) } else { None })
    }
    /// Whether the outfit is mirror-symmetric from the front and back:
    /// straps, one-shoulder pieces, scarf tails and wrap fronts aren't.
    fn symmetric(&self) -> bool {
        !self.pieces.iter().any(|p| matches!(p, Piece::Satchel | Piece::Quiver | Piece::HalfCape(_) | Piece::Wrap | Piece::Scarf(_) | Piece::DiagSash(_) | Piece::Pauldrons(false, _)))
    }
}

fn leg(cv: &mut Canvas, look: &Look, b: &Build, c: &Colors, (hip, knee, ankle): (V2, V2, V2), far: bool, dir: u8) {
    let g = garb(look);
    let ramp = dye(c, g.legs);
    // The far leg sits clearly in shadow so the stride reads as alternating.
    let bias = if far { -0.32 } else { 0.0 };
    let wide = if g.wide { 1.22 } else { 1.0 };
    let (tr, cr) = (b.thigh_r * wide, b.calf_r * wide);
    let ankle_r = b.calf_r * 0.62;
    let front = dir != DIR_SIDE;
    if front {
        let left = hip.x < b.cx;
        cv.side = if left { 1 } else { 2 };
        cv.clip = Some(if left { (0, 31) } else { (32, FW - 1) });
    }
    cv.begin();
    // Thigh: soft and full most of the way, easing into the knee.
    let mid = v(hip.x + (knee.x - hip.x) * 0.55, hip.y + (knee.y - hip.y) * 0.55);
    cv.capsule(hip, mid, tr, tr * 0.94, &ramp, bias);
    cv.capsule(mid, knee, tr * 0.94, cr * 1.02, &ramp, bias);
    // Calf: swells on the outside just below the knee, then slims to a
    // narrow ankle (baggy trousers gather at the ankle).
    let out = if front { if hip.x < b.cx { -0.6 } else { 0.6 } } else { -0.6 };
    let calf_mid = v(knee.x + (ankle.x - knee.x) * 0.32 + out, knee.y + (ankle.y - knee.y) * 0.32);
    cv.capsule(knee, calf_mid, cr * 1.0, cr * 1.1, &ramp, bias);
    cv.capsule(calf_mid, ankle, cr * 1.1, ankle_r * 0.9, &ramp, bias);
    let side = dir == DIR_SIDE;
    if let Some((d, h)) = g.boots {
        // Shaft up the shin: knee-high riders reach over the knee.
        // Up the actual shin, so a raised, foreshortened leg keeps its boot
        // below the knee.
        let k = (h / b.calf).min(1.25);
        let top = v(ankle.x + (knee.x - ankle.x) * k, ankle.y + (knee.y - ankle.y) * k);
        cv.begin();
        cv.capsule(top, ankle, b.calf_r * 1.04, ankle_r + 0.6, &dye(c, d), bias);
    }
    cv.begin();
    let shoe = match g.boots {
        Some((d, _)) => dye(c, d),
        None if g.barefoot => c.skin,
        None => c.shoe,
    };
    if side {
        cv.ellipse(ankle.add(v(2.4, 1.6)), 4.4, 2.2, &shoe, bias);
    } else {
        cv.ellipse(ankle.add(v(0.0, 1.8)), 3.2, 2.4, &shoe, bias);
    }
    cv.clip = None;
    cv.side = 0;
}

#[allow(clippy::too_many_arguments)]
fn arm(cv: &mut Canvas, look: &Look, b: &Build, c: &Colors, (sh, el, wr): (V2, V2, V2), far: bool, dir: u8, pose: Pose) {
    let g = garb(look);
    let r = b.arm_r;
    let (kind, d) = g.sleeve;
    let sleeve = dye(c, d);
    let bias = if far { -0.18 } else { 0.0 };
    cv.begin();
    match kind {
        Sleeve::Bare => {
            cv.capsule(sh, el, r, r * 0.88, &c.skin, bias);
            cv.capsule(el, wr, r * 0.86, r * 0.72, &c.skin, bias);
        }
        Sleeve::Rolled => {
            cv.capsule(sh, el, r, r * 0.9, &sleeve, bias);
            cv.begin();
            cv.capsule(el, wr, r * 0.86, r * 0.72, &c.skin, bias);
            // The rolled cuff just above the elbow.
            cv.begin();
            cv.capsule(el.add(v((sh.x - el.x) * 0.12, (sh.y - el.y) * 0.12)), el, r * 1.02, r * 0.98, &sleeve, bias + 0.06);
        }
        Sleeve::Wide => {
            cv.capsule(sh, el, r, r * 0.95, &sleeve, bias);
            cv.capsule(el, wr, r * 0.95, r * 1.45, &sleeve, bias);
        }
        Sleeve::Long | Sleeve::Puff => {
            cv.capsule(sh, el, r, r * 0.88, &sleeve, bias);
            cv.capsule(el, wr, r * 0.88, r * 0.8, &sleeve, bias);
        }
    }
    if kind == Sleeve::Puff {
        cv.begin();
        cv.ellipse(sh.add(v(0.0, 2.2)), r + 2.0, r + 1.8, &sleeve, bias);
    }
    for p in &g.pieces {
        if let Piece::Pauldrons(both, trim) = *p {
            // A lone pauldron sits on the left shoulder: screen right from
            // the front, screen left from behind, the far side in profile.
            let mine = both || match dir {
                DIR_DOWN => sh.x > b.cx,
                DIR_UP => sh.x < b.cx,
                _ => far,
            };
            if mine {
                cv.begin();
                cv.ellipse(sh.add(v(0.0, 1.6)), r + 2.4, r + 1.6, &dye(c, trim), bias);
                cv.begin();
                cv.ellipse(sh.add(v(0.0, 0.4)), r + 2.4, r + 1.6, &c.metal, bias + 0.05);
            }
        }
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
    // The same hand in every view: screen right from the front, screen
    // left from behind, the near hand in profile.
    let holding = !far && match dir {
        DIR_DOWN => wr.x > b.cx,
        DIR_UP => wr.x < b.cx,
        _ => true,
    };
    if holding && look.held != held::NONE {
        cv.begin();
        held_item(cv, look, hand, dir, pose.work == 1, c);
    }
}

fn torso(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
    let side = dir == DIR_SIDE;
    let back = dir == DIR_UP;
    let front = dir == DIR_DOWN;
    let g = garb(look);
    let shirt = dye(c, g.shirt);
    let (sy, cy) = (b.shoulder_y, b.crotch_y);
    let fig = b.woman;
    // Profile: separate front and back edges give the body its S-curve —
    // chest forward, the small of the back curving in, the seat curving
    // out, belly and pelvis flat in front.
    let hk = b.hip_w / 11.2;
    let (front_keys, back_keys): (Vec<(f32, f32)>, Vec<(f32, f32)>) = if look.female {
        (
            vec![(sy - 0.5, 4.0), (sy + 4.0, 5.8), (sy + 10.0, 5.6), (b.waist_y, 5.0), (b.hip_y, 5.6), (cy + 1.0, 5.0)],
            vec![(sy - 0.5, -4.2), (sy + 4.0, -5.8), (b.waist_y - 1.0, -4.6), (b.hip_y + 1.0, -4.4 - 3.6 * hk), (cy + 1.0, -5.6 * hk)],
        )
    } else {
        let bk = b.chest_w / 11.5;
        (
            vec![(sy - 0.5, 4.6), (sy + 4.0, 7.0 * bk), (sy + 11.0, 6.6 * bk), (b.waist_y, 5.6 * b.waist_w / 9.0), (b.hip_y, 5.6), (cy + 1.0, 5.2)],
            vec![(sy - 0.5, -4.8), (sy + 4.0, -6.6 * bk), (b.waist_y - 1.0, -5.4), (b.hip_y + 1.0, -6.2 * hk), (cy + 1.0, -5.6)],
        )
    };
    let keys: Vec<(f32, f32)> = if side {
        Vec::new()
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
    let cx0 = b.cx;
    let (fk, bkk, kk) = (&front_keys, &back_keys, &keys);
    let lean = move |y: f32| -> f32 { if side { cx0 + 0.5 + (keyed(fk, y) + keyed(bkk, y)) * 0.5 } else { cx0 } };
    let width = move |y: f32| -> f32 { if side { (keyed(fk, y) - keyed(bkk, y)) * 0.5 } else { keyed(kk, y) } };
    // Front and back edges of the body in profile.
    let fwd = move |y: f32| lean(y) + width(y);
    let aft = move |y: f32| lean(y) - width(y);
    // Neck: its own part so the chin and collar line it.
    cv.begin();
    let nw = if look.female { 3.4 } else { 4.2 };
    // From behind the nape sits in the hair's shadow.
    cv.profile(sy - 6.0, sy + 1.5, |_| b.cx + if side { 0.5 } else { 0.0 }, |_| nw, &c.skin, if back { -0.3 } else { -0.06 });
    let (lower_kind, lower_dye) = g.lower;
    let skirted = lower_kind != Lower::Pants;
    let lower = dye(c, g.legs);
    // Bare legs under a short skirt get shorts underneath, never bare skin.
    let short_skirt = matches!(lower_kind, Lower::Skirt(len, _) if len < 0.7);
    let seat = if short_skirt && g.legs == Dye::Skin { c.drab } else { lower };
    cv.begin();
    // Pelvis: hip curve, then a V into the crotch so the thighs carry the
    // silhouette instead of a flat box edge.
    let pelvis: Vec<(f32, f32)> = vec![(b.waist_y, b.waist_w), (b.hip_y - 1.0, b.hip_w), (cy - 2.0, b.hip_w - 1.2), (cy + 1.5, 2.5)];
    // Same cloth as the thighs: drawn without an id so no seam is traced
    // where it meets them; only the inner-leg line runs down from the crotch.
    let part = cv.cur;
    cv.cur = 0;
    cv.profile(b.waist_y, cy + 1.5, lean, |y| if side { width(y) } else { keyed(&pelvis, y) }, &seat, 0.0);
    cv.cur = part;
    if fig && back && !skirted {
        // Seat: a soft rounding with a short crease, no outlined lobes.
        let part = cv.cur;
        cv.cur = 0;
        for s in [-1.0f32, 1.0] {
            cv.ellipse(v(b.cx + s * b.hip_w * 0.45, b.hip_y + 3.0), b.hip_w * 0.5, 4.8, &lower, 0.0);
        }
        cv.cur = part;
        for y in (b.hip_y + 3.0) as i32..(cy + 2.0) as i32 {
            cv.put(31, y, lower[1]);
            cv.put(32, y, lower[1]);
        }
    }

    cv.begin();
    cv.profile(sy, b.waist_y + 3.5, lean, width, &shirt, 0.0);
    // Fuller busts sit a little lower.
    let by = (sy + 9.5 + (b.bust - 5.4).max(0.0) * 0.6).round() + b.jig;
    let cx = b.cx;
    // Neckline: a little skin under the collarbone for open-necked tops.
    if front && g.neckline {
        for y in sy as i32..(sy + if fig { 5.0 } else { 3.0 }) as i32 {
            let half = (4.5 - (y as f32 - sy) * 0.6).max(1.0);
            hspan(cv, half, y, if y == sy as i32 { c.skin[3] } else { c.skin[2] });
        }
    }
    let sway = b.sway;
    // Skirts and gowns from the waist.
    match lower_kind {
        Lower::Skirt(len, flare) => {
            let top = b.waist_y + 1.0;
            let len = b.thigh * len + 2.0;
            let ramp = dye(c, lower_dye);
            cv.begin();
            let (w0, k) = skirt_shape(side, b.waist_w + 1.2, flare);
            let sc = if side { cx - 0.5 } else { cx };
            cv.profile(top, top + len, |y| sc + (y - top) / len * sway * 0.8, |y| w0 + (y - top) * k, &ramp, -0.02);
            hem_folds(cv, top, top + len, sc, w0, k, &ramp, sway);
            // The skirt shades the legs just under its hem.
            let hy = (top + len).ceil() as i32 + 1;
            let (hc, hw) = (sc + sway * 0.8, w0 + len * k);
            for x in (hc - hw) as i32..=(hc + hw) as i32 {
                cv.darken_on(x, hy, &lower);
            }
        }
        Lower::Gown(flare) => {
            let top = b.waist_y - 1.0;
            let bottom = b.ground - 1.5;
            let len = bottom - top;
            let ramp = dye(c, lower_dye);
            cv.begin();
            let (w0, k) = skirt_shape(side, b.waist_w + 1.4, flare);
            cv.profile(top, bottom, |y| cx + (y - top) / len * sway * 0.7, |y| w0 + (y - top) * k, &ramp, 0.0);
            hem_folds(cv, top, bottom, cx, w0, k, &ramp, sway);
        }
        Lower::Pants => {}
    }
    // Chest garments go on first; the bust is then shaped in whatever is
    // outermost over it, so a coat or vest wraps the curve instead of a
    // shirt-coloured disc sitting on top. Everything else layers after.
    let chest = |p: &Piece| matches!(p, Piece::Coat(..) | Piece::Vest(_, true) | Piece::Corset(_) | Piece::ChestGuard);
    let bust_ramp = g.pieces.iter().rev().find_map(|p| match *p {
        Piece::Coat(d, ..) | Piece::Vest(d, true) => Some(dye(c, d)),
        Piece::ChestGuard => Some(c.leather),
        _ => None,
    }).unwrap_or(shirt);
    let ordered: Vec<Piece> = g.pieces.iter().copied().filter(|p| chest(p)).collect();
    let rest: Vec<Piece> = g.pieces.iter().copied().filter(|p| !chest(p)).collect();
    for (pass, list) in [(0, ordered), (1, rest)] {
        if pass == 1 && b.bust > 0.0 {
            if side {
                bust_side(cv, b, by, &bust_ramp, fwd);
            } else {
                bust(cv, b, dir, by, &bust_ramp);
                // An open coat still shows the shirt down the middle.
                if let (true, Some(Piece::Coat(_, inner, ..))) = (front, g.pieces.iter().find(|p| matches!(p, Piece::Coat(..)))) {
                    let inn = dye(c, *inner);
                    for y in (by - b.bust) as i32..(by + b.bust) as i32 {
                        hspan(cv, 1.5, y, inn[3]);
                    }
                }
            }
        }
        for p in list {
            match p {
                Piece::Belt(d) => belt(cv, b, &dye(c, d), lean, width, front),
                Piece::Sash(d) => sash(cv, b, &dye(c, d), lean, width),
                Piece::DiagSash(d) if !side => {
                    let s = if back { -1.0 } else { 1.0 };
                    cv.begin();
                    cv.capsule(v(cx - s * (b.shoulder_w - 2.5), sy + 1.5), v(cx + s * (b.waist_w - 0.5), b.waist_y + 2.0), 1.5, 1.5, &dye(c, d), 0.05);
                }
                Piece::Apron(d, bib, len) => {
                    let ap = dye(c, d);
                    let bottom = b.hip_y + len;
                    if back {
                        // From behind only the ties show: strings round the
                        // waist, a knot, two loops and two tails.
                        let y = b.waist_y + 2.5;
                        hspan(cv, width(y) - 0.4, y as i32, ap[2]);
                        cv.begin();
                        for s in [-1.0f32, 1.0] {
                            cv.ellipse(v(cx + s * 2.0, y - 0.5), 1.8, 1.2, &ap, 0.05);
                            cv.capsule(v(cx + s * 0.8, y + 0.5), v(cx + s * 1.8, y + 5.0), 0.8, 0.6, &ap, 0.0);
                        }
                        cv.begin();
                        cv.ellipse(v(cx, y), 1.0, 1.0, &ap, -0.15);
                        continue;
                    }
                    let at = if bib { if fig && !side { by - 1.0 } else { sy + 4.0 } } else { b.waist_y + 1.0 };
                    let (ax, aw) = if side { (fwd(b.waist_y) - 1.6, 2.2) } else if bib { (cx, b.chest_w - 3.0) } else { (cx, b.waist_w + 0.5) };
                    cv.begin();
                    cv.profile(at, bottom, |_| ax, |y| if side { aw } else { aw + (y - at).max(0.0) * 0.06 }, &ap, 0.06);
                    if front && bib {
                        // Straps up to the shoulders.
                        let strap = (cx - aw + 1.0).floor() as i32;
                        for k in 0..(at - sy) as i32 {
                            cv.put(strap, sy as i32 + k, ap[2]);
                            cv.put(FW - 1 - strap, sy as i32 + k, ap[2]);
                        }
                    }
                    if front {
                        // Tie line at the waist.
                        hspan(cv, aw - 0.5, (b.waist_y + 2.0) as i32, ap[2]);
                    }
                }
                Piece::Tabard(d, band) => {
                    let tb = dye(c, d);
                    let top = if b.bust > 0.0 && front { by + b.bust * 0.6 } else { sy + 1.0 };
                    let bottom = b.hip_y + if look.female { 9.0 } else { 7.0 };
                    cv.begin();
                    if side {
                        for e in [fwd(b.waist_y) - 1.2, aft(b.waist_y) + 1.2] {
                            cv.profile(sy + 2.0, bottom, |_| e, |_| 1.4, &tb, 0.0);
                        }
                    } else {
                        cv.profile(top, bottom, |_| cx, |y| 4.0 + (y - top) * 0.03, &tb, 0.0);
                        let bd = dye(c, band);
                        for k in 0..2 {
                            hspan(cv, 4.0, (b.waist_y + 6.0) as i32 + k, bd[2 + k as usize % 2]);
                        }
                    }
                }
                Piece::Vest(d, closed) => {
                    let vd = dye(c, d);
                    cv.begin();
                    if side {
                        cv.profile(sy + 0.5, b.hip_y, |y| lean(y) - 0.3, |y| width(y) - 0.2, &vd, 0.0);
                        cv.begin();
                        cv.profile(sy + 0.5, b.hip_y, |y| fwd(y) - 1.5, |_| 1.0, &shirt, 0.0);
                    } else if back || closed {
                        let top = if closed && front { sy + 4.0 } else { sy + 0.5 };
                        cv.profile(top, b.hip_y, lean, |y| width(y) - 0.2, &vd, 0.0);
                        if closed && front {
                            // Blouse in the V of the collar.
                            for y in (sy + 4.0) as i32..(sy + 8.0) as i32 {
                                hspan(cv, (8.0 - (y as f32 - sy)).max(1.0) * 0.6, y, shirt[3]);
                            }
                        }
                    } else {
                        let w = b.chest_w * 0.42;
                        cv.profile(sy + 0.5, b.hip_y, |_| cx - b.chest_w + w, |_| w, &vd, 0.0);
                        cv.begin();
                        cv.profile(sy + 0.5, b.hip_y, |_| cx + b.chest_w - w, |_| w, &vd, 0.0);
                    }
                }
                Piece::Coat(d, inner, len, cutaway) => {
                    let ct = dye(c, d);
                    let bottom = (b.crotch_y + b.thigh * len).min(b.ground - 1.5);
                    cv.begin();
                    cv.profile(sy, b.waist_y + 1.0, lean, |y| width(y) + 0.8, &ct, 0.0);
                    let top = b.waist_y;
                    let span = bottom - top;
                    let (w0, k) = if side { (5.8, 0.18) } else { (b.waist_w + 2.4, 0.22) };
                    let sc = if side { cx - 1.0 } else { cx };
                    cv.profile(top, bottom, |y| sc + (y - top) / span * sway * 0.9, |y| w0 + (y - top) * k, &ct, 0.0);
                    hem_folds(cv, top, bottom, sc, w0, k, &ct, sway);
                    if front {
                        // Front opening: the shirt above the waist, the legs (or
                        // the dress) below it; a cutaway opens wide at once.
                        let inn = dye(c, inner);
                        let legs = if skirted { dye(c, lower_dye) } else { lower };
                        for y in sy as i32..bottom as i32 {
                            let fy = y as f32 + 0.5;
                            let below = fy >= b.waist_y + 2.0;
                            let half = if !below { 2.0 + (fy - sy) * 0.05 } else if cutaway { 4.5 + (fy - b.waist_y) * 0.5 } else { 2.5 + (fy - b.waist_y) * 0.18 };
                            let col = if !below { inn[if fy < sy + 2.0 { 2 } else { 3 }] } else { legs[2] };
                            hspan(cv, half.min(w0 + (fy - top).max(0.0) * k - 0.5), y, col);
                        }
                        // Lapels.
                        for y in sy as i32..(sy + 7.0) as i32 {
                            let off = 2.0 + (y as f32 - sy) * 0.5;
                            cv.put((cx - off - 1.0).floor() as i32, y, ct[1]);
                            cv.put(FW - 1 - (cx - off - 1.0).floor() as i32, y, ct[1]);
                        }
                    }
                    if back {
                        for y in (b.waist_y + 3.0) as i32..bottom as i32 {
                            cv.put(31, y, ct[1]);
                            cv.put(32, y, ct[1]);
                        }
                    }
                }
                Piece::Corset(d) => {
                    let cd = dye(c, d);
                    let top = if fig && !back { by + b.bust * 0.55 } else { sy + 6.0 };
                    cv.begin();
                    cv.profile(top, b.waist_y + 3.0, lean, |y| width(y) - 0.3, &cd, 0.0);
                }
                Piece::Capelet(d) => {
                    let cp = dye(c, d);
                    cv.begin();
                    if side {
                        cv.profile(sy - 0.5, sy + 7.0, |y| lean(y) - 0.5, |y| width(y) + 1.6 + (y - sy) * 0.1, &cp, 0.05);
                    } else {
                        cv.profile(sy - 0.5, sy + 8.0, |_| cx, |y| b.shoulder_w + 2.6 + (y - sy) * 0.12, &cp, 0.05);
                    }
                }
                Piece::HalfCape(d) => {
                    let cp = dye(c, d);
                    cv.begin();
                    if side {
                        cv.profile(sy - 0.5, sy + 6.0, |y| lean(y) - 0.5, |y| width(y) + 1.4, &cp, 0.05);
                    } else {
                        // Over the left shoulder (screen right in front).
                        let s = if back { -1.0 } else { 1.0 };
                        let (top, bottom) = (sy - 0.5, b.waist_y + 5.0);
                        let c0 = cx + s * (b.shoulder_w + 0.5);
                        // Draped over the shoulder and arm, rounding off at the hem.
                        cv.profile(top, bottom, |y| c0 + s * (y - top) * 0.04, |y| {
                            let t = (y - top) / (bottom - top);
                            4.4 * (1.0 - t.powi(4)).sqrt().max(0.3)
                        }, &cp, 0.05);
                    }
                }
                Piece::Scarf(d) => {
                    let sc = dye(c, d);
                    cv.begin();
                    let ncx = b.cx + if side { 0.5 } else { 0.0 };
                    cv.profile(sy - 3.0, sy + 1.0, |_| ncx, |_| nw + 2.2, &sc, 0.05);
                    cv.begin();
                    if side {
                        let a0 = v(ncx - nw - 0.5, sy - 0.5);
                        cv.capsule(a0, a0.add(v(-2.5 - sway * 2.0, 8.0)), 1.6, 1.2, &sc, 0.0);
                    } else if back {
                        cv.capsule(v(cx - 1.0, sy), v(cx - 1.5 + sway, sy + 9.0), 1.5, 1.2, &sc, 0.0);
                    } else {
                        cv.capsule(v(cx + 3.0, sy), v(cx + 4.0, sy + 9.0), 1.5, 1.2, &sc, 0.0);
                    }
                }
                Piece::Kerchief(d) => {
                    let kc = dye(c, d);
                    if front {
                        for (k, w) in [3.0f32, 2.0, 1.0].into_iter().enumerate() {
                            hspan(cv, w, sy as i32 + k as i32, kc[if k == 0 { 3 } else { 2 }]);
                        }
                    } else if side {
                        let x = fwd(sy) as i32 - 1;
                        for k in 0..3 {
                            cv.put(x - k / 2, sy as i32 + k, kc[2]);
                        }
                    } else {
                        hspan(cv, nw, sy as i32 - 1, kc[2]);
                    }
                }
                Piece::Stole(d) => {
                    let st = dye(c, d);
                    let bottom = b.hip_y + 8.0;
                    cv.begin();
                    if side {
                        cv.profile(sy, bottom, |y| fwd(y.min(b.waist_y)) - 1.4, |_| 0.9, &st, 0.0);
                    } else if front {
                        for s in [-1.0f32, 1.0] {
                            cv.profile(sy - 0.5, bottom, |_| cx + s * 3.2, |_| 1.1, &st, 0.0);
                        }
                    }
                }
                Piece::Collar(lining) => {
                    // High standing collar, lined in the second dye.
                    let out = dye(c, g.shirt);
                    cv.begin();
                    if side {
                        cv.capsule(v(cx - 3.5, sy + 1.0), v(cx - 5.5, sy - 7.0), 1.8, 2.4, &out, 0.0);
                    } else {
                        for s in [-1.0f32, 1.0] {
                            cv.capsule(v(cx + s * 5.0, sy + 1.0), v(cx + s * 8.0, sy - 7.0), 1.8, 2.6, &out, 0.0);
                            if front {
                                cv.capsule(v(cx + s * 4.6, sy + 0.5), v(cx + s * 7.0, sy - 5.5), 0.8, 1.2, &dye(c, lining), 0.0);
                            }
                        }
                    }
                }
                Piece::Bow(d) if !back => {
                    let bw = dye(c, d);
                    let ty = sy + 2.0;
                    cv.begin();
                    if side {
                        cv.ellipse(v(fwd(ty) - 0.5, ty + 1.0), 1.8, 1.6, &bw, 0.0);
                    } else {
                        cv.ellipse(v(cx - 2.4, ty + 1.0), 2.2, 1.7, &bw, 0.0);
                        cv.ellipse(v(cx + 2.4, ty + 1.0), 2.2, 1.7, &bw, 0.0);
                        cv.ellipse(v(cx, ty + 1.2), 1.1, 1.1, &bw, -0.1);
                    }
                }
                Piece::Lacing(d) if front => {
                    let ld = dye(c, d);
                    let top = if fig { by + b.bust * 0.55 } else { sy + 3.0 };
                    let n = ((b.waist_y + 1.0 - top) / 3.0).max(1.0) as i32;
                    for k in 0..n {
                        let y = top as i32 + 1 + k * 3;
                        cv.put(cx as i32 - 2, y, ld[3]);
                        cv.put(cx as i32 + 1, y, ld[3]);
                        cv.put(cx as i32 - 1, y + 1, ld[2]);
                        cv.put(cx as i32, y + 1, ld[2]);
                    }
                }
                Piece::Gem(d) if !back => {
                    let gm = dye(c, d);
                    cv.begin();
                    let gx = if side { fwd(sy + 2.5) - 1.0 } else { cx };
                    cv.ellipse(v(gx, sy + 2.5), 1.5, 1.6, &gm, 0.15);
                }
                Piece::Wrap => {
                    // Left over right: an asymmetric lapel line from the collar.
                    if front {
                        cv.line(v(cx - 4.5, sy), v(cx + 2.5, b.waist_y), |_| Some(shirt[1]));
                    } else if side {
                        cv.line(v(fwd(sy) - 1.0, sy), v(fwd(b.waist_y) - 0.5, b.waist_y), |_| Some(shirt[1]));
                    }
                }
                Piece::Pouches => {
                    let y = b.waist_y + 4.5;
                    cv.begin();
                    if side {
                        cv.ellipse(v(aft(y) + 1.0, y), 1.8, 2.0, &c.leather, 0.05);
                    } else {
                        for s in [-1.0f32, 1.0] {
                            cv.ellipse(v(cx + s * (b.waist_w - 0.5), y), 1.8, 2.0, &c.leather, 0.05);
                        }
                    }
                }
                Piece::Satchel => {
                    // Strap over the right shoulder, bag on the left hip.
                    cv.begin();
                    if side {
                        cv.ellipse(v(aft(b.hip_y), b.hip_y), 2.6, 3.0, &c.leather, 0.05);
                    } else {
                        let s = if back { -1.0 } else { 1.0 };
                        cv.line(v(cx - s * (b.shoulder_w - 2.0), sy + 0.5), v(cx + s * (b.hip_w + 0.5), b.hip_y - 1.0), |_| Some(c.leather[1]));
                        cv.begin();
                        cv.ellipse(v(cx + s * (b.hip_w + 1.0), b.hip_y + 1.0), 2.8, 2.6, &c.leather, 0.05);
                    }
                }
                Piece::Quiver => {
                    let fletch = [hex(0x6a5a50), hex(0xa89888), hex(0xe0d4c4), hex(0xf8f0e4), hex(0xffffff)];
                    if back {
                        if g.cape().is_none() {
                            quiver_back(cv, b, c);
                        }
                    } else if side {
                        cv.begin();
                        cv.capsule(v(aft(sy) - 1.0, sy - 3.0), v(aft(b.waist_y) + 0.5, b.waist_y), 2.2, 2.0, &c.leather, 0.05);
                        cv.begin();
                        cv.ellipse(v(aft(sy) - 1.5, sy - 5.0), 1.2, 1.4, &fletch, 0.1);
                    } else {
                        cv.line(v(cx + b.shoulder_w - 2.5, sy + 0.5), v(cx - b.waist_w + 0.5, b.waist_y + 1.0), |_| Some(c.leather[1]));
                    }
                }
                Piece::Faulds => {
                    for k in 0..3 {
                        let top = b.waist_y + 1.0 + k as f32 * 3.2;
                        cv.begin();
                        if side {
                            cv.profile(top, top + 3.5, |y| lean(y.min(cy)), |y| width(y.min(cy)) + 0.8 + k as f32 * 0.4, &c.metal, 0.0);
                        } else {
                            cv.profile(top, top + 3.5, |_| cx, |_| b.hip_w + 0.6 + k as f32 * 0.7, &c.metal, 0.0);
                        }
                    }
                }
                Piece::ChestGuard => {
                    let top = if fig && !back { by - b.bust * 0.3 } else { sy + 3.0 };
                    cv.begin();
                    if side {
                        cv.profile(top, b.waist_y, |y| lean(y) + 0.3, |y| width(y) - 0.2, &c.leather, 0.0);
                    } else {
                        cv.profile(top, b.waist_y, lean, |y| width(y) - 0.8, &c.leather, 0.0);
                    }
                }
                Piece::Yoke(d) => {
                    let yk = dye(c, d);
                    if front {
                        // Chemise across the top of the bodice, then the scooped
                        // neckline inside it.
                        for y in sy as i32..(sy + if fig { 5.0 } else { 3.5 }) as i32 {
                            let fy = y as f32 + 0.5;
                            hspan(cv, (width(fy) - 1.2).max(1.0), y, yk[if y == sy as i32 { 2 } else { 3 }]);
                            let half = (4.0 - (fy - sy) * 0.7).max(0.0);
                            if half >= 1.0 {
                                hspan(cv, half, y, c.skin[if y == sy as i32 { 3 } else { 2 }]);
                            }
                        }
                    } else if back {
                        for y in sy as i32..(sy + 2.0) as i32 {
                            hspan(cv, (width(y as f32 + 0.5) - 1.2).max(1.0), y, yk[3]);
                        }
                    } else {
                        for y in sy as i32..(sy + 3.0) as i32 {
                            let fy = y as f32 + 0.5;
                            for x in (aft(fy) + 1.0) as i32..fwd(fy) as i32 {
                                cv.put(x, y, yk[3]);
                            }
                        }
                    }
                }
                Piece::Bikini(d, ties) => swim_top(cv, b, c, dir, by, &dye(c, d), &dye(c, ties), fwd),
                Piece::Trunks(d, band) => swim_trunks(cv, b, c, dir, &dye(c, d), &dye(c, band), aft, fwd),
                Piece::Ridge if front => {
                    for y in (sy + 2.0) as i32..(b.waist_y + 2.0) as i32 {
                        hspan(cv, 0.5, y, c.metal[3]);
                    }
                }
                _ => {}
            }
        }
    }
    if back {
        if let Some((outside, _)) = g.cape() {
            cloak_back(cv, b, &dye(c, outside), false, false);
            // A quiver is slung over the cloak.
            if g.pieces.contains(&Piece::Quiver) {
                quiver_back(cv, b, c);
            }
        }
    }
}

/// Fill the already-drawn body pixels in `rows`, where `inside(x, y)`
/// says so, as the current part; mirrored pairs stay mirrored.
fn cover(cv: &mut Canvas, y0: f32, y1: f32, mut inside: impl FnMut(i32, f32) -> Option<Rgb>) {
    for y in y0.floor() as i32..=y1.ceil() as i32 {
        let fy = y as f32 + 0.5;
        if fy < y0 || fy > y1 {
            continue;
        }
        for x in 0..FW {
            if cv.get(x, y).is_some() {
                if let Some(col) = inside(x, fy) {
                    cv.fill(x, y, col);
                }
            }
        }
    }
}

/// Triangle bikini top and tie-side bottoms over bare skin.
#[allow(clippy::too_many_arguments)]
fn swim_top(cv: &mut Canvas, b: &Build, c: &Colors, dir: u8, by: f32, cloth: &R5, ties: &R5, fwd: impl Fn(f32) -> f32) {
    let (cx, sy) = (b.cx, b.shoulder_y);
    let r = b.bust.max(2.5);
    let (top, bottom) = (by - r * 0.75, by + r * 0.85);
    let tone = |fy: f32| cloth[if fy > bottom - 1.0 { 1 } else if fy > by + r * 0.3 { 2 } else { 3 }];
    match dir {
        DIR_DOWN => {
            // One cup on the left, mirrored: apex up at the strap, widening
            // to the underline; the band joins them.
            let ax = cx - (r * 0.85).max(3.0);
            let first = cv.cur + 1;
            cv.begin();
            cover(cv, top, bottom, |x, fy| {
                let half = ((fy - top) * 0.8 + 0.6).min(r * 0.95 + 0.5);
                (x < 32 && (x as f32 + 0.5 - ax).abs() <= half).then(|| tone(fy))
            });
            cv.mirror_from(first);
            for y in (bottom - 1.0) as i32..=bottom as i32 {
                hspan(cv, 2.0, y, cloth[1]);
            }
            // Halter strings up to the neck.
            cv.line(v(ax, top), v(cx - 3.0, sy - 1.0), |_| Some(ties[2]));
            cv.line(v(64.0 - ax, top), v(cx + 3.0, sy - 1.0), |_| Some(ties[2]));
            navel(cv, b, c);
        }
        DIR_UP => {
            // The band and the tied strings across the back.
            cv.begin();
            let half = b.chest_w - 0.3;
            cover(cv, bottom - 2.0, bottom, |x, fy| ((x as f32 + 0.5 - 32.0).abs() <= half).then(|| cloth[if fy > bottom - 1.0 { 1 } else { 2 }]));
            // The halter ties in a small bow at the nape.
            cv.begin();
            for s in [-1.0f32, 1.0] {
                cv.ellipse(v(cx + s * 1.6, sy - 1.5), 1.3, 0.9, ties, 0.05);
            }
        }
        _ => {
            let front = fwd(by);
            cv.begin();
            cover(cv, top, bottom, |x, fy| (x as f32 + 0.5 >= front - 2.5 - (fy - top) * 0.35).then(|| tone(fy)));
            cv.line(v(front - 1.0, top), v(cx - 2.5, sy - 1.5), |_| Some(ties[2]));
            cv.line(v(front - 2.5, bottom - 1.0), v(fwd(by) - 8.0, bottom - 1.0), |_| Some(cloth[1]));
        }
    }
    // Bottoms: high-cut in front, fuller behind, tied at the hips.
    let (top, low) = (b.hip_y - 1.0, b.crotch_y + 1.5);
    let span = low - top;
    cv.begin();
    match dir {
        DIR_SIDE => {
            let (f0, a0) = (fwd(top), fwd(top) - 2.0 * (fwd(top) - b.cx));
            cover(cv, top, low, |x, fy| {
                let u = ((x as f32 + 0.5 - a0) / (f0 - a0).max(1.0)).clamp(0.0, 1.0);
                (fy <= low - u * 3.0).then(|| cloth[if fy > low - u * 3.0 - 1.0 { 2 } else { 3 }])
            });
        }
        _ => {
            let flare = if dir == DIR_UP { 0.5 } else { 0.8 };
            cover(cv, top, low, |x, fy| {
                let t = (fy - top) / span;
                let half = (b.hip_w - 0.6) * (1.0 - t * flare) + 0.6;
                ((x as f32 + 0.5 - 32.0).abs() <= half).then(|| cloth[if t > 0.75 { 2 } else { 3 }])
            });
            if dir == DIR_DOWN {
                cv.begin();
                for s in [-1.0f32, 1.0] {
                    cv.ellipse(v(32.0 + s * (b.hip_w - 0.4), top + 1.0), 1.1, 1.1, ties, 0.1);
                }
            }
        }
    }
}

/// Swim trunks from the hips over the tops of the thighs; the bare chest
/// gets a soft pec line and a navel.
#[allow(clippy::too_many_arguments)]
fn swim_trunks(cv: &mut Canvas, b: &Build, c: &Colors, dir: u8, cloth: &R5, band: &R5, aft: impl Fn(f32) -> f32, fwd: impl Fn(f32) -> f32) {
    if dir == DIR_DOWN {
        let y = (b.shoulder_y + 7.0) as i32;
        for x in (32.0 - b.chest_w * 0.75) as i32..30 {
            cv.put(x, y + ((30 - x) as f32 * 0.15) as i32, c.skin[2]);
            cv.put(FW - 1 - x, y + ((30 - x) as f32 * 0.15) as i32, c.skin[2]);
        }
        navel(cv, b, c);
    }
    let (top, low) = (b.waist_y + 2.0, b.crotch_y + 5.0);
    cv.begin();
    cover(cv, top, low, |x, fy| {
        let px = x as f32 + 0.5;
        let ok = if dir == DIR_SIDE { px >= aft(fy) - 1.5 && px <= fwd(fy) + 1.5 } else { (px - 32.0).abs() <= b.hip_w + 1.0 };
        ok.then(|| if fy < top + 1.5 { band[2] } else { cloth[if fy > low - 1.0 { 2 } else { 3 }] })
    });
}

fn navel(cv: &mut Canvas, b: &Build, c: &Colors) {
    let y = (b.waist_y + 1.0) as i32;
    cv.put(31, y, c.skin[2]);
    cv.put(32, y, c.skin[2]);
}

fn quiver_back(cv: &mut Canvas, b: &Build, c: &Colors) {
    let (cx, sy) = (b.cx, b.shoulder_y);
    let fletch = [hex(0x6a5a50), hex(0xa89888), hex(0xe0d4c4), hex(0xf8f0e4), hex(0xffffff)];
    cv.begin();
    cv.capsule(v(cx - 4.5, sy - 2.0), v(cx + 4.0, b.waist_y + 1.0), 2.4, 2.2, &c.leather, 0.05);
    cv.begin();
    for k in 0..3 {
        cv.ellipse(v(cx - 6.0 + k as f32 * 1.5, sy - 4.5 + k as f32 * 0.5), 0.9, 1.4, &fletch, 0.1);
    }
}

/// Side-view bust grown out of the chest line, as part of the same garment:
/// a gentle slope from the collarbone to the fullest point, then a round
/// underside curving back to the ribs, shaded darker underneath.
fn bust_side(cv: &mut Canvas, b: &Build, by: f32, ramp: &R5, front: impl Fn(f32) -> f32) {
    let r = b.bust;
    let top = by - r * 1.35;
    let apex = by + r * 0.1;
    // Squash pushes it forward and lifts the underside; stretch droops.
    let bottom = by + r * 0.95 + b.jsq;
    let depth = r - b.jsq;
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
            cv.ellipse(v(b.cx + 4.6, by), r * 0.85 - b.jsq * 0.5, r * 0.92 + b.jsq, ramp, 0.0);
        }
        DIR_DOWN => {
            // One soft mass, not two outlined balls: lit on top with a small
            // highlight, shaded underneath, a short cleavage line low in the
            // middle. The top blends into the chest; the contour pass lines
            // only the sides and underside.
            // Squash: a row shorter and half a pixel wider each side;
            // stretch: the opposite.
            let (rx, ry) = (r - b.jsq * 0.5, r * 0.9 + b.jsq);
            cv.begin();
            for y in (by - ry - 1.0).floor() as i32..=(by + ry + 1.0).ceil() as i32 {
                for x in 0..FW {
                    let px = x as f32 + 0.5;
                    let s = if px < b.cx { -1.0 } else { 1.0 };
                    let nx = (px - (b.cx + s * r * 0.92)) / rx;
                    let ny = (y as f32 + 0.5 - by) / ry;
                    if nx * nx + ny * ny > 1.0 + EPS {
                        continue;
                    }
                    // nx measured outward from the cleavage on both sides.
                    let nxo = nx * s;
                    let i = if ny > 0.78 {
                        1
                    } else if ny > 0.4 {
                        2
                    } else if (ny + 0.5 - b.jhl / ry).abs() < 0.1 && nxo > 0.0 && nxo < 0.42 {
                        4
                    } else {
                        3
                    };
                    cv.fill(x, y, ramp[i]);
                }
            }
            let c0 = (by - ry * 0.1).round() as i32;
            for y in c0..(by + ry * 0.75).round() as i32 {
                let col = if y < c0 + 2 { ramp[2] } else { ramp[1] };
                cv.put(31, y, col);
                cv.put(32, y, col);
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
        hspan(cv, 2.0, y as i32 - 1, g[3]);
        hspan(cv, 2.0, y as i32, g[2]);
        hspan(cv, 1.0, y as i32 - 1, g[1]);
    }
}

fn sash(cv: &mut Canvas, b: &Build, ramp: &R5, lean: impl Fn(f32) -> f32, width: impl Fn(f32) -> f32) {
    let y = b.waist_y + 1.0;
    let w = width(y) + 0.3;
    let c = lean(y);
    cv.begin();
    cv.profile(y - 1.0, y + 1.5, |_| c, |_| w, ramp, 0.05);
}

/// Skirt start half-width and flare per row. In profile a skirt only has
/// the body's depth to hang from and flares far less.
fn skirt_shape(side: bool, w0: f32, k: f32) -> (f32, f32) {
    if side { (6.2, k * 0.5) } else { (w0, k) }
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

/// Cape: over the back in the back view; behind the body in front and
/// profile views (`behind`), where only its lining shows past the arms.
fn cloak_back(cv: &mut Canvas, b: &Build, ramp: &R5, side: bool, behind: bool) {
    let top = b.shoulder_y - 0.5;
    let bottom = b.ground - 7.0;
    let cx = if side { b.cx - 3.5 } else { b.cx };
    let lag = b.sway;
    let sw = b.shoulder_w;
    cv.begin();
    cv.profile(top, bottom, |y| cx - (y - top) / (bottom - top) * lag * 1.5, |y| if side { 5.5 + (y - top) * 0.14 } else if behind { sw + 1.0 + (y - top) * 0.1 } else { sw + 2.0 + (y - top) * 0.2 }, ramp, -0.1);
}

fn held_item(cv: &mut Canvas, look: &Look, hand: V2, dir: u8, raised: bool, c: &Colors) {
    let wood = [hex(0x2e1c1c), hex(0x4a2e24), hex(0x6b412f), hex(0x8c5a38), hex(0xab7645)];
    let (x, y) = (hand.x, hand.y);
    match look.held {
        held::STAFF => {
            // Taller than its bearer: the orb rides above the head. In
            // profile it leans a little forward, clear of the face.
            let top = (if raised { y - 26.0 } else { y - 56.0 }).max(9.0);
            let lean = if dir == DIR_SIDE { 3.0 } else { 0.0 };
            let x = (x + lean * 0.3).min(FW as f32 - 8.0 - lean);
            cv.capsule(v(x + lean, top), v(x - lean * 0.4, (y + 14.0).min(FH as f32 - 4.0)), 1.3, 1.3, &wood, 0.0);
            let x = x + lean;
            let gem = [hex(0x2a1050), hex(0x4a2490), hex(0x7a48d0), hex(0xb090f8), hex(0xf0e6ff)];
            cv.capsule(v(x - 3.0, top + 1.0), v(x, top - 2.5), 1.2, 1.0, &wood, 0.0);
            cv.capsule(v(x + 3.0, top + 1.0), v(x, top - 2.5), 1.2, 1.0, &wood, 0.0);
            cv.ellipse(v(x, top - 3.5), 3.0, 3.2, &gem, 0.1);
        }
        held::HAMMER => {
            // Gripped near the end of the handle: head up when swinging,
            // head down and resting otherwise.
            let (grip, head) = if raised { (v(x, y + 2.0), v(x, y - 11.0)) } else { (v(x, y - 2.0), v((x + 1.0).min(FW as f32 - 9.0), (y + 9.0).min(FH as f32 - 6.0))) };
            cv.capsule(grip, head, 1.2, 1.2, &wood, 0.0);
            cv.begin();
            cv.capsule(v(head.x - 4.0, head.y), v(head.x + 4.0, head.y), 2.6, 2.6, &c.metal, 0.05);
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
            // Raised overhead, or carried low with the tip angled forward so
            // it stays clear of the ground.
            let (a, bb) = if raised { (v(x, y - 22.0), v(x, y + 1.0)) } else { (v(x, y + 1.0), v((x + 7.0).min(FW as f32 - 5.0), y + 17.0)) };
            cv.capsule(a, bb, 1.2, 0.9, &c.metal, 0.1);
            let g = if raised { bb } else { v(x + 0.6, y + 2.0) };
            cv.capsule(v(g.x - 3.5, g.y), v(g.x + 3.5, g.y), 1.1, 1.1, &gold5(), 0.0);
        }
        held::AXE | held::PICKAXE => {
            // Gripped near the handle's end: head down at rest, up to swing.
            let (grip, head) = if raised { (v(x, y + 2.0), v(x, y - 11.0)) } else { (v(x, y - 3.0), v((x + 1.0).min(FW as f32 - 9.0), (y + 10.0).min(FH as f32 - 6.0))) };
            cv.capsule(grip, head, 1.2, 1.2, &wood, 0.0);
            cv.begin();
            if look.held == held::AXE {
                // Bearded blade flaring out to a curved, bright cutting edge.
                let up = if raised { -1.0 } else { 1.0 };
                for k in 0..6 {
                    let t = k as f32;
                    let half = 1.2 + t * 0.55;
                    let x = (head.x + 1.0 + t).round() as i32;
                    for dy in (-half).round() as i32..=(half).round() as i32 {
                        let edge = k == 5;
                        let col = if edge { c.metal[4] } else if dy as f32 * up < 0.0 { c.metal[3] } else { c.metal[2] };
                        cv.fill(x, (head.y + dy as f32 * up).round() as i32, col);
                    }
                }
            } else {
                // Pick: a curved spike either way.
                for s in [-1.0f32, 1.0] {
                    cv.capsule(head, head.add(v(s * 5.0, 1.5)), 1.8, 0.6, &c.metal, 0.05);
                }
            }
        }
        held::HOE => {
            let (top, bot) = if raised { (v(x - 2.0, y - 16.0), v(x + 1.0, y + 8.0)) } else { (v(x, (y - 14.0).max(11.0)), v(x, (y + 15.0).min(FH as f32 - 6.0))) };
            cv.capsule(top, bot, 1.1, 1.1, &wood, 0.0);
            cv.begin();
            let blade = if raised { top } else { bot };
            cv.capsule(blade, blade.add(v(4.0, if raised { -1.0 } else { 1.0 })), 1.6, 1.4, &c.metal, 0.05);
        }
        held::ROD => {
            let tip = v((x + 9.0).min(FW as f32 - 4.0), (y - 26.0).max(6.0));
            cv.capsule(v(x, y + 2.0), tip, 1.1, 0.6, &wood, 0.0);
            cv.begin();
            let line = [hex(0x8a8a9a), hex(0xb8b8c8), hex(0xd8d8e4), hex(0xeeeef6), hex(0xffffff)];
            cv.capsule(tip, v(tip.x + 1.0, y + 6.0), 0.5, 0.5, &line, 0.0);
            let red = [hex(0x5a1010), hex(0x9a2020), hex(0xd84040), hex(0xf07070), hex(0xffa0a0)];
            cv.ellipse(v(tip.x + 1.0, y + 7.0), 1.1, 1.1, &red, 0.0);
        }
        held::SPEAR => {
            let top = (y - 34.0).max(13.0);
            cv.capsule(v(x, top), v(x, (y + 16.0).min(FH as f32 - 6.0)), 1.1, 1.1, &wood, 0.0);
            cv.begin();
            cv.capsule(v(x, top), v(x, top - 7.0), 2.0, 0.4, &c.metal, 0.1);
            let red = [hex(0x5a1010), hex(0x9a2020), hex(0xd84040), hex(0xf07070), hex(0xffa0a0)];
            cv.ellipse(v(x, top + 1.5), 1.6, 1.2, &red, 0.0);
        }
        held::LADLE if dir != DIR_UP => {
            cv.capsule(v(x, y - 6.0), v(x, y + 4.0), 0.8, 0.8, &c.metal, 0.0);
            cv.begin();
            cv.ellipse(v(x + 0.5, y + 6.0), 2.4, 1.8, &c.metal, 0.1);
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
    // Men: a lean, angled jaw from below the eyes to a defined chin.
    const MALE: [f32; 15] = [12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 11.0, 11.0, 10.0, 9.0, 7.0, 5.0];
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
    // Cranium (under the hair).
    (-6, 5),
    (-9, 8),
    (-10, 9),
    (-11, 10),
    (-12, 11),
    (-12, 11),
    (-13, 11),
    (-13, 12),
    (-13, 12),
    (-13, 12),
    (-13, 12),
    (-13, 12),
    (-13, 12),
    // Brow, a slight eye socket, cheek, nose tip at the eye's bottom, lips,
    // chin. Behind the ear the skull narrows straight into the nape, and the
    // jaw angles from under the ear down to the chin.
    (-13, 12),
    (-13, 12),
    (-13, 11),
    (-12, 11),
    (-12, 11),
    (-11, 11),
    (-5, 11),
    (-4, 12),
    (-4, 12),
    (-4, 13),
    (-4, 12),
    (-4, 11),
    (-3, 11),
    (2, 10),
    (4, 9),
];

fn head(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
    let hc = b.head_c;
    let (cx, cy) = (32, hc.y.round() as i32);
    cv.begin();
    match dir {
        DIR_SIDE => {
            for dy in -13..=14 {
                let (l, r) = SIDE_HEAD[(dy + 13) as usize];
                for x in cx + l..cx + r {
                    let edge = x == cx + r - 1 || (dy >= 9 && x < cx + l + 2);
                    cv.fill(x, cy + dy, c.skin[if edge { 2 } else { 3 }]);
                }
            }
        }
        _ => {
            // From behind, anime heads hide the ears under the hair.
            for dy in -13..=14 {
                // From behind there is no jaw: the skull under the hair,
                // then only the neck.
                // From behind, the back of the skull and jaw narrow smoothly
                // into the neck under the hairline.
                let hw = if dir == DIR_UP && dy > 4 {
                    let neck = if look.female { 4.5 } else { 5.5 };
                    let top = face_hw(look, 4);
                    top + (neck - top) * ((dy - 4) as f32 / 8.0).min(1.0).powf(0.7)
                } else {
                    face_hw(look, dy)
                };
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

/// Human ear from the front, left side (mirrored for the right): the rim
/// outside, the shadowed bowl, joining the head on the right.
const EAR_FRONT: [&str; 6] = [".LM", "LLM", "LDM", "LDM", ".LM", "..M"];

/// Human ear in profile, facing right: the helix rim at the top and back,
/// the shadowed bowl in the middle, the lobe at the bottom.
const EAR_SIDE: [&str; 6] = [
    ".ML.",
    "MLLL",
    "MLDL",
    "MLDL",
    "MLL.",
    ".ML.",
];

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
        facial_hair(cv, look, dir, cy, c);
    } else if dir == DIR_SIDE {
        let ex = 36;
        let ey = cy + 3;
        stamp(cv, ex, ey, if pose.blink { &EYE_SIDE_CLOSED } else { &EYE_SIDE }, false, &eye_pal);
        if soft {
            for k in 0..3 {
                cv.put(ex - 1 + k, cy + 11, blush);
            }
        }
        cv.put(32 + 9, cy + 13, lip);
        facial_hair(cv, look, dir, cy, c);
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

/// Facial hair as bold flat shapes, anime-style: each reads by its outline
/// alone, in two tones (hair colour and one step darker along the bottom
/// and outer edges), no highlight dots or strands; the mouth stays clear.
/// Shapes are rows of (dy below the head centre, x0, x1): the front view
/// gives the left half and mirrors it, the profile gives it directly.
fn facial_hair(cv: &mut Canvas, look: &Look, dir: u8, cy: i32, c: &Colors) {
    let style = look.facial_style();
    if style == 0 || dir == DIR_UP {
        return;
    }
    let h = &c.hair;
    let front = dir == DIR_DOWN;
    if style == 1 {
        // Stubble: a flat shadow tone on the jaw and upper lip, no outline.
        let tone = mix(c.skin[2], h[1], 0.32);
        let rows: &[(i32, i32, i32)] = if front {
            &[(10, 21, 22), (11, 22, 23), (12, 23, 24), (12, 29, 31), (13, 25, 27), (14, 27, 31)]
        } else {
            &[(10, 31, 32), (11, 32, 33), (12, 33, 35), (12, 39, 40), (13, 34, 38), (14, 36, 39)]
        };
        for &(dy, x0, x1) in rows {
            for x in x0..=x1 {
                cv.put(x, cy + dy, tone);
                if front {
                    cv.put(FW - 1 - x, cy + dy, tone);
                }
            }
        }
        return;
    }
    // (dy, x0, x1) spans; front = left half up to the centre column 31.
    let shape: Vec<(i32, i32, i32)> = match (style, front) {
        // Hero block (Omni-Man): flat-topped under the nose, thick over the
        // outer thirds, square ends dropping past the mouth corners.
        (2, true) => vec![(11, 26, 30), (12, 25, 31), (13, 25, 28), (14, 25, 27)],
        (2, false) => vec![(11, 37, 41), (12, 36, 41), (13, 36, 38)],
        // Van Dyke: a thin mustache and a pointed wedge under the lip.
        (3, true) => vec![(12, 28, 31), (14, 29, 31), (15, 30, 31), (16, 31, 31)],
        (3, false) => vec![(12, 39, 41), (14, 37, 40), (15, 37, 39), (16, 37, 38)],
        // Short boxed: a band from the sideburn along the jaw, joined to
        // the mustache, flat across under the chin.
        (4, true) => vec![(4, 20, 21), (5, 20, 21), (6, 20, 21), (7, 20, 21), (8, 20, 21), (9, 21, 22), (10, 21, 22), (11, 22, 24), (12, 23, 25), (12, 28, 31), (13, 25, 29), (14, 27, 31), (15, 28, 31)],
        (4, false) => vec![(4, 31, 32), (5, 31, 32), (6, 31, 32), (7, 31, 32), (8, 31, 32), (9, 31, 33), (10, 31, 33), (11, 32, 34), (12, 33, 36), (12, 39, 41), (13, 34, 38), (14, 36, 40), (15, 35, 39)],
        // Full warrior: cheeks below the cheekbone, past the chin, a lobed
        // bottom edge.
        (5, true) => vec![(4, 20, 21), (5, 20, 21), (6, 20, 21), (7, 20, 21), (8, 20, 22), (9, 21, 23), (10, 21, 24), (11, 22, 26), (12, 23, 31), (13, 25, 31), (14, 26, 31), (15, 26, 31), (16, 27, 31), (17, 27, 29), (17, 31, 31)],
        (5, false) => vec![(4, 31, 32), (5, 31, 32), (6, 31, 32), (7, 31, 32), (8, 31, 33), (9, 31, 34), (10, 31, 35), (11, 31, 37), (12, 31, 41), (13, 32, 41), (14, 33, 41), (15, 34, 41), (16, 34, 40), (17, 35, 36), (17, 38, 39)],
        // Sage: full, then a long wedge narrowing to a point well below the
        // chin.
        (6, true) => {
            let mut v = vec![(4, 20, 21), (5, 20, 21), (6, 20, 21), (7, 20, 21), (8, 20, 22), (9, 21, 23), (10, 21, 24), (11, 22, 26), (12, 23, 31), (13, 25, 31)];
            for (k, x0) in [26, 26, 27, 27, 28, 28, 29, 29, 30, 30, 31].into_iter().enumerate() {
                v.push((14 + k as i32, x0, 31));
            }
            v
        }
        (6, false) => {
            let mut v = vec![(4, 31, 32), (5, 31, 32), (6, 31, 32), (7, 31, 32), (8, 31, 33), (9, 31, 34), (10, 31, 35), (11, 31, 37), (12, 31, 41), (13, 32, 41)];
            for k in 0..11 {
                let t = k as f32;
                v.push((14 + k, (33.0 + t * 0.55).round() as i32, (41.0 - t * 0.25).round() as i32));
            }
            v
        }
        // Crescent (Whitebeard): a long sweep over the lip, tips curling up.
        (_, true) => vec![(10, 23, 23), (11, 23, 25), (12, 25, 31), (13, 26, 29)],
        (_, false) => vec![(10, 36, 36), (11, 36, 37), (12, 37, 41)],
    };
    let inside = |x: i32, dy: i32| shape.iter().any(|&(d, x0, x1)| d == dy && x >= x0 && x <= x1);
    cv.begin();
    for &(dy, x0, x1) in &shape {
        for x in x0..=x1 {
            // Edge tone along the bottom and the outer (cheek or back) side.
            let edge = !inside(x, dy + 1) || !inside(x - 1, dy);
            let col = h[if edge { 1 } else { 2 }];
            cv.fill(x, cy + dy, col);
            if front && x < 32 {
                cv.fill(FW - 1 - x, cy + dy, col);
            }
        }
    }
    // Beards over the lips keep the mouth as a dark notch.
    if matches!(style, 5 | 6) {
        let dark = mix(h[0], pal::INK, 0.3);
        if front {
            hspan(cv, 1.5, cy + 13, dark);
        } else {
            cv.put(40, cy + 13, dark);
            cv.put(41, cy + 13, dark);
        }
    }
}

// ---------------------------------------------------------------- hair

/// What a hairstyle adds on top of the basic cap, bangs, locks and back
/// sheet.
#[derive(Clone, Copy, PartialEq)]
enum Extra {
    None,
    /// High ponytail tied at the back of the crown.
    Ponytail,
    /// One bun on top.
    Bun,
    /// Shonen spikes fanned over the crown.
    Spikes,
    /// Twin tails tied high at the sides.
    TwinTails,
    /// Ojou-sama tate-roll ringlets hanging at the temples.
    Drills,
    /// Two braids from behind the ears, over the shoulders.
    TwinBraids,
    /// One thick braid over the right shoulder.
    SideBraid,
    /// Very long hair with a small half-up loop and ribbon at the back.
    HalfUp,
    /// Messy protagonist hair: an ahoge strand and tufts at the nape.
    Messy,
    /// Long hair tied low at the nape (butler, elf lord).
    LowTie,
    /// Combed straight back off the forehead.
    Slick,
    /// Long top with clipped sides.
    Undercut,
    /// Samurai: hair tied high at the crown, the tail to the shoulders.
    TopKnot,
    /// Bald crown with a horseshoe of hair at the sides and back.
    Bald,
}

/// How a hairstyle is cut.
struct Cut {
    /// Bang tips: (x offset from centre, tip y offset from head centre,
    /// slope of the clump's sides). Lower slope = blunter cut.
    tips: &'static [(f32, f32, f32)],
    /// The same bangs seen in profile (x offset forward of the head centre).
    side_tips: &'static [(f32, f32, f32)],
    /// Side locks framing the face: length below the head centre, width.
    lock: (f32, f32),
    /// Length of the back mass below the head centre (front view behind the
    /// body); 0 = short.
    back: f32,
    /// How far the hair reaches below the head centre at the nape.
    nape: f32,
    /// Hair pulled up or tied: the nape and the hairline show from behind.
    gathered: bool,
    /// Locks cut straight across instead of tapering to a point.
    blunt: bool,
    /// Depth of the V in the back sheet's hem (0 = straight across).
    hem: f32,
    /// Close to the skull (short men's cuts).
    tight: bool,
    extra: Extra,
}

const SIDE_SOFT: &[(f32, f32, f32)] = &[(5.0, 2.0, 1.5), (9.0, 0.5, 1.6), (12.5, 1.5, 1.4)];
const SIDE_SHORT: &[(f32, f32, f32)] = &[(5.0, 0.0, 1.6), (8.5, 1.0, 1.5), (12.0, -1.0, 1.3)];

fn cut(style: u8) -> Cut {
    let base = Cut { tips: &[], side_tips: SIDE_SOFT, lock: (0.0, 3.0), back: 0.0, nape: 9.0, gathered: false, blunt: false, hem: 6.0, tight: false, extra: Extra::None };
    match style {
        // Short, slightly messy (masculine).
        0 => Cut { tips: &[(-11.0, -1.0, 1.6), (-7.5, 0.0, 1.8), (-3.5, 1.0, 1.8), (0.5, 0.0, 1.8), (4.5, 1.0, 1.8), (8.5, -0.5, 1.6), (12.0, -2.0, 1.4)], side_tips: SIDE_SHORT, lock: (4.0, 3.0), nape: 8.0, tight: true, ..base },
        // Long straight.
        1 => Cut { tips: &[(-10.5, 4.5, 1.5), (-6.5, 0.5, 1.7), (-2.0, 4.0, 1.7), (2.0, 4.0, 1.7), (6.5, 0.5, 1.7), (10.5, 4.5, 1.5)], lock: (24.0, 4.0), back: 46.0, nape: 12.0, ..base },
        // Ponytail.
        2 => Cut { tips: &[(-10.5, 4.0, 1.5), (-6.5, 1.5, 1.6), (-2.0, 3.5, 1.7), (2.5, 2.5, 1.7), (6.5, 1.0, 1.6), (10.5, 3.5, 1.5)], lock: (12.0, 3.5), nape: 6.0, gathered: true, extra: Extra::Ponytail, ..base },
        // Bun.
        3 => Cut { tips: &[(-10.5, 3.5, 1.5), (-6.5, 1.0, 1.8), (-2.5, 3.0, 1.8), (2.5, 3.0, 1.8), (6.5, 1.0, 1.8), (10.5, 3.5, 1.5)], lock: (9.0, 3.0), nape: 6.0, gathered: true, extra: Extra::Bun, ..base },
        // Spiky hero.
        4 => Cut { tips: &[(-11.0, 0.5, 1.3), (-7.0, 1.5, 1.5), (-3.0, 2.0, 1.6), (1.0, 1.0, 1.6), (5.0, 2.0, 1.5), (9.0, 0.5, 1.4), (12.0, -1.0, 1.3)], side_tips: SIDE_SHORT, lock: (6.0, 3.5), tight: true, extra: Extra::Spikes, ..base },
        // Bob with a blunt fringe.
        5 => Cut { tips: &[(-10.0, 2.0, 0.6), (-6.5, 1.5, 0.6), (-2.0, 2.0, 0.6), (2.0, 2.0, 0.6), (6.5, 1.5, 0.6), (10.0, 2.0, 0.6)], side_tips: &[(5.0, 1.5, 0.7), (9.0, 1.5, 0.7), (12.5, 1.0, 0.7)], lock: (13.0, 5.5), back: 12.0, nape: 13.0, ..base },
        // Twin tails.
        6 => Cut { tips: &[(-10.5, 4.5, 1.5), (-6.5, 0.5, 1.7), (-2.0, 4.0, 1.7), (2.0, 4.0, 1.7), (6.5, 0.5, 1.7), (10.5, 4.5, 1.5)], lock: (11.0, 3.5), extra: Extra::TwinTails, ..base },
        // Side-swept.
        7 => Cut { tips: &[(-10.0, 4.0, 1.2), (-6.0, 3.5, 1.2), (-2.0, 2.5, 1.3), (2.0, 1.0, 1.4), (6.0, -0.5, 1.5), (10.0, -1.0, 1.5)], side_tips: &[(5.0, 3.5, 1.2), (9.0, 2.5, 1.3), (12.5, 0.0, 1.4)], lock: (14.0, 4.0), back: 10.0, nape: 11.0, ..base },
        // Hime cut: dead-straight bangs at the brow, side locks cut level
        // at the chin, a sleek sheet to the waist with a straight hem.
        8 => Cut { tips: &[(-10.5, 2.0, 0.2), (-6.5, 2.0, 0.2), (-2.0, 2.0, 0.2), (2.0, 2.0, 0.2), (6.5, 2.0, 0.2), (10.5, 2.0, 0.2)], side_tips: &[(5.0, 2.0, 0.25), (9.0, 2.0, 0.25), (12.5, 2.0, 0.25)], lock: (13.5, 5.0), back: 46.0, nape: 12.0, blunt: true, hem: 0.0, ..base },
        // Ojou drills: swept bangs, ringlets at the temples, hair to the
        // shoulder blades behind.
        9 => Cut { tips: &[(-10.5, 4.0, 1.4), (-6.0, 1.0, 1.6), (-2.0, 3.0, 1.7), (2.0, 3.0, 1.7), (6.0, 1.0, 1.6), (10.5, 4.0, 1.4)], lock: (7.0, 3.0), back: 18.0, nape: 12.0, hem: 3.0, extra: Extra::Drills, ..base },
        // Twin braids (osage): centre part, a braid from behind each ear.
        10 => Cut { tips: &[(-10.0, 4.5, 1.2), (-6.0, 3.0, 1.4), (-2.5, 0.0, 1.8), (2.5, 0.0, 1.8), (6.0, 3.0, 1.4), (10.0, 4.5, 1.2)], lock: (6.0, 3.0), gathered: true, extra: Extra::TwinBraids, ..base },
        // Side braid: soft swept bangs, one thick plait over the shoulder.
        11 => Cut { tips: &[(-10.0, 4.0, 1.3), (-6.0, 3.0, 1.3), (-2.0, 2.0, 1.4), (2.0, 1.5, 1.4), (6.0, 0.5, 1.5), (10.0, 0.0, 1.5)], side_tips: &[(5.0, 3.0, 1.3), (9.0, 2.0, 1.3), (12.5, 0.5, 1.4)], lock: (10.0, 3.5), gathered: true, extra: Extra::SideBraid, ..base },
        // Very long with a half-up loop: parted bangs, locks past the chest.
        12 => Cut { tips: &[(-10.5, 5.0, 1.3), (-6.5, 2.0, 1.5), (-2.5, -0.5, 1.8), (2.5, -0.5, 1.8), (6.5, 2.0, 1.5), (10.5, 5.0, 1.3)], lock: (28.0, 4.0), back: 58.0, nape: 12.0, hem: 7.0, extra: Extra::HalfUp, ..base },
        // Messy isekai protagonist: uneven bangs to the eyes, an ahoge.
        13 => Cut { tips: &[(-11.0, 1.0, 1.5), (-7.5, 3.0, 1.6), (-3.5, 4.5, 1.8), (0.5, 2.5, 1.9), (4.0, 5.0, 1.7), (8.0, 2.5, 1.6), (11.5, 0.0, 1.4)], side_tips: &[(5.0, 2.5, 1.5), (8.5, 3.5, 1.5), (12.5, 1.0, 1.3)], lock: (6.0, 3.2), nape: 10.0, tight: true, extra: Extra::Messy, ..base },
        // Long-haired pretty boy: parted bangs, long locks, tied low.
        14 => Cut { tips: &[(-10.5, 3.0, 1.4), (-6.5, 2.0, 1.6), (-2.5, -1.0, 1.8), (2.5, -1.0, 1.8), (6.5, 2.0, 1.6), (10.5, 3.0, 1.4)], lock: (20.0, 3.5), nape: 10.0, gathered: true, extra: Extra::LowTie, ..base },
        // Slicked back: the forehead bare, combed over the crown.
        15 => Cut { tips: &[(-10.0, -4.0, 0.9), (-5.0, -5.5, 0.6), (0.0, -6.0, 0.5), (5.0, -5.5, 0.6), (10.0, -4.0, 0.9)], side_tips: &[(5.0, -5.5, 0.6), (9.0, -6.0, 0.6), (12.5, -5.0, 0.8)], nape: 9.0, tight: true, extra: Extra::Slick, ..base },
        // Undercut: centre-parted top over clipped sides.
        16 => Cut { tips: &[(-8.0, 0.0, 1.4), (-4.5, 1.5, 1.6), (-1.5, -3.0, 1.8), (1.5, -3.0, 1.8), (4.5, 1.5, 1.6), (8.0, 0.0, 1.4)], side_tips: &[(5.0, 0.5, 1.5), (8.5, 1.0, 1.5), (12.0, -0.5, 1.3)], nape: 8.0, tight: true, extra: Extra::Undercut, ..base },
        // Samurai: widow's peak, long locks, tied high at the crown.
        17 => Cut { tips: &[(-10.5, 3.0, 1.4), (-6.0, 0.5, 1.6), (-2.0, -3.0, 1.8), (2.0, -3.0, 1.8), (6.0, 0.5, 1.6), (10.5, 3.0, 1.4)], lock: (16.0, 3.0), nape: 8.0, gathered: true, extra: Extra::TopKnot, ..base },
        // Bald with a horseshoe fringe (elders).
        _ => Cut { tips: &[], side_tips: &[], nape: 8.0, tight: true, extra: Extra::Bald, ..base },
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
    if cut(look.hair_style).tight { (14.0, 14.5) } else { (14.5, 15.0) }
}

/// Long-hair width down the back: just under the skull width at the nape,
/// a gentle flare over the shoulders, then a taper toward the tips.
fn sheet_w(rx: f32, t: f32) -> f32 {
    rx * (0.93 + 0.1 * (std::f32::consts::PI * (t / 0.35).min(1.0)).sin() * (1.0 - t * 0.3)) * (1.0 - 0.28 * t * t)
}

fn back_hair(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
    let h = &c.hair;
    let hc = b.head_c;
    let cy = hc.y;
    let side = dir == DIR_SIDE;
    let sway = b.sway;
    let k = cut(look.hair_style);
    cv.begin();
    if k.back > 0.0 {
        let (rx, _) = mass_r(look);
        if side {
            // Continues the skull's back curve, then falls along the back,
            // narrowing toward the tips.
            let top = cy - 4.0;
            let len = k.back + 2.0;
            let back_x = |t: f32| 17.0 + 5.0 * t.powf(1.2) + t * t * sway * 2.2;
            let front_x = |t: f32| 33.0 - 4.0 * t + t * t * sway * 1.2;
            hair_sheet(cv, h, top, top + len, true, k.hem * 0.5, |y| {
                let t = ((y - top) / len).clamp(0.0, 1.0);
                (back_x(t) + front_x(t)) * 0.5
            }, |y| {
                let t = ((y - top) / len).clamp(0.0, 1.0);
                (front_x(t) - back_x(t)) * 0.5
            });
        } else {
            // From the front only what shows past the body matters; ending
            // above the knee keeps it from boxing in the legs.
            let top = cy - 2.0;
            let len = k.back.min(b.crotch_y + 6.0 - top);
            hair_sheet(cv, h, top, top + len, false, (k.hem - 1.0).max(0.0), |_| 32.0, |y| sheet_w(rx, ((y - top) / len).clamp(0.0, 1.0)));
        }
    }
    // Tails that hang behind the body.
    let tie = r5(&ramp4(&pal::CLOTH[1]));
    match k.extra {
        Extra::Ponytail if side => {
            // Tied high at the back of the head: sweeps out, then falls.
            let at = v(32.0 - 12.5, cy - 7.0);
            let mid = at.add(v(-4.5 - sway, 7.0));
            let end = mid.add(v(-1.0 - sway * 2.0, 17.0));
            lock_segment(cv, h, at, mid, 3.6, false);
            tail_lock(cv, h, mid.add(v(0.5, -1.0)), end, 4.2);
            cv.begin();
            cv.ellipse(at.add(v(0.5, 0.0)), 2.0, 2.4, &tie, 0.05);
        }
        Extra::Ponytail => {
            let base = v(32.0 + 11.0, cy + 2.0);
            tail_lock(cv, h, base, base.add(v(2.0 + sway, 18.0)), 3.2);
        }
        Extra::TwinTails => {
            let sides: &[f32] = if side { &[-1.0] } else { &[-1.0, 1.0] };
            for &s in sides {
                let base = v(32.0 + s * 14.5 + if side { 4.0 } else { 0.0 }, cy - 4.0);
                tail_lock(cv, h, base, base.add(v(s * 3.0 + sway, 30.0)), 4.0);
            }
        }
        // A long tail tied at the nape, straight down the back.
        Extra::LowTie if side => {
            let at = v(20.5, cy + 6.0);
            tail_lock(cv, h, at, v(19.5 - sway * 1.5, cy + 34.0), 3.0);
        }
        // Samurai: tied high at the back of the crown, the tail arcing out
        // and down to the shoulders.
        Extra::TopKnot if side => {
            let at = v(20.5, cy - 10.0);
            let mid = at.add(v(-4.0 - sway * 0.5, 4.0));
            lock_segment(cv, h, at, mid, 3.0, false);
            tail_lock(cv, h, mid.add(v(0.3, -0.8)), mid.add(v(-1.0 - sway * 2.0, 20.0)), 3.4);
        }
        _ => {}
    }
}

/// A falling sheet of hair: clumps laid out from the centre (so both halves
/// mirror) ending in pointed tips of uneven length on a V-shaped hem
/// (`vdepth` px deeper at the centre). Flat banded shading: the upper part
/// on the base tone, the lower part and outer edges a step darker; flow
/// lines are drawn separately so they follow the head.
#[allow(clippy::too_many_arguments)]
fn hair_sheet(cv: &mut Canvas, h: &R5, top: f32, end: f32, lit: bool, vdepth: f32, cx: impl Fn(f32) -> f32, hw: impl Fn(f32) -> f32) {
    let len = (end - top).max(1.0);
    let base = if lit { 3 } else { 2 };
    for y in top.floor() as i32..=end.ceil() as i32 {
        let fy = y as f32 + 0.5;
        let t = ((fy - top) / len).clamp(0.0, 1.0);
        let c = cx(fy);
        let w = hw(fy);
        if !c.is_finite() || !w.is_finite() || w <= 0.0 {
            continue;
        }
        for x in (c - w).floor() as i32..=(c + w).ceil() as i32 {
            let px = x as f32 + 0.5;
            let d = (px - c).abs();
            let u = (px - c) / w.max(0.5);
            if u.abs() > 1.0 + EPS {
                continue;
            }
            let q = (d + 3.5) / 7.0;
            let (k, uc) = (q.floor(), q - q.floor());
            // Each clump has its own length (deterministic, mirrored).
            let extra = [0.0, 2.0, 0.5, 3.0, 1.0][(k as usize) % 5];
            let tip = (1.0 - (uc - 0.5).abs() * 2.0) * 4.0;
            // A zero-depth hem is cut straight across (hime cut).
            let hem = if vdepth <= 0.0 { end } else { end - u.abs().powf(1.3) * vdepth - (4.0 - tip) - extra };
            if fy > hem {
                continue;
            }
            let edge = u.abs() > 0.88;
            let lower = t > 0.6 && (uc < 0.3 || uc > 0.75);
            let tone = if edge || lower { base - 1 } else { base };
            cv.fill(x, y, h[tone]);
        }
    }
}

/// Flow lines over a hair mass: from the crown, over the curve of the
/// skull, then straight down to `bottom`. Mirrored pairs at each offset.
fn flow_lines(cv: &mut Canvas, h: &R5, cx: f32, crown_y: f32, skull_y: f32, bottom: f32, offsets: &[f32]) {
    for &o in offsets {
        for s in [-1.0f32, 1.0] {
            let a = v(cx + s * o * 0.35, crown_y);
            let m = v(cx + s * o * 1.05, skull_y);
            let e = v(cx + s * o, bottom);
            cv.line(a, m, |t| if t < 0.35 { None } else { Some(h[2]) });
            cv.line(m, e, |t| if t > 0.85 { Some(h[1]) } else { Some(h[2]) });
        }
    }
}

/// A tapered lock (ponytail, twin tail) with flat banded shading.
fn tail_lock(cv: &mut Canvas, h: &R5, a: V2, b: V2, r: f32) {
    lock_segment(cv, h, a, b, r, true);
}

/// Point and unit direction at arc length `s` along a polyline.
fn along(pts: &[V2], s: f32) -> (V2, V2) {
    let mut left = s;
    let last = pts.len().saturating_sub(2);
    for (i, w) in pts.windows(2).enumerate() {
        let (dx, dy) = (w[1].x - w[0].x, w[1].y - w[0].y);
        let l = (dx * dx + dy * dy).sqrt().max(1e-4);
        if left <= l || i == last {
            let t = (left / l).min(1.0);
            return (v(w[0].x + dx * t, w[0].y + dy * t), v(dx / l, dy / l));
        }
        left -= l;
    }
    (pts[0], v(0.0, 1.0))
}

fn path_len(pts: &[V2]) -> f32 {
    pts.windows(2).map(|w| ((w[1].x - w[0].x).powi(2) + (w[1].y - w[0].y).powi(2)).sqrt()).sum()
}

/// A three-strand braid along a path: round lobes stepping left and right,
/// each its own part so the contour pass draws the tuck where it overlaps
/// the next; a tie and a small brush tuft at the end.
fn braid(cv: &mut Canvas, h: &R5, pts: &[V2], r0: f32, r1: f32, tie: &R5, flip: f32) {
    let total = path_len(pts);
    let step = (r0 * 1.15).max(2.2);
    let n = ((total - 2.0) / step).floor().max(1.0) as i32;
    // Bottom lobe first, so each upper lobe overlaps the one below it.
    for i in (0..=n).rev() {
        let s = i as f32 * step;
        let (p, d) = along(pts, s);
        let t = s / total;
        let r = r0 + (r1 - r0) * t;
        let side = if i % 2 == 0 { flip } else { -flip };
        let c = v(p.x - d.y * side * r * 0.32, p.y + d.x * side * r * 0.32);
        // Each lobe is a slanted lozenge, tilted alternately: the chevron
        // pattern that reads as a plait rather than a stack of beads.
        let (sn, cs) = (side * 0.6f32).sin_cos();
        let u = v(d.x * cs - d.y * sn, d.x * sn + d.y * cs);
        let (a, bb) = (r * 1.1, r * 0.62);
        cv.begin();
        let span = a.ceil() as i32 + 1;
        for y in (c.y as i32 - span)..=(c.y as i32 + span) {
            for x in (c.x as i32 - span)..=(c.x as i32 + span) {
                let (ox, oy) = (x as f32 + 0.5 - c.x, y as f32 + 0.5 - c.y);
                let pu = ox * u.x + oy * u.y;
                let pv = -ox * u.y + oy * u.x;
                let q = (pu / a).powi(2) + (pv / bb).powi(2);
                if q > 1.0 + EPS {
                    continue;
                }
                // Lit along the upper flank, shadowed where it tucks under.
                let tone = if q > 0.55 && pv * side > 0.0 { 1 } else if pv * side > 0.15 || q > 0.7 { 2 } else { 3 };
                cv.fill(x, y, h[tone]);
            }
        }
    }
    let (end, d) = along(pts, total);
    let tie_at = v(end.x - d.x * 1.5, end.y - d.y * 1.5);
    cv.begin();
    tail_lock(cv, h, tie_at, v(end.x + d.x * 4.0, end.y + d.y * 4.0), (r1 * 0.9).max(1.6));
    cv.begin();
    cv.ellipse(tie_at, (r1 * 0.75).max(1.3), 1.0, tie, 0.1);
}

/// Tate-roll drill: a tight ringlet hanging from `top`, coils stacked down
/// the length, fullest in the middle and narrowing to a curled tip, each
/// coil its own part with a diagonal shadow showing the spiral.
fn drill(cv: &mut Canvas, h: &R5, top: V2, len: f32, r: f32, lean: f32) {
    let coil = (r * 0.95).max(2.4);
    let n = (len / coil).floor().max(2.0) as i32;
    for i in 0..n {
        let t = i as f32 / (n - 1) as f32;
        let rr = r * (0.75 + 0.35 * (std::f32::consts::PI * t.min(0.7) / 0.7).sin()) * (1.0 - 0.45 * t * t);
        let c = v(top.x + lean * t * t, top.y + coil * (i as f32 + 0.5));
        cv.begin();
        cv.ellipse(c, rr.max(1.2), coil * 0.62, h, 0.04);
        // The spiral: the coil's lower edge sweeps across in shadow.
        let y = (c.y + coil * 0.25).floor() as i32;
        let x0 = (c.x - rr * 0.7).round() as i32;
        let x1 = (c.x + rr * 0.7).round() as i32;
        for x in x0..=x1 {
            // Mirrored drills spiral the mirrored way.
            let run = if lean < 0.0 { x1 - x } else { x - x0 };
            let yy = y - (run as f32 / (x1 - x0).max(1) as f32 * 1.4).round() as i32;
            if cv.owns(x, yy) {
                cv.put(x, yy, h[2]);
            }
        }
    }
}

/// Ahoge: one antenna strand springing from the crown and curling over.
fn ahoge(cv: &mut Canvas, h: &R5, root: V2, dir: f32) {
    cv.begin();
    let pts = [root, v(root.x + dir * 1.0, root.y - 4.0), v(root.x + dir * 3.5, root.y - 6.5), v(root.x + dir * 5.5, root.y - 5.0)];
    curve(cv, &pts, 1.3, 0.5, h);
}

/// One segment of a lock; `taper` false keeps it full (for joining a curve).
fn lock_segment(cv: &mut Canvas, h: &R5, a: V2, b: V2, r: f32, taper: bool) {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len2 = (dx * dx + dy * dy).max(1e-4);
    let (x0, x1) = ((a.x.min(b.x) - r - 1.0) as i32, (a.x.max(b.x) + r + 1.0) as i32);
    let (y0, y1) = ((a.y.min(b.y) - r - 1.0) as i32, (a.y.max(b.y) + r + 1.0) as i32);
    for y in y0..=y1 {
        for x in x0..=x1 {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let t = (((px - a.x) * dx + (py - a.y) * dy) / len2).clamp(0.0, 1.0);
            // Full near the tie, swelling, then tapering to a point.
            let rr = if taper { r * (0.75 + (t * 3.0).min(1.0) * 0.35) * (1.0 - t.powf(1.6) * 0.92) } else { r * (0.85 + t * 0.15) };
            let (ox, oy) = (px - (a.x + dx * t), py - (a.y + dy * t));
            let d2 = ox * ox + oy * oy;
            if d2 > rr * rr {
                continue;
            }
            let u = ox / rr.max(0.5);
            let tone = if u.abs() > 0.58 || t > 0.8 { 2 } else { 3 };
            let tone = if u.abs() < 0.15 && t > 0.2 && t < 0.85 { 2 } else { tone };
            cv.fill(x, y, h[tone]);
        }
    }
}

fn hair_front(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
    let h = &c.hair;
    let hc = b.head_c;
    let cy = hc.y;
    let k = cut(look.hair_style);
    let ex = k.extra;
    let sway = b.sway;
    let (rx, ry) = mass_r(look);
    let shine = h[4];
    let tie = r5(&ramp4(&pal::CLOTH[1]));
    // Clipped hair (undercut, a bald man's fringe): skin showing through.
    let clipped: R5 = [mix(c.skin[0], h[0], 0.7), mix(c.skin[1], h[0], 0.7), mix(c.skin[2], h[1], 0.72), mix(c.skin[3], h[1], 0.65), mix(c.skin[4], h[2], 0.6)];
    cv.register(&clipped);
    cv.begin();
    if ex == Extra::Spikes {
        spikes(cv, h, cy, dir);
    }
    match dir {
        DIR_UP => {
            // Back of the head: the whole cap, nape, and the long sheet over
            // the back.
            let mc = v(32.0, cy - 1.0);
            // From behind, hair comes down to about the jaw; gathered styles
            // show more of the nape.
            let nape = cy + if k.gathered { 10.0 } else { k.nape.max(12.0) };
            let neck_w = if look.female { 6.0 } else { 7.0 };
            for y in (mc.y - ry).floor() as i32..=nape.ceil() as i32 {
                let fy = y as f32 + 0.5;
                // From behind: a round crown, full width to the ears, then
                // the back of the skull curves in quickly under the ears and
                // eases into the neck, like the jaw line seen from the front
                // (a late pinch reads as a bell or a bowling ball on a stick).
                // A hair under the full radius keeps the widest row from
                // poking out a one-pixel nub on each side.
                // A bald man's fringe lies flat on the skull.
                let er = if ex == Extra::Bald { 12.8 } else { rx - 0.4 };
                let ear = cy + 3.0;
                let half = if fy < mc.y {
                    let ny = (fy - mc.y) / ry;
                    er * (1.0 - ny * ny).max(0.0).sqrt()
                } else if fy < ear {
                    er
                } else {
                    let t = ((fy - ear) / (nape - ear).max(1.0)).clamp(0.0, 1.0);
                    er + (neck_w + 0.5 - er) * (1.0 - (1.0 - t).powf(1.8))
                };
                for x in (32.0 - rx - 1.0) as i32..=(32.0 + rx + 1.0) as i32 {
                    let nx = (x as f32 + 0.5 - mc.x) / rx;
                    let ny = (fy - mc.y) / ry;
                    let d = (x as f32 + 0.5 - 32.0).abs();
                    // The hairline curves up behind the ears into a rounded U
                    // at the nape; gathered hair shows more of it.
                    let rise = (d / rx).min(1.0).powf(1.6) * if k.gathered { 7.0 } else { 4.0 };
                    if d > half + EPS || fy > nape - rise {
                        continue;
                    }
                    let col = match ex {
                        // Only the horseshoe below the crown.
                        // A horseshoe at ear height; the crown stays bare.
                        Extra::Bald if fy < cy - 1.0 || fy > cy + 7.0 => continue,
                        Extra::Bald => h[if nx.abs() > 0.8 { 2 } else { 3 }],
                        Extra::Undercut if fy > cy - 2.0 => clipped[if nx.abs() > 0.8 { 1 } else { 2 }],
                        _ => h[if nx.abs() > 0.86 || ny > 0.7 { 2 } else { 3 }],
                    };
                    cv.fill(x, y, col);
                }
            }
            let top = cy + 5.0;
            let len = (k.back - 5.0).max(0.0);
            if k.back > 0.0 {
                hair_sheet(cv, h, top, top + len, true, k.hem, |_| 32.0, |y| sheet_w(rx, ((y - top) / len).clamp(0.0, 1.0)));
            }
            // Flow lines that follow the skull, then fall with the hair.
            let bottom = match ex {
                Extra::Undercut => cy - 3.0,
                _ if k.back > 0.0 => top + len - 7.0,
                _ => nape - 2.0,
            };
            if ex != Extra::Bald {
                flow_lines(cv, h, 32.0, cy - 12.0, cy + 4.0, bottom, &[4.0, 9.5]);
                highlight_ring(cv, h, shine, v(32.0, cy - 1.0), rx, ry, &[]);
            }
            match ex {
                Extra::Ponytail => {
                    // Tied high at the back of the crown, as in profile.
                    let base = v(32.0, cy - 7.0);
                    cv.begin();
                    tail_lock(cv, h, base, base.add(v(sway, 30.0)), 4.2);
                    cv.begin();
                    cv.ellipse(base.add(v(0.0, -0.5)), 2.2, 1.6, &tie, 0.1);
                }
                Extra::Bun => {
                    cv.begin();
                    cv.ellipse(v(32.0, cy - ry - 0.5), 5.5, 5.0, h, 0.05);
                }
                Extra::TwinTails => {
                    for s in [-1.0f32, 1.0] {
                        let base = v(32.0 + s * (rx + 0.5), cy - 4.0);
                        cv.begin();
                        tail_lock(cv, h, base, base.add(v(s * 3.0 + sway, 30.0)), 4.0);
                        cv.ellipse(base.add(v(0.0, -1.0)), 2.4, 2.0, &tie, 0.1);
                    }
                }
                Extra::Drills => {
                    let first = cv.cur + 1;
                    drill(cv, h, v(18.5, cy - 1.0), 24.0, 3.6, -1.5);
                    cv.mirror_from(first);
                }
                // Both braids go forward over the shoulders: only their
                // roots show from behind.
                Extra::TwinBraids => {
                    let first = cv.cur + 1;
                    braid(cv, h, &[v(21.5, cy + 6.0), v(19.5, cy + 16.0)], 2.4, 2.2, &tie, -1.0);
                    cv.mirror_from(first);
                }
                // Gathered to the right shoulder (screen right from behind).
                Extra::SideBraid => {
                    braid(cv, h, &[v(40.5, cy + 6.0), v(43.5, cy + 17.0)], 2.9, 2.6, &tie, 1.0);
                }
                Extra::HalfUp => {
                    // A ribbon bow: two loops, a knot and two tails falling
                    // over the hair.
                    cv.begin();
                    for s in [-1.0f32, 1.0] {
                        cv.ellipse(v(32.0 + s * 2.6, cy - 5.5), 2.0, 1.3, &tie, 0.1);
                        cv.capsule(v(32.0 + s * 0.6, cy - 4.5), v(32.0 + s * 1.8, cy + 1.0), 0.8, 0.6, &tie, 0.0);
                    }
                    cv.begin();
                    cv.ellipse(v(32.0, cy - 5.0), 1.0, 1.1, &tie, -0.2);
                }
                Extra::Messy => {
                    cv.begin();
                    // Tufts overlapping across the nape, no skin between them.
                    for (dx, len) in [(-3.5f32, 3.0f32), (0.0, 4.0), (3.5, 3.0)] {
                        tail_lock(cv, h, v(32.0 + dx, nape - 4.0), v(32.0 + dx * 1.15, nape + len - 2.0), 2.7);
                    }
                    ahoge(cv, h, v(31.5, cy - 15.0), -1.0);
                }
                Extra::LowTie => {
                    let at = v(32.0, cy + 8.0);
                    cv.begin();
                    tail_lock(cv, h, at, v(32.0 + sway, cy + 36.0), 3.0);
                    cv.begin();
                    cv.ellipse(at, 2.2, 1.2, &tie, 0.1);
                }
                Extra::TopKnot => {
                    let at = v(32.0, cy - 11.0);
                    cv.begin();
                    tail_lock(cv, h, at, v(32.0 + sway, cy + 16.0), 3.2);
                    cv.begin();
                    cv.ellipse(at, 2.6, 2.0, h, 0.1);
                    cv.begin();
                    cv.ellipse(at.add(v(0.0, 1.5)), 2.0, 0.9, &tie, 0.1);
                }
                _ => {}
            }
        }
        DIR_SIDE => {
            let mc = v(31.0, cy - 1.0);
            let front_tips = k.side_tips;
            let nape = cy + k.nape;
            // Long hair falls over the ear and the nape down to the jaw; only
            // shorter cuts show the ear.
            let long = k.back > 0.0;
            for y in (mc.y - ry).floor() as i32..=(if long { cy + 13.0 } else { nape }).ceil() as i32 {
                for x in (mc.x - rx - 1.0) as i32..=(mc.x + rx + 1.0) as i32 {
                    let nx = (x as f32 + 0.5 - mc.x) / rx;
                    let ny = (y as f32 + 0.5 - mc.y) / ry;
                    // Round skull in profile too: the back of the head curves
                    // down into the nape instead of ending in a square corner.
                    let nyk = if ny > 0.0 { ny * 0.8 } else { ny };
                    if nx * nx + nyk * nyk > 1.0 + EPS {
                        continue;
                    }
                    let fy = y as f32 + 0.5;
                    if x >= 35 {
                        if fy > fringe_bottom(front_tips, cy, x) {
                            continue;
                        }
                    } else if x >= 33 {
                        // Between the fringe and the side lock: the temple.
                        if fy > cy - 1.0 || (ex == Extra::Slick && fy > cy - 7.0) {
                            continue;
                        }
                    } else if fy > if long { cy + 13.0 } else { nape - ((x as f32 - 22.0) / 11.0).clamp(0.0, 1.0) * 3.0 } {
                        // Behind the side lock the head is all hair, down to
                        // the nape; only the face shows in front of it.
                        continue;
                    }
                    let rim = nx * nx + ny * ny > 0.8 && ny > -0.2;
                    let col = match ex {
                        Extra::Bald if fy < cy - 3.0 || x > 29 => continue,
                        Extra::Bald => h[if rim { 2 } else { 3 }],
                        Extra::Undercut if fy > cy - 5.0 && x < 33 => clipped[if rim { 1 } else { 2 }],
                        _ => h[if rim { 2 } else { 3 }],
                    };
                    cv.fill(x, y, col);
                }
            }
            // The ear sits on the hair just behind the jaw hinge, from the
            // eye line to the nose, tucked behind the side lock. Beastfolk
            // hear with their animal ears and elves draw their own.
            if !long && !matches!(look.race, RACE_ELF | RACE_BEAST) {
                let s = c.skin;
                let pal = |ch: char| match ch {
                    'L' => Some(s[3]),
                    'M' => Some(s[2]),
                    'D' => Some(s[1]),
                    _ => None,
                };
                cv.begin();
                stamp_fill(cv, 27, cy.round() as i32 + 3, &EAR_SIDE, false, &pal);
                cv.begin();
            }
            // Side lock in front of the ear.
            let (ll, lw) = k.lock;
            if ll > 5.0 {
                // In profile the lock in front of the ear stops at the jaw.
                lock_strip(cv, h, 31.5, cy - 1.0, cy + ll.min(12.0), (lw * 0.7).max(2.6), sway, 1.0, k.blunt);
            }
            for &(tx, _, _) in front_tips.iter().skip(1) {
                let nb = fringe_bottom(front_tips, cy, (32.0 + tx - 2.0) as i32);
                cv.line(v(32.0 + tx * 0.5, cy - 11.0), v(32.0 + tx - 2.0, nb), |t| if t < 0.1 { None } else if t > 0.8 { Some(h[1]) } else { Some(h[2]) });
            }
            if ex != Extra::Bald {
                let low = if ex == Extra::Undercut { cy - 6.0 } else { nape - 2.0 };
                for dx in [-8.0f32, -3.0] {
                    cv.line(v(31.0 + dx * 0.4, cy - 12.0), v(31.0 + dx * 1.3, low), |t| if t < 0.15 { None } else { Some(h[2]) });
                }
                highlight_ring(cv, h, shine, mc, rx, ry, &[]);
            }
            match ex {
                Extra::Bun => {
                    cv.begin();
                    cv.ellipse(v(28.0, cy - ry - 0.5), 5.5, 5.0, h, 0.05);
                }
                Extra::Drills => drill(cv, h, v(25.0, cy - 1.0), 24.0, 3.6, -1.2),
                Extra::TwinBraids => braid(cv, h, &[v(27.5, cy + 8.0), v(29.5, cy + 16.0), v(31.0, cy + 24.0), v(31.5, cy + 30.0)], 2.4, 1.8, &tie, 1.0),
                Extra::SideBraid => braid(cv, h, &[v(27.5, cy + 8.0), v(29.5, cy + 16.0), v(31.5, cy + 24.0), v(32.5, cy + 32.0)], 2.9, 2.0, &tie, 1.0),
                Extra::HalfUp => {
                    cv.begin();
                    cv.ellipse(v(18.5, cy - 5.0), 1.6, 2.4, &tie, 0.1);
                }
                Extra::Messy => {
                    cv.begin();
                    for (x, len) in [(20.5f32, 4.0f32), (24.5, 3.5)] {
                        tail_lock(cv, h, v(x + 1.0, nape - 3.0), v(x - 0.5, nape + len - 2.0), 2.0);
                    }
                    ahoge(cv, h, v(31.5, cy - 15.5), 1.0);
                }
                Extra::LowTie => {
                    cv.begin();
                    cv.ellipse(v(21.0, cy + 6.5), 1.3, 2.0, &tie, 0.1);
                }
                Extra::TopKnot => {
                    cv.begin();
                    cv.ellipse(v(21.0, cy - 10.5), 2.4, 2.4, h, 0.1);
                    cv.begin();
                    cv.ellipse(v(22.0, cy - 10.0), 0.9, 2.0, &tie, 0.1);
                }
                // Combed lines sweeping back over the crown.
                Extra::Slick => {
                    for (y0, y1) in [(-7.5f32, -12.5f32), (-6.0, -9.0)] {
                        cv.line(v(42.0, cy + y0), v(22.0, cy + y1), |t| if t < 0.05 { None } else { Some(h[2]) });
                    }
                }
                Extra::Bald => {
                    cv.put(30, (cy - 11.0) as i32, c.skin[4]);
                    cv.put(31, (cy - 11.0) as i32, c.skin[4]);
                    cv.put(29, (cy - 10.0) as i32, c.skin[4]);
                }
                _ => {}
            }
        }
        _ => {
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
                    let temple = dxa > face_w - 1.0 && fy > cy - 4.0;
                    let col = match ex {
                        // Only the fringe of hair around the sides.
                        // A thin fringe hugging the sides of the skull.
                        Extra::Bald if dxa > face_w - 0.6 && dxa < face_w + 1.0 && fy > cy - 4.0 && fy < cy + 3.5 => h[if dxa > face_w { 2 } else { 3 }],
                        Extra::Bald => continue,
                        // Clipped sides from the temples down, close to the skull.
                        Extra::Undercut if dxa > 9.0 && fy > cy - 6.0 => {
                            if fy > cy + 1.0 || dxa > face_w + 0.6 {
                                continue;
                            }
                            clipped[if dxa > face_w - 0.6 { 1 } else { 2 }]
                        }
                        _ => {
                            // Slicked-back sides lie flat above the ears.
                            let side_end = if ex == Extra::Slick { cy - 5.0 } else { cy + 3.0 };
                            let limit = if dxa < face_w + 0.5 { fringe_bottom(tips, cy, x) } else { side_end };
                            if fy > limit {
                                continue;
                            }
                            h[if temple { 2 } else { 3 }]
                        }
                    };
                    cv.fill(x, y, col);
                }
            }
            // Strand lines from the crown into each notch between clumps.
            let mut xs: Vec<f32> = tips.iter().map(|t| t.0).collect();
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let mut notches = Vec::new();
            for w in xs.windows(2) {
                let nx = (w[0] + w[1]) * 0.5;
                // Nudged toward the centre so notches on whole pixels mirror.
                let px = (32.0 + nx - nx.signum() * 1e-3).floor() as i32;
                let nb = fringe_bottom(tips, cy, px);
                if nx.abs() >= 0.6 {
                    notches.push(px);
                }
                if nx.abs() < 0.6 || ex == Extra::Slick {
                    // A single centre column can't be mirrored; the clumps
                    // either side already meet there.
                    continue;
                }
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
            if ex != Extra::Bald {
                highlight_ring(cv, h, shine, mc, rx, ry, &notches);
            }
            // Cuts that clear the sides of the head show the ears.
            if matches!(ex, Extra::Bald | Extra::Slick | Extra::Undercut) && !matches!(look.race, RACE_ELF | RACE_BEAST) {
                let sk = c.skin;
                let pal = |ch: char| match ch {
                    'L' => Some(sk[3]),
                    'M' => Some(sk[2]),
                    'D' => Some(sk[1]),
                    _ => None,
                };
                cv.begin();
                stamp_fill(cv, 17, cy.round() as i32 + 3, &EAR_FRONT, false, &pal);
                stamp_fill(cv, 17, cy.round() as i32 + 3, &EAR_FRONT, true, &pal);
            }
            // Side locks framing the face.
            let (ll, lw) = k.lock;
            if ll > 0.0 {
                for s in [-1.0f32, 1.0] {
                    let cxl = 32.0 + s * (rx - lw * 0.5);
                    lock_strip(cv, h, cxl, cy - 4.0, cy + ll, lw, sway, s, k.blunt);
                }
            }
            match ex {
                Extra::Bun => {
                    cv.begin();
                    cv.ellipse(v(32.0, cy - ry - 1.0), 5.5, 5.0, h, 0.05);
                }
                Extra::TwinTails => {
                    for s in [-1.0f32, 1.0] {
                        let base = v(32.0 + s * (rx + 0.5), cy - 4.0);
                        cv.begin();
                        cv.ellipse(base.add(v(0.0, -1.0)), 2.4, 2.0, &tie, 0.1);
                    }
                }
                Extra::Drills => {
                    let first = cv.cur + 1;
                    drill(cv, h, v(18.5, cy - 1.0), 24.0, 3.6, -1.5);
                    cv.mirror_from(first);
                }
                // Over the shoulders and down the front.
                Extra::TwinBraids => {
                    let first = cv.cur + 1;
                    let p = |x: f32, y: f32| v(32.0 - x, cy + y);
                    braid(cv, h, &[p(11.5, 5.0), p(12.0, 13.0), p(11.0, 22.0), p(10.5, 30.0)], 2.4, 1.8, &tie, 1.0);
                    cv.mirror_from(first);
                }
                // Over her right shoulder (screen left).
                Extra::SideBraid => {
                    braid(cv, h, &[v(20.5, cy + 5.0), v(20.5, cy + 13.0), v(22.5, cy + 22.0), v(24.0, cy + 32.0)], 2.9, 2.0, &tie, 1.0);
                }
                Extra::Messy => ahoge(cv, h, v(32.5, cy - 15.0), 1.0),
                // The knot peeks over the crown.
                Extra::TopKnot => {
                    cv.begin();
                    cv.ellipse(v(32.0, cy - ry - 0.5), 2.6, 2.2, h, 0.1);
                }
                // Combed lines back from the hairline.
                Extra::Slick => {
                    for o in [3.0f32, 7.0, 10.5] {
                        for s in [-1.0f32, 1.0] {
                            cv.line(v(32.0 + s * o, cy - 7.0), v(32.0 + s * o * 0.55, cy - 14.0), |t| if t < 0.1 { None } else { Some(h[2]) });
                        }
                    }
                }
                Extra::Bald => {
                    for (x, y) in [(27, -11), (28, -11), (26, -10)] {
                        cv.put(x, (cy + y as f32) as i32, c.skin[4]);
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
fn lock_strip(cv: &mut Canvas, h: &R5, cx: f32, top: f32, end: f32, w: f32, sway: f32, s: f32, blunt: bool) {
    let len = (end - top).max(1.0);
    for y in top.floor() as i32..=end.ceil() as i32 {
        let fy = y as f32 + 0.5;
        if fy < top || fy > end {
            continue;
        }
        let t = (fy - top) / len;
        // Pointed tip, or cut straight across.
        let ww = if t > 0.6 && !blunt { w * (1.0 - (t - 0.6) / 0.4 * 0.9) } else { w };
        // The point swings outward with the lag.
        let c = cx + t * t * sway * 0.6 + if t > 0.6 && !blunt { s * (t - 0.6) * 2.0 } else { 0.0 };
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
    let x0 = (mc.x - rx * 0.62).round() as i32;
    let x1 = (2.0 * mc.x).round() as i32 - x0;
    for x in x0..x1 {
        if skip.contains(&x) {
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

/// Spiky hair: tapered triangular clumps fanned along the crown, bases sunk
/// into the hair mass (drawn first, so the cap covers them). Front and back
/// views fan symmetrically; the profile sweeps them back.
fn spikes(cv: &mut Canvas, h: &R5, cy: f32, dir: u8) {
    let spike = |cv: &mut Canvas, base: V2, ang: f32, len: f32, hw: f32| {
        // Axis points along `ang` (0 = straight up, + = toward +x).
        let (ax, ay) = (ang.sin(), -ang.cos());
        let r = len + hw + 1.0;
        for y in (base.y - r).floor() as i32..=(base.y + r).ceil() as i32 {
            for x in (base.x - r).floor() as i32..=(base.x + r).ceil() as i32 {
                let (dx, dy) = (x as f32 + 0.5 - base.x, y as f32 + 0.5 - base.y);
                let u = dx * ax + dy * ay;
                let w = dx * -ay + dy * ax;
                if u < -2.0 || u > len {
                    continue;
                }
                let half = hw * (1.0 - u.max(0.0) / len);
                if w.abs() > half + EPS {
                    continue;
                }
                // Lit flank and shaded flank, mirrored with the spike angle.
                let lit = if ang.abs() < 0.05 { w < 0.0 } else { (w < 0.0) == (ang < 0.0) };
                cv.fill(x, y, h[if u > len * 0.75 { 3 } else if lit { 3 } else { 2 }]);
            }
        }
    };
    if dir == DIR_SIDE {
        // Profile: swept back over the crown.
        for (k, (x, ang, len)) in [(36.0f32, -0.35f32, 6.0f32), (31.0, -0.6, 7.5), (26.0, -0.85, 7.5), (21.5, -1.15, 6.0)].into_iter().enumerate() {
            let y = cy - 12.5 + (k as f32 - 0.5).abs() * 1.2;
            spike(cv, v(x, y), ang, len, 3.0);
        }
    } else {
        for (dx, ang, len) in [(0.0f32, 0.0f32, 7.5f32), (5.5, 0.35, 7.0), (10.5, 0.75, 6.0)] {
            let sides: &[f32] = if dx == 0.0 { &[1.0] } else { &[-1.0, 1.0] };
            for &sgn in sides {
                let y = cy - 13.5 + (dx / 10.5) * (dx / 10.5) * 5.0;
                spike(cv, v(32.0 + sgn * dx, y), sgn * ang, len, 3.0);
            }
        }
    }
}

// Authored ear and horn stamps (left-side piece; the right is mirrored).
// Legend: H highlight, L light, M mid, D dark, I inner, J inner shadow,
// T tip (darker fur), R ridge. Column 0 is the outer/tip side.

/// Elf ear, front view: long, angled up and out, inner fold shaded.
const ELF_EAR: [&str; 9] = [
    "HL..........",
    ".LLM........",
    "..LLMM......",
    "...LLDMM....",
    "....LLDDMM..",
    ".....LLDDMMM",
    "......LLDDMM",
    "........MMMM",
    "..........MM",
];
/// Elf ear, side view (tip points back): the whole fold is visible.
const ELF_EAR_SIDE: [&str; 9] = [
    "HL........",
    ".LLM......",
    "..LLDM....",
    "...LLDDM..",
    "....LLDDM.",
    ".....LLDDM",
    "......LLDM",
    ".......LMM",
    "........MM",
];
/// Cat ear: neat triangle with a soft inner.
const CAT_EAR: [&str; 12] = [
    ".H.......",
    ".LL......",
    ".LLL.....",
    "LLILM....",
    "LLIIJM...",
    "LIIIIJM..",
    "LIIIIIJM.",
    "LIIIIIJMM",
    "LLIIIIJMM",
    "LLIIIJMMM",
    "MLLLMMMMM",
    "MMMMMMMMM",
];
/// Fox ear: tall, dark tip, cream fur inner.
const FOX_EAR: [&str; 12] = [
    "..TT.......",
    "..TTT......",
    ".TTLT......",
    ".LLIIL.....",
    ".LIIIJL....",
    "LLIIIIJL...",
    "LIIIIIJML..",
    "LIIIIIIJML.",
    "LIIIIIIJMML",
    "LLIIIIIJMMM",
    "MLLIIIJMMMM",
    "MMMMMMMMMMM",
];
/// Wolf ear: broad, upright, tufted inner.
const WOLF_EAR: [&str; 11] = [
    "...H.......",
    "...LL......",
    "..LLLM.....",
    "..LIIJM....",
    ".LLIIIJM...",
    ".LIIIIIJM..",
    "LLIIIIIIJM.",
    "LIIIIIIIJMM",
    "LLIIIIIIJMM",
    "MLLIIIIJMMM",
    "MMMMMMMMMMM",
];
/// Rabbit ear: long and upright with a pink inner stripe.
const RABBIT_EAR: [&str; 17] = [
    ".HL..",
    "LLLM.",
    "LLIM.",
    "LIIJM",
    "LIIJM",
    "LIIJM",
    "LIIJM",
    "LIIJM",
    "LIIJM",
    "LIIJM",
    "LIIJM",
    "LLIJM",
    "LLIJM",
    "LLLMM",
    "LLLMM",
    "LLMMM",
    "MMMMM",
];
/// Demon horn: sweeps out from the head, then hooks upward to a sharp
/// point (a crescent, concave side toward the head), with growth ridges.
const HORN: [&str; 12] = [
    ".H......",
    ".LH.....",
    "LLM.....",
    "LLMD....",
    "RRRD....",
    "LLMMD...",
    ".LLMD...",
    ".RRRRD..",
    "..LLMMD.",
    "...LLMMD",
    "....RRRD",
    ".....LMD",
];

/// Beast ears grow out of the hair: drawn before it, so the hair mass covers
/// their base and only the ear itself shows above the crown.
fn beast_ears(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
    let hc = b.head_c;
    let cy = hc.y.round() as i32;
    let back = dir == DIR_UP;
    match look.race {
        RACE_BEAST => {
            let inner = if look.beast == 1 || look.beast == 2 {
                [hex(0xb09a88), hex(0xd8c4b0), hex(0xf0e2d0), hex(0xfaf2e6), hex(0xfffaf2)]
            } else {
                [hex(0xa8606a), hex(0xd08890), hex(0xf0a8b0), hex(0xf8c8cc), hex(0xffe0e4)]
            };
            let h = c.hair;
            let tip = mix(h[0], pal::INK, 0.35);
            let pal = |ch: char| -> Option<Rgb> {
                match ch {
                    'H' => Some(h[4]),
                    'L' => Some(h[3]),
                    'M' => Some(h[2]),
                    'D' => Some(h[1]),
                    // From behind there is no inner fur, only the ear's back.
                    'I' => Some(if back { h[3] } else { inner[2] }),
                    'J' => Some(if back { h[2] } else { inner[1] }),
                    'T' => Some(if look.beast == 1 { tip } else { h[3] }),
                    _ => None,
                }
            };
            let art: &[&str] = match look.beast {
                1 => &FOX_EAR,
                2 => &WOLF_EAR,
                3 => &RABBIT_EAR,
                _ => &CAT_EAR,
            };
            let (w, hgt) = (art[0].len() as i32, art.len() as i32);
            // Upper-side of the crown, bases sunk three rows into the hair.
            let base_y = cy - 9;
            cv.begin();
            match dir {
                DIR_SIDE => {
                    // Profile: on top of the crown, leaning slightly forward;
                    // the far ear peeks just behind the near one.
                    let near = 33 - w / 2;
                    let far = near - 5;
                    stamp_at(cv, far, base_y - hgt + 2, art, true, &|ch| pal(ch).map(|p| mix(p, h[1], 0.3)));
                    cv.begin();
                    stamp_at(cv, near, base_y - hgt + 1, art, true, &pal);
                }
                _ => {
                    let x0 = 32 - 9 - w / 2;
                    stamp_fill(cv, x0, base_y - hgt + 1, art, false, &pal);
                    stamp_fill(cv, x0, base_y - hgt + 1, art, true, &pal);
                }
            }
        }
        _ => {}
    }
}

fn horns(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, _c: &Colors) {
    let cy = b.head_c.y.round() as i32;
    match look.race {
        RACE_DEMON => {
            let horn = [hex(0x1e1622), hex(0x3a2e40), hex(0x5a4a62), hex(0x857590), hex(0xb0a0ba)];
            let pal = |ch: char| -> Option<Rgb> {
                match ch {
                    'H' => Some(horn[4]),
                    'L' => Some(horn[3]),
                    'M' => Some(horn[2]),
                    'D' => Some(horn[1]),
                    'R' => Some(horn[1]),
                    _ => None,
                }
            };
            if look.beast == 1 {
                // Ram horns: curling back and around beside the head.
                let cyf = cy as f32;
                let left = [v(25.0, cyf - 11.0), v(21.0, cyf - 15.0), v(16.0, cyf - 14.0), v(13.5, cyf - 9.5), v(15.0, cyf - 5.5), v(18.0, cyf - 4.5)];
                let hr = [horn[0], horn[1], horn[2], horn[3], horn[4]];
                cv.begin();
                match dir {
                    DIR_SIDE => {
                        let pts = [v(31.0, cyf - 12.0), v(27.0, cyf - 16.0), v(22.5, cyf - 14.5), v(20.5, cyf - 10.0), v(22.0, cyf - 6.0), v(25.0, cyf - 5.0)];
                        curve(cv, &pts, 2.8, 1.0, &hr);
                    }
                    _ => {
                        let right: Vec<V2> = left.iter().map(|p| v(64.0 - p.x, p.y)).collect();
                        curve(cv, &left, 2.8, 1.0, &hr);
                        curve(cv, &right, 2.8, 1.0, &hr);
                    }
                }
                return;
            }
            let hgt = HORN.len() as i32;
            let base_y = cy - 11;
            cv.begin();
            match dir {
                // Profile: the horn sweeps forward over the brow.
                DIR_SIDE => stamp_at(cv, 31, base_y - hgt + 1, &HORN, true, &pal),
                _ => {
                    stamp_fill(cv, 32 - 6 - 8, base_y - hgt + 1, &HORN, false, &pal);
                    stamp_fill(cv, 32 - 6 - 8, base_y - hgt + 1, &HORN, true, &pal);
                }
            }
        }
        _ => {}
    }
}

/// Elf ears are drawn before the hair so locks fall over their base.
fn elf_ears(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors, after_hair: bool) {
    if look.race != RACE_ELF {
        return;
    }
    let s = c.skin;
    let back = dir == DIR_UP;
    let pal = |ch: char| -> Option<Rgb> {
        match ch {
            'H' => Some(s[4]),
            'L' => Some(s[3]),
            'M' => Some(s[2]),
            'D' => Some(if back { s[2] } else { s[1] }),
            _ => None,
        }
    };
    let cy = b.head_c.y.round() as i32;
    cv.begin();
    match dir {
        // In profile the ear pokes out through the hair behind the side
        // lock; drawn after the hair, its base stays hidden under the lock.
        DIR_SIDE => {
            if after_hair {
                let trimmed: Vec<&str> = ELF_EAR_SIDE.iter().map(|r| &r[..8]).collect();
                stamp_fill(cv, 20, cy - 2, &trimmed, false, &pal);
            }
        }
        _ if after_hair => {}
        _ => {
            stamp_fill(cv, 10, cy - 1, &ELF_EAR, false, &pal);
            stamp_fill(cv, 10, cy - 1, &ELF_EAR, true, &pal);
        }
    }
}

/// Smooth tapered curve through `pts` (radius r0 at the root to r1 at the
/// end), shaded as a cylinder; segments share one part so joints are seamless.
fn curve(cv: &mut Canvas, pts: &[V2], r0: f32, r1: f32, ramp: &R5) {
    // Catmull-Rom through the control points, finely sampled, so the line
    // bends smoothly instead of in visible angles.
    let n = pts.len();
    let at = |i: isize| pts[i.clamp(0, n as isize - 1) as usize];
    let mut samples = Vec::new();
    for i in 0..n - 1 {
        let (p0, p1, p2, p3) = (at(i as isize - 1), at(i as isize), at(i as isize + 1), at(i as isize + 2));
        for k in 0..8 {
            let t = k as f32 / 8.0;
            let (t2, t3) = (t * t, t * t * t);
            let f = |a: f32, b: f32, c: f32, d: f32| 0.5 * (2.0 * b + (c - a) * t + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2 + (3.0 * b - a - 3.0 * c + d) * t3);
            samples.push(v(f(p0.x, p1.x, p2.x, p3.x), f(p0.y, p1.y, p2.y, p3.y)));
        }
    }
    samples.push(pts[n - 1]);
    let m = samples.len() - 1;
    for i in 0..m {
        let (ta, tb) = (i as f32 / m as f32, (i + 1) as f32 / m as f32);
        cv.capsule(samples[i], samples[i + 1], r0 + (r1 - r0) * ta, r0 + (r1 - r0) * tb, ramp, 0.0);
    }
}

/// Demon tail spade, pointing along +x (mirrored for -x).
const SPADE: [&str; 5] = [".LL..", "LMMLL", "MMMMH", "DMMLL", ".DD.."];

/// Tails grow from the small of the back. Side view: curving out behind.
/// Back view: drawn over the body (it faces the camera). Front view: only
/// the tip peeks out to one side from behind.
fn tail(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
    if !matches!(look.race, RACE_BEAST | RACE_DEMON) {
        return;
    }
    let sway = b.lag;
    let side = dir == DIR_SIDE;
    // Direction away from the body: backward in profile, to one side otherwise.
    let (bx, k) = if side { (b.cx - 5.0, -1.0) } else if dir == DIR_UP { (b.cx, 0.95) } else { (b.cx, 0.55) };
    let base = v(bx, b.hip_y);
    let p = |dx: f32, dy: f32| v(base.x + dx * k, base.y + dy);
    let white = [hex(0xc8bcb0), hex(0xe0d8ce), hex(0xf0ece6), hex(0xfaf8f4), hex(0xffffff)];
    cv.begin();
    match look.race {
        RACE_BEAST => match look.beast {
            // Rabbit: a round puff at the base.
            3 => {
                if dir != DIR_DOWN {
                    cv.ellipse(p(2.0, 1.0), 3.4, 3.2, &white, 0.0);
                }
            }
            // Fox: big, bushy, sweeping up, with a white tip.
            1 => {
                // A full brush, thickest towards the end, with a big white tip.
                let pts = [p(0.0, 0.0), p(4.0, 4.5), p(9.0, 5.5), p(13.5, 1.5 + sway), p(15.0, -4.0 + sway)];
                curve(cv, &pts, 2.8, 5.8, &c.hair);
                cv.begin();
                cv.ellipse(pts[4].add(v(0.0, -1.0)), 4.0, 4.2, &white, 0.0);
            }
            // Wolf: bushy, hanging low with a slight curl.
            2 => {
                let pts = [p(0.0, 0.0), p(3.5, 4.0), p(6.0, 9.0), p(7.0, 14.0 + sway * 0.5)];
                curve(cv, &pts, 2.2, 3.8, &c.hair);
                cv.ellipse(pts[3], 2.6, 2.8, &c.hair, -0.05);
            }
            // Cat: slender S-curve rising at the tip.
            _ => {
                let pts = [p(0.0, 0.0), p(4.0, 4.0), p(8.0, 5.0), p(11.0, 2.0), p(12.5, -4.0 + sway), p(12.0, -8.0 + sway)];
                curve(cv, &pts, 1.9, 1.4, &c.hair);
            }
        },
        _ => {
            // Demon: thin dark whip curving out and up, ending in a spade
            // (dark whatever the skin, so it never reads as a pale hand).
            let s = [hex(0x140c18), hex(0x24162c), hex(0x3a2446), hex(0x56366a), hex(0x7a5090)];
            cv.register(&s);
            let pts = [p(0.0, 0.0), p(4.0, 5.0), p(9.0, 7.0), p(13.0, 4.0 + sway * 0.5), p(14.5, -1.0 + sway)];
            curve(cv, &pts, 1.5, 1.0, &s);
            let pal = |ch: char| -> Option<Rgb> {
                match ch {
                    'H' => Some(s[4]),
                    'L' => Some(s[3]),
                    'M' => Some(s[2]),
                    'D' => Some(s[1]),
                    _ => None,
                }
            };
            let tip = pts[4];
            let flip = k < 0.0;
            let x0 = if flip { tip.x as i32 - 4 } else { tip.x as i32 };
            // The spade stamp points along +x; mirror it in place for -x.
            for (r, row) in SPADE.iter().enumerate() {
                for (i, ch) in row.chars().enumerate() {
                    if let Some(col) = pal(ch) {
                        let xi = if flip { x0 + 4 - i as i32 } else { x0 + i as i32 };
                        cv.fill(xi, tip.y as i32 - 3 + r as i32, col);
                    }
                }
            }
        }
    }
}

fn hat(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
    let hc = b.head_c;
    let (rx, ry) = b.head_r;
    let side = dir == DIR_SIDE;
    let top = hc.y - ry;
    let hcol = &c.accent;
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
            if side {
                let knot = v(hc.x - rx, top + 6.0);
                cv.capsule(knot, knot.add(v(-2.5, 7.0)), 1.9, 1.2, hcol, -0.05);
                cv.capsule(knot, knot.add(v(-4.0, 5.0)), 1.9, 1.2, hcol, -0.1);
            } else if dir == DIR_UP {
                // Knot at the centre back, two tails hanging either side.
                let knot = v(hc.x, top + 7.0);
                cv.begin();
                for s in [-1.0f32, 1.0] {
                    cv.capsule(knot, knot.add(v(s * 3.0, 7.0)), 1.8, 1.1, hcol, -0.05);
                }
                cv.ellipse(knot, 2.2, 1.8, hcol, 0.0);
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
            // Hood: close over the crown, opening framed by a shadowed rim
            // around the face, draping down onto the shoulders.
            let (hr, vr) = (rx + 1.6, ry + 1.4);
            let cc = v(hc.x - if side { 1.5 } else { 0.0 }, hc.y - 1.0);
            let cyi = hc.y.round() as i32;
            let bottom = hc.y + ry + 3.0;
            for y in (cc.y - vr - 1.0) as i32..bottom as i32 {
                for x in (cc.x - hr - 3.0) as i32..=(cc.x + hr + 3.0) as i32 {
                    let fy = y as f32 + 0.5;
                    let nx = (x as f32 + 0.5 - cc.x) / hr;
                    let ny = (fy - cc.y) / vr;
                    // Round over the head, then flaring a little to the shoulders.
                    let flare = if ny > 0.3 { 1.0 + (ny - 0.3) * 0.35 } else { 1.0 };
                    let inside = if ny < 0.0 { nx * nx + ny * ny <= 1.0 } else { nx.abs() <= flare };
                    if !inside {
                        continue;
                    }
                    let dy = y - cyi;
                    let open = |w: f32| match dir {
                        DIR_DOWN => dy > -5 && (x as f32 + 0.5 - 32.0).abs() < face_hw(look, dy.min(14)) - w,
                        DIR_SIDE => x as f32 + 0.5 > 33.0 + w && dy > -5,
                        _ => false,
                    };
                    if open(0.5) {
                        continue;
                    }
                    let rim = open(-1.5);
                    let i = if rim { 1 } else if nx.abs() > 0.82 || ny > 0.8 { 2 } else { 3 };
                    cv.fill(x, y, hcol[i]);
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
            let tip = v(hc.x + if side { -7.0 } else { 5.0 }, top - 15.0);
            cv.begin();
            cv.capsule(v(hc.x, top + 2.5), v(hc.x + 1.0, top - 7.0), rx * 0.8, rx * 0.5, hcol, 0.04);
            cv.capsule(v(hc.x + 1.0, top - 7.0), tip, rx * 0.5, 0.7, hcol, 0.06);
            cv.begin();
            cv.profile(top + 2.0, top + 4.5, |_| hc.x, |_| rx * 0.82, &gold5(), 0.0);
            cv.ellipse(v(hc.x, top + 3.0), 1.7, 1.7, &[hex(0x2a1050), hex(0x3a1a6a), hex(0x6a3ac0), hex(0xa070f0), hex(0xe8d8ff)], 0.1);
        }
        7 => {
            let f = pal::FLOWERS[look.accent as usize % pal::FLOWERS.len()];
            let pet = [mix(hex(f[0]), pal::INK, 0.3), hex(f[0]), hex(f[1]), hex(f[2]), mix(hex(f[2]), hex(0xffffff), 0.4)];
            // The same side of her head from behind: mirrored on screen.
            let fc = v(hc.x + if side { -3.5 } else if dir == DIR_UP { -rx * 0.62 } else { rx * 0.62 }, top + 4.0);
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
            let bc = v(hc.x + if side { -5.0 } else if dir == DIR_UP { -rx * 0.58 } else { rx * 0.58 }, top + 2.0);
            cv.ellipse(bc.add(v(-3.0, 0.0)), 2.9, 2.4, hcol, 0.05);
            cv.ellipse(bc.add(v(3.0, 0.0)), 2.9, 2.4, hcol, -0.05);
            cv.begin();
            cv.ellipse(bc, 1.2, 1.2, hcol, -0.2);
            cv.capsule(bc, bc.add(v(-1.8, 5.0)), 1.1, 0.8, hcol, -0.1);
            cv.capsule(bc, bc.add(v(1.8, 5.0)), 1.1, 0.8, hcol, -0.15);
        }
        11 => {
            // Miner's helmet: hard ochre dome, brim, and a lamp at the front.
            let shell = [hex(0x5a3c10), hex(0x9a6e1c), hex(0xd0a030), hex(0xf0cc58), hex(0xfff0a0)];
            let cc = v(hc.x - if side { 1.0 } else { 0.0 }, hc.y - 1.0);
            for y in (top - 3.0) as i32..(top + 8.0) as i32 {
                for x in (hc.x - rx - 2.0) as i32..=(hc.x + rx + 2.0) as i32 {
                    let nx = (x as f32 + 0.5 - cc.x) / (rx + 1.6);
                    let ny = (y as f32 + 0.5 - cc.y) / (ry + 1.6);
                    let d = nx * nx + ny * ny;
                    if d <= 1.0 {
                        cv.fill(x, y, shell[tone(ny, (1.0 - d).sqrt(), 0.06)]);
                    }
                }
            }
            cv.begin();
            cv.profile(top + 7.0, top + 9.0, |_| cc.x, |_| rx + 2.6, &shell, -0.1);
            let lamp = [hex(0x8a6a20), hex(0xd8b040), hex(0xfff0a0), hex(0xfffce0), hex(0xffffff)];
            cv.begin();
            match dir {
                DIR_DOWN => cv.ellipse(v(hc.x, top + 3.5), 2.4, 2.2, &lamp, 0.2),
                DIR_SIDE => cv.ellipse(v(hc.x + rx + 1.0, top + 4.0), 1.6, 2.2, &lamp, 0.2),
                _ => {}
            }
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

// ---------------------------------------------------------------- quality

impl Look {
    /// Any combination the generator supports, for randomize buttons and
    /// bulk QA.
    pub fn random_any(rng: &mut Rng) -> Look {
        let female = rng.chance(0.55);
        // No child characters for now.
        let age = *rng.pick(&[1u8, 1, 1, 1, 2]);
        let mut l = Look::random(rng, female, age);
        l.race = rng.below(5) as u8;
        l.beast = rng.below(4) as u8;
        l.hair_style = rng.below(HAIR_STYLES as usize) as u8;
        l.hair = rng.below(pal::HAIR.len()) as u8;
        l.outfit = rng.below(outfit::COUNT as usize) as u8;
        l.hat = if rng.chance(0.4) { rng.below(HATS as usize) as u8 } else { 0 };
        l.held = if rng.chance(0.4) { rng.below(held::COUNT as usize) as u8 } else { held::NONE };
        l.glasses = rng.chance(0.15);
        l.figure = rng.below(5) as u8;
        l.build = rng.below(4) as u8;
        l.height = rng.below(3) as u8;
        l.beard = rng.chance(0.3);
        l.facial = if rng.chance(0.5) { rng.below(FACIAL.len()) as u8 } else { 0 };
        if l.race == RACE_DEMON {
            l.skin = rng.below(pal::DEMON_SKIN.len()) as u8;
        }
        l.normalized()
    }

    /// Enforce traits that only exist for some bodies: beards only on grown
    /// men, figure (bust) only on grown women; children are never mature
    /// (teens keep `mature` false until 18). Every render goes through this,
    /// so no combination can break the rule.
    pub fn normalized(mut self) -> Look {
        if self.age == 0 {
            self.mature = false;
        }
        if self.female || !self.mature {
            self.beard = false;
            self.facial = 0;
        }
        if !self.female || !self.mature {
            self.figure = default_figure();
        }
        self
    }

    /// Whether the front (or back) view should be mirror-symmetric.
    fn symmetric(&self, _dir: u8) -> bool {
        matches!(self.hair_style, 1 | 3 | 5 | 6 | 8 | 9 | 10 | 12 | 15)
            && matches!(self.hat, 0 | 2 | 3 | 4 | 5 | 8 | 10)
            && self.held == held::NONE
            // Tails peek out to one side; tailed races aren't mirror-exact.
            && matches!(self.race, RACE_HUMAN | RACE_ELF)
            && garb(self).symmetric()

    }
}

/// A visual defect found by [`lint`].
#[derive(Debug, Clone)]
pub struct Issue {
    pub kind: &'static str,
    pub dir: u8,
    pub frame: u16,
    pub detail: String,
}

/// Automated visual QA for one character: renders every frame and checks
/// for the defects a pixel artist would reject.
pub fn lint(look: &Look) -> Vec<Issue> {
    let mut out = Vec::new();
    let mut push = |kind: &'static str, dir: u8, frame: u16, detail: String| out.push(Issue { kind, dir, frame, detail });
    let iris = if look.race == RACE_DEMON && look.eyes < 4 { EYE_COLORS[4 + (look.eyes % 2) as usize] } else { EYE_COLORS[look.eyes as usize % EYE_COLORS.len()] };
    for dir in [DIR_DOWN, DIR_UP, DIR_SIDE] {
        let mut tops = Vec::new();
        for f in 0..=FRAMES_PER_DIR {
            if f == FRAMES_PER_DIR && dir != DIR_DOWN {
                continue;
            }
            let img = frame(look, dir, f);
            let (w, h) = (img.w, img.h);
            let a = |x: i32, y: i32| img.alpha(x, y) != 0;
            // Clipping: nothing may touch the frame edge.
            let mut clip = 0;
            for x in 0..w {
                clip += a(x, 0) as i32 + a(x, h - 1) as i32;
            }
            for y in 0..h {
                clip += a(0, y) as i32 + a(w - 1, y) as i32;
            }
            if clip > 0 {
                push("clipped", dir, f, format!("{clip} edge px"));
            }
            // Connected components (8-way): small separate islands are debris.
            let mut seen = vec![false; (w * h) as usize];
            let mut sizes = Vec::new();
            for y in 0..h {
                for x in 0..w {
                    if !a(x, y) || seen[(y * w + x) as usize] {
                        continue;
                    }
                    let mut stack = vec![(x, y)];
                    seen[(y * w + x) as usize] = true;
                    let mut n = 0;
                    while let Some((cx, cy)) = stack.pop() {
                        n += 1;
                        for dy in -1..=1 {
                            for dx in -1..=1 {
                                let (nx, ny) = (cx + dx, cy + dy);
                                if img.inside(nx, ny) && a(nx, ny) && !seen[(ny * w + nx) as usize] {
                                    seen[(ny * w + nx) as usize] = true;
                                    stack.push((nx, ny));
                                }
                            }
                        }
                    }
                    sizes.push(n);
                }
            }
            sizes.sort_unstable_by(|a, b| b.cmp(a));
            let debris: Vec<_> = sizes.iter().skip(1).filter(|&&s| s < 24).collect();
            if !debris.is_empty() {
                push("debris", dir, f, format!("{} islands {:?}", debris.len(), debris));
            }
            // Pinholes: tiny transparent regions enclosed by the sprite.
            let mut outside = vec![false; (w * h) as usize];
            let mut stack: Vec<(i32, i32)> = Vec::new();
            for x in 0..w {
                stack.push((x, 0));
                stack.push((x, h - 1));
            }
            for y in 0..h {
                stack.push((0, y));
                stack.push((w - 1, y));
            }
            while let Some((x, y)) = stack.pop() {
                if !img.inside(x, y) || a(x, y) || outside[(y * w + x) as usize] {
                    continue;
                }
                outside[(y * w + x) as usize] = true;
                stack.extend([(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]);
            }
            let holes = (0..w * h).filter(|&i| !outside[i as usize] && img.px[i as usize * 4 + 3] == 0).count();
            if holes > 0 && holes <= 6 {
                push("pinhole", dir, f, format!("{holes} px"));
            }
            // Feet on the ground.
            let bottom = (0..h).rev().find(|&y| (0..w).any(|x| a(x, y))).unwrap_or(0);
            if !(PIVOT.1 - 1..=PIVOT.1 + 2).contains(&bottom) {
                push("feet", dir, f, format!("lowest row {bottom}, pivot {}", PIVOT.1));
            }
            let top = (0..h).find(|&y| (0..w).any(|x| a(x, y))).unwrap_or(0);
            if f < IDLE_FRAMES || f == FRAMES_PER_DIR {
                tops.push(top);
            }
            // Palette discipline.
            let mut cols: Vec<[u8; 3]> = Vec::new();
            for i in 0..(w * h) as usize {
                if img.px[i * 4 + 3] != 0 {
                    let c = [img.px[i * 4], img.px[i * 4 + 1], img.px[i * 4 + 2]];
                    if !cols.contains(&c) {
                        cols.push(c);
                    }
                }
            }
            if cols.len() > 72 {
                push("palette", dir, f, format!("{} colours", cols.len()));
            }
            // Front and back views: symmetry; front: visible eyes.
            if dir != DIR_SIDE && f < IDLE_FRAMES && look.symmetric(dir) && !(dir == DIR_UP && look.hair_style == 2) {
                {
                    let mut diff = 0;
                    for y in 0..h {
                        for x in 0..w / 2 {
                            if img.get(x, y) != img.get(w - 1 - x, y) {
                                diff += 1;
                            }
                        }
                    }
                    if diff > 3 {
                        push("asymmetric", dir, f, format!("{diff} px differ"));
                    }
                }
            }
            if dir == DIR_DOWN && f != FRAMES_PER_DIR {
                let hits = |x0: i32, x1: i32| {
                    let mut n = 0;
                    for y in 0..h {
                        for x in x0..x1 {
                            if let Some(c) = img.get(x, y) {
                                if iris.iter().any(|&k| hex(k) == c) {
                                    n += 1;
                                }
                            }
                        }
                    }
                    n
                };
                let (l, r) = (hits(0, 32), hits(32, w));
                if l < 6 || r < 6 {
                    push("eyes", dir, f, format!("iris px left {l} right {r}"));
                }
            }
        }
        if let (Some(mn), Some(mx)) = (tops.iter().min(), tops.iter().max()) {
            if mx - mn > 2 {
                push("jitter", dir, 0, format!("head top varies {mn}..{mx}"));
            }
        }
    }
    out
}

#[cfg(test)]
mod qa {
    use super::*;

    #[test]
    fn random_characters_pass_visual_qa() {
        let mut rng = Rng::new(77);
        let mut failures = Vec::new();
        for i in 0..240 {
            let look = Look::random_any(&mut rng);
            // Palette size is advisory; everything else is a defect.
            let issues: Vec<_> = lint(&look).into_iter().filter(|i| i.kind != "palette").collect();
            if !issues.is_empty() {
                failures.push(format!("#{i} {:?}: {:?}", look, &issues[..issues.len().min(3)]));
            }
        }
        assert!(failures.is_empty(), "{} looks failed QA:\n{}", failures.len(), failures[..failures.len().min(12)].join("\n"));
    }
}

#[cfg(test)]
mod traits {
    use super::*;

    #[test]
    fn beards_only_on_men_and_figures_only_on_women() {
        let mut rng = Rng::new(5);
        for _ in 0..200 {
            let l = Look::random_any(&mut rng);
            // Asking for a beard on a woman or child changes nothing.
            let mut bearded = l;
            bearded.beard = true;
            bearded.facial = 5;
            if l.female || !l.mature {
                assert_eq!(frame(&bearded, DIR_DOWN, 0).px, frame(&Look { beard: false, facial: 0, ..l }, DIR_DOWN, 0).px);
            }
            // Figure has no effect on men, teens or children.
            if !l.female || !l.mature {
                assert_eq!(frame(&Look { figure: 4, ..l }, DIR_DOWN, 0).px, frame(&Look { figure: 0, ..l }, DIR_DOWN, 0).px);
            }
            let n = l.normalized();
            assert!(!(n.beard && (n.female || !n.mature)));
            assert!(!n.mature || n.age >= 1);
            // A teen (adult stage, not yet 18) gets neither.
            let teen = Look { mature: false, age: 1, beard: true, figure: 4, ..l }.normalized();
            assert!(!teen.beard && teen.figure == default_figure());
        }
    }
}

#[cfg(test)]
mod motion {
    use super::*;

    #[test]
    fn bust_motion_steps_are_small() {
        for f in 0..5 {
            for t in [&JIGGLE_WALK[f][..], &JIGGLE_IDLE[f][..]] {
                for k in 0..t.len() {
                    let (a, b) = (t[k], t[(k + 1) % t.len()]);
                    assert!((b.0 - a.0).abs() <= 2, "figure {f}: offset jump {a:?}->{b:?}");
                    assert!((b.1 - a.1).abs() <= 2 && b.1.abs() <= 1, "figure {f}: shape step {a:?}->{b:?}");
                }
            }
        }
        // Bigger busts never move less than smaller ones.
        let span = |t: &[Jig]| {
            let ys: Vec<i32> = t.iter().map(|j| j.0).collect();
            ys.iter().max().unwrap() - ys.iter().min().unwrap() + t.iter().map(|j| j.1.abs()).max().unwrap()
        };
        for f in 1..5 {
            assert!(span(&JIGGLE_WALK[f]) >= span(&JIGGLE_WALK[f - 1]));
        }
    }
}

// ---------------------------------------------------------------- core cast

/// The hand-tuned core cast: the faces a typical isekai story is built on.
/// Each is curated for quality first; procedural villagers come later.
pub fn cast() -> Vec<(&'static str, Look)> {
    let base = Look { skin: 1, hair: 0, hair_style: 1, primary: 0, secondary: 0, outfit: outfit::PEASANT, hat: 0, accent: 1, female: true, age: 1, beard: false, eyes: 0, mature: true, race: RACE_HUMAN, beast: 0, held: held::NONE, glasses: false, figure: 2, build: 1, height: 1, facial: 0 };
    vec![
        ("Reincarnated Hero", Look { build: 1, female: false, hair: 0, hair_style: 13, outfit: outfit::HERO, primary: 11, secondary: 4, accent: 9, eyes: 2, skin: 1, held: held::SWORD, height: 1, ..base }),
        ("Guild Receptionist", Look { figure: 3, build: 1, hair: 1, hair_style: 1, outfit: outfit::GUILD, primary: 13, secondary: 10, accent: 1, hat: 9, eyes: 1, skin: 1, held: held::BOOK, ..base }),
        ("Elf Mage", Look { figure: 1, build: 0, race: RACE_ELF, hair: 5, hair_style: 8, outfit: outfit::MAGE, primary: 10, secondary: 2, accent: 6, eyes: 6, skin: 0, held: held::STAFF, ..base }),
        ("Cat-girl Adventurer", Look { figure: 2, build: 2, race: RACE_BEAST, beast: 0, hair: 2, hair_style: 5, outfit: outfit::ADVENTURER, primary: 7, secondary: 3, accent: 1, eyes: 5, skin: 2, held: held::SWORD, ..base }),
        ("Female Knight", Look { figure: 3, build: 2, hair: 3, hair_style: 2, outfit: outfit::KNIGHT, primary: 1, secondary: 13, accent: 6, eyes: 0, skin: 1, held: held::SWORD, ..base }),
        ("Healer", Look { figure: 3, build: 1, hair: 3, hair_style: 11, outfit: outfit::PRIEST, primary: 10, secondary: 1, accent: 6, eyes: 0, skin: 0, held: held::STAFF, hat: 7, ..base }),
        ("Dwarf Smith", Look { facial: 6, build: 2, female: false, race: RACE_DWARF, hair: 4, hair_style: 0, outfit: outfit::SMITH, primary: 3, secondary: 8, accent: 1, beard: true, hat: 2, eyes: 2, skin: 3, held: held::HAMMER, ..base }),
        ("Demon Lord", Look { figure: 4, build: 2, race: RACE_DEMON, skin: 3, beast: 1, hair: 0, hair_style: 1, outfit: outfit::DEMON, primary: 11, secondary: 1, accent: 5, eyes: 4, height: 2, ..base }),
        ("Barmaid", Look { figure: 4, build: 3, hair: 3, hair_style: 10, outfit: outfit::INNKEEPER, primary: 1, secondary: 2, accent: 6, hat: 10, eyes: 1, skin: 2, held: held::MUG, ..base }),
        ("Fox Merchant", Look { figure: 2, build: 2, race: RACE_BEAST, beast: 1, hair: 4, hair_style: 7, outfit: outfit::MERCHANT, primary: 5, secondary: 7, accent: 6, eyes: 5, skin: 1, held: held::BASKET, ..base }),
        ("Village Elder", Look { facial: 6, build: 0, female: false, age: 2, hair: 5, hair_style: 18, outfit: outfit::SCHOLAR, primary: 2, secondary: 7, accent: 6, beard: true, eyes: 2, skin: 3, held: held::STAFF, ..base }),
        ("Shopkeeper", Look { facial: 2, build: 3, female: false, hair: 1, hair_style: 0, outfit: outfit::INNKEEPER, primary: 9, secondary: 3, accent: 6, hat: 5, glasses: true, eyes: 2, skin: 2, held: held::BASKET, ..base }),
    ]
}
