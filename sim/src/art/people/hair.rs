//! Hairstyles: each cut as data plus an attachment kind, and the passes
//! that draw them in every view.

use super::*;

/// What a hairstyle adds on top of the basic cap, bangs, locks and back
/// sheet.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Extra {
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
pub(super) struct Cut {
    /// Bang tips: (x offset from centre, tip y offset from head centre,
    /// slope of the clump's sides). Lower slope = blunter cut.
    pub(super) tips: &'static [(f32, f32, f32)],
    /// The same bangs seen in profile (x offset forward of the head centre).
    pub(super) side_tips: &'static [(f32, f32, f32)],
    /// Side locks framing the face: length below the head centre, width.
    pub(super) lock: (f32, f32),
    /// Length of the back mass below the head centre (front view behind the
    /// body); 0 = short.
    pub(super) back: f32,
    /// How far the hair reaches below the head centre at the nape.
    pub(super) nape: f32,
    /// Hair pulled up or tied: the nape and the hairline show from behind.
    pub(super) gathered: bool,
    /// Locks cut straight across instead of tapering to a point.
    pub(super) blunt: bool,
    /// Depth of the V in the back sheet's hem (0 = straight across).
    pub(super) hem: f32,
    /// Close to the skull (short men's cuts).
    pub(super) tight: bool,
    pub(super) extra: Extra,
}

pub(super) const SIDE_SOFT: &[(f32, f32, f32)] = &[(5.0, 2.0, 1.5), (9.0, 0.5, 1.6), (12.5, 1.5, 1.4)];

pub(super) const SIDE_SHORT: &[(f32, f32, f32)] = &[(5.0, 0.0, 1.6), (8.5, 1.0, 1.5), (12.0, -1.0, 1.3)];

pub(super) fn cut(style: u8) -> Cut {
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
pub(super) fn fringe_bottom(tips: &[(f32, f32, f32)], cy: f32, x: i32) -> f32 {
    let px = x as f32 + 0.5;
    tips.iter().map(|&(tx, ty, s)| cy + ty - s * (px - (32.0 + tx)).abs()).fold(f32::MIN, f32::max)
}

/// Hair mass radii (front view).
pub(super) fn mass_r(look: &Look) -> (f32, f32) {
    if cut(look.hair_style).tight { (14.0, 14.5) } else { (14.5, 15.0) }
}

/// Long-hair width down the back: just under the skull width at the nape,
/// a gentle flare over the shoulders, then a taper toward the tips.
pub(super) fn sheet_w(rx: f32, t: f32) -> f32 {
    rx * (0.93 + 0.1 * (std::f32::consts::PI * (t / 0.35).min(1.0)).sin() * (1.0 - t * 0.3)) * (1.0 - 0.28 * t * t)
}

pub(super) fn back_hair(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
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
pub(super) fn hair_sheet(cv: &mut Canvas, h: &R5, top: f32, end: f32, lit: bool, vdepth: f32, cx: impl Fn(f32) -> f32, hw: impl Fn(f32) -> f32) {
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

/// Flow lines over a hair mass: from the crown, over the curve of the
/// skull, then straight down to `bottom`. Mirrored pairs at each offset.
pub(super) fn flow_lines(cv: &mut Canvas, h: &R5, cx: f32, crown_y: f32, skull_y: f32, bottom: f32, offsets: &[f32]) {
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
pub(super) fn tail_lock(cv: &mut Canvas, h: &R5, a: V2, b: V2, r: f32) {
    lock_segment(cv, h, a, b, r, true);
}

/// Point and unit direction at arc length `s` along a polyline.
pub(super) fn along(pts: &[V2], s: f32) -> (V2, V2) {
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

pub(super) fn path_len(pts: &[V2]) -> f32 {
    pts.windows(2).map(|w| ((w[1].x - w[0].x).powi(2) + (w[1].y - w[0].y).powi(2)).sqrt()).sum()
}

/// A three-strand braid along a path: round lobes stepping left and right,
/// each its own part so the contour pass draws the tuck where it overlaps
/// the next; a tie and a small brush tuft at the end.
pub(super) fn braid(cv: &mut Canvas, h: &R5, pts: &[V2], r0: f32, r1: f32, tie: &R5, flip: f32) {
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
pub(super) fn drill(cv: &mut Canvas, h: &R5, top: V2, len: f32, r: f32, lean: f32) {
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
pub(super) fn ahoge(cv: &mut Canvas, look: &Look, h: &R5, root: V2, dir: f32) {
    if look.covers_crown() {
        return;
    }
    cv.begin();
    let pts = [root, v(root.x + dir * 1.0, root.y - 4.0), v(root.x + dir * 3.5, root.y - 6.5), v(root.x + dir * 5.5, root.y - 5.0)];
    curve(cv, &pts, 1.3, 0.5, h);
}

/// One segment of a lock; `taper` false keeps it full (for joining a curve).
pub(super) fn lock_segment(cv: &mut Canvas, h: &R5, a: V2, b: V2, r: f32, taper: bool) {
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

pub(super) fn hair_front(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
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
                // Long sheets get more strands so they don't read as a slab.
                let offsets: &[f32] = if k.back > 20.0 { &[2.5, 6.0, 10.0] } else { &[4.0, 9.5] };
                flow_lines(cv, h, 32.0, cy - 12.0, cy + 4.0, bottom, offsets);
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
                    ahoge(cv, look, h, v(31.5, cy - 15.0), -1.0);
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
                    ahoge(cv, look, h, v(31.5, cy - 15.5), 1.0);
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
                Extra::Messy => ahoge(cv, look, h, v(32.5, cy - 15.0), 1.0),
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
pub(super) fn lock_strip(cv: &mut Canvas, h: &R5, cx: f32, top: f32, end: f32, w: f32, sway: f32, s: f32, blunt: bool) {
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
pub(super) fn highlight_ring(cv: &mut Canvas, h: &R5, shine: Rgb, mc: V2, rx: f32, ry: f32, skip: &[i32]) {
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
pub(super) fn spikes(cv: &mut Canvas, h: &R5, cy: f32, dir: u8) {
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
