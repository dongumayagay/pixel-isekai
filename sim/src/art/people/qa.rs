//! Visual QA: automated checks for defects a viewer would notice, run by
//! tests and the roster loop.

use super::*;

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
/// Thresholds for the visual checks, calibrated on the cast and thousands of
/// random looks so they flag real outliers.
/// Check kinds that are reported but don't fail QA.
pub const ADVISORY: [&str; 3] = ["palette", "nubs", "boxy"];

pub(super) const NUB_LIMIT: usize = 9;

pub(super) const FLAT_LIMIT: usize = 1100;

pub(super) const BOXY_LIMIT: i32 = 40;

pub(super) const POP_LIMIT: usize = 260;

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
            // A big separate island: a prop or piece floating off the body.
            if let Some(&s2) = sizes.get(1).filter(|&&s| s >= 24) {
                push("detached", dir, f, format!("separate part of {s2} px"));
            }
            // Nubs: single pixels jutting out of the silhouette (one
            // opaque neighbour), beyond the few legitimate pointed tips.
            let nubs = (0..h)
                .flat_map(|y| (0..w).map(move |x| (x, y)))
                .filter(|&(x, y)| a(x, y) && [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().filter(|(dx, dy)| a(x + dx, y + dy)).count() == 1)
                .count();
            if nubs > NUB_LIMIT {
                push("nubs", dir, f, format!("{nubs} one-pixel spurs"));
            }
            // Flat panels: a large patch of one exact colour reads as an
            // unshaded cut-out (big white aprons, slab capes).
            let mut region = vec![false; (w * h) as usize];
            let mut flat = 0;
            for y in 0..h {
                for x in 0..w {
                    let i = (y * w + x) as usize;
                    if region[i] || !a(x, y) {
                        continue;
                    }
                    let col = img.get(x, y);
                    let mut stack = vec![(x, y)];
                    region[i] = true;
                    let mut n = 0;
                    while let Some((cx, cy)) = stack.pop() {
                        n += 1;
                        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                            let (nx, ny) = (cx + dx, cy + dy);
                            if img.inside(nx, ny) && !region[(ny * w + nx) as usize] && img.get(nx, ny) == col {
                                region[(ny * w + nx) as usize] = true;
                                stack.push((nx, ny));
                            }
                        }
                    }
                    flat = flat.max(n);
                }
            }
            if flat > FLAT_LIMIT {
                push("flat", dir, f, format!("unshaded patch of {flat} px"));
            }
            // Boxy: a silhouette side running ruler-straight for many rows.
            let mut run = (0, 0, i32::MIN, i32::MIN);
            let mut longest = 0;
            for y in 0..h {
                let l = (0..w).find(|&x| a(x, y)).unwrap_or(i32::MIN);
                let r = (0..w).rev().find(|&x| a(x, y)).unwrap_or(i32::MIN);
                run.0 = if l != i32::MIN && l == run.2 { run.0 + 1 } else { 1 };
                run.1 = if r != i32::MIN && r == run.3 { run.1 + 1 } else { 1 };
                run.2 = l;
                run.3 = r;
                longest = longest.max(run.0).max(run.1);
            }
            if longest > BOXY_LIMIT {
                push("boxy", dir, f, format!("straight side {longest} rows"));
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
                            if img.get(x, y).is_some_and(|c| iris.iter().any(|&k| hex(k) == c)) {
                                n += 1;
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
        if let (Some(mn), Some(mx)) = (tops.iter().min(), tops.iter().max())
            && mx - mn > 2
        {
            push("jitter", dir, 0, format!("head top varies {mn}..{mx}"));
        }
        // Popping: neighbouring idle frames should differ by a breath (the
        // upper body rising a pixel), not a redraw. Compare after allowing
        // that one-pixel shift, row by row.
        for f in 0..IDLE_FRAMES {
            let (p, q) = (frame(look, dir, f), frame(look, dir, (f + 1) % IDLE_FRAMES));
            let row_diff = |y: i32, dy: i32| (0..p.w).filter(|&x| p.get(x, y) != q.get(x, y + dy)).count();
            let diff: usize = (0..p.h).map(|y| (-1..=1).map(|dy| row_diff(y, dy)).min().unwrap_or(0)).sum();
            if diff > POP_LIMIT {
                push("pop", dir, f, format!("{diff} px change to the next idle frame"));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The core roster is held to the full bar: every frame, every view.
    #[test]
    pub(super) fn core_roster_passes_visual_qa() {
        let mut failures = Vec::new();
        for (name, look) in cast() {
            let issues: Vec<_> = lint(&look).into_iter().filter(|i| !ADVISORY.contains(&i.kind)).collect();
            if !issues.is_empty() {
                failures.push(format!("{name}: {:?}", &issues[..issues.len().min(3)]));
            }
        }
        assert!(failures.is_empty(), "core roster failed QA:\n{}", failures.join("\n"));
    }

    /// A light smoke test over random combinations.
    #[test]
    pub(super) fn random_characters_pass_visual_qa() {
        let mut rng = Rng::new(77);
        let mut failures = Vec::new();
        for i in 0..40 {
            let look = Look::random_any(&mut rng);
            // Palette size, nubs and straight sides are advisory (props and
            // pointed tips trip them legitimately); everything else is a defect.
            let issues: Vec<_> = lint(&look).into_iter().filter(|i| !ADVISORY.contains(&i.kind)).collect();
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
    pub(super) fn beards_only_on_men_and_figures_only_on_women() {
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
    pub(super) fn bust_motion_steps_are_small() {
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
