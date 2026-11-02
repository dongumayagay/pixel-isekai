//! Clothing: each outfit as a recipe of garment pieces and dye slots, and
//! the torso pass that draws them over the body.

use super::*;

/// Profile key points: (y, half-width or edge offset).
type Keys = Vec<(f32, f32)>;

/// Where a garment piece takes its colour from: one of the look's three
/// dyes, or a fixed material.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Dye {
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

pub(super) fn dye(c: &Colors, d: Dye) -> R5 {
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
pub(super) enum Sleeve {
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
pub(super) enum Lower {
    Pants,
    /// Skirt from the waist: length as a fraction of the thigh, flare.
    Skirt(f32, f32),
    /// Floor-length robe or gown with this flare.
    Gown(f32),
}

/// One layer over the body, drawn in list order.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Piece {
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
    /// Micro bikini: small cups over the front of each breast and a tiny
    /// high-cut bottom on thin strings.
    Micro(Dye, Dye),
}

/// A whole outfit: body garment, sleeves, legs, lower garment, footwear and
/// the layers over them.
pub(super) struct Garb {
    pub(super) shirt: Dye,
    pub(super) sleeve: (Sleeve, Dye),
    pub(super) legs: Dye,
    pub(super) lower: (Lower, Dye),
    /// Boots: material and height above the ankle.
    pub(super) boots: Option<(Dye, f32)>,
    /// Baggy trousers.
    pub(super) wide: bool,
    /// Skin showing at an open collar.
    pub(super) neckline: bool,
    pub(super) barefoot: bool,
    /// Revealing cut (0 modest, 1 bold, 2 daring), where the outfit allows.
    pub(super) cut: u8,
    pub(super) pieces: Vec<Piece>,
}

pub(super) fn garb(look: &Look) -> Garb {
    use Dye::*;
    use Piece::*;
    let f = look.female;
    let mut g = Garb { shirt: P, sleeve: (Sleeve::Long, P), legs: Drab, lower: (Lower::Pants, Drab), boots: None, wide: false, neckline: false, barefoot: false, cut: 0, pieces: Vec::new() };
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
        outfit::SWIM | outfit::MICRO => {
            g.shirt = Skin;
            g.sleeve = (Sleeve::Bare, Skin);
            g.legs = Skin;
            g.barefoot = true;
            g.pieces = vec![match (f, look.outfit == outfit::MICRO) {
                (false, _) => Trunks(P, S),
                (true, false) => Bikini(P, A),
                (true, true) => Micro(P, A),
            }];
        }
        _ => {
            // Demon lord: floor-length gown or coat, a high collar, a cape
            // lined in the second dye.
            g.lower = (Lower::Gown(0.36), P);
            g.legs = Skin;
            g.pieces = vec![Cape(P, S), Sash(Gold), Collar(S), Gem(A)];
        }
    }
    // A revealing cut: armour and swimwear stay as they are; skirts rise.
    if !matches!(look.outfit, outfit::KNIGHT | outfit::GUARD | outfit::SWIM | outfit::MICRO) {
        g.cut = look.cut.min(2);
        if let (true, Lower::Skirt(len, flare)) = (f, g.lower.0) {
            g.lower.0 = Lower::Skirt(len * [1.0, 0.78, 0.58][g.cut as usize], flare);
        }
    }
    g
}

impl Garb {
    /// The cape's outside and lining, if the outfit has one.
    pub(super) fn cape(&self) -> Option<(Dye, Dye)> {
        self.pieces.iter().find_map(|p| if let Piece::Cape(o, l) = p { Some((*o, *l)) } else { None })
    }
    /// Whether the outfit is mirror-symmetric from the front and back:
    /// straps, one-shoulder pieces, scarf tails and wrap fronts aren't.
    pub(super) fn symmetric(&self) -> bool {
        !self.pieces.iter().any(|p| matches!(p, Piece::Satchel | Piece::Quiver | Piece::HalfCape(_) | Piece::Wrap | Piece::Scarf(_) | Piece::DiagSash(_) | Piece::Pauldrons(false, _)))
    }
}

pub(super) fn torso(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
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
    let (front_keys, back_keys): (Keys, Keys) = if look.female {
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
    // From behind the nape sits in the hair's shadow.
    cv.profile(b.neck_top, sy + 1.5, |_| b.neck.x + if side { 0.5 } else { 0.0 }, |_| b.neck_w, &c.skin, if back { -0.3 } else { -0.06 });
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
    // Seen from above, the tops of the shoulders catch the light.
    if !front {
        for x in 0..FW {
            cv.lighten_on(x, sy.ceil() as i32, &shirt);
        }
    }
    let by = b.bust_y;
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
            cv.drape(top, top + len, |y| sc + (y - top) / len * sway * 0.8, |y| w0 + (y - top) * k, &ramp, -0.02, if side { 1.5 } else { 2.5 });
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
            cv.drape(top, bottom, |y| cx + (y - top) / len * sway * 0.7, |y| w0 + (y - top) * k, &ramp, 0.0, if side { 2.0 } else { 3.5 });
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
        if pass == 1 && g.cut > 0 && !back {
            // Only a separate top can be cropped; dresses and gowns stay whole.
            let crop = !matches!(lower_kind, Lower::Gown(_)) && (lower_kind == Lower::Pants || lower_dye != g.shirt);
            reveal(cv, look, b, dir, by, g.cut, &[shirt, bust_ramp], c, &width, &fwd, crop);
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
                    cv.drape(at, bottom, |_| ax, |y| if side { aw } else { aw + (y - at).max(0.0) * 0.06 }, &ap, 0.06, if side { 0.0 } else { 1.5 });
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
                    cv.profile(sy - 3.0, sy + 1.0, |_| ncx, |_| b.neck_w + 2.2, &sc, 0.05);
                    cv.begin();
                    if side {
                        let a0 = v(ncx - b.neck_w - 0.5, sy - 0.5);
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
                        hspan(cv, b.neck_w, sy as i32 - 1, kc[2]);
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
                Piece::Micro(d, strings) => micro_bikini(cv, b, c, dir, by, &dye(c, d), &dye(c, strings), fwd, aft),
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
    if let (true, Some((outside, _))) = (back, g.cape()) {
        cloak_back(cv, b, &dye(c, outside), false, false);
        // A quiver is slung over the cloak.
        if g.pieces.contains(&Piece::Quiver) {
            quiver_back(cv, b, c);
        }
    }
}

/// Fill the already-drawn body pixels in `rows`, where `inside(x, y)`
/// says so, as the current part; mirrored pairs stay mirrored.
pub(super) fn cover(cv: &mut Canvas, y0: f32, y1: f32, mut inside: impl FnMut(i32, f32) -> Option<Rgb>) {
    for y in y0.floor() as i32..=y1.ceil() as i32 {
        let fy = y as f32 + 0.5;
        if fy < y0 || fy > y1 {
            continue;
        }
        for x in 0..FW {
            if let (true, Some(col)) = (cv.get(x, y).is_some(), inside(x, fy)) {
                cv.fill(x, y, col);
            }
        }
    }
}

/// Triangle bikini top and tie-side bottoms over bare skin.
#[allow(clippy::too_many_arguments)]
pub(super) fn swim_top(cv: &mut Canvas, b: &Build, c: &Colors, dir: u8, by: f32, cloth: &R5, ties: &R5, fwd: impl Fn(f32) -> f32) {
    let (cx, sy) = (b.cx, b.shoulder_y);
    let r = b.bust.max(2.5);
    let bottom = by + r * 0.85;
    match dir {
        DIR_DOWN => {
            // One cup on the left, mirrored. It wraps the breast's own
            // shading, stops short of its outer side (side curve shows) and
            // of its underside (a crescent of under-bust shows), and ties
            // with strings instead of a solid band.
            let ry = r * 0.9;
            let bc = cx - r * 0.92;
            let (ctop, cbase) = (by - ry * 0.7, by + ry * 0.62);
            let first = cv.cur + 1;
            cv.begin();
            for y in ctop.floor() as i32..=cbase.ceil() as i32 {
                let fy = y as f32 + 0.5;
                if fy < ctop || fy > cbase {
                    continue;
                }
                let half = ((fy - ctop) * 0.85 + 0.6).min(r * 0.78 + 0.3);
                for x in 0..32 {
                    if (x as f32 + 0.5 - bc).abs() <= half {
                        wrap_skin(cv, x, y, &c.skin, cloth);
                    }
                }
            }
            cv.mirror_from(first);
            let y0 = cbase.floor() as i32;
            // Strings: halter to the neck, round the ribs, a knot between.
            cv.line(v(bc, ctop), v(cx - 3.0, sy - 1.0), |_| Some(ties[2]));
            cv.line(v(64.0 - bc, ctop), v(cx + 3.0, sy - 1.0), |_| Some(ties[2]));
            for x in (cx - b.chest_w + 0.5) as i32..(bc - r * 0.78) as i32 {
                cv.put(x, y0, ties[2]);
                cv.put(FW - 1 - x, y0, ties[2]);
            }
            hspan(cv, (r * 0.92 - r * 0.78).max(1.0), y0, ties[2]);
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
            // The cup over the front of the near breast, ending above its
            // underside; a string round the ribs.
            let ry = r * 0.9;
            let front = fwd(by) + r * 0.6;
            let (ctop, cbase) = (by - ry * 0.7, by + ry * 0.62);
            cv.begin();
            for y in ctop.floor() as i32..=cbase.ceil() as i32 {
                let fy = y as f32 + 0.5;
                if fy < ctop || fy > cbase {
                    continue;
                }
                let depth = ((fy - ctop) * 0.8 + 1.0).min(r * 1.05);
                for x in 0..FW {
                    if x as f32 + 0.5 >= front - depth {
                        wrap_skin(cv, x, y, &c.skin, cloth);
                    }
                }
            }
            cv.line(v(front - 1.0, ctop), v(cx - 2.5, sy - 1.5), |_| Some(ties[2]));
            let y0 = cbase.floor() as i32;
            for x in (fwd(by) - 2.0 * (fwd(by) - cx)) as i32..(front - r * 1.05) as i32 {
                cv.put(x, y0, ties[2]);
            }
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
pub(super) fn swim_trunks(cv: &mut Canvas, b: &Build, c: &Colors, dir: u8, cloth: &R5, band: &R5, aft: impl Fn(f32) -> f32, fwd: impl Fn(f32) -> f32) {
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

/// Turn fabric back into the skin underneath, keeping its shade step.
pub(super) fn unwrap_to_skin(cv: &mut Canvas, x: i32, y: i32, fabrics: &[R5], skin: &R5) -> bool {
    if !cv.img.inside(x, y) {
        return false;
    }
    let i = (y * cv.img.w + x) as usize;
    let r = cv.rid[i] as usize;
    if r < cv.ramps.len() && fabrics.contains(&cv.ramps[r]) {
        let k = (cv.ridx[i] as usize).clamp(1, 4);
        cv.put(x, y, skin[k]);
        return true;
    }
    false
}

/// The revealing cut: a neckline that shows the top of the bust and the
/// cleavage (deeper and off the shoulders when daring), a bare midriff on
/// daring tops; on men an open collar or shirt. The fabric edge along the
/// neckline gets a darker hem.
#[allow(clippy::too_many_arguments)]
pub(super) fn reveal(cv: &mut Canvas, look: &Look, b: &Build, dir: u8, by: f32, cut: u8, fabrics: &[R5], c: &Colors, width: &impl Fn(f32) -> f32, fwd: &impl Fn(f32) -> f32, crop: bool) {
    let sy = b.shoulder_y;
    let skin = c.skin;
    let daring = cut >= 2;
    let side = dir == DIR_SIDE;
    if !look.female || !b.woman {
        // Men: an open collar, or the shirt open to the sternum.
        let depth = if daring { 9.0 } else { 4.0 };
        for y in sy as i32..(sy + depth) as i32 {
            let t = (y as f32 + 0.5 - sy) / depth;
            if side {
                let x = fwd(y as f32 + 0.5) as i32 - 1;
                for k in 0..2 {
                    unwrap_to_skin(cv, x - k, y, fabrics, &skin);
                }
            } else {
                let half = (if daring { 4.5 } else { 3.5 }) * (1.0 - t) + 0.6;
                for x in 0..FW {
                    if (x as f32 + 0.5 - 32.0).abs() <= half {
                        unwrap_to_skin(cv, x, y, fabrics, &skin);
                    }
                }
            }
        }
        return;
    }
    let r = b.bust.max(3.0);
    let ry = r * 0.9;
    // The neckline: a sweetheart over each breast dipping into a V at the
    // cleavage; daring sits lower and runs off the shoulders.
    let base = if daring { by - ry * 0.15 } else { by - ry * 0.5 };
    let neck = |x: i32| {
        let d = (x as f32 + 0.5 - 32.0).abs();
        base + (if daring { 4.0 } else { 3.0 } - d).max(0.0) * if daring { 1.1 } else { 0.8 }
    };
    let edge = if daring { 0.6 } else { 2.2 };
    for x in 0..FW {
        let ny = if side { base } else { neck(x) };
        let mut last = None;
        for y in (sy - 1.0) as i32..=ny.floor() as i32 {
            let fy = y as f32 + 0.5;
            let px = x as f32 + 0.5;
            let inside = if side { px >= b.cx } else { (px - 32.0).abs() <= width(fy) - edge };
            if inside && unwrap_to_skin(cv, x, y, fabrics, &skin) {
                last = Some(y);
            }
        }
        // Hem the neckline one pixel below the last bared row.
        if let Some(y) = last {
            let i = ((y + 1) * cv.img.w + x) as usize;
            if cv.img.inside(x, y + 1) {
                let rid = cv.rid[i] as usize;
                if rid < cv.ramps.len() && fabrics.contains(&cv.ramps[rid]) {
                    let ramp = cv.ramps[rid];
                    cv.put(x, y + 1, ramp[1]);
                }
            }
        }
    }
    if daring && crop && !side {
        // A crop top: the midriff from under the bust to the waist.
        let (top, low) = ((by + ry + 1.5).floor() as i32, (b.waist_y + 1.0) as i32);
        for y in top..low {
            for x in 0..FW {
                if (x as f32 + 0.5 - 32.0).abs() <= width(y as f32 + 0.5) - 0.5 {
                    unwrap_to_skin(cv, x, y, &fabrics[..1], &skin);
                }
            }
        }
        navel(cv, b, c);
    }
}

/// Recolour a bare-skin pixel in fabric, keeping its shade step, so a cup
/// follows the curve already shaded underneath it.
pub(super) fn wrap_skin(cv: &mut Canvas, x: i32, y: i32, skin: &R5, cloth: &R5) {
    if !cv.img.inside(x, y) {
        return;
    }
    let i = (y * cv.img.w + x) as usize;
    let r = cv.rid[i] as usize;
    if r < cv.ramps.len() && cv.ramps[r] == *skin {
        let k = (cv.ridx[i] as usize).clamp(1, 4);
        cv.fill(x, y, cloth[k]);
    }
}

/// Micro bikini: small triangle cups over the front of each breast (the
/// breast's own shading shows through the fabric), thin strings round the
/// neck and ribs, and a tiny high-cut bottom on hip strings.
#[allow(clippy::too_many_arguments)]
pub(super) fn micro_bikini(cv: &mut Canvas, b: &Build, c: &Colors, dir: u8, by: f32, cloth: &R5, strings: &R5, fwd: impl Fn(f32) -> f32, aft: impl Fn(f32) -> f32) {
    let (cx, sy) = (b.cx, b.shoulder_y);
    let r = b.bust.max(2.6);
    let ry = r * 0.9;
    let (apex_y, base_y) = (by - ry * 0.32, by + ry * 0.45);
    let string = strings[2];
    match dir {
        DIR_DOWN => {
            // Left cup, mirrored; the breast centres sit at cx -/+ r*0.92.
            let bc = cx - r * 0.92;
            let first = cv.cur + 1;
            cv.begin();
            for y in apex_y.floor() as i32..=base_y.ceil() as i32 {
                let fy = y as f32 + 0.5;
                if fy < apex_y || fy > base_y {
                    continue;
                }
                let half = ((fy - apex_y) * 0.8 + 0.5).min(r * 0.42 + 0.4);
                for x in 0..32 {
                    if (x as f32 + 0.5 - bc).abs() <= half {
                        wrap_skin(cv, x, y, &c.skin, cloth);
                    }
                }
            }
            cv.mirror_from(first);
            // Halter strings to the neck, a string under the bust round the
            // ribs, and a little knot between the cups.
            let by0 = base_y.floor();
            cv.line(v(bc, apex_y), v(cx - 3.0, sy - 1.0), |_| Some(string));
            cv.line(v(64.0 - bc, apex_y), v(cx + 3.0, sy - 1.0), |_| Some(string));
            for x in (cx - b.chest_w + 0.5) as i32..(bc - r * 0.42) as i32 {
                cv.put(x, by0 as i32, string);
                cv.put(FW - 1 - x, by0 as i32, string);
            }
            hspan(cv, (r * 0.92 - r * 0.42).max(1.0), by0 as i32, string);
            navel(cv, b, c);
        }
        DIR_UP => {
            // A single string across the back and a bow at the nape.
            hspan(cv, b.chest_w - 0.4, base_y.floor() as i32, string);
            cv.begin();
            for s in [-1.0f32, 1.0] {
                cv.ellipse(v(cx + s * 1.5, sy - 1.5), 1.2, 0.8, strings, 0.05);
            }
        }
        _ => {
            // The cup on the front of the near breast, its string round the
            // ribs to the back.
            let front = fwd(by) + r * 0.6;
            cv.begin();
            for y in apex_y.floor() as i32..=base_y.ceil() as i32 {
                let fy = y as f32 + 0.5;
                if fy < apex_y || fy > base_y {
                    continue;
                }
                let depth = ((fy - apex_y) * 0.7 + 0.8).min(r * 0.55);
                for x in 0..FW {
                    if x as f32 + 0.5 >= front - depth {
                        wrap_skin(cv, x, y, &c.skin, cloth);
                    }
                }
            }
            cv.line(v(front - 1.0, apex_y), v(cx - 2.0, sy - 1.5), |_| Some(string));
            let y0 = base_y.floor() as i32;
            for x in aft(by) as i32..(front - r * 0.7) as i32 {
                cv.put(x, y0, string);
            }
        }
    }
    // Bottoms: a small triangle, low on the hips, on strings.
    let (top, low) = (b.hip_y + 0.5, b.crotch_y + 1.5);
    let span = low - top;
    cv.begin();
    match dir {
        DIR_SIDE => {
            let f0 = fwd(top);
            cover(cv, top, low, |x, fy| {
                let t = (fy - top) / span;
                let px = x as f32 + 0.5;
                (px >= f0 - 2.5 - t * 1.5 && px <= f0 + 0.5).then(|| cloth[if t > 0.7 { 2 } else { 3 }])
            });
            let y0 = top.floor() as i32;
            for x in aft(top) as i32..(f0 - 2.0) as i32 {
                cv.put(x, y0, string);
            }
        }
        _ => {
            // Front: a narrow V; behind: a small cheeky triangle.
            let (w0, w1) = if dir == DIR_UP { (b.hip_w * 0.42, 1.2) } else { (b.hip_w * 0.4, 0.8) };
            cover(cv, top, low, |x, fy| {
                let t = (fy - top) / span;
                let half = w0 + (w1 - w0) * t;
                ((x as f32 + 0.5 - 32.0).abs() <= half).then(|| cloth[if t > 0.7 { 2 } else { 3 }])
            });
            // Hip strings rising to the hip bones.
            let y0 = top.floor();
            cv.line(v(32.0 - w0, y0 + 0.5), v(32.0 - b.hip_w + 0.3, y0 - 1.0), |_| Some(string));
            cv.line(v(32.0 + w0, y0 + 0.5), v(32.0 + b.hip_w - 0.3, y0 - 1.0), |_| Some(string));
        }
    }
}

pub(super) fn navel(cv: &mut Canvas, b: &Build, c: &Colors) {
    let y = (b.waist_y + 1.0) as i32;
    cv.put(31, y, c.skin[2]);
    cv.put(32, y, c.skin[2]);
}

pub(super) fn quiver_back(cv: &mut Canvas, b: &Build, c: &Colors) {
    let (cx, sy) = (b.cx, b.shoulder_y);
    let fletch = [hex(0x6a5a50), hex(0xa89888), hex(0xe0d4c4), hex(0xf8f0e4), hex(0xffffff)];
    cv.begin();
    cv.capsule(v(cx - 4.5, sy - 2.0), v(cx + 4.0, b.waist_y + 1.0), 2.4, 2.2, &c.leather, 0.05);
    cv.begin();
    for k in 0..3 {
        cv.ellipse(v(cx - 6.0 + k as f32 * 1.5, sy - 4.5 + k as f32 * 0.5), 0.9, 1.4, &fletch, 0.1);
    }
}

pub(super) fn belt(cv: &mut Canvas, b: &Build, leather: &R5, lean: impl Fn(f32) -> f32, width: impl Fn(f32) -> f32, buckle: bool) {
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

pub(super) fn sash(cv: &mut Canvas, b: &Build, ramp: &R5, lean: impl Fn(f32) -> f32, width: impl Fn(f32) -> f32) {
    let y = b.waist_y + 1.0;
    let w = width(y) + 0.3;
    let c = lean(y);
    cv.begin();
    cv.profile(y - 1.0, y + 1.5, |_| c, |_| w, ramp, 0.05);
}

/// Skirt start half-width and flare per row. In profile a skirt only has
/// the body's depth to hang from and flares far less.
pub(super) fn skirt_shape(side: bool, w0: f32, k: f32) -> (f32, f32) {
    if side { (6.2, k * 0.5) } else { (w0, k) }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn hem_folds(cv: &mut Canvas, top: f32, bottom: f32, cx: f32, w0: f32, k: f32, ramp: &R5, sway: f32) {
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
pub(super) fn cloak_back(cv: &mut Canvas, b: &Build, ramp: &R5, side: bool, behind: bool) {
    let top = b.shoulder_y - 0.5;
    let bottom = b.ground - 7.0;
    let cx = if side { b.cx - 3.5 } else { b.cx };
    let lag = b.sway;
    let sw = b.shoulder_w;
    cv.begin();
    let folds = if side || behind { 1.5 } else { 3.5 };
    cv.drape(top, bottom, |y| cx - (y - top) / (bottom - top) * lag * 1.5, |y| if side { 5.5 + (y - top) * 0.14 } else if behind { sw + 1.0 + (y - top) * 0.1 } else { sw + 2.0 + (y - top) * 0.2 }, ramp, -0.1, folds);
}
