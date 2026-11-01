//! wasm-bindgen boundary. The worker owns one `Engine`; everything crossing
//! the boundary is plain bytes, floats or JSON strings.

use wasm_bindgen::prelude::*;

use crate::game::Game;

#[wasm_bindgen]
pub struct Engine {
    g: Game,
}

#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: f64) -> Engine {
        Engine { g: Game::new(seed as u64) }
    }

    pub fn tick(&mut self, dt: f32) {
        self.g.tick(dt);
    }

    pub fn command(&mut self, json: &str) {
        self.g.command(json);
    }

    pub fn snapshot(&self) -> Vec<f32> {
        self.g.snapshot()
    }

    pub fn world_info(&self) -> String {
        serde_json::json!({
            "w": self.g.world.w, "h": self.g.world.h,
            "tile": crate::world::TILE, "chunk": crate::world::CHUNK,
            "chunkPx": crate::world::bake::CHUNK_PX,
            "pages": crate::game::sprites::PAGES, "pageSize": crate::art::atlas::PAGE,
            "seed": self.g.world.seed,
        })
        .to_string()
    }

    pub fn atlas_page(&self, i: usize) -> Vec<u8> {
        self.g.sprites.atlas.pages.get(i).map(|p| p.px.clone()).unwrap_or_default()
    }

    /// Rects changed since the last call: [page, x, y, w, h] per rect.
    pub fn take_atlas_dirty(&mut self) -> Vec<u32> {
        let mut out = Vec::new();
        for (page, rects) in self.g.sprites.atlas.dirty.iter_mut().enumerate() {
            for (x, y, w, h) in rects.drain(..) {
                out.extend_from_slice(&[page as u32, x as u32, y as u32, w as u32, h as u32]);
            }
        }
        out
    }

    /// Pixels of a sub-rect of a page (for incremental uploads).
    pub fn atlas_rect(&self, page: usize, x: u32, y: u32, w: u32, h: u32) -> Vec<u8> {
        let img = &self.g.sprites.atlas.pages[page];
        let mut out = Vec::with_capacity((w * h * 4) as usize);
        for yy in y..y + h {
            let s = ((yy as i32 * img.w + x as i32) * 4) as usize;
            out.extend_from_slice(&img.px[s..s + (w * 4) as usize]);
        }
        out
    }

    /// Sprite table: [page, x, y, w, h, pivot_x, pivot_y] per sprite.
    pub fn sprite_table(&self) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.g.sprites.atlas.sprites.len() * 7);
        for s in &self.g.sprites.atlas.sprites {
            out.extend_from_slice(&[s.page as f32, s.x as f32, s.y as f32, s.w as f32, s.h as f32, s.px as f32, s.py as f32]);
        }
        out
    }

    /// Ids of named sprites the renderer needs directly (emotes).
    pub fn named_sprites(&self) -> String {
        let mut m = serde_json::Map::new();
        for n in 1..=11 {
            let k = format!("emote_{n}");
            if let Some(id) = self.g.sprites.atlas.get(&k) {
                m.insert(k, id.into());
            }
        }
        serde_json::Value::Object(m).to_string()
    }

    /// RGBA pixels of one sprite (for UI portraits): [w, h, ...pixels].
    pub fn sprite_pixels(&self, id: u16) -> Vec<u8> {
        let Some(r) = self.g.sprites.atlas.sprites.get(id as usize) else { return vec![] };
        let img = &self.g.sprites.atlas.pages[r.page as usize];
        let mut out = vec![r.w as u8, r.h as u8];
        for y in r.y as i32..(r.y + r.h) as i32 {
            let s = ((y * img.w + r.x as i32) * 4) as usize;
            out.extend_from_slice(&img.px[s..s + r.w as usize * 4]);
        }
        out
    }

    pub fn sprite_count(&self) -> usize {
        self.g.sprites.atlas.sprites.len()
    }

    pub fn static_props(&self) -> Vec<f32> {
        self.g.static_props()
    }

    pub fn static_lights(&self) -> Vec<f32> {
        self.g.static_lights()
    }

    pub fn house_info(&self) -> Vec<f32> {
        self.g.house_info()
    }

    pub fn props_changed(&mut self) -> bool {
        self.g.take_changed_props()
    }

    pub fn season(&self) -> u8 {
        self.g.season()
    }

    pub fn bake_chunk(&self, cx: i32, cy: i32) -> Vec<u8> {
        self.g.bake_chunk(cx, cy)
    }

    pub fn minimap(&self) -> Vec<u8> {
        self.g.minimap()
    }

    pub fn inspect(&self, id: u32) -> String {
        self.g.inspect(id)
    }

    pub fn status(&self) -> String {
        self.g.status()
    }

    pub fn take_ui(&mut self) -> String {
        self.g.take_ui()
    }

    pub fn chronicle(&self) -> String {
        self.g.chronicle_json()
    }

    pub fn villagers(&self) -> String {
        self.g.villager_list()
    }
}

// ------------------------------------------------------------ creator

use crate::art::people::{self, Look};

fn look_from(json: &str) -> Look {
    serde_json::from_str(json).unwrap_or_else(|_| Look::random_any(&mut crate::rng::Rng::new(1)))
}

/// Frame size and sheet layout, for the character creator.
#[wasm_bindgen]
pub fn creator_info() -> String {
    serde_json::json!({
        "fw": people::FW, "fh": people::FH, "pivot": [people::PIVOT.0, people::PIVOT.1],
        "framesPerDir": people::FRAMES_PER_DIR, "sheetFrames": people::SHEET_FRAMES,
        "races": people::RACE_NAMES,
        "skins": crate::palette::SKIN.len(), "demonSkins": crate::palette::DEMON_SKIN.len(),
        "hairs": crate::palette::HAIR.len(), "cloths": crate::palette::CLOTH.len(),
        "hairStyles": people::HAIR_STYLES,
        "swatch": {
            "skin": crate::palette::SKIN.iter().map(|r| r[2]).collect::<Vec<_>>(),
            "demon": crate::palette::DEMON_SKIN.iter().map(|r| r[2]).collect::<Vec<_>>(),
            "hair": crate::palette::HAIR.iter().map(|r| r[2]).collect::<Vec<_>>(),
            "cloth": crate::palette::CLOTH.iter().map(|r| r[2]).collect::<Vec<_>>(),
            "eyes": people::EYE_COLORS.iter().map(|r| r[1]).collect::<Vec<_>>(),
        },
    })
    .to_string()
}

/// One frame as RGBA bytes (FW × FH).
#[wasm_bindgen]
pub fn render_frame(look_json: &str, dir: u8, frame: u16) -> Vec<u8> {
    people::frame(&look_from(look_json), dir, frame).px
}

/// A random look from a seed, as JSON.
#[wasm_bindgen]
pub fn random_look(seed: f64) -> String {
    serde_json::to_string(&Look::random_any(&mut crate::rng::Rng::new(seed as u64))).unwrap()
}

/// Visual QA results for a look, as JSON.
#[wasm_bindgen]
pub fn lint_look(look_json: &str) -> String {
    let issues: Vec<_> = people::lint(&look_from(look_json))
        .into_iter()
        .map(|i| serde_json::json!({ "kind": i.kind, "dir": i.dir, "frame": i.frame, "detail": i.detail }))
        .collect();
    serde_json::Value::Array(issues).to_string()
}
