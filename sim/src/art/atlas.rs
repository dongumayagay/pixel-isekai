//! Texture atlas: shelf-packs generated sprites into 2048² pages and keeps a
//! registry of rects and pivots that the renderer reads.

use serde::Serialize;
use std::collections::HashMap;

use super::canvas::Img;

pub const PAGE: i32 = 2048;

/// Transparent gap between packed sprites, so texture sampling at a
/// sprite's edge never picks up its neighbour.
const PAD: i32 = 1;

#[derive(Clone, Copy, Serialize, Debug)]
pub struct SpriteRect {
    pub page: u8,
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
    /// Pivot (feet/base point) relative to the sprite's top-left.
    pub px: i16,
    pub py: i16,
}

/// A row of the page being filled left to right.
struct Shelf {
    x: i32,
    y: i32,
    h: i32,
}

pub struct Atlas {
    pub pages: Vec<Img>,
    pub sprites: Vec<SpriteRect>,
    names: HashMap<String, u16>,
    shelves: Vec<Vec<Shelf>>,
    next_y: Vec<i32>,
    /// Rects changed since the last upload, per page: (x, y, w, h).
    pub dirty: Vec<Vec<(i32, i32, i32, i32)>>,
}

impl Atlas {
    pub fn new(pages: usize) -> Atlas {
        Atlas {
            pages: (0..pages).map(|_| Img::new(PAGE, PAGE)).collect(),
            sprites: Vec::new(),
            names: HashMap::new(),
            shelves: (0..pages).map(|_| Vec::new()).collect(),
            next_y: vec![0; pages],
            dirty: vec![Vec::new(); pages],
        }
    }

    /// First shelf with room wins; otherwise a new shelf as tall as this
    /// sprite opens below the last one.
    fn alloc(&mut self, page: usize, w: i32, h: i32) -> Option<(i32, i32)> {
        let (w, h) = (w + PAD, h + PAD);
        if let Some(s) = self.shelves[page].iter_mut().find(|s| h <= s.h && s.x + w <= PAGE) {
            let pos = (s.x, s.y);
            s.x += w;
            return Some(pos);
        }
        let y = self.next_y[page];
        if y + h > PAGE {
            return None;
        }
        self.shelves[page].push(Shelf { x: w, y, h });
        self.next_y[page] = y + h;
        Some((0, y))
    }

    /// Add a named sprite. Returns its id. Panics if the page is full (a
    /// build-time bug, not a runtime condition, for static pages).
    pub fn add(&mut self, page: usize, name: &str, img: &Img, pivot: (i32, i32)) -> u16 {
        let id = self.try_add(page, img, pivot).unwrap_or_else(|| panic!("atlas page {page} full adding {name}"));
        if !name.is_empty() {
            self.names.insert(name.to_string(), id);
        }
        id
    }

    /// Add an anonymous sprite, or `None` if the page has no room.
    pub fn try_add(&mut self, page: usize, img: &Img, pivot: (i32, i32)) -> Option<u16> {
        let (x, y) = self.alloc(page, img.w, img.h)?;
        self.pages[page].blit(img, x, y, false);
        self.dirty[page].push((x, y, img.w, img.h));
        let id = self.sprites.len() as u16;
        self.sprites.push(SpriteRect { page: page as u8, x: x as u16, y: y as u16, w: img.w as u16, h: img.h as u16, px: pivot.0 as i16, py: pivot.1 as i16 });
        Some(id)
    }

    pub fn id(&self, name: &str) -> u16 {
        self.get(name).unwrap_or_else(|| panic!("missing sprite {name}"))
    }

    pub fn get(&self, name: &str) -> Option<u16> {
        self.names.get(name).copied()
    }
}
