//! The drawing surface for one frame: parts painted under ids, flat-band
//! lighting, pixel stamps, and the contour, outline and pinhole passes.

use super::*;

/// Inside-test tolerance, so shapes mirrored across the centre line include
/// exactly the same boundary pixels despite float rounding.
pub(super) const EPS: f32 = 1e-3;

/// Colour the contour pass pulls overlapped edges toward.
pub(super) const CONTOUR: Rgb = hex(0x2a1430);

// ---------------------------------------------------------------- geometry

#[derive(Clone, Copy, Default)]
pub(super) struct V2 {
    pub(super) x: f32,
    pub(super) y: f32,
}

pub(super) fn v(x: f32, y: f32) -> V2 {
    V2 { x, y }
}

impl V2 {
    pub(super) fn add(self, o: V2) -> V2 {
        v(self.x + o.x, self.y + o.y)
    }
    /// Angle 0 points straight down; positive swings toward +x.
    pub(super) fn polar(len: f32, ang: f32) -> V2 {
        v(ang.sin() * len, ang.cos() * len)
    }
}

/// Light comes from the front and above, so symmetric shapes shade
/// symmetrically.
pub(super) const LY: f32 = -0.55;

pub(super) const LZ: f32 = 0.83;

/// Map a surface normal to a ramp index (0..=4) in clean bands: most of a
/// form sits on the base tone, edges step down, a small cap steps up.
pub(super) fn tone(ny: f32, nz: f32, bias: f32) -> usize {
    let d = ny * LY + nz * LZ;
    ((0.30 + d * 0.52 + bias).clamp(0.0, 0.999) * 5.0) as usize
}

pub(super) struct Canvas<'a> {
    pub(super) img: &'a mut Img,
    /// Owning part per pixel (0 = none). Later parts have higher ids.
    pub(super) ids: Vec<u8>,
    pub(super) cur: u8,
    /// Ramps in use, and per pixel which ramp/step its colour came from
    /// (255 = a one-off detail colour). Contours and outlines step down the
    /// pixel's own ramp, so the palette stays fixed and hue-matched.
    pub(super) ramps: Vec<R5>,
    pub(super) rid: Vec<u8>,
    pub(super) ridx: Vec<u8>,
    /// Mirror group per part id (0 none, 1 left, 2 right): where a left
    /// part meets a right one (legs, shoes, bust) the seam is shaded on both
    /// sides so front views stay symmetric.
    pub(super) groups: Vec<u8>,
    pub(super) side: u8,
    /// Optional column range a part may paint (keeps paired limbs on their
    /// own side of the centre line).
    pub(super) clip: Option<(i32, i32)>,
    /// Direct-mapped cache of colour -> (ramp, step) lookups for `mark`;
    /// cleared whenever a ramp is registered.
    pub(super) marks: [Option<(Rgb, u8, u8)>; 64],
}

impl Canvas<'_> {
    /// Start a new part: its edges get contour lines where it overlaps
    /// anything drawn before.
    pub(super) fn begin(&mut self) {
        self.cur = self.cur.saturating_add(1);
        self.groups[self.cur as usize] = self.side;
    }

    pub(super) fn new(img: &mut Img) -> Canvas<'_> {
        let n = (img.w * img.h) as usize;
        Canvas { img, ids: vec![0; n], cur: 0, ramps: Vec::new(), rid: vec![255; n], ridx: vec![0; n], groups: vec![0; 256], side: 0, clip: None, marks: [None; 64] }
    }

    pub(super) fn register(&mut self, r: &R5) {
        if !self.ramps.contains(r) && self.ramps.len() < 250 {
            self.ramps.push(*r);
            self.marks = [None; 64];
        }
    }

    /// Record which ramp (first registered match) and step a colour is.
    pub(super) fn mark(&mut self, i: usize, c: Rgb) {
        let slot = (c[0] as usize * 7 + c[1] as usize * 13 + c[2] as usize * 31) & 63;
        let (r, j) = match self.marks[slot] {
            Some((k, r, j)) if k == c => (r, j),
            _ => {
                let found = self.ramps.iter().enumerate().find_map(|(k, r)| r.iter().position(|&q| q == c).map(|j| (k as u8, j as u8)));
                let (r, j) = found.unwrap_or((255, 0));
                self.marks[slot] = Some((c, r, j));
                (r, j)
            }
        };
        self.rid[i] = r;
        self.ridx[i] = j;
    }

    /// Detail pixel: recolours without changing which part owns it.
    pub(super) fn put(&mut self, x: i32, y: i32, c: Rgb) {
        if self.img.inside(x, y) {
            self.img.set(x, y, c);
            self.mark((y * self.img.w + x) as usize, c);
        }
    }

    /// Body pixel owned by the current part.
    pub(super) fn fill(&mut self, x: i32, y: i32, c: Rgb) {
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

    pub(super) fn get(&self, x: i32, y: i32) -> Option<Rgb> {
        self.img.get(x, y)
    }

    /// Step a pixel one shade darker if it was painted from `ramp`.
    pub(super) fn darken_on(&mut self, x: i32, y: i32, ramp: &R5) {
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

    /// Step a pixel one shade lighter if it was painted from `ramp`.
    pub(super) fn lighten_on(&mut self, x: i32, y: i32, ramp: &R5) {
        if !self.img.inside(x, y) {
            return;
        }
        let i = (y * self.img.w + x) as usize;
        let r = self.rid[i] as usize;
        if r < self.ramps.len() && self.ramps[r] == *ramp && self.ridx[i] < 4 {
            let c = ramp[self.ridx[i] as usize + 1];
            self.put(x, y, c);
        }
    }

    /// Whether the current part painted this pixel.
    pub(super) fn owns(&self, x: i32, y: i32) -> bool {
        self.img.inside(x, y) && self.ids[(y * self.img.w + x) as usize] == self.cur
    }

    /// Copy every part from `first` to the current one across the centre
    /// line as new parts, so paired pieces (drills, braids) mirror exactly.
    pub(super) fn mirror_from(&mut self, first: u8) {
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
    pub(super) fn ellipse(&mut self, c: V2, rx: f32, ry: f32, ramp: &R5, bias: f32) {
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
    pub(super) fn capsule(&mut self, a: V2, b: V2, ra: f32, rb: f32, ramp: &R5, bias: f32) {
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
    pub(super) fn profile(&mut self, y0: f32, y1: f32, cx: impl Fn(f32) -> f32, hw: impl Fn(f32) -> f32, ramp: &R5, bias: f32) {
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

    /// Hanging cloth (capes, gowns, skirts): like `profile`, but with
    /// `folds` soft vertical folds that deepen toward the hem (a shadowed
    /// valley and a lit ridge per fold, mirrored about the centre) and a
    /// gently scalloped hem instead of a ruler-straight one.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn drape(&mut self, y0: f32, y1: f32, cx: impl Fn(f32) -> f32, hw: impl Fn(f32) -> f32, ramp: &R5, bias: f32, folds: f32) {
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
            let t = (fy - y0) / (y1 - y0);
            for x in (c - w).floor() as i32..=(c + w).ceil() as i32 {
                let dx = x as f32 + 0.5 - c;
                let nx = dx / w;
                if nx.abs() > 1.0 + EPS {
                    continue;
                }
                // Scalloped hem: each fold's valley lifts the hem a little.
                let u = dx.abs() / w * folds * std::f32::consts::PI;
                if fy > y1 - (1.0 - u.cos()) * 0.75 {
                    continue;
                }
                let nz = (1.0 - nx * nx).max(0.0).sqrt();
                let ny = (t - 0.5) * 0.4;
                let mut k = tone(ny, nz, bias) as i32;
                // Folds fade in below the top quarter.
                if t > 0.22 && folds > 0.0 {
                    let cu = u.cos();
                    if cu < -0.72 {
                        k -= 1;
                    } else if cu > 0.9 && t > 0.4 {
                        k += 1;
                    }
                }
                self.fill(x, y, ramp[k.clamp(0, 4) as usize]);
            }
        }
    }

    pub(super) fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Rgb) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.put(xx, yy, c);
            }
        }
    }

    /// Recolour a pixel only where something is already drawn.
    pub(super) fn tint(&mut self, x: i32, y: i32, c: Rgb) {
        if self.img.alpha(x, y) != 0 {
            self.put(x, y, c);
        }
    }

    /// Pixel line (DDA) drawn as detail.
    pub(super) fn line(&mut self, a: V2, b: V2, mut col: impl FnMut(f32) -> Option<Rgb>) {
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
pub(super) fn keyed(keys: &[(f32, f32)], y: f32) -> f32 {
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
pub(super) fn hspan(cv: &mut Canvas, half: f32, y: i32, c: Rgb) {
    let x0 = (32.0 - half).round() as i32;
    for x in x0..FW - x0 {
        cv.put(x, y, c);
    }
}

/// An authored stamp painted as part of the current body part (so it joins
/// the contour pass), mirrored across the centre line when asked.
pub(super) fn stamp_fill(cv: &mut Canvas, x0: i32, y0: i32, rows: &[&str], mirror: bool, pal: &impl Fn(char) -> Option<Rgb>) {
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
pub(super) fn stamp_at(cv: &mut Canvas, x0: i32, y0: i32, rows: &[&str], flip: bool, pal: &impl Fn(char) -> Option<Rgb>) {
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
pub(super) fn stamp(cv: &mut Canvas, x0: i32, y0: i32, rows: &[&str], mirror: bool, pal: &impl Fn(char) -> Option<Rgb>) {
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

/// Per-pixel ramp provenance, for picking contour and outline shades.
pub(super) struct Shades<'a> {
    pub(super) ramps: &'a [R5],
    pub(super) rid: &'a [u8],
    pub(super) ridx: &'a [u8],
}

impl Shades<'_> {
    /// Contour shade: `steps` down the pixel's own ramp.
    pub(super) fn contour(&self, i: usize, c: Rgb, steps: u8) -> Rgb {
        match self.ramps.get(self.rid[i] as usize) {
            Some(r) if self.ridx[i] >= steps => r[(self.ridx[i] - steps) as usize],
            Some(r) => mix(r[0], CONTOUR, 0.35),
            None => mix(c, CONTOUR, 0.5),
        }
    }

    /// Outline shade: the ramp's darkest step pulled toward ink.
    pub(super) fn outline(&self, i: usize, c: Rgb) -> Rgb {
        match self.ramps.get(self.rid[i] as usize) {
            Some(r) => mix(r[0], pal::INK, 0.55),
            None => mix(c, pal::INK, 0.72),
        }
    }
}

/// Close tiny enclosed gaps (slivers between outlines) with the darkest
/// neighbouring colour; they read as noise rather than negative space.
/// The sprite's opaque bounding box grown by one pixel (for outlines),
/// clamped to the frame: (x0, y0, x1, y1) inclusive.
pub(super) fn padded_bounds(img: &Img) -> (i32, i32, i32, i32) {
    let (mut x0, mut y0, mut x1, mut y1) = (img.w, img.h, -1, -1);
    for y in 0..img.h {
        for x in 0..img.w {
            if img.alpha(x, y) != 0 {
                x0 = x0.min(x);
                x1 = x1.max(x);
                y0 = y0.min(y);
                y1 = y1.max(y);
            }
        }
    }
    if x1 < 0 {
        return (0, 0, -1, -1);
    }
    ((x0 - 1).max(0), (y0 - 1).max(0), (x1 + 1).min(img.w - 1), (y1 + 1).min(img.h - 1))
}

/// Fill small enclosed gaps (8px or less) with their darkest neighbour.
/// Only the sprite's padded box is searched: everything beyond it is open
/// background.
pub(super) fn close_pinholes(img: &mut Img, (x0, y0, x1, y1): (i32, i32, i32, i32)) {
    if x1 < x0 {
        return;
    }
    let (bw, bh) = (x1 - x0 + 1, y1 - y0 + 1);
    let at = |x: i32, y: i32| ((y - y0) * bw + (x - x0)) as usize;
    let inbox = |x: i32, y: i32| x >= x0 && x <= x1 && y >= y0 && y <= y1;
    let clear = |img: &Img, x: i32, y: i32| img.alpha(x, y) == 0;
    // Open background: flood from the box border (marking on push).
    let mut outside = vec![false; (bw * bh) as usize];
    let mut stack: Vec<(i32, i32)> = Vec::with_capacity((bw * bh) as usize / 2);
    let seed = |x: i32, y: i32, outside: &mut Vec<bool>, stack: &mut Vec<(i32, i32)>| {
        if clear(img, x, y) && !outside[at(x, y)] {
            outside[at(x, y)] = true;
            stack.push((x, y));
        }
    };
    for x in x0..=x1 {
        seed(x, y0, &mut outside, &mut stack);
        seed(x, y1, &mut outside, &mut stack);
    }
    for y in y0..=y1 {
        seed(x0, y, &mut outside, &mut stack);
        seed(x1, y, &mut outside, &mut stack);
    }
    while let Some((x, y)) = stack.pop() {
        for (nx, ny) in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
            if inbox(nx, ny) && clear(img, nx, ny) && !outside[at(nx, ny)] {
                outside[at(nx, ny)] = true;
                stack.push((nx, ny));
            }
        }
    }
    let mut seen = vec![false; (bw * bh) as usize];
    for y in y0..=y1 {
        for x in x0..=x1 {
            let i = at(x, y);
            if outside[i] || seen[i] || !clear(img, x, y) {
                continue;
            }
            let mut region = Vec::new();
            let mut st = vec![(x, y)];
            seen[i] = true;
            while let Some((cx, cy)) = st.pop() {
                region.push((cx, cy));
                for (nx, ny) in [(cx + 1, cy), (cx - 1, cy), (cx, cy + 1), (cx, cy - 1)] {
                    if inbox(nx, ny) && clear(img, nx, ny) && !seen[at(nx, ny)] {
                        seen[at(nx, ny)] = true;
                        st.push((nx, ny));
                    }
                }
            }
            if region.len() > 8 {
                continue;
            }
            // Fill inward in layers (each layer's colours are all chosen
            // before any is written), so the result doesn't depend on
            // visiting order and mirrored gaps stay identical.
            let mut left = region;
            while !left.is_empty() {
                let mut next = Vec::new();
                let mut fills = Vec::new();
                for &(px, py) in &left {
                    let dark = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                        .iter()
                        .filter_map(|&(dx, dy)| img.get(px + dx, py + dy))
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
pub(super) fn contours(img: &mut Img, ids: &[u8], groups: &[u8], sh: &Shades, (x0, y0, x1, y1): (i32, i32, i32, i32)) {
    let src = img.clone();
    let w = img.w;
    for y in y0..=y1 {
        for x in x0..=x1 {
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
pub(super) fn outline(img: &mut Img, sh: &Shades, (x0, y0, x1, y1): (i32, i32, i32, i32)) {
    let src = img.clone();
    for y in y0..=y1 {
        for x in x0..=x1 {
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
