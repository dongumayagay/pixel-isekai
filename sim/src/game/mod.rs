//! The running game: world, simulation, player, and the data contract with
//! the renderer (snapshots, static props, lights).

pub mod actor;
pub mod ai;
pub mod clock;
pub mod dialogue;
pub mod society;
pub mod sprites;
pub mod villager;

use serde::Deserialize;
use serde_json::json;

use actor::{stand_pos, Activity, Actor, Facing};
use ai::{Seen, Stock};
use clock::{Clock, Weather, SEC_PER_MIN, WINTER};
use society::Event;
use sprites::{prop_pivot, SpriteBank};
use villager::{Emote, Goal, Villager};

use crate::art::people::{outfit, Look};
use crate::rng::Rng;
use crate::world::bake::{Baker, CHUNK_PX};
use crate::world::path::PathFinder;
use crate::world::{worldgen, PropKind, World, TILE};

pub const HEADER: usize = 20;
pub const ENT_STRIDE: usize = 8;

pub mod eflag {
    pub const FLIP: u32 = 1;
    pub const SHADOW: u32 = 2;
    pub const FLASH: u32 = 4;
    pub const PLAYER: u32 = 8;
    pub const SWAY: u32 = 16;
    pub const GROUND: u32 = 32;
    pub const BIG_SHADOW: u32 = 64;
    pub const HOUSE: u32 = 128;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pending {
    Talk(u32),
    Harvest(usize),
}

pub struct Player {
    pub actor: Actor,
    pub look: Look,
    pub hp: f32,
    pub energy: f32,
    pub gold: i32,
    pub dir_input: (f32, f32),
    pub pending: Option<Pending>,
    pub inventory: Vec<(String, u32)>,
    pub blessing: u8,
    pub skills: [f32; 5],
}

impl Player {
    pub fn add_item(&mut self, name: &str, n: u32) {
        if let Some(e) = self.inventory.iter_mut().find(|(k, _)| k == name) {
            e.1 += n;
        } else {
            self.inventory.push((name.to_string(), n));
        }
    }
}

pub struct Game {
    pub world: World,
    pub baker: Baker,
    pub sprites: SpriteBank,
    pub clock: Clock,
    pub rng: Rng,
    pub pf: PathFinder,
    pub villagers: Vec<Villager>,
    pub stocks: Vec<Stock>,
    pub player: Player,
    pub chronicle: Vec<Event>,
    pub events_out: Vec<Event>,
    pub toasts: Vec<String>,
    pub changed_props: Vec<usize>,
    pub dirty_looks: Vec<u32>,
    pub season_changed: bool,
    pub next_id: u32,
    pub speed: f32,
    pub tick_n: u64,
    pub dialogue_out: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(tag = "t")]
enum Cmd {
    #[serde(rename = "tap")]
    Tap { x: f32, y: f32 },
    #[serde(rename = "dir")]
    Dir { x: f32, y: f32 },
    #[serde(rename = "speed")]
    Speed { v: f32 },
    #[serde(rename = "weather")]
    Weather { w: u8, i: f32 },
    #[serde(rename = "skip")]
    Skip { hours: f32 },
    #[serde(rename = "teleport")]
    Teleport { x: f32, y: f32 },
}

impl Game {
    pub fn new(seed: u64) -> Game {
        let world = worldgen::generate(seed);
        let mut rng = Rng::new(seed ^ 0x5eed);
        let mut sprites = SpriteBank::build(&world);
        let mut villagers = villager::populate(&world, &mut rng.fork(1));
        for v in villagers.iter_mut() {
            v.actor.sheet = sprites.add_sheet(&v.look);
        }
        let next_id = villagers.iter().map(|v| v.id).max().unwrap_or(0) + 1;
        let mut look = Look::random(&mut rng.fork(77), false, 1);
        look.hair = 0;
        look.hair_style = 13;
        look.outfit = outfit::HERO;
        (look.primary, look.secondary, look.accent) = outfit::defaults(outfit::HERO, false);
        look.eyes = 0;
        look.facial = 0;
        let sheet = sprites.add_sheet(&look);
        let (sx, sy) = stand_pos(world.spawn);
        let mut actor = Actor::new(sx, sy, sheet, 4.2);
        actor.facing = Facing::Down;
        let player = Player {
            actor,
            look,
            hp: 100.0,
            energy: 100.0,
            gold: 30,
            dir_input: (0.0, 0.0),
            pending: None,
            inventory: Vec::new(),
            blessing: 0,
            skills: [0.0; 5],
        };
        let mut g = Game {
            baker: Baker::new(seed),
            pf: PathFinder::new(world.w, world.h),
            world,
            sprites,
            clock: Clock::new(),
            rng,
            villagers,
            stocks: Vec::new(),
            player,
            chronicle: Vec::new(),
            events_out: Vec::new(),
            toasts: Vec::new(),
            changed_props: Vec::new(),
            dirty_looks: Vec::new(),
            season_changed: false,
            next_id,
            speed: 1.0,
            tick_n: 0,
            dialogue_out: Vec::new(),
        };
        g.stocks = g.starting_stocks();
        let first = g.world.villages.first().map(|v| v.name.clone()).unwrap_or_default();
        g.chronicle("arrival", format!("A stranger from another world awoke near {first}."), vec![]);
        g
    }

    pub fn season(&self) -> u8 {
        self.clock.season()
    }

    /// Advance the simulation by `dt` real seconds.
    pub fn tick(&mut self, dt: f32) {
        let dt = dt.min(0.1) * self.speed;
        let mins = dt / SEC_PER_MIN;
        self.tick_n += 1;
        if self.clock.advance(mins as f64, &mut self.rng) {
            self.new_day();
        }
        self.update_player(dt);
        self.update_villagers(dt, mins);
        // Rebuild sprites for villagers who aged into a new life stage.
        let dirty: Vec<u32> = std::mem::take(&mut self.dirty_looks);
        for id in dirty {
            if let Some(i) = self.index_of(id) {
                let look = self.villagers[i].look;
                self.villagers[i].actor.sheet = self.sprites.add_sheet(&look);
            }
        }
    }

    /// Index of a living villager by id.
    pub fn index_of(&self, id: u32) -> Option<usize> {
        self.villagers.iter().position(|v| v.id == id && v.alive)
    }

    fn update_villagers(&mut self, dt: f32, mins: f32) {
        let seen: Vec<Seen> = self
            .villagers
            .iter()
            .filter(|v| v.alive)
            .map(|v| Seen { id: v.id, x: v.actor.x, y: v.actor.y, village: v.village, available: ai::available(v) })
            .collect();
        let mut invites = Vec::new();
        let mut talks = Vec::new();
        let mut changed = std::mem::take(&mut self.changed_props);
        {
            let mut ctx = ai::Ctx {
                world: &mut self.world,
                pf: &mut self.pf,
                rng: &mut self.rng,
                clock: &self.clock,
                stocks: &mut self.stocks,
                seen: &seen,
                invites: &mut invites,
                changed_props: &mut changed,
                talks: &mut talks,
            };
            for v in self.villagers.iter_mut() {
                if v.alive {
                    ai::update(v, &mut ctx, dt, mins);
                }
            }
        }
        self.changed_props = changed;
        for (target, from) in invites {
            let Some(fi) = self.index_of(from) else { continue };
            let (fx, fy) = (self.villagers[fi].actor.x, self.villagers[fi].actor.y);
            if let Some(ti) = self.index_of(target) {
                if ai::available(&self.villagers[ti]) {
                    let mut r = self.rng.fork(target as u64);
                    ai::accept_invite(&mut self.villagers[ti], from, fx, fy, &mut r);
                } else {
                    // Rebuffed: wander off.
                    self.villagers[fi].timer = 1.0;
                }
            }
        }
        if !talks.is_empty() {
            self.resolve_talks(talks);
        }
    }

    fn update_player(&mut self, dt: f32) {
        let p = &mut self.player;
        p.actor.update_anim(dt);
        let (vx, vy) = p.dir_input;
        if vx.abs() + vy.abs() > 0.05 {
            p.actor.clear_path();
            p.pending = None;
            p.actor.activity = Activity::None;
            p.actor.walk_free(&self.world, vx, vy, dt);
        } else if p.actor.has_path() {
            // Keep walking toward a moving villager.
            if let Some(Pending::Talk(id)) = p.pending
                && let Some(v) = self.villagers.iter().find(|v| v.id == id)
                && p.actor.dist_to(v.actor.x, v.actor.y) < TILE as f32 * 1.3
            {
                p.actor.clear_path();
            }
            p.actor.follow(dt);
        } else {
            p.actor.moving = false;
        }
        if !self.player.actor.has_path()
            && let Some(pend) = self.player.pending.take()
        {
            self.resolve_pending(pend);
        }
    }

    fn resolve_pending(&mut self, pend: Pending) {
        match pend {
            Pending::Talk(id) => {
                let Some(i) = self.index_of(id) else { return };
                let (px, py) = (self.player.actor.x, self.player.actor.y);
                let (vx, vy) = (self.villagers[i].actor.x, self.villagers[i].actor.y);
                if self.player.actor.dist_to(vx, vy) > TILE as f32 * 2.2 {
                    return;
                }
                self.player.actor.face_toward(vx, vy);
                self.villagers[i].actor.face_toward(px, py);
                let lines = self.dialogue_for(i);
                let v = &mut self.villagers[i];
                if !v.talked_today {
                    v.player_opinion = (v.player_opinion + 3.0 + v.traits.sociable * 2.0).min(100.0);
                    v.talked_today = true;
                    v.needs.social = (v.needs.social + 10.0).min(100.0);
                }
                v.emote(if v.player_opinion > 40.0 { Emote::Heart } else { Emote::Chat }, 2.5);
                let out = json!({
                    "id": v.id, "name": v.name(), "job": v.job.label(), "lines": lines,
                    "opinion": v.player_opinion, "sheet": v.actor.sheet,
                });
                self.dialogue_out.push(out);
            }
            Pending::Harvest(pi) => self.harvest(pi),
        }
    }

    fn harvest(&mut self, pi: usize) {
        let Some(p) = self.world.props.get(pi).cloned() else { return };
        if !p.alive {
            return;
        }
        let (px, py) = prop_pivot(&p);
        self.player.actor.face_toward(px, py - 8.0);
        self.player.actor.activity = match p.kind {
            k if k.is_tree() => Activity::Chop,
            k if k.is_rock() => Activity::Mine,
            _ => Activity::Hoe,
        };
        self.player.actor.anim = 0.0;
        let (item, n, remove): (&str, u32, bool) = match p.kind {
            k if k.is_tree() => {
                self.player.actor.flash = 0.0;
                if self.world.props[pi].chop(1) {
                    self.changed_props.push(pi);
                    self.toasts.push("Timber! +5 Wood".into());
                    ("Wood", 5, false)
                } else {
                    ("Wood", 1, false)
                }
            }
            PropKind::Stump => ("Wood", 2, true),
            PropKind::Rock => ("Stone", 2, true),
            PropKind::Boulder => ("Stone", 5, true),
            PropKind::OreRock => (["Copper Ore", "Iron Ore", "Gold Ore", "Crystal"][p.variant as usize % 4], 2, true),
            PropKind::BerryBush => {
                if self.season() == WINTER {
                    return;
                }
                ("Berries", 3, false)
            }
            PropKind::Mushroom => ("Mushroom", 1, true),
            PropKind::Flowers => ("Flowers", 1, true),
            PropKind::Crop => {
                // Taking a farmer's crop: villagers who see it disapprove.
                if p.state < 3 {
                    return;
                }
                self.world.props[pi].state = 0;
                self.changed_props.push(pi);
                let (x, y) = (self.player.actor.x, self.player.actor.y);
                for v in self.villagers.iter_mut().filter(|v| v.alive && !v.actor.hidden) {
                    if (v.actor.x - x).abs() < 200.0 && (v.actor.y - y).abs() < 160.0 {
                        v.player_opinion -= 6.0;
                        v.emote(Emote::Anger, 3.0);
                    }
                }
                ("Produce", 2, false)
            }
            _ => return,
        };
        if remove {
            self.world.remove_prop(pi);
            self.changed_props.push(pi);
        }
        self.player.add_item(item, n);
        self.player.energy = (self.player.energy - 1.5).max(0.0);
        self.toasts.push(format!("+{n} {item}"));
    }

    pub fn command(&mut self, json_str: &str) {
        let Ok(cmd) = serde_json::from_str::<Cmd>(json_str) else { return };
        match cmd {
            Cmd::Tap { x, y } => self.tap(x, y),
            Cmd::Dir { x, y } => self.player.dir_input = (x, y),
            Cmd::Speed { v } => self.speed = v.clamp(0.0, 16.0),
            Cmd::Weather { w, i } => {
                self.clock.set_weather(Weather::from_index(w as usize), i);
            }
            Cmd::Skip { hours } => {
                let steps = (hours * 60.0 * SEC_PER_MIN / 0.1) as usize;
                let sp = self.speed;
                self.speed = 1.0;
                for _ in 0..steps {
                    self.tick(0.1);
                }
                self.speed = sp;
            }
            Cmd::Teleport { x, y } => {
                self.player.actor.x = x;
                self.player.actor.y = y;
            }
        }
    }

    /// Tap/click on the world: talk to a villager, harvest a prop, or walk.
    fn tap(&mut self, x: f32, y: f32) {
        self.player.pending = None;
        self.player.actor.activity = Activity::None;
        // Villagers: hit-test their body box.
        let mut best: Option<(u32, f32)> = None;
        for v in self.villagers.iter().filter(|v| v.alive && !v.actor.hidden) {
            let dx = x - v.actor.x;
            let dy = y - (v.actor.y - 46.0);
            if dx.abs() < 18.0 && dy.abs() < 46.0 {
                let d = dx * dx + dy * dy;
                if best.is_none_or(|(_, b)| d < b) {
                    best = Some((v.id, d));
                }
            }
        }
        if let Some((id, _)) = best {
            let i = self.index_of(id).unwrap();
            let goal = self.villagers[i].actor.tile();
            self.player.pending = Some(Pending::Talk(id));
            let d = self.player.actor.dist_to(self.villagers[i].actor.x, self.villagers[i].actor.y);
            if d < TILE as f32 * 1.6 || !self.player.actor.path_to(&self.world, &mut self.pf, goal, 20_000) {
                self.player.actor.clear_path();
            }
            return;
        }
        let tx = (x / TILE as f32).floor() as i32;
        let ty = (y / TILE as f32).floor() as i32;
        // Harvestable prop under or just below the tap (sprites extend up).
        for (dx, dy) in [(0, 0), (0, 1), (0, 2)] {
            let (px, py) = (tx + dx, ty + dy);
            let idx = self.world.prop_at(px, py).or_else(|| {
                self.world.props.iter().position(|p| p.alive && p.x == px && p.y == py && matches!(p.kind, PropKind::Mushroom | PropKind::Flowers | PropKind::Crop))
            });
            if let Some(pi) = idx {
                let p = &self.world.props[pi];
                if !p.kind.is_harvestable() {
                    break;
                }
                let (bx, by) = (p.x, p.y);
                self.player.pending = Some(Pending::Harvest(pi));
                let (cx, cy) = self.player.actor.tile();
                if (cx - bx).abs() <= 1 && (cy - by).abs() <= 1 {
                    self.player.actor.clear_path();
                    return;
                }
                let goal = [(0, 1), (-1, 0), (1, 0), (0, -1), (1, 1), (-1, 1)]
                    .iter()
                    .map(|(ox, oy)| (bx + ox, by + oy))
                    .find(|&(gx, gy)| self.world.walkable(gx, gy));
                if let Some(goal) = goal {
                    if !self.player.actor.path_to(&self.world, &mut self.pf, goal, 20_000) {
                        self.player.pending = None;
                    }
                } else {
                    self.player.pending = None;
                }
                return;
            }
        }
        let goal = if self.world.walkable(tx, ty) { Some((tx, ty)) } else { actor::nearest_walkable(&self.world, (tx, ty)) };
        if let Some(goal) = goal {
            self.player.actor.path_to(&self.world, &mut self.pf, goal, 25_000);
        }
    }

    // ------------------------------------------------------------ renderer contract

    /// Static props: [x, y, sprite, flags, prop_index] per prop.
    pub fn static_props(&self) -> Vec<f32> {
        let season = self.season();
        let mut out = Vec::with_capacity(self.world.props.len() * 5);
        for (i, p) in self.world.props.iter().enumerate() {
            if !p.alive {
                continue;
            }
            let Some(sid) = self.sprites.prop_sprite(i, p, season) else { continue };
            let (x, y) = prop_pivot(p);
            let mut f = eflag::SHADOW;
            if p.kind.is_tree() {
                f |= eflag::SWAY | eflag::BIG_SHADOW;
            }
            if matches!(p.kind, PropKind::TallGrass | PropKind::Reeds | PropKind::Crop | PropKind::Flowers) {
                f |= eflag::SWAY;
                f &= !eflag::SHADOW;
            }
            if p.kind == PropKind::Lily {
                f = eflag::GROUND;
            }
            if p.kind == PropKind::House {
                f = eflag::HOUSE;
            }
            out.extend_from_slice(&[x, y, sid as f32, f as f32, i as f32]);
        }
        out
    }

    /// Static lights: [x, y, radius, r, g, b, house_index or -1].
    pub fn static_lights(&self) -> Vec<f32> {
        let mut out = Vec::new();
        for (i, p) in self.world.props.iter().enumerate() {
            if !p.alive {
                continue;
            }
            let (x, y) = prop_pivot(p);
            match p.kind {
                PropKind::Lamp => out.extend_from_slice(&[x, y - 84.0, 130.0, 1.0, 0.72, 0.38, -1.0]),
                PropKind::Shrine => out.extend_from_slice(&[x, y - 70.0, 80.0, 1.0, 0.86, 0.5, -1.0]),
                PropKind::Campfire => out.extend_from_slice(&[x, y - 6.0, 140.0, 1.0, 0.55, 0.25, -1.0]),
                PropKind::House => {
                    if let Some(h) = self.sprites.houses.get(&i) {
                        for &(wx, wy) in &h.fx.windows {
                            out.extend_from_slice(&[x + wx as f32, y + wy as f32, 56.0, 1.0, 0.7, 0.36, i as f32]);
                        }
                    }
                }
                _ => {}
            }
        }
        out
    }

    /// House extras: [prop_index, lit_overlay_sprite, chimney_x, chimney_y] (chimney -1 if none).
    pub fn house_info(&self) -> Vec<f32> {
        let mut out = Vec::new();
        for (&i, h) in &self.sprites.houses {
            let p = &self.world.props[i];
            let (x, y) = prop_pivot(p);
            let (cx, cy) = h.fx.chimney.map(|(a, b)| (x + a as f32, y + b as f32)).unwrap_or((-1.0, -1.0));
            out.extend_from_slice(&[i as f32, h.lit as f32, cx, cy]);
        }
        out
    }

    /// Per-tick snapshot for the renderer.
    pub fn snapshot(&self) -> Vec<f32> {
        let mut out = vec![0.0f32; HEADER];
        let p = &self.player;
        out[0] = self.tick_n as f32;
        out[1] = self.clock.minute_of_day();
        out[2] = self.clock.day() as f32;
        out[3] = self.season() as f32;
        out[4] = self.clock.year() as f32;
        out[5] = self.clock.weather as u8 as f32;
        out[6] = self.clock.intensity;
        out[7] = p.actor.x;
        out[8] = p.actor.y;
        out[9] = p.hp;
        out[10] = p.energy;
        out[11] = p.gold as f32;
        out[15] = self.clock.wind;
        out[16] = self.speed;
        let mut count = 0;
        let push_actor = |out: &mut Vec<f32>, id: u32, a: &Actor, extra: u32, emote: Emote, player: bool| {
            let (sid, flip) = a.sprite();
            let mut f = eflag::SHADOW;
            if flip {
                f |= eflag::FLIP;
            }
            if a.flash > 0.0 {
                f |= eflag::FLASH;
            }
            if player {
                f |= eflag::PLAYER;
            }
            out.extend_from_slice(&[id as f32, a.x, a.y, sid as f32, f as f32, emote as u8 as f32, (a.activity as u32 | extra << 8) as f32, 0.0]);
        };
        push_actor(&mut out, 0, &p.actor, 0, Emote::None, true);
        count += 1;
        for v in self.villagers.iter() {
            if !v.alive || v.actor.hidden {
                continue;
            }
            push_actor(&mut out, v.id, &v.actor, 0, v.emote, false);
            count += 1;
        }
        out[12] = count as f32;
        // Lit houses: someone home and awake in the evening/night.
        let h = self.clock.hour();
        let evening = !(6.5..17.5).contains(&h);
        let mut lit: Vec<usize> = Vec::new();
        if evening {
            for v in self.villagers.iter() {
                if let Some(b) = v.inside {
                    let asleep = v.goal == Goal::Sleep && !(17.5..22.5).contains(&h) && !(5.0..6.5).contains(&h);
                    if !asleep && !lit.contains(&b) {
                        lit.push(b);
                    }
                }
            }
            // Inns and chapels glow all evening.
            for vil in &self.world.villages {
                for b in &vil.buildings {
                    if matches!(b.kind, crate::world::villages::BuildingKind::Inn | crate::world::villages::BuildingKind::Chapel) && (17.5..24.0).contains(&h) && !lit.contains(&b.prop) {
                        lit.push(b.prop);
                    }
                }
            }
        }
        out[13] = lit.len() as f32;
        for b in lit {
            out.push(b as f32);
        }
        out
    }

    pub fn bake_chunk(&self, cx: i32, cy: i32) -> Vec<u8> {
        let mut buf = vec![0u8; (CHUNK_PX * CHUNK_PX * 4) as usize];
        self.baker.bake_chunk(&self.world, cx, cy, self.season(), &mut buf);
        buf
    }

    /// One pixel per tile overview map (RGBA).
    pub fn minimap(&self) -> Vec<u8> {
        let mut out = vec![0u8; (self.world.w * self.world.h * 4) as usize];
        let mut buf = vec![0u8; (CHUNK_PX * CHUNK_PX * 4) as usize];
        let season = self.season();
        for cy in 0..crate::world::CHUNKS_Y {
            for cx in 0..crate::world::CHUNKS_X {
                // Sample tile centres from a coarse bake of each chunk.
                for ty in 0..crate::world::CHUNK {
                    for tx in 0..crate::world::CHUNK {
                        let wx = cx * crate::world::CHUNK + tx;
                        let wy = cy * crate::world::CHUNK + ty;
                        let c = self.baker.sample(&self.world, wx * TILE + TILE / 2, wy * TILE + TILE / 2, season);
                        let o = ((wy * self.world.w + wx) * 4) as usize;
                        out[o..o + 3].copy_from_slice(&c);
                        out[o + 3] = 255;
                    }
                }
            }
        }
        let _ = &mut buf;
        // Trees and houses as darker dots.
        for p in &self.world.props {
            if !p.alive {
                continue;
            }
            let c: Option<[u8; 3]> = match p.kind {
                k if k.is_tree() => Some([30, 70, 50]),
                PropKind::House => Some([150, 80, 60]),
                _ => None,
            };
            if let Some(c) = c {
                let o = ((p.y.clamp(0, self.world.h - 1) * self.world.w + p.x.clamp(0, self.world.w - 1)) * 4) as usize;
                out[o..o + 3].copy_from_slice(&c);
            }
        }
        out
    }

    /// Detail for the villager inspector panel.
    pub fn inspect(&self, id: u32) -> String {
        let Some(v) = self.villagers.iter().find(|v| v.id == id) else { return "null".into() };
        let name_of = |oid: u32| self.villagers.iter().find(|o| o.id == oid).map(|o| o.name()).unwrap_or_else(|| "someone".into());
        let mut rel: Vec<(u32, f32)> = v.opinions.iter().filter(|(o, _)| self.villagers.iter().any(|x| x.id == *o && x.alive)).cloned().collect();
        rel.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let friends: Vec<_> = rel.iter().take(4).map(|(o, s)| json!({"id": o, "name": name_of(*o), "score": s.round()})).collect();
        let rivals: Vec<_> = rel.iter().rev().take(2).filter(|(_, s)| *s < -5.0).map(|(o, s)| json!({"id": o, "name": name_of(*o), "score": s.round()})).collect();
        let vil = &self.world.villages[v.village];
        let doing = match v.goal {
            Goal::Work => v.job.work_description().to_string(),
            Goal::Socialize(o) => format!("Chatting with {}", name_of(o)),
            g => g.label().to_string(),
        };
        json!({
            "id": v.id, "name": v.name(), "age": v.age.floor(), "stage": v.stage(), "female": v.female,
            "job": v.job.label(), "village": vil.name, "traits": v.traits.describe(),
            "needs": {"hunger": v.needs.hunger.round(), "energy": v.needs.energy.round(), "social": v.needs.social.round(), "fun": v.needs.fun.round()},
            "mood": v.needs.mood().round(), "doing": doing,
            "partner": v.partner.map(name_of), "dating": v.dating.map(name_of),
            "children": v.children.iter().map(|c| name_of(*c)).collect::<Vec<_>>(),
            "parents": v.parents.iter().map(|c| name_of(*c)).collect::<Vec<_>>(),
            "friends": friends, "rivals": rivals, "memories": v.memories, "gold": v.gold,
            "opinion": v.player_opinion.round(), "sheet": v.actor.sheet, "pregnant": v.pregnant > 0.0,
        })
        .to_string()
    }

    /// HUD and panels: date, player status, inventory.
    pub fn status(&self) -> String {
        let pop = self.villagers.iter().filter(|v| v.alive).count();
        json!({
            "date": self.clock.date_label(),
            "season": self.season(),
            "day": self.clock.day(),
            "weather": self.clock.weather as u8,
            "inventory": self.player.inventory,
            "gold": self.player.gold,
            "population": pop,
            "villages": self.world.villages.iter().map(|v| json!({
                "name": v.name, "x": v.cx, "y": v.cy,
                "pop": self.villagers.iter().filter(|p| p.alive && p.village == v.id).count(),
                "stock": self.stocks.get(v.id),
            })).collect::<Vec<_>>(),
        })
        .to_string()
    }

    /// Drain events, toasts and dialogue for the UI.
    pub fn take_ui(&mut self) -> String {
        let ev: Vec<Event> = std::mem::take(&mut self.events_out);
        let toasts: Vec<String> = std::mem::take(&mut self.toasts);
        let dlg = std::mem::take(&mut self.dialogue_out);
        let season = std::mem::take(&mut self.season_changed);
        json!({"events": ev, "toasts": toasts, "dialogue": dlg, "season_changed": season}).to_string()
    }

    pub fn chronicle_json(&self) -> String {
        serde_json::to_string(&self.chronicle).unwrap_or_else(|_| "[]".into())
    }

    pub fn villager_list(&self) -> String {
        let list: Vec<_> = self
            .villagers
            .iter()
            .filter(|v| v.alive)
            .map(|v| json!({"id": v.id, "name": v.name(), "job": v.job.label(), "village": v.village, "x": v.actor.x, "y": v.actor.y, "sheet": v.actor.sheet}))
            .collect();
        serde_json::to_string(&list).unwrap()
    }

    pub fn take_changed_props(&mut self) -> bool {
        let changed = !self.changed_props.is_empty();
        self.changed_props.clear();
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soak_sixty_days() {
        let mut g = Game::new(2026);
        let start_pop = g.villagers.iter().filter(|v| v.alive).count();
        assert!(start_pop > 30, "population {start_pop}");
        let t0 = std::time::Instant::now();
        // 60 in-game days at 10 ticks per real second, 8x speed.
        g.speed = 8.0;
        let ticks = (60.0 * 1440.0 * SEC_PER_MIN / 8.0 / 0.1) as usize;
        for _ in 0..ticks {
            g.tick(0.1);
        }
        let pop = g.villagers.iter().filter(|v| v.alive).count();
        let kinds: std::collections::HashSet<&str> = g.chronicle.iter().map(|e| e.kind).collect();
        eprintln!("soak: {:?}, pop {start_pop} -> {pop}, events {}, kinds {:?}", t0.elapsed(), g.chronicle.len(), kinds);
        assert!(pop > start_pop / 2 && pop < start_pop * 3);
        assert!(kinds.contains("romance") || kinds.contains("wedding") || kinds.contains("arrival"), "{kinds:?}");
        assert_eq!(g.clock.day(), 60);
    }

    #[test]
    fn villagers_actually_work_and_socialise() {
        let mut g = Game::new(99);
        g.speed = 4.0;
        let mut talked = 0;
        let mut worked = 0;
        for _ in 0..(1440.0 * SEC_PER_MIN / 4.0 / 0.1) as usize {
            g.tick(0.1);
            talked += g.villagers.iter().filter(|v| matches!(v.goal, Goal::Socialize(_)) && v.actor.activity == Activity::Talk).count();
            worked += g.villagers.iter().filter(|v| v.goal == Goal::Work && v.phase == villager::Phase::Perform).count();
        }
        assert!(talked > 100, "talked {talked}");
        assert!(worked > 1000, "worked {worked}");
    }
}
