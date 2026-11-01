//! Sprite bank: generates every sprite into the atlas at startup and maps
//! world props and actors to sprite ids.

use std::collections::HashMap;

use crate::art::atlas::Atlas;
use crate::art::buildings::{self, HouseFx};
use crate::art::canvas::Img;
use crate::art::{flora, fx, people};
use crate::palette as pal;
use crate::world::villages::BuildingKind;
use crate::world::{Prop, PropKind, World, TILE};

pub const PAGE_NATURE: usize = 0;
pub const PAGE_HOUSES: usize = 1;
pub const PAGE_CHARS: usize = 2;
pub const PAGES: usize = 5;

pub struct HouseArt {
    pub sprite: u16,
    pub lit: u16,
    pub fx: HouseFx,
}

pub struct SpriteBank {
    pub atlas: Atlas,
    pub houses: HashMap<usize, HouseArt>,
    next_char_page: usize,
}

fn add(atlas: &mut Atlas, name: &str, (img, pivot): (Img, (i32, i32))) {
    atlas.add(PAGE_NATURE, name, &img, pivot);
}

/// Overlay of the pixels that differ between the lit and unlit images,
/// cropped to their bounding box. Drawn on top of a house at night.
fn diff_overlay(dark: &Img, lit: &Img, pivot: (i32, i32)) -> (Img, (i32, i32)) {
    let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, -1, -1);
    for y in 0..dark.h {
        for x in 0..dark.w {
            if dark.get(x, y) != lit.get(x, y) {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    if x1 < 0 {
        return (Img::new(1, 1), (0, 0));
    }
    let mut o = Img::new(x1 - x0 + 1, y1 - y0 + 1);
    for y in y0..=y1 {
        for x in x0..=x1 {
            if dark.get(x, y) != lit.get(x, y) {
                if let Some(c) = lit.get(x, y) {
                    o.set(x - x0, y - y0, c);
                }
            }
        }
    }
    (o, (pivot.0 - x0, pivot.1 - y0))
}

impl SpriteBank {
    pub fn build(world: &World) -> SpriteBank {
        let mut atlas = Atlas::new(PAGES);
        for s in 0..4u8 {
            for v in 0..3u8 {
                add(&mut atlas, &format!("oak_{v}_{s}"), flora::deciduous(0, v, s));
            }
            for v in 0..2u8 {
                add(&mut atlas, &format!("fruit_{v}_{s}"), flora::deciduous(1, v, s));
                add(&mut atlas, &format!("birch_{v}_{s}"), flora::deciduous(2, v, s));
                add(&mut atlas, &format!("pine_{v}_{s}"), flora::pine(v, s));
                add(&mut atlas, &format!("bush_{v}_{s}"), flora::bush(v, s, false));
                add(&mut atlas, &format!("berry_{v}_{s}"), flora::bush(v, s, true));
                add(&mut atlas, &format!("tallgrass_{v}_{s}"), flora::tall_grass(v, s));
            }
            let snow = if s == 3 { 3 } else { 1 };
            if s == 1 || s == 3 {
                for v in 0..3u8 {
                    add(&mut atlas, &format!("rock_{v}_{snow}"), flora::rock(0, v, None, snow));
                    add(&mut atlas, &format!("boulder_{v}_{snow}"), flora::rock(1, v, None, snow));
                }
                add(&mut atlas, &format!("stump_{snow}"), flora::stump(snow));
            }
        }
        let ores: [&[u32]; 4] = [&[0x5a2a1a, 0x9a4a2a, 0xd07a3a, 0xf0b070], pal::METAL, pal::GOLD, &[0x3a1a5a, 0x6a3aa0, 0xa070e0, 0xe0c0ff]];
        for (i, ore) in ores.iter().enumerate() {
            add(&mut atlas, &format!("ore_{i}"), flora::rock(1, i as u8 + 3, Some(ore), 1));
        }
        for v in 0..2u8 {
            add(&mut atlas, &format!("reeds_{v}"), flora::reeds(v));
            add(&mut atlas, &format!("mushroom_{v}"), flora::mushroom(v));
            add(&mut atlas, &format!("lily_{v}"), flora::lily(v));
        }
        for v in 0..6u8 {
            add(&mut atlas, &format!("flowers_{v}"), flora::flower_patch(v));
        }
        for k in 0..6u8 {
            for st in 0..4u8 {
                add(&mut atlas, &format!("crop_{k}_{st}"), flora::crop(k, st));
            }
        }
        for c in 0..4u8 {
            add(&mut atlas, &format!("well_{c}"), buildings::well(c));
        }
        add(&mut atlas, "lamp", buildings::lamp());
        add(&mut atlas, "shrine", buildings::shrine());
        for k in ["barrel", "crate", "haystack", "bench", "fence", "campfire"] {
            add(&mut atlas, k, buildings::small_prop(k, 0));
        }
        for v in 0..4u8 {
            add(&mut atlas, &format!("tent_{v}"), buildings::small_prop("tent", v));
            add(&mut atlas, &format!("stall_{v}"), buildings::small_prop("stall", v));
        }

        for n in 1..=11 {
            add(&mut atlas, &format!("emote_{n}"), fx::emote(n));
        }
        let mut houses = HashMap::new();
        for (i, p) in world.props.iter().enumerate() {
            if p.kind != PropKind::House {
                continue;
            }
            let kind = building_kind(p.state);
            let culture = p.variant / 4;
            let seed = (world.seed as u32).wrapping_mul(31).wrapping_add(i as u32 * 7919);
            let (dark, pivot, fx) = buildings::house(kind, culture, seed, p.fw as i32, p.fh as i32, false);
            let (lit, _, _) = buildings::house(kind, culture, seed, p.fw as i32, p.fh as i32, true);
            let sprite = atlas
                .try_add(PAGE_HOUSES, &dark, pivot)
                .or_else(|| atlas.try_add(PAGE_NATURE, &dark, pivot))
                .expect("house atlas full");
            let (ov, opiv) = diff_overlay(&dark, &lit, pivot);
            let lit_id = atlas.try_add(PAGE_HOUSES, &ov, opiv).or_else(|| atlas.try_add(PAGE_NATURE, &ov, opiv)).expect("overlay atlas full");
            houses.insert(i, HouseArt { sprite, lit: lit_id, fx });
        }
        SpriteBank { atlas, houses, next_char_page: PAGE_CHARS }
    }

    /// Add a character sheet; returns the id of its first frame. Frames are
    /// consecutive ids in sheet order.
    pub fn add_sheet(&mut self, look: &people::Look) -> u16 {
        let frames = people::sheet(look);
        let mut first = None;
        for full in frames.iter() {
            // Crop transparent margins to save atlas space.
            let (f, pivot) = crop(full, people::PIVOT);
            let f = &f;
            let id = loop {
                if let Some(id) = self.atlas.try_add(self.next_char_page, f, pivot) {
                    break id;
                }
                if self.next_char_page + 1 >= PAGES {
                    // Out of space: fall back to recycling the last page's
                    // first sheet would corrupt others, so reuse frame 0 ids.
                    break first.unwrap_or(0);
                }
                self.next_char_page += 1;
            };
            if first.is_none() {
                first = Some(id);
            }
        }
        first.unwrap_or(0)
    }

    pub fn id(&self, name: &str) -> u16 {
        self.atlas.id(name)
    }

    /// Sprite for a static prop in a given season. None = not drawn as a sprite.
    pub fn prop_sprite(&self, idx: usize, p: &Prop, season: u8) -> Option<u16> {
        let s = season;
        let snow = if s == 3 { 3 } else { 1 };
        let name = match p.kind {
            PropKind::Oak => format!("oak_{}_{s}", p.variant % 3),
            PropKind::FruitTree => format!("fruit_{}_{s}", p.variant % 2),
            PropKind::Birch => format!("birch_{}_{s}", p.variant % 2),
            PropKind::Pine => format!("pine_{}_{s}", p.variant % 2),
            PropKind::Bush => format!("bush_{}_{s}", p.variant % 2),
            PropKind::BerryBush => format!("berry_{}_{s}", p.variant % 2),
            PropKind::Rock => format!("rock_{}_{snow}", p.variant % 3),
            PropKind::Boulder => format!("boulder_{}_{snow}", p.variant % 3),
            PropKind::OreRock => format!("ore_{}", p.variant % 4),
            PropKind::Stump => format!("stump_{snow}"),
            PropKind::TallGrass => {
                if s == 3 {
                    return None;
                }
                format!("tallgrass_{}_{s}", p.variant % 2)
            }
            PropKind::Reeds => format!("reeds_{}", p.variant % 2),
            PropKind::Mushroom => {
                if s == 3 {
                    return None;
                }
                format!("mushroom_{}", p.variant % 2)
            }
            PropKind::Lily => format!("lily_{}", p.variant % 2),
            PropKind::Flowers => {
                if s == 3 {
                    return None;
                }
                format!("flowers_{}", p.variant % 6)
            }
            PropKind::Crop => format!("crop_{}_{}", p.variant % 6, p.state.min(3)),
            PropKind::Well => format!("well_{}", p.variant % 4),
            PropKind::Lamp => "lamp".into(),
            PropKind::Shrine => "shrine".into(),
            PropKind::Barrel => "barrel".into(),
            PropKind::Crate => "crate".into(),
            PropKind::Haystack => "haystack".into(),
            PropKind::Bench => "bench".into(),
            PropKind::Fence => "fence".into(),
            PropKind::Campfire => "campfire".into(),
            PropKind::Tent => format!("tent_{}", p.variant % 4),
            PropKind::Stall => format!("stall_{}", p.variant % 4),
            PropKind::House => return self.houses.get(&idx).map(|h| h.sprite),
            PropKind::Sign => return None,
        };
        self.atlas.get(&name)
    }
}

/// Crop an image to its opaque bounding box, adjusting the pivot.
fn crop(img: &Img, pivot: (i32, i32)) -> (Img, (i32, i32)) {
    let (mut x0, mut y0, mut x1, mut y1) = (img.w, img.h, -1, -1);
    for y in 0..img.h {
        for x in 0..img.w {
            if img.alpha(x, y) != 0 {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    if x1 < 0 {
        return (Img::new(1, 1), (0, 0));
    }
    let mut o = Img::new(x1 - x0 + 1, y1 - y0 + 1);
    for y in y0..=y1 {
        for x in x0..=x1 {
            if let Some(c) = img.get(x, y) {
                o.set(x - x0, y - y0, c);
            }
        }
    }
    (o, (pivot.0 - x0, pivot.1 - y0))
}

pub fn building_kind(state: u8) -> BuildingKind {
    match state {
        1 => BuildingKind::Inn,
        2 => BuildingKind::Store,
        3 => BuildingKind::Smithy,
        4 => BuildingKind::Chapel,
        _ => BuildingKind::Home,
    }
}

/// World-space pivot of a prop's sprite.
pub fn prop_pivot(p: &Prop) -> (f32, f32) {
    match p.kind {
        PropKind::House | PropKind::Stall | PropKind::Tent if p.fw > 1 => ((p.x * TILE) as f32, ((p.y + 1) * TILE) as f32),
        PropKind::Lily => ((p.x * TILE + TILE / 2) as f32, (p.y * TILE + TILE / 2) as f32),
        _ => ((p.x * TILE + TILE / 2) as f32, (p.y * TILE + TILE * 3 / 4) as f32),
    }
}
