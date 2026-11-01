//! The world grid: terrain, elevation, flags and static props.

pub mod bake;
pub mod worldgen;
pub mod path;
pub mod villages;

use serde::{Deserialize, Serialize};

pub const TILE: i32 = 32;
pub const WORLD_W: i32 = 192;
pub const WORLD_H: i32 = 192;
/// Chunk edge in tiles. Chunks are the unit of terrain baking and streaming.
pub const CHUNK: i32 = 16;
pub const CHUNKS_X: i32 = WORLD_W / CHUNK;
pub const CHUNKS_Y: i32 = WORLD_H / CHUNK;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Terrain {
    Deep = 0,
    Shallow = 1,
    Sand = 2,
    Dirt = 3,
    Grass = 4,
    Rock = 5,
    Snow = 6,
}

impl Terrain {
    pub fn from_u8(v: u8) -> Terrain {
        match v {
            0 => Terrain::Deep,
            1 => Terrain::Shallow,
            2 => Terrain::Sand,
            3 => Terrain::Dirt,
            4 => Terrain::Grass,
            5 => Terrain::Rock,
            _ => Terrain::Snow,
        }
    }
    pub fn is_water(self) -> bool {
        matches!(self, Terrain::Deep | Terrain::Shallow)
    }
}

pub mod flag {
    pub const ROAD: u8 = 1;
    pub const RIVER: u8 = 2;
    pub const STAIRS: u8 = 4;
    pub const BRIDGE: u8 = 8;
    pub const TILLED: u8 = 16;
    pub const PLAZA: u8 = 32;
    pub const NO_PROPS: u8 = 64;
    pub const WATERED: u8 = 128;
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum PropKind {
    Oak = 0,
    Pine,
    Birch,
    FruitTree,
    Bush,
    BerryBush,
    Rock,
    Boulder,
    OreRock,
    Stump,
    TallGrass,
    Reeds,
    Mushroom,
    House,
    Well,
    Lamp,
    Fence,
    Sign,
    Shrine,
    Crate,
    Barrel,
    Haystack,
    Stall,
    Bench,
    Lily,
    Flowers,
    Crop,
    Tent,
    Campfire,
}

impl PropKind {
    /// Whether the prop stops movement on its anchor tile.
    pub fn blocks(self) -> bool {
        !matches!(
            self,
            PropKind::TallGrass
                | PropKind::Reeds
                | PropKind::Mushroom
                | PropKind::Lily
                | PropKind::Flowers
                | PropKind::Crop
        )
    }
    pub fn is_tree(self) -> bool {
        matches!(self, PropKind::Oak | PropKind::Pine | PropKind::Birch | PropKind::FruitTree)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prop {
    pub kind: PropKind,
    /// Anchor tile. For trees and rocks this is the trunk/base tile. For
    /// buildings it's the bottom-left footprint tile.
    pub x: i32,
    pub y: i32,
    pub variant: u8,
    /// Kind-specific state: growth stage, chop damage, lit, etc.
    pub state: u8,
    /// Hit points for harvestable props. 0 means removed.
    pub hp: u8,
    /// Footprint in tiles (buildings are larger than 1x1).
    pub fw: u8,
    pub fh: u8,
    pub alive: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct World {
    pub seed: u64,
    pub w: i32,
    pub h: i32,
    pub terrain: Vec<u8>,
    pub level: Vec<u8>,
    pub flags: Vec<u8>,
    /// Smooth fields kept for baking: height in [0,1], moisture, forest density.
    pub height: Vec<f32>,
    pub moist: Vec<f32>,
    pub forest: Vec<f32>,
    pub props: Vec<Prop>,
    /// Index+1 of the prop blocking each tile (0 = none).
    pub blocker: Vec<u32>,
    pub villages: Vec<villages::Village>,
    pub spawn: (i32, i32),
}

impl World {
    pub fn empty(seed: u64) -> World {
        let n = (WORLD_W * WORLD_H) as usize;
        World {
            seed,
            w: WORLD_W,
            h: WORLD_H,
            terrain: vec![Terrain::Grass as u8; n],
            level: vec![1; n],
            flags: vec![0; n],
            height: vec![0.0; n],
            moist: vec![0.0; n],
            forest: vec![0.0; n],
            props: Vec::new(),
            blocker: vec![0; n],
            villages: Vec::new(),
            spawn: (WORLD_W / 2, WORLD_H / 2),
        }
    }

    #[inline]
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.w && y < self.h
    }

    #[inline]
    pub fn idx(&self, x: i32, y: i32) -> usize {
        (y * self.w + x) as usize
    }

    #[inline]
    pub fn terrain(&self, x: i32, y: i32) -> Terrain {
        let x = x.clamp(0, self.w - 1);
        let y = y.clamp(0, self.h - 1);
        Terrain::from_u8(self.terrain[self.idx(x, y)])
    }

    #[inline]
    pub fn level(&self, x: i32, y: i32) -> u8 {
        let x = x.clamp(0, self.w - 1);
        let y = y.clamp(0, self.h - 1);
        self.level[self.idx(x, y)]
    }

    #[inline]
    pub fn flags(&self, x: i32, y: i32) -> u8 {
        if !self.in_bounds(x, y) {
            return 0;
        }
        self.flags[self.idx(x, y)]
    }

    #[inline]
    pub fn has(&self, x: i32, y: i32, f: u8) -> bool {
        self.flags(x, y) & f != 0
    }

    pub fn set_flag(&mut self, x: i32, y: i32, f: u8, on: bool) {
        if !self.in_bounds(x, y) {
            return;
        }
        let i = self.idx(x, y);
        if on {
            self.flags[i] |= f;
        } else {
            self.flags[i] &= !f;
        }
    }

    /// Which row of a cliff face this tile is: 0 = not a face, 1 = upper
    /// row (directly below the plateau), 2 = lower row. Faces are two tiles
    /// tall so elevation reads clearly next to 24px-tall characters.
    #[inline]
    pub fn face_row(&self, x: i32, y: i32) -> u8 {
        if y <= 0 {
            return 0;
        }
        let l = self.level(x, y);
        if self.level(x, y - 1) > l {
            return 1;
        }
        if y > 1 && self.level(x, y - 1) == l && self.level(x, y - 2) > l {
            return 2;
        }
        0
    }

    /// A cliff face: the visible vertical wall of the higher ground to the north.
    #[inline]
    pub fn is_face(&self, x: i32, y: i32) -> bool {
        self.face_row(x, y) != 0
    }

    /// Whether a tile can be stood on at all (ignoring level transitions).
    pub fn walkable(&self, x: i32, y: i32) -> bool {
        if !self.in_bounds(x, y) {
            return false;
        }
        let i = self.idx(x, y);
        let f = self.flags[i];
        if self.blocker[i] != 0 {
            return false;
        }
        if f & flag::BRIDGE != 0 {
            return true;
        }
        if Terrain::from_u8(self.terrain[i]).is_water() {
            return false;
        }
        if self.is_face(x, y) && f & flag::STAIRS == 0 {
            return false;
        }
        true
    }

    /// Whether an agent can step from tile a to adjacent tile b.
    pub fn can_step(&self, ax: i32, ay: i32, bx: i32, by: i32) -> bool {
        if !self.walkable(bx, by) {
            return false;
        }
        let la = self.level(ax, ay);
        let lb = self.level(bx, by);
        if la == lb {
            if ay == by && (self.is_face(ax, ay) || self.is_face(bx, by)) {
                return false;
            }
            // Diagonal moves must not squeeze between two blocked corners.
            if ax != bx && ay != by {
                return self.walkable(ax, by) && self.walkable(bx, ay)
                    && self.level(ax, by) == la && self.level(bx, ay) == la;
            }
            return true;
        }
        // Level change only straight north/south through the top stair row.
        if ax != bx {
            return false;
        }
        let top_a = self.has(ax, ay, flag::STAIRS) && self.face_row(ax, ay) == 1;
        let top_b = self.has(bx, by, flag::STAIRS) && self.face_row(bx, by) == 1;
        (top_a && by == ay - 1 && lb == la + 1) || (top_b && ay == by - 1 && la == lb + 1)
    }

    pub fn add_prop(&mut self, p: Prop) -> usize {
        let id = self.props.len();
        if p.kind.blocks() {
            if p.kind == PropKind::House || p.kind == PropKind::Stall || p.kind == PropKind::Tent {
                // Footprint grows up (north) from the anchor row.
                for dy in 0..p.fh as i32 {
                    for dx in 0..p.fw as i32 {
                        let (tx, ty) = (p.x + dx, p.y - dy);
                        if self.in_bounds(tx, ty) {
                            let i = self.idx(tx, ty);
                            self.blocker[i] = id as u32 + 1;
                        }
                    }
                }
            } else if self.in_bounds(p.x, p.y) {
                let i = self.idx(p.x, p.y);
                self.blocker[i] = id as u32 + 1;
            }
        }
        self.props.push(p);
        id
    }

    pub fn remove_prop(&mut self, id: usize) {
        let p = &mut self.props[id];
        p.alive = false;
        let (x, y, fw, fh) = (p.x, p.y, p.fw as i32, p.fh as i32);
        for dy in 0..fh.max(1) {
            for dx in 0..fw.max(1) {
                let (tx, ty) = (x + dx, y - dy);
                if self.in_bounds(tx, ty) {
                    let i = self.idx(tx, ty);
                    if self.blocker[i] == id as u32 + 1 {
                        self.blocker[i] = 0;
                    }
                }
            }
        }
    }

    pub fn prop_at(&self, x: i32, y: i32) -> Option<usize> {
        if !self.in_bounds(x, y) {
            return None;
        }
        let b = self.blocker[self.idx(x, y)];
        if b == 0 { None } else { Some(b as usize - 1) }
    }

    /// Tile area free for building or decoration.
    pub fn free_for_prop(&self, x: i32, y: i32) -> bool {
        self.in_bounds(x, y)
            && self.walkable(x, y)
            && !self.is_face(x, y)
            && !self.is_face(x, y + 1)
            && self.flags(x, y) & (flag::ROAD | flag::PLAZA | flag::NO_PROPS | flag::TILLED | flag::STAIRS | flag::BRIDGE) == 0
    }

    /// Stable checksum of the generated world, used by determinism tests.
    pub fn checksum(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |b: u8| {
            h ^= b as u64;
            h = h.wrapping_mul(0x100_0000_01b3);
        };
        for i in 0..self.terrain.len() {
            eat(self.terrain[i]);
            eat(self.level[i]);
            eat(self.flags[i]);
        }
        for p in &self.props {
            eat(p.kind as u8);
            eat(p.x as u8);
            eat(p.y as u8);
            eat(p.variant);
        }
        h
    }
}
