//! Heads and faces: the face silhouette tables, eyes, ears and facial hair.

use super::*;

/// Front-view face half-width for row `dy` relative to the head centre
/// (-13..=14). Straight cheeks, then a soft U jaw stepping 1,1,2,2 so the
/// curve reads clean at 1x. Above the eyes it is covered by hair anyway.
pub(super) fn face_hw(look: &Look, dy: i32) -> f32 {
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
pub(super) const SIDE_HEAD: [(i32, i32); 28] = [
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

pub(super) fn head(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
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
pub(super) const EAR_FRONT: [&str; 6] = [".LM", "LLM", "LDM", "LDM", ".LM", "..M"];

/// Human ear in profile, facing right: the helix rim at the top and back,
/// the shadowed bowl in the middle, the lobe at the bottom.
pub(super) const EAR_SIDE: [&str; 6] = [
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
pub(super) const EYE_F: [&str; 8] = [
    "aKKKKa.",
    "KKKKKKa",
    "abDDDKK",
    ".WWDDDS",
    ".WDDMMS",
    ".SMMLMS",
    "..MLLW.",
    "...bb..",
];

pub(super) const EYE_M: [&str; 6] = [
    ".aKKKKa",
    "aKKKKKK",
    ".SDDDKK",
    ".SWDMS.",
    "..MLM..",
    "...bb..",
];

pub(super) const EYE_CLOSED: [&str; 8] = [
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
pub(super) const EYE_SIDE: [&str; 8] = [
    "aKKKa",
    "KKKKK",
    ".bDDK",
    ".WDDS",
    ".WDMS",
    ".MMLS",
    "..LM.",
    "..b..",
];

pub(super) const EYE_SIDE_CLOSED: [&str; 8] = [".....", ".....", ".....", ".....", ".....", "KKKKa", ".bb..", "....."];

pub(super) fn face(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors, pose: Pose) {
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
pub(super) fn facial_hair(cv: &mut Canvas, look: &Look, dir: u8, cy: i32, c: &Colors) {
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
