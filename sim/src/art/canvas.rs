//! A tiny RGBA pixel canvas with the drawing primitives the sprite
//! generators need. Every write is clipped, so generators can draw freely
//! past the edges without bounds checks of their own.

use crate::palette::Rgb;

/// The four edge neighbours, in the order outlines probe them.
const NEIGHBOURS: [(i32, i32); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];

#[derive(Clone)]
pub struct Img {
    pub w: i32,
    pub h: i32,
    /// Row-major RGBA bytes, ready to upload as a texture.
    pub px: Vec<u8>,
}

impl Img {
    pub fn new(w: i32, h: i32) -> Img {
        Img { w, h, px: vec![0; (w * h * 4) as usize] }
    }

    /// Byte offset of a pixel. The caller must check `inside` first.
    #[inline]
    fn offset(&self, x: i32, y: i32) -> usize {
        ((y * self.w + x) * 4) as usize
    }

    #[inline]
    pub fn inside(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.w && y < self.h
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, c: Rgb) {
        self.set_a(x, y, c, 255);
    }

    #[inline]
    pub fn set_a(&mut self, x: i32, y: i32, c: Rgb, a: u8) {
        if self.inside(x, y) {
            let i = self.offset(x, y);
            self.px[i..i + 4].copy_from_slice(&[c[0], c[1], c[2], a]);
        }
    }

    /// Colour of an opaque pixel; `None` if transparent or off the canvas.
    #[inline]
    pub fn get(&self, x: i32, y: i32) -> Option<Rgb> {
        if !self.inside(x, y) {
            return None;
        }
        let i = self.offset(x, y);
        (self.px[i + 3] != 0).then(|| [self.px[i], self.px[i + 1], self.px[i + 2]])
    }

    /// Alpha of a pixel, 0 off the canvas.
    #[inline]
    pub fn alpha(&self, x: i32, y: i32) -> u8 {
        if !self.inside(x, y) {
            return 0;
        }
        self.px[self.offset(x, y) + 3]
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Rgb) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, c);
            }
        }
    }

    /// Horizontal run, inclusive of both ends in either order.
    pub fn hline(&mut self, x0: i32, x1: i32, y: i32, c: Rgb) {
        for x in x0.min(x1)..=x0.max(x1) {
            self.set(x, y, c);
        }
    }

    /// Vertical run, inclusive of both ends in either order.
    pub fn vline(&mut self, x: i32, y0: i32, y1: i32, c: Rgb) {
        for y in y0.min(y1)..=y0.max(y1) {
            self.set(x, y, c);
        }
    }

    /// Bresenham line, inclusive of both end points.
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: Rgb) {
        let (mut x, mut y) = (x0, y0);
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.set(x, y, c);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Filled ellipse, calling `f(x, y, nx, ny)` for each covered pixel
    /// with the normalised offset from the centre (for shading).
    pub fn ellipse_with(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, mut f: impl FnMut(&mut Img, i32, i32, f32, f32)) {
        let x0 = (cx - rx).floor() as i32;
        let x1 = (cx + rx).ceil() as i32;
        let y0 = (cy - ry).floor() as i32;
        let y1 = (cy + ry).ceil() as i32;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let nx = (x as f32 + 0.5 - cx) / rx;
                let ny = (y as f32 + 0.5 - cy) / ry;
                if nx * nx + ny * ny <= 1.0 {
                    f(self, x, y, nx, ny);
                }
            }
        }
    }

    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, c: Rgb) {
        self.ellipse_with(cx, cy, rx, ry, |img, x, y, _, _| img.set(x, y, c));
    }

    /// Selective outline: every transparent pixel touching an opaque one
    /// becomes a dark, hue-matched version of that neighbour. Lighter on the
    /// top-left (lit) side, darker on the bottom-right.
    pub fn outline(&mut self, ink: Rgb) {
        let src = self.clone();
        for y in 0..self.h {
            for x in 0..self.w {
                if src.alpha(x, y) != 0 {
                    continue;
                }
                let touching = NEIGHBOURS.iter().find_map(|&(dx, dy)| src.get(x + dx, y + dy).map(|c| (c, dx, dy)));
                if let Some((c, dx, dy)) = touching {
                    // A neighbour below or to the right puts us on the
                    // shape's lit top-left edge.
                    let lit = dx > 0 || dy > 0;
                    let amt = if lit { 0.62 } else { 0.8 };
                    self.set(x, y, crate::palette::mix(c, ink, amt));
                }
            }
        }
    }

    /// Hard outline in a single colour.
    pub fn outline_solid(&mut self, c: Rgb) {
        let src = self.clone();
        for y in 0..self.h {
            for x in 0..self.w {
                if src.alpha(x, y) == 0 && NEIGHBOURS.iter().any(|&(dx, dy)| src.alpha(x + dx, y + dy) != 0) {
                    self.set(x, y, c);
                }
            }
        }
    }

    /// Copy `src` onto self at (dx, dy), skipping transparent pixels.
    pub fn blit(&mut self, src: &Img, dx: i32, dy: i32, flip: bool) {
        for y in 0..src.h {
            for x in 0..src.w {
                let sx = if flip { src.w - 1 - x } else { x };
                let i = src.offset(sx, y);
                let a = src.px[i + 3];
                if a != 0 {
                    self.set_a(dx + x, dy + y, [src.px[i], src.px[i + 1], src.px[i + 2]], a);
                }
            }
        }
    }

    /// Mirror image, used for left-facing frames.
    pub fn flipped(&self) -> Img {
        let mut o = Img::new(self.w, self.h);
        o.blit(self, 0, 0, true);
        o
    }
}

