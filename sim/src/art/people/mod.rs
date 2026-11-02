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

mod body;
mod cast;
mod clothing;
mod face;
mod hair;
mod paint;
mod props;
mod qa;

use serde::{Deserialize, Serialize};

use super::canvas::Img;
use crate::palette::{self as pal, hex, mix, Rgb};
use crate::rng::Rng;
use body::*;
use clothing::*;
use face::*;
use hair::*;
use paint::*;
use props::*;

pub use cast::cast;
pub use qa::{lint, Issue, ADVISORY};

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
    pub const MICRO: u8 = 23;
    pub const COUNT: u8 = 24;

    pub const NAMES: [&str; COUNT as usize] = [
        "Peasant", "Labourer", "Fisher", "Smith", "Cook", "Merchant", "Innkeeper", "Guard", "Knight", "Hero", "Adventurer", "Rogue", "Ranger", "Mage", "Scholar", "Priest", "Monk", "Bard", "Maid / Butler", "Noble", "Guild clerk", "Demon lord", "Swimwear", "Micro bikini",
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
            (SWIM, false) | (MICRO, false) => ["Trunks", "Waistband", ""],
            (MICRO, true) => ["Bikini", "", "Strings"],
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
            MICRO => if female { (WHITE, WHITE, WHITE) } else { (BLACK, WHITE, WHITE) },
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
    /// How revealing the outfit is cut (see [`CUTS`]).
    #[serde(default)]
    pub cut: u8,
}

/// Modest as designed; bold: a sweetheart neckline with cleavage and
/// shorter skirts (an open collar on men); daring: a plunging, off-shoulder
/// neckline, bare midriff and mini skirts (an open shirt on men).
pub const CUTS: [&str; 3] = ["Modest", "Bold", "Daring"];

pub const FACIAL: [&str; 8] = ["None", "Stubble", "Hero mustache", "Goatee", "Short beard", "Full beard", "Sage beard", "Crescent mustache"];

impl Look {
    /// Headwear over the crown (bandana, helmet, hood, cap, chef, miner):
    /// hair that would rise above it is not drawn.
    fn covers_crown(&self) -> bool {
        matches!(self.hat, 2 | 3 | 4 | 5 | 8 | 11)
    }

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
            cut: 0,
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
    let mut cv = Canvas::new(&mut img);
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
    // Post passes only need the sprite's box (plus one pixel of outline).
    let bounds = padded_bounds(&img);
    contours(&mut img, &ids, &groups, &shades, bounds);
    outline(&mut img, &shades, bounds);
    let bounds = padded_bounds(&img);
    close_pinholes(&mut img, bounds);
    img
}

/// Build all frames of a sheet in atlas order: for each dir (down, up,
/// side) its `FRAMES_PER_DIR` poses, then the front blink frame.
pub fn sheet(look: &Look) -> Vec<Img> {
    let mut out = Vec::with_capacity(SHEET_FRAMES as usize);
    for dir in [DIR_DOWN, DIR_UP, DIR_SIDE] {
        for f in 0..FRAMES_PER_DIR {
            out.push(frame(look, dir, f));
        }
    }
    out.push(frame(look, DIR_DOWN, FRAMES_PER_DIR));
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
        l.cut = rng.below(CUTS.len()) as u8;
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
