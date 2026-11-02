//! Hairstyles as pieces on the skull (the SpriteKiln hair-kit split: crown
//! cap, fringe, temple locks, nape/back mass, tied attachments) with one
//! parameter set per style. Each view is a projection of the same pieces:
//! the profile fringe is the heavier half of the front fringe laid along
//! the head's depth, so a cut that is full from the front is just as full
//! in profile and from behind. Drawn in two layers as in LPC sheets:
//! `back_hair` behind the body, `hair_front` over the face.
//!
//! Research: pixel-dissected 64x128 anime idle/walk sheets (long hair, 3
//! views); Clip Studio Tips "How to draw anime hair" (three growth zones,
//! bangs start just above the brows); animeoutline.com female hair (back
//! mass follows the skull a little off it, widens to the shoulders; tied
//! styles show little side hair); kawaiihannah pixel shading (ribbons not
//! strands, one shine halo on the skull sphere, a big shadow behind the
//! neck). Rules extracted:
//!
//! | rule                           | reference (px, 64x128)             | here (64x120)                    |
//! |--------------------------------|------------------------------------|----------------------------------|
//! | crown top at one y in 3 views  | y=17 down, 16 side, 17 up          | head_c.y - ry - 1, every view    |
//! | hair volume over the skull     | 3 px above the skin dome           | head_r + 1 (tight cuts +0.5)     |
//! | fringe bottom vs the eyes      | brow row, 4 px above the eye       | tips end cy-1 .. cy+5            |
//! | profile depth vs front width   | 31 deep vs 28 wide: +3 behind      | back of cap rx+1 behind cx       |
//! | profile: fringe share of depth | front 45% (14 of 31 px)            | px >= cx + 2                     |
//! | profile: temple lock           | in front of the ear, to the jaw    | always drawn, >= 6 px, short too |
//! | profile: no bare temple        | fringe runs into the lock, no gap  | fringe limit spans the temple    |
//! | back view, short hair          | solid dome to the nape, U hairline | cap to nape, hairline rise       |
//! | back view, long hair           | sheet 2 px wider than the skull    | sheet_w()                        |
//! | hats over the crown            | only fringe, locks and tails show  | covers_crown(): no crown pieces  |
//!
//! Side and back views, re-derived from the 64x128 idle sheet (frame 0, all
//! four directions, top-down 3/4 camera), the 8x head crops, the itch.io
//! turnaround gifs and SpriteKiln's hair kit (`specs/kits/hair.yaml`: back
//! mass = three tubes hugging 2-3 px behind the skull, temple lock = one
//! tube from the temple past the ear to the jaw):
//!
//! | rule                            | reference                            | here                                   |
//! |---------------------------------|--------------------------------------|----------------------------------------|
//! | profile back edge is vertical   | x46 rows 25-55: straight to shoulders| SIDE_HEAD row + volume, sheet follows  |
//! | profile depth = front width + 2 | 29 deep at the ear row vs 27 wide    | skull row +2.5 behind, +2.5 in front   |
//! | profile fringe: level bottom    | 4 px thick, ends at the brow, 1 step | clump slopes capped at 0.5 (no V's)    |
//! | strands inside, not notches     | dark columns x23-33 inside the fringe| near-vertical lines to the fringe edge |
//! | temple lock in front of the ear | 3 px wide, fringe bottom to the jaw  | lock_strip 3 px at ear_x + 4           |
//! | ear shows, even with long hair  | skin x33-36 between lock and mass    | EAR_SIDE always stamped over the cap   |
//! | back mass behind the ear        | 9-10 px wide, darkest beside the neck| cap/sheet to the neck, 2 dark columns  |
//! | profile tones (3/4 camera)      | light crown, mid mass, dark at neck  | h3 crown + fringe, h2 mass, h1 at neck |
//! | tail hangs, never fins out      | tied at the occiput, falls to the    | tie at skull back, cy-4; tail straight |
//! |                                 | shoulders behind the neck            | down to neck.y + 8                     |
//! | profile shows a crown sliver    | hair top 1 row above front and back  | side dome ellipse raised 1 row         |
//! | back view: crown seen from above| light top 55% in a U to ear level,   | h3 above U (cy+5 centre, cy-1 sides),  |
//! |                                 | mid below, no stripes on the dome    | h2 below; no flow lines                |
//! | back view: splits only low down | darker clump lines in bottom 10 rows | hair_sheet lower clumps only           |
//! | back-view tail                  | wide (4-5 px) from the occiput       | r 4.5 from cy-4, strands converge      |
//! | every joint from the Build      | head sits lower / neck shorter later | head_c, head_r, neck_top, neck, neck_w |

use super::*;

/// What a hairstyle adds on top of the cap, fringe, locks and back sheet.
#[derive(Clone, Copy, PartialEq)]
enum Extra {
    Plain,
    Ponytail,
    Bun,
    Spikes,
    TwinTails,
    /// Ojou-sama tate-roll ringlets at the temples.
    Drills,
    TwinBraids,
    SideBraid,
    /// Very long hair with a half-up loop and ribbon at the back.
    HalfUp,
    /// Protagonist hair: an ahoge and tufts at the nape.
    Messy,
    /// Tied low at the nape (butler, elf lord).
    LowTie,
    /// Combed straight back off the forehead.
    Slick,
    Undercut,
    /// Samurai: tied high at the crown, the tail to the shoulders.
    TopKnot,
    /// A horseshoe of hair at the sides and back.
    Bald,
}

/// Fringe clumps: (x offset from centre, tip y offset from head centre,
/// slope of the clump's sides). Lower slope = blunter cut.
type Tips = &'static [(f32, f32, f32)];

/// A hairstyle's parameters. Every view is drawn from these.
struct Style {
    tips: Tips,
    /// Temple locks framing the face: length below the head centre, width.
    lock: (f32, f32),
    /// Length of the back mass below the head centre; 0 = short.
    back: f32,
    /// How far the cap reaches below the head centre at the nape.
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

fn style(s: u8) -> Style {
    let base = Style { tips: &[], lock: (0.0, 3.0), back: 0.0, nape: 9.0, gathered: false, blunt: false, hem: 6.0, tight: false, extra: Extra::Plain };
    match s {
        // Short, slightly messy (masculine).
        0 => Style { tips: &[(-11.0, -1.0, 1.6), (-7.5, 0.0, 1.8), (-3.5, 1.0, 1.8), (0.5, 0.0, 1.8), (4.5, 1.0, 1.8), (8.5, -0.5, 1.6), (12.0, -2.0, 1.4)], lock: (4.0, 3.0), nape: 8.0, tight: true, ..base },
        // Long straight.
        1 => Style { tips: &[(-10.5, 4.5, 1.5), (-6.5, 0.5, 1.7), (-2.0, 4.0, 1.7), (2.0, 4.0, 1.7), (6.5, 0.5, 1.7), (10.5, 4.5, 1.5)], lock: (24.0, 4.0), back: 46.0, nape: 12.0, ..base },
        // Ponytail.
        2 => Style { tips: &[(-10.5, 4.0, 1.5), (-6.5, 1.5, 1.6), (-2.0, 3.5, 1.7), (2.5, 2.5, 1.7), (6.5, 1.0, 1.6), (10.5, 3.5, 1.5)], lock: (12.0, 3.5), nape: 6.0, gathered: true, extra: Extra::Ponytail, ..base },
        // Bun.
        3 => Style { tips: &[(-10.5, 3.5, 1.5), (-6.5, 1.0, 1.8), (-2.5, 3.0, 1.8), (2.5, 3.0, 1.8), (6.5, 1.0, 1.8), (10.5, 3.5, 1.5)], lock: (9.0, 3.0), nape: 6.0, gathered: true, extra: Extra::Bun, ..base },
        // Spiky hero.
        4 => Style { tips: &[(-11.0, 0.5, 1.3), (-7.0, 1.5, 1.5), (-3.0, 2.0, 1.6), (1.0, 1.0, 1.6), (5.0, 2.0, 1.5), (9.0, 0.5, 1.4), (12.0, -1.0, 1.3)], lock: (6.0, 3.5), tight: true, extra: Extra::Spikes, ..base },
        // Bob with a blunt fringe.
        5 => Style { tips: &[(-10.0, 2.0, 0.6), (-6.5, 1.5, 0.6), (-2.0, 2.0, 0.6), (2.0, 2.0, 0.6), (6.5, 1.5, 0.6), (10.0, 2.0, 0.6)], lock: (13.0, 5.5), back: 12.0, nape: 13.0, ..base },
        // Twin tails.
        6 => Style { tips: &[(-10.5, 4.5, 1.5), (-6.5, 0.5, 1.7), (-2.0, 4.0, 1.7), (2.0, 4.0, 1.7), (6.5, 0.5, 1.7), (10.5, 4.5, 1.5)], lock: (11.0, 3.5), extra: Extra::TwinTails, ..base },
        // Side-swept.
        7 => Style { tips: &[(-10.0, 4.0, 1.2), (-6.0, 3.5, 1.2), (-2.0, 2.5, 1.3), (2.0, 1.0, 1.4), (6.0, -0.5, 1.5), (10.0, -1.0, 1.5)], lock: (14.0, 4.0), back: 10.0, nape: 11.0, ..base },
        // Hime cut: dead-straight bangs at the brow, side locks cut level
        // at the chin, a sleek sheet to the waist with a straight hem.
        8 => Style { tips: &[(-10.5, 2.0, 0.2), (-6.5, 2.0, 0.2), (-2.0, 2.0, 0.2), (2.0, 2.0, 0.2), (6.5, 2.0, 0.2), (10.5, 2.0, 0.2)], lock: (13.5, 5.0), back: 46.0, nape: 12.0, blunt: true, hem: 0.0, ..base },
        // Ojou drills: swept bangs, ringlets at the temples, hair to the
        // shoulder blades behind.
        9 => Style { tips: &[(-10.5, 4.0, 1.4), (-6.0, 1.0, 1.6), (-2.0, 3.0, 1.7), (2.0, 3.0, 1.7), (6.0, 1.0, 1.6), (10.5, 4.0, 1.4)], lock: (7.0, 3.0), back: 18.0, nape: 12.0, hem: 3.0, extra: Extra::Drills, ..base },
        // Twin braids (osage): centre part, a braid from behind each ear.
        10 => Style { tips: &[(-10.0, 4.5, 1.2), (-6.0, 3.0, 1.4), (-2.5, 0.0, 1.8), (2.5, 0.0, 1.8), (6.0, 3.0, 1.4), (10.0, 4.5, 1.2)], lock: (6.0, 3.0), gathered: true, extra: Extra::TwinBraids, ..base },
        // Side braid: soft swept bangs, one thick plait over the shoulder.
        11 => Style { tips: &[(-10.0, 4.0, 1.3), (-6.0, 3.0, 1.3), (-2.0, 2.0, 1.4), (2.0, 1.5, 1.4), (6.0, 0.5, 1.5), (10.0, 0.0, 1.5)], lock: (10.0, 3.5), gathered: true, extra: Extra::SideBraid, ..base },
        // Very long with a half-up loop: parted bangs, locks past the chest.
        12 => Style { tips: &[(-10.5, 5.0, 1.3), (-6.5, 2.0, 1.5), (-2.5, -0.5, 1.8), (2.5, -0.5, 1.8), (6.5, 2.0, 1.5), (10.5, 5.0, 1.3)], lock: (28.0, 4.0), back: 58.0, nape: 12.0, hem: 7.0, extra: Extra::HalfUp, ..base },
        // Messy isekai protagonist: uneven bangs to the eyes, an ahoge.
        13 => Style { tips: &[(-11.0, 1.0, 1.5), (-7.5, 3.0, 1.6), (-3.5, 4.5, 1.8), (0.5, 2.5, 1.9), (4.0, 5.0, 1.7), (8.0, 2.5, 1.6), (11.5, 0.0, 1.4)], lock: (6.0, 3.2), nape: 10.0, tight: true, extra: Extra::Messy, ..base },
        // Long-haired pretty boy: parted bangs, long locks, tied low.
        14 => Style { tips: &[(-10.5, 3.0, 1.4), (-6.5, 2.0, 1.6), (-2.5, -1.0, 1.8), (2.5, -1.0, 1.8), (6.5, 2.0, 1.6), (10.5, 3.0, 1.4)], lock: (20.0, 3.5), nape: 10.0, gathered: true, extra: Extra::LowTie, ..base },
        // Slicked back: the forehead bare, combed over the crown.
        15 => Style { tips: &[(-10.0, -4.0, 0.9), (-5.0, -5.5, 0.6), (0.0, -6.0, 0.5), (5.0, -5.5, 0.6), (10.0, -4.0, 0.9)], nape: 9.0, tight: true, extra: Extra::Slick, ..base },
        // Undercut: centre-parted top over clipped sides.
        16 => Style { tips: &[(-8.0, 0.0, 1.4), (-4.5, 1.5, 1.6), (-1.5, -3.0, 1.8), (1.5, -3.0, 1.8), (4.5, 1.5, 1.6), (8.0, 0.0, 1.4)], nape: 8.0, tight: true, extra: Extra::Undercut, ..base },
        // Samurai: widow's peak, long locks, tied high at the crown.
        17 => Style { tips: &[(-10.5, 3.0, 1.4), (-6.0, 0.5, 1.6), (-2.0, -3.0, 1.8), (2.0, -3.0, 1.8), (6.0, 0.5, 1.6), (10.5, 3.0, 1.4)], lock: (16.0, 3.0), nape: 8.0, gathered: true, extra: Extra::TopKnot, ..base },
        // Bald with a horseshoe fringe (elders).
        _ => Style { nape: 8.0, tight: true, extra: Extra::Bald, ..base },
    }
}

impl Style {
    /// The fringe seen in profile: the clumps of the heavier half of the
    /// front fringe laid along the head's depth, the centre clump just
    /// behind the brow and the outermost at the temple. The clump sides are
    /// flattened (reference: the profile fringe ends in a near-level edge
    /// with strand lines inside it, not a saw of V notches).
    fn side_tips(&self, out: &mut [(f32, f32, f32); 8]) -> usize {
        let hang = |s: f32| self.tips.iter().filter(|t| t.0 * s > -0.5).map(|t| t.1).sum::<f32>();
        let s = if hang(-1.0) > hang(1.0) + 0.5 { -1.0 } else { 1.0 };
        let mut n = 0;
        for &(tx, ty, slope) in self.tips.iter().filter(|t| t.0 * s > -0.5) {
            if n < out.len() {
                out[n] = (3.0 + tx.abs() * 0.9, ty, slope.min(0.5));
                n += 1;
            }
        }
        n
    }

    fn long(&self) -> bool {
        self.back > 0.0
    }
}

/// Bottom edge of the fringe at pixel column `px` (pixel centre): each
/// clump hangs as a downward V from its tip, and the fringe is their union.
fn fringe_bottom(tips: &[(f32, f32, f32)], cy: f32, px: f32) -> f32 {
    tips.iter().map(|&(tx, ty, s)| cy + ty - s * (px - (32.0 + tx)).abs()).fold(f32::MIN, f32::max)
}

/// The hair mass around the skull in one view: its centre column and the
/// radii of the cap (the skull plus the hair's volume).
struct Skull {
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
}

impl Skull {
    fn new(look: &Look, b: &Build, dir: u8) -> Skull {
        let tight = style(look.hair_style).tight;
        let (hx, hy) = b.head_r;
        Skull { cx: if dir == DIR_SIDE { b.head_c.x - 1.0 } else { b.head_c.x }, cy: b.head_c.y, rx: hx + if tight { 1.0 } else { 1.5 }, ry: hy + if tight { 0.5 } else { 1.0 } }
    }

    /// Centre of the cap ellipse (a pixel above the head centre).
    fn mc(&self) -> V2 {
        v(self.cx, self.cy - 1.0)
    }

    /// Fill the cap from the crown down to `bottom`: pixels passing `inside`
    /// (normalised offsets) get `col(px, fy, nx, ny)`, None where the view
    /// shows something else (face, ear, nape).
    fn cap(&self, cv: &mut Canvas, bottom: f32, inside: impl Fn(f32, f32) -> bool, col: impl Fn(f32, f32, f32, f32) -> Option<Rgb>) {
        let mc = self.mc();
        for y in (mc.y - self.ry).floor() as i32..=bottom.ceil() as i32 {
            let fy = y as f32 + 0.5;
            for x in (mc.x - self.rx - 1.0) as i32..=(mc.x + self.rx + 1.0) as i32 {
                let px = x as f32 + 0.5;
                let (nx, ny) = ((px - mc.x) / self.rx, (fy - mc.y) / self.ry);
                if !inside(nx, ny) {
                    continue;
                }
                if let Some(c) = col(px, fy, nx, ny) {
                    cv.fill(x, y, c);
                }
            }
        }
    }

    /// Profile landmarks from the skeleton: the hair's resting back edge
    /// (the skull's deepest row plus the hair's volume), the ear's left
    /// column and the temple lock's left column, just in front of the ear.
    fn profile(&self, b: &Build, tight: bool) -> Profile {
        let vb = if tight { 1.5 } else { 2.5 };
        let deepest = SIDE_HEAD.iter().map(|r| r.0).min().unwrap_or(-13) as f32;
        let ear_x = b.head_c.x - 5.0;
        Profile { back: b.head_c.x + deepest - vb, vb, vf: if tight { 2.0 } else { 2.5 }, ear_x, lock_x: ear_x + 4.0 }
    }

    /// Hair extents on one profile row: the dome ellipse over the crown
    /// unioned with the skull row (`SIDE_HEAD`) plus the hair's volume, so
    /// the back edge falls straight to the nape instead of bulging out.
    /// Rows below the skull's nape keep curving in toward the neck.
    fn side_span(&self, b: &Build, p: &Profile, fy: f32) -> (f32, f32) {
        let mc = self.mc();
        // The 3/4 camera shows a sliver of the crown in profile: the dome
        // sits one row higher than the front and back views (ref sheet:
        // hair top y=16 side vs 17 front/back).
        let ny = (fy - mc.y + 1.0) / self.ry;
        let e = if ny.abs() <= 1.0 { self.rx * (1.0 - ny * ny).sqrt() } else { f32::NAN };
        let dy = (fy - b.head_c.y).floor() as i32;
        let (l, r) = match dy {
            -13..=5 => SIDE_HEAD[(dy + 13) as usize],
            d if d > 5 => (SIDE_HEAD[18].0 + (d - 5), SIDE_HEAD[18].1),
            _ => (i32::MAX, i32::MIN),
        };
        let (l, r) = if l == i32::MAX { (f32::NAN, f32::NAN) } else { (b.head_c.x + l as f32 - p.vb, b.head_c.x + r as f32 + p.vf) };
        ((mc.x - e).min(l), (mc.x + e).max(r))
    }
}

struct Profile {
    back: f32,
    vb: f32,
    vf: f32,
    ear_x: f32,
    lock_x: f32,
}

impl Profile {
    /// Where a tied tail hangs from in profile: at the occiput, on the hair's
    /// back edge, so it falls behind the neck instead of finning out.
    fn tie(&self, cy: f32) -> V2 {
        v(self.back + 0.5, cy - 4.0)
    }
}

/// Long-hair width down the back: just under the skull width at the nape,
/// a gentle flare over the shoulders, then a taper toward the tips.
fn sheet_w(rx: f32, t: f32) -> f32 {
    rx * (0.93 + 0.1 * (std::f32::consts::PI * (t / 0.35).min(1.0)).sin() * (1.0 - t * 0.3)) * (1.0 - 0.28 * t * t)
}

/// Pieces hanging behind the body: the back mass and tails, in the views
/// where the body is in front of them.
pub(super) fn back_hair(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
    let h = &c.hair;
    let k = style(look.hair_style);
    let sk = Skull::new(look, b, dir);
    let cy = sk.cy;
    let sway = b.sway;
    cv.begin();
    if k.long() {
        if dir == DIR_SIDE {
            // A straight fall from the hair's back edge: 9-10 px show behind
            // the neck (the front half hides behind the body), a hair wider
            // over the shoulders, then a taper toward the tips.
            let p = sk.profile(b, k.tight);
            let top = cy - 2.0;
            let len = k.back + 2.0;
            let back_x = |t: f32| p.back - 1.5 * (std::f32::consts::PI * (t / 0.3).min(1.0)).sin() * (1.0 - t * 0.5) + 4.0 * t * t + t * t * sway * 2.2;
            let front_x = |t: f32| p.ear_x + 3.0 + t * t * sway * 1.2;
            let tt = |y: f32| ((y - top) / len).clamp(0.0, 1.0);
            hair_sheet(cv, h, top, top + len, false, k.hem * 0.5, |y| (back_x(tt(y)) + front_x(tt(y))) * 0.5, |y| (front_x(tt(y)) - back_x(tt(y))) * 0.5);
        } else {
            // From the front only what shows past the body matters; ending
            // above the knee keeps it from boxing in the legs.
            let top = cy - 2.0;
            let len = k.back.min(b.crotch_y + 6.0 - top);
            hair_sheet(cv, h, top, top + len, false, (k.hem - 1.0).max(0.0), |_| 32.0, |y| sheet_w(sk.rx, ((y - top) / len).clamp(0.0, 1.0)));
        }
    }
    attachments(cv, look, dir, b, c, &k, true);
}

/// Tied pieces (tails, buns, braids, ribbons, tufts) and crown details.
/// `behind` draws the ones hanging behind the body in this view; the rest
/// come after the cap. Pieces rising above the crown hide under hats.
fn attachments(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors, k: &Style, behind: bool) {
    use Extra::*;
    let h = &c.hair;
    let sk = Skull::new(look, b, dir);
    let (cy, ry) = (sk.cy, sk.ry);
    let sway = b.sway;
    let side = dir == DIR_SIDE;
    let crown = !look.covers_crown();
    let tie = r5(&ramp4(&pal::CLOTH[1]));
    // Mirrored pairs in the front and back views; only the near one in
    // profile.
    let pair: &[f32] = if side { &[-1.0] } else { &[-1.0, 1.0] };
    let nape = if k.gathered { cy + 10.0 } else { b.neck_top.max(cy + k.nape) };
    let p = sk.profile(b, k.tight);
    // Where tails end: flowing over the shoulders, not on a neck pole.
    let shoulder = b.neck.y + 8.0;
    match (k.extra, dir, behind) {
        // Tied at the occiput: a short sweep back off the tie, then the
        // tail falls straight behind the neck to the shoulders.
        (Ponytail, DIR_SIDE, true) => {
            let at = p.tie(cy);
            let mid = at.add(v(-3.0 - sway * 0.5, 6.0));
            lock_segment(cv, h, at, mid, 2.6, false);
            tail_lock(cv, h, mid.add(v(0.3, -1.0)), v(mid.x - 0.5 - sway * 2.0, shoulder), 4.2);
        }
        (Ponytail, DIR_SIDE, false) => {
            cv.begin();
            cv.ellipse(p.tie(cy).add(v(0.5, 0.0)), 1.4, 2.2, &tie, 0.05);
        }
        // From the front the tail peeks out past the head.
        (Ponytail, DIR_DOWN, true) => {
            let base = v(43.0, cy + 2.0);
            tail_lock(cv, h, base, base.add(v(2.0 + sway, 18.0)), 3.2);
        }
        (Ponytail, DIR_UP, false) => {
            let base = v(32.0, cy - 4.0);
            // Strands gathered up into the tie.
            for s in [-1.0f32, 1.0] {
                cv.line(v(32.0 + s * 8.5, cy + 4.0), v(32.0 + s * 2.0, cy - 3.0), |t| if t > 0.9 { None } else { Some(h[2]) });
            }
            cv.begin();
            tail_lock(cv, h, base.add(v(0.0, 0.5)), v(32.0 + sway, shoulder + 2.0), 4.5);
            cv.begin();
            cv.ellipse(base, 2.4, 1.6, &tie, 0.1);
        }
        (TwinTails, DIR_DOWN, true) => {
            for &s in pair {
                let base = v(32.0 + s * 14.5, cy - 4.0);
                tail_lock(cv, h, base, base.add(v(s * 3.0 + sway, 30.0)), 4.0);
            }
        }
        // In profile the far tail shows as a sliver behind the neck; the
        // near one hangs beside the ear, over the shoulder.
        (TwinTails, DIR_SIDE, true) => {
            let base = v(p.ear_x - 4.0, cy - 3.0);
            tail_lock(cv, h, base, v(base.x - 1.0 - sway, shoulder), 3.4);
        }
        (TwinTails, DIR_SIDE, false) => {
            let base = v(p.ear_x + 1.5, cy - 3.0);
            cv.begin();
            tail_lock(cv, h, base, v(base.x + sway * 2.0, shoulder + 1.0), 3.8);
            cv.begin();
            cv.ellipse(base.add(v(0.0, -1.0)), 2.2, 1.8, &tie, 0.1);
        }
        (TwinTails, DIR_DOWN | DIR_UP, false) => {
            for &s in pair {
                let base = v(32.0 + s * (sk.rx + 0.5), cy - 4.0);
                cv.begin();
                if dir == DIR_UP {
                    tail_lock(cv, h, base, v(base.x + s * 3.0 + sway, shoulder), 4.0);
                }
                cv.ellipse(base.add(v(0.0, -1.0)), 2.4, 2.0, &tie, 0.1);
            }
        }
        // A long tail tied at the nape, straight down the back.
        (LowTie, DIR_SIDE, true) => tail_lock(cv, h, v(p.back + 3.5, b.neck_top - 5.0), v(p.back + 2.5 - sway * 1.5, shoulder + 16.0), 3.0),
        (LowTie, DIR_SIDE, false) => {
            cv.begin();
            cv.ellipse(v(p.back + 4.0, b.neck_top - 4.5), 1.3, 2.0, &tie, 0.1);
        }
        (LowTie, DIR_UP, false) => {
            let at = v(32.0, b.neck_top - 4.0);
            cv.begin();
            tail_lock(cv, h, at, v(32.0 + sway, shoulder + 18.0), 3.0);
            cv.begin();
            cv.ellipse(at, 2.2, 1.2, &tie, 0.1);
        }
        // Samurai: tied high at the back of the crown, the tail arcing out
        // and down to the shoulders.
        (TopKnot, DIR_SIDE, true) => {
            let at = v(p.back + 4.0, cy - 10.0);
            let mid = at.add(v(-4.5 - sway * 0.5, 5.0));
            lock_segment(cv, h, at, mid, 3.0, false);
            tail_lock(cv, h, mid.add(v(0.3, -0.8)), v(mid.x - 0.5 - sway * 2.0, shoulder), 3.4);
        }
        (TopKnot, _, false) if crown => {
            cv.begin();
            match dir {
                DIR_DOWN => cv.ellipse(v(32.0, cy - ry - 0.5), 2.6, 2.2, h, 0.1),
                DIR_SIDE => {
                    cv.ellipse(v(p.back + 4.5, cy - 10.5), 2.4, 2.4, h, 0.1);
                    cv.begin();
                    cv.ellipse(v(p.back + 5.5, cy - 10.0), 0.9, 2.0, &tie, 0.1);
                }
                _ => {
                    let at = v(32.0, cy - 11.0);
                    tail_lock(cv, h, at, v(32.0 + sway, shoulder - 2.0), 3.2);
                    cv.begin();
                    cv.ellipse(at, 2.6, 2.0, h, 0.1);
                    cv.begin();
                    cv.ellipse(at.add(v(0.0, 1.5)), 2.0, 0.9, &tie, 0.1);
                }
            }
        }
        (Bun, _, false) if crown => {
            cv.begin();
            let (x, dy) = match dir {
                DIR_DOWN => (32.0, -1.0),
                DIR_SIDE => (28.0, -0.5),
                _ => (32.0, -0.5),
            };
            cv.ellipse(v(x, cy - ry + dy), 5.5, 5.0, h, 0.05);
        }
        (Drills, DIR_SIDE, false) => drill(cv, h, v(25.0, cy - 1.0), 24.0, 3.6, -1.2),
        (Drills, _, false) => {
            let first = cv.cur + 1;
            drill(cv, h, v(18.5, cy - 1.0), 24.0, 3.6, -1.5);
            cv.mirror_from(first);
        }
        // Over the shoulders and down the front; from behind only the roots
        // show.
        (TwinBraids, DIR_DOWN, false) => {
            let first = cv.cur + 1;
            let p = |x: f32, y: f32| v(32.0 - x, cy + y);
            braid(cv, h, &[p(11.5, 5.0), p(12.0, 13.0), p(11.0, 22.0), p(10.5, 30.0)], 2.4, 1.8, &tie, 1.0);
            cv.mirror_from(first);
        }
        (TwinBraids, DIR_SIDE, false) => braid(cv, h, &[v(27.5, cy + 8.0), v(29.5, cy + 16.0), v(31.0, cy + 24.0), v(31.5, cy + 30.0)], 2.4, 1.8, &tie, 1.0),
        (TwinBraids, DIR_UP, false) => {
            let first = cv.cur + 1;
            braid(cv, h, &[v(21.5, cy + 6.0), v(19.5, cy + 16.0)], 2.4, 2.2, &tie, -1.0);
            cv.mirror_from(first);
        }
        // Over her right shoulder (screen left from the front).
        (SideBraid, DIR_DOWN, false) => braid(cv, h, &[v(20.5, cy + 5.0), v(20.5, cy + 13.0), v(22.5, cy + 22.0), v(24.0, cy + 32.0)], 2.9, 2.0, &tie, 1.0),
        (SideBraid, DIR_SIDE, false) => braid(cv, h, &[v(27.5, cy + 8.0), v(29.5, cy + 16.0), v(31.5, cy + 24.0), v(32.5, cy + 32.0)], 2.9, 2.0, &tie, 1.0),
        (SideBraid, DIR_UP, false) => braid(cv, h, &[v(40.5, cy + 6.0), v(43.5, cy + 17.0)], 2.9, 2.6, &tie, 1.0),
        (HalfUp, DIR_SIDE, false) => {
            cv.begin();
            cv.ellipse(v(p.back + 2.0, cy - 5.0), 1.6, 2.4, &tie, 0.1);
        }
        // A ribbon bow: two loops, a knot and two tails falling over the hair.
        (HalfUp, DIR_UP, false) => {
            cv.begin();
            for s in [-1.0f32, 1.0] {
                cv.ellipse(v(32.0 + s * 2.6, cy - 5.5), 2.0, 1.3, &tie, 0.1);
                cv.capsule(v(32.0 + s * 0.6, cy - 4.5), v(32.0 + s * 1.8, cy + 1.0), 0.8, 0.6, &tie, 0.0);
            }
            cv.begin();
            cv.ellipse(v(32.0, cy - 5.0), 1.0, 1.1, &tie, -0.2);
        }
        (Messy, _, false) => {
            cv.begin();
            // Tufts overlapping across the nape, no skin between them.
            if dir == DIR_UP {
                for (dx, len) in [(-3.5f32, 3.0f32), (0.0, 4.0), (3.5, 3.0)] {
                    tail_lock(cv, h, v(32.0 + dx, nape - 4.0), v(32.0 + dx * 1.15, nape + len - 2.0), 2.7);
                }
            } else if side {
                let nape = cy + k.nape;
                for (dx, len) in [(3.0f32, 4.0f32), (7.0, 3.5)] {
                    let x = p.back + dx;
                    tail_lock(cv, h, v(x + 1.0, nape - 3.0), v(x - 0.5, nape + len - 2.0), 2.0);
                }
            }
            let (x, d) = if dir == DIR_DOWN { (32.5, 1.0) } else { (31.5, if side { 1.0 } else { -1.0 }) };
            ahoge(cv, look, h, v(x, cy - if side { 15.5 } else { 15.0 }), d);
        }
        // Combed lines sweeping back over the crown.
        (Slick, DIR_SIDE, false) => {
            for (y0, y1) in [(-7.5f32, -12.5f32), (-6.0, -9.0)] {
                cv.line(v(42.0, cy + y0), v(22.0, cy + y1), |t| if t < 0.05 { None } else { Some(h[2]) });
            }
        }
        (Slick, DIR_DOWN, false) => {
            for o in [3.0f32, 7.0, 10.5] {
                for s in [-1.0f32, 1.0] {
                    cv.line(v(32.0 + s * o, cy - 7.0), v(32.0 + s * o * 0.55, cy - 14.0), |t| if t < 0.1 { None } else { Some(h[2]) });
                }
            }
        }
        // A shine on the bare crown.
        (Bald, DIR_DOWN | DIR_SIDE, false) => {
            let ox = if side { 3 } else { 0 };
            for (x, y) in [(27, -11), (28, -11), (26, -10)] {
                cv.put(x + ox, (cy + y as f32) as i32, c.skin[4]);
            }
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
            // A blunt-cut sheet has no tips to break it up, so its clumps
            // are parted by shadow lines down the length.
            let parted = vdepth <= 0.0 && t > 0.12 && uc < 0.13;
            let tone = if edge || lower || parted { base - 1 } else { base };
            cv.fill(x, y, h[tone]);
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
fn ahoge(cv: &mut Canvas, look: &Look, h: &R5, root: V2, dir: f32) {
    if look.covers_crown() {
        return;
    }
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

/// The cap, fringe, temple locks and crown-top details, over the head.
pub(super) fn hair_front(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
    use Extra::*;
    let h = &c.hair;
    let k = style(look.hair_style);
    let ex = k.extra;
    let sk = Skull::new(look, b, dir);
    let (cy, rx, ry) = (sk.cy, sk.rx, sk.ry);
    let mc = sk.mc();
    let sway = b.sway;
    let shine = h[4];
    let crown = !look.covers_crown();
    let human_ears = !matches!(look.race, RACE_ELF | RACE_BEAST);
    // Clipped hair (undercut, a bald man's fringe): skin showing through.
    let clipped: R5 = [mix(c.skin[0], h[0], 0.7), mix(c.skin[1], h[0], 0.7), mix(c.skin[2], h[1], 0.72), mix(c.skin[3], h[1], 0.65), mix(c.skin[4], h[2], 0.6)];
    cv.register(&clipped);
    let skin_pal = |ch: char| match ch {
        'L' => Some(c.skin[3]),
        'M' => Some(c.skin[2]),
        'D' => Some(c.skin[1]),
        _ => None,
    };
    cv.begin();
    if ex == Spikes && crown {
        spikes(cv, h, cy, dir);
    }
    let (ll, lw) = k.lock;
    match dir {
        DIR_UP => {
            // The whole cap down to the nape, then the sheet over the back.
            let nape = if k.gathered { cy + 10.0 } else { b.neck_top.max(cy + k.nape) };
            let neck_w = b.neck_w + 2.8;
            // Round crown, full width to the ears, then the skull curves in
            // under the ears and eases into the neck (a hair under the full
            // radius keeps the widest row from poking out a nub).
            let er = if ex == Bald { 12.8 } else { rx - 0.4 };
            let ear = cy + 3.0;
            let half = |fy: f32| {
                if fy < mc.y {
                    let ny = (fy - mc.y) / ry;
                    er * (1.0 - ny * ny).max(0.0).sqrt()
                } else if fy < ear {
                    er
                } else {
                    let t = ((fy - ear) / (nape - ear).max(1.0)).clamp(0.0, 1.0);
                    er + (neck_w + 0.5 - er) * (1.0 - (1.0 - t).powf(1.8))
                }
            };
            sk.cap(cv, nape, |nx, ny| (nx * rx).abs() <= half(mc.y + ny * ry) + EPS, |px, fy, nx, _| {
                let d = (px - 32.0).abs();
                // The hairline rises behind the ears into a U at the nape.
                let rise = (d / rx).min(1.0).powf(1.6) * if k.gathered { 7.0 } else { 4.0 };
                if fy > nape - rise {
                    return None;
                }
                // The 3/4 camera sees the top of the dome: lit in a U down
                // to ear level at the centre, the fall below it a step
                // darker, no stripes across the crown.
                let under = fy > cy + 5.0 - (d / rx).min(1.0).powi(2) * 6.0;
                Some(match ex {
                    // A horseshoe at ear height; the crown stays bare.
                    Bald if !(cy - 1.0..=cy + 7.0).contains(&fy) => return None,
                    Bald => h[if nx.abs() > 0.8 { 2 } else { 3 }],
                    Undercut if fy > cy - 2.0 => clipped[if nx.abs() > 0.8 { 1 } else { 2 }],
                    _ => h[if under || nx.abs() > 0.9 { 2 } else { 3 }],
                })
            });
            let top = cy + 5.0;
            let len = (k.back - 5.0).max(0.0);
            if k.long() {
                hair_sheet(cv, h, top, top + len, false, k.hem, |_| 32.0, |y| sheet_w(rx, ((y - top) / len).clamp(0.0, 1.0)));
            } else if !matches!(ex, Bald | Undercut) {
                // Short cuts: a pair of clump splits low on the nape only.
                for s in [-1.0f32, 1.0] {
                    cv.line(v(32.0 + s * 4.0, nape - 6.0), v(32.0 + s * 4.5, nape - 1.5), |t| if t < 0.1 { None } else { Some(h[1]) });
                }
            }
            if ex != Bald {
                highlight_ring(cv, h, shine, mc, rx, ry, &[]);
            }
        }
        DIR_SIDE => {
            let mut buf = [(0.0, 0.0, 0.0); 8];
            let n = k.side_tips(&mut buf);
            let stips = &buf[..n];
            let long = k.long();
            let p = sk.profile(b, k.tight);
            let nape = cy + k.nape;
            // Short cuts end at the nape, the bottom edge rising behind the
            // ear; long hair's cap stops at the skull's nape and the sheet
            // (behind the body) carries on down.
            // Long hair covers the occiput down to the skull's bottom row in
            // profile (the sheet behind the body carries on below); the old
            // cap stopped at the nape and left the lower back of the head bald.
            let skull_nape = cy + 16.0;
            let bottom = if long { skull_nape } else { nape };
            let ear_top = cy + 3.5;
            for y in (mc.y - ry).floor() as i32 - 1..=bottom.ceil() as i32 {
                let fy = y as f32 + 0.5;
                let (xl, xr) = sk.side_span(b, &p, fy);
                if xl.is_nan() {
                    continue;
                }
                for x in xl.floor() as i32..=xr.ceil() as i32 {
                    let px = x as f32 + 0.5;
                    if px < xl || px > xr {
                        continue;
                    }
                    // Three zones along the depth: the fringe over the brow
                    // (level bottom), the ear column (the ear is stamped
                    // over), and the mass behind the ear down to the nape.
                    let limit = if px >= p.lock_x {
                        if ex == Slick { cy - 7.0 } else { fringe_bottom(stips, cy, px) }
                    } else if px >= p.ear_x {
                        ear_top
                    } else if long {
                        skull_nape
                    } else {
                        nape - (px - (p.ear_x - 5.0)).max(0.0) * 0.8
                    };
                    if fy > limit {
                        continue;
                    }
                    // Rounded bottom-back corner.
                    if fy > bottom - 1.0 && px < xl + 1.0 {
                        continue;
                    }
                    let back_edge = px < xl + 1.0 && fy > cy - 7.0;
                    let under = fy > limit - 1.0 && px < p.lock_x;
                    // The 3/4 camera: the crown and fringe lit, the mass
                    // behind the ear a step darker, darkest beside the neck.
                    let mass = px < p.ear_x && fy > cy - 3.5 + ((px - xl) / 10.0).powi(2) * 2.0;
                    let neck = px >= p.ear_x - 2.0 && px < p.ear_x && fy > cy + 1.0;
                    let tone = if neck { 1 } else if back_edge || under || mass { 2 } else { 3 };
                    let c = match ex {
                        Bald if fy < cy - 3.0 || px > p.ear_x + 2.5 => continue,
                        Undercut if fy > cy - 5.0 && px < p.lock_x + 2.0 => clipped[tone.min(2)],
                        _ => h[tone],
                    };
                    cv.fill(x, y, c);
                }
            }
            // The ear shows between the lock and the back mass, even under
            // long hair; beastfolk and elves draw their own ears.
            if human_ears {
                cv.begin();
                stamp_fill(cv, p.ear_x as i32, cy.round() as i32 + 3, &EAR_SIDE, false, &skin_pal);
                cv.begin();
            }
            // Every cut has a temple lock (a sideburn at least) in front of
            // the ear, from the fringe's edge to the jaw.
            if ll > 0.0 && !matches!(ex, Slick | Undercut | Bald) {
                let top = fringe_bottom(stips, cy, p.lock_x + 1.5) - 1.5;
                lock_strip(cv, h, p.lock_x + 1.5, top.min(cy + 1.0), cy + ll.clamp(6.0, 12.0), (lw * 0.7).clamp(2.6, 3.2), sway, 1.0, k.blunt);
            }
            // Strands inside the fringe, falling to its edge.
            for &(tx, _, _) in stips.iter().skip(1) {
                let ex0 = 32.0 + tx - 2.0;
                let nb = fringe_bottom(stips, cy, ex0.floor() + 0.5);
                cv.line(v(ex0 + 0.5, cy - 9.0), v(ex0, nb - 0.5), |t| if t < 0.1 { None } else if t > 0.75 { Some(h[1]) } else { Some(h[2]) });
            }
            if ex != Bald {
                // One strand over the back of the crown into the mass.
                let low = if ex == Undercut { cy - 6.0 } else { bottom - 2.0 };
                cv.line(v(p.back + 6.5, cy - 11.0), v(p.back + 4.0, low), |t| if t < 0.15 { None } else { Some(h[2]) });
                highlight_ring(cv, h, shine, mc, rx, ry, &[]);
            }
        }
        _ => {
            let tips = k.tips;
            let face_w = 12.0;
            sk.cap(cv, cy + 4.0, |nx, ny| nx * nx + ny * ny <= 1.0, |px, fy, _, _| {
                let dxa = (px - 32.0).abs();
                let temple = dxa > face_w - 1.0 && fy > cy - 4.0;
                Some(match ex {
                    // A thin fringe hugging the sides of the skull.
                    Bald if dxa > face_w - 0.6 && dxa < face_w + 1.0 && fy > cy - 4.0 && fy < cy + 3.5 => h[if dxa > face_w { 2 } else { 3 }],
                    Bald => return None,
                    // Clipped sides from the temples down, close to the skull.
                    Undercut if dxa > 9.0 && fy > cy - 6.0 => {
                        if fy > cy + 1.0 || dxa > face_w + 0.6 {
                            return None;
                        }
                        clipped[if dxa > face_w - 0.6 { 1 } else { 2 }]
                    }
                    _ => {
                        // Slicked-back sides lie flat above the ears.
                        let side_end = if ex == Slick { cy - 5.0 } else { cy + 3.0 };
                        let limit = if dxa < face_w + 0.5 { fringe_bottom(tips, cy, px) } else { side_end };
                        if fy > limit {
                            return None;
                        }
                        h[if temple { 2 } else { 3 }]
                    }
                })
            });
            // Strand lines from the crown into each notch between clumps.
            let mut xs: Vec<f32> = tips.iter().map(|t| t.0).collect();
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let mut notches = Vec::new();
            for w in xs.windows(2) {
                let nx = (w[0] + w[1]) * 0.5;
                // Nudged toward the centre so notches on whole pixels mirror.
                let px = (32.0 + nx - nx.signum() * 1e-3).floor() as i32;
                let nb = fringe_bottom(tips, cy, px as f32 + 0.5);
                if nx.abs() >= 0.6 {
                    notches.push(px);
                }
                if nx.abs() < 0.6 || ex == Slick {
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
                let fb = fringe_bottom(tips, cy, x as f32 + 0.5);
                let y = (fb - 0.5).floor() as i32;
                if cv.get(x, y).is_some() && fb > cy - 3.0 {
                    cv.put(x, y, h[2]);
                }
            }
            if ex != Bald {
                highlight_ring(cv, h, shine, mc, rx, ry, &notches);
            }
            // Cuts that clear the sides of the head show the ears.
            if matches!(ex, Bald | Slick | Undercut) && human_ears {
                cv.begin();
                stamp_fill(cv, 17, cy.round() as i32 + 3, &EAR_FRONT, false, &skin_pal);
                stamp_fill(cv, 17, cy.round() as i32 + 3, &EAR_FRONT, true, &skin_pal);
            }
            // Temple locks framing the face.
            if ll > 0.0 {
                for s in [-1.0f32, 1.0] {
                    let cxl = 32.0 + s * (rx - lw * 0.5);
                    lock_strip(cv, h, cxl, cy - 4.0, cy + ll, lw, sway, s, k.blunt);
                }
            }
        }
    }
    attachments(cv, look, dir, b, c, &k, false);
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
                cv.fill(x, y, h[if u > len * 0.75 || lit { 3 } else { 2 }]);
            }
        }
    };
    if dir == DIR_SIDE {
        // Profile: swept back over the crown.
        for (k, (x, ang, len)) in [(36.0f32, -0.35f32, 7.0f32), (31.0, -0.6, 8.5), (25.5, -0.85, 8.5), (20.5, -1.15, 7.0)].into_iter().enumerate() {
            let y = cy - 13.5 + (k as f32 - 0.5).abs() * 1.2;
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
