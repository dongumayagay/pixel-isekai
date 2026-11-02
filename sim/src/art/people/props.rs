//! What characters carry and wear on the head: held tools, hats, animal
//! and elf ears, horns and tails.

use super::*;

pub(super) fn held_item(cv: &mut Canvas, look: &Look, hand: V2, dir: u8, raised: bool, c: &Colors) {
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

/// Elf ear, front view: long, angled up and out, inner fold shaded.
pub(super) const ELF_EAR: [&str; 9] = [
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
pub(super) const ELF_EAR_SIDE: [&str; 9] = [
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
pub(super) const CAT_EAR: [&str; 12] = [
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
pub(super) const FOX_EAR: [&str; 12] = [
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
pub(super) const WOLF_EAR: [&str; 11] = [
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
pub(super) const RABBIT_EAR: [&str; 17] = [
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
pub(super) const HORN: [&str; 12] = [
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
pub(super) fn beast_ears(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
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

pub(super) fn horns(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, _c: &Colors) {
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
pub(super) fn elf_ears(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors, after_hair: bool) {
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
pub(super) fn curve(cv: &mut Canvas, pts: &[V2], r0: f32, r1: f32, ramp: &R5) {
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
pub(super) const SPADE: [&str; 5] = [".LL..", "LMMLL", "MMMMH", "DMMLL", ".DD.."];

/// Tails grow from the small of the back. Side view: curving out behind.
/// Back view: drawn over the body (it faces the camera). Front view: only
/// the tip peeks out to one side from behind.
pub(super) fn tail(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
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

pub(super) fn hat(cv: &mut Canvas, look: &Look, dir: u8, b: &Build, c: &Colors) {
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
            let bottom = b.shoulder_y + 2.0;
            let cape_w = (b.shoulder_w * 0.75).max(hr);
            for y in (cc.y - vr - 1.0) as i32..bottom as i32 {
                let fy = y as f32 + 0.5;
                let ny = (fy - cc.y) / vr;
                // Round over the crown, then falling in a cone onto the shoulders.
                let hw = if ny < 0.0 { hr * (1.0 - ny * ny).max(0.0).sqrt() } else { hr + (cape_w - hr) * ((fy - cc.y) / (bottom - cc.y)).powi(2) };
                for x in (cc.x - hw).floor() as i32..=(cc.x + hw).ceil() as i32 {
                    let nx = (x as f32 + 0.5 - cc.x) / hw;
                    if nx.abs() > 1.0 {
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
                    let nz = (1.0 - nx * nx).max(0.0).sqrt();
                    cv.fill(x, y, hcol[if rim { 1 } else { tone(ny.clamp(-0.4, 1.0), nz, -0.08) }]);
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
