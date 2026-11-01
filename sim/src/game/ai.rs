//! Villager decision making (utility AI) and goal execution.

use super::actor::{nearest_walkable, Activity};
use super::clock::Clock;
use super::villager::{Emote, Goal, Job, Phase, Villager};
use crate::rng::Rng;
use crate::world::path::PathFinder;
use crate::world::villages::BuildingKind;
use crate::world::{flag, PropKind, Terrain, World, TILE};

/// Per-village shared resources.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Stock {
    pub food: f32,
    pub wood: f32,
    pub ore: f32,
    pub goods: f32,
    pub gold: f32,
}

/// Lightweight view of another villager for social decisions.
#[derive(Clone, Copy)]
pub struct Seen {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub village: usize,
    pub available: bool,
    pub adult: bool,
}

pub struct Ctx<'a> {
    pub world: &'a mut World,
    pub pf: &'a mut PathFinder,
    pub rng: &'a mut Rng,
    pub clock: &'a Clock,
    pub stocks: &'a mut Vec<Stock>,
    pub seen: &'a [Seen],
    /// Requests for another villager to join a conversation: (target, from).
    pub invites: &'a mut Vec<(u32, u32)>,
    /// Chronicle lines produced this tick.
    pub log: &'a mut Vec<String>,
    /// Props whose sprite changed (chopped trees, harvested crops).
    pub changed_props: &'a mut Vec<usize>,
    /// Finished conversations: (speaker, listener).
    pub talks: &'a mut Vec<(u32, u32)>,
}

fn door_of(world: &World, prop: usize) -> (i32, i32) {
    let p = &world.props[prop];
    (p.x + p.fw as i32 / 2, p.y + 1)
}

fn building_door(world: &World, village: usize, kind: BuildingKind) -> Option<(i32, i32)> {
    world.villages.get(village)?.buildings.iter().find(|b| b.kind == kind).map(|b| b.door)
}

/// Find a walkable tile adjacent to (x, y), preferring the south side so
/// workers face up toward what they're working on.
fn beside(world: &World, x: i32, y: i32) -> Option<(i32, i32)> {
    for (dx, dy) in [(0, 1), (-1, 0), (1, 0), (0, -1), (1, 1), (-1, 1)] {
        if world.walkable(x + dx, y + dy) && world.level(x + dx, y + dy) == world.level(x, y) {
            return Some((x + dx, y + dy));
        }
    }
    None
}

fn find_prop_near(world: &World, rng: &mut Rng, cx: i32, cy: i32, radius: i32, pred: impl Fn(PropKind) -> bool) -> Option<usize> {
    let mut found: Vec<(i32, usize)> = Vec::new();
    for (i, p) in world.props.iter().enumerate() {
        if !p.alive || !pred(p.kind) {
            continue;
        }
        let d = (p.x - cx).abs() + (p.y - cy).abs();
        if d <= radius {
            found.push((d, i));
        }
    }
    if found.is_empty() {
        return None;
    }
    found.sort_unstable();
    let k = found.len().min(8);
    Some(found[rng.below(k)].1)
}

fn random_tile_near(world: &World, rng: &mut Rng, cx: i32, cy: i32, r: i32, same_level: bool) -> Option<(i32, i32)> {
    let lv = world.level(cx, cy);
    for _ in 0..30 {
        let x = cx + rng.range(-r, r + 1);
        let y = cy + rng.range(-r, r + 1);
        if world.walkable(x, y) && (!same_level || world.level(x, y) == lv) {
            return Some((x, y));
        }
    }
    None
}

fn work_hours(job: Job, h: f32) -> bool {
    match job {
        Job::Guard => !(4.0..9.0).contains(&h),
        Job::Innkeeper => (10.0..23.0).contains(&h),
        Job::Fisher => (5.0..14.0).contains(&h),
        Job::Child | Job::Retired => false,
        _ => (8.0..17.5).contains(&h),
    }
}

/// Score the options and pick a goal.
pub fn decide(v: &mut Villager, ctx: &mut Ctx) {
    let h = ctx.clock.hour();
    let n = &v.needs;
    let night = !(6.0..22.0).contains(&h) || (v.job == Job::Child && !(6.5..20.5).contains(&h));
    let fatigue = 1.0 - n.energy / 100.0;
    let hungry = 1.0 - n.hunger / 100.0;
    let lonely = 1.0 - n.social / 100.0;
    let bored = 1.0 - n.fun / 100.0;
    let t = &v.traits;
    let noise = |r: &mut Rng| r.f32() * 0.12;

    let mut best = (Goal::Leisure, 0.12 + bored * 0.6 + t.curious * 0.1 + noise(ctx.rng));
    let mut consider = |g: Goal, s: f32| {
        if s > best.1 {
            best = (g, s);
        }
    };
    let sleep_s = if night && v.job != Job::Guard { 1.1 + fatigue } else { fatigue * fatigue * 1.6 };
    consider(Goal::Sleep, sleep_s + noise(ctx.rng));
    let meal = if (6.5..8.5).contains(&h) || (11.5..13.5).contains(&h) || (17.5..19.5).contains(&h) { 0.35 } else { 0.0 };
    consider(Goal::Eat, hungry.powf(1.5) * 1.6 + meal + noise(ctx.rng));
    if v.is_adult() && work_hours(v.job, h) {
        consider(Goal::Work, 0.5 + t.diligent * 0.4 + noise(ctx.rng));
    }
    if v.job == Job::Child && (8.0..19.0).contains(&h) {
        consider(Goal::Play, 0.55 + bored * 0.5 + noise(ctx.rng));
    }
    if t.devout > 0.55 && (6.0..9.0).contains(&h) {
        consider(Goal::Pray, t.devout * 0.6 + noise(ctx.rng));
    }
    if v.gold > 25 && (9.0..18.0).contains(&h) {
        consider(Goal::Shop, 0.18 + bored * 0.2 + noise(ctx.rng));
    }
    // Pick a conversation partner: someone liked and nearby.
    let mut partner: Option<(u32, f32)> = None;
    if !night {
        for s in ctx.seen {
            if s.id == v.id || !s.available || s.village != v.village {
                continue;
            }
            let d = ((s.x - v.actor.x).powi(2) + (s.y - v.actor.y).powi(2)).sqrt() / TILE as f32;
            if d > 22.0 {
                continue;
            }
            let op = v.opinion_of(s.id);
            let score = op / 100.0 - d / 40.0 + ctx.rng.f32() * 0.3 + if Some(s.id) == v.partner || Some(s.id) == v.dating { 0.4 } else { 0.0 };
            if partner.map_or(true, |(_, b)| score > b) {
                partner = Some((s.id, score));
            }
        }
        if let Some((id, sc)) = partner {
            let evening = if (17.0..21.5).contains(&h) { 0.18 } else { 0.0 };
            consider(Goal::Socialize(id), lonely * 0.95 * (0.45 + t.sociable) + evening + sc * 0.15 + noise(ctx.rng));
        }
    }
    begin(v, best.0, ctx);
}

/// Set up travel for a goal.
pub fn begin(v: &mut Villager, goal: Goal, ctx: &mut Ctx) {
    v.goal = goal;
    v.phase = Phase::Travel;
    v.work_prop = None;
    v.actor.activity = Activity::None;
    let world = &*ctx.world;
    let vil = &world.villages[v.village.min(world.villages.len() - 1)];
    let (cx, cy) = (vil.cx, vil.cy);
    let home_door = door_of(world, v.home);
    let target: Option<(i32, i32)> = match goal {
        Goal::Sleep => Some(home_door),
        Goal::Eat => {
            let h = ctx.clock.hour();
            if v.gold > 12 && (17.5..21.5).contains(&h) && ctx.rng.chance(0.45) {
                building_door(world, v.village, BuildingKind::Inn).or(Some(home_door))
            } else {
                Some(home_door)
            }
        }
        Goal::Work => work_target(v, ctx),
        Goal::Socialize(id) => ctx.seen.iter().find(|s| s.id == id).map(|s| ((s.x / TILE as f32) as i32, (s.y / TILE as f32) as i32)),
        Goal::Leisure => {
            let r = ctx.rng.below(3);
            if r == 0 {
                random_tile_near(world, ctx.rng, cx, cy, 3, true)
            } else {
                random_tile_near(world, ctx.rng, cx, cy, vil.radius + 4, false)
            }
        }
        Goal::Pray => world
            .props
            .iter()
            .position(|p| p.kind == PropKind::Shrine && (p.x - cx).abs() < 10 && (p.y - cy).abs() < 10)
            .and_then(|i| beside(world, world.props[i].x, world.props[i].y))
            .or_else(|| building_door(world, v.village, BuildingKind::Chapel)),
        Goal::Play => random_tile_near(world, ctx.rng, cx, cy + 2, 6, true),
        Goal::Shop => building_door(world, v.village, BuildingKind::Store),
        Goal::Visit(other) => world.villages.get(other).map(|o| (o.cx, o.cy + 2)),
        Goal::Idle => None,
    };
    let Some(target) = target else {
        v.goal = Goal::Idle;
        v.think_cd = 0.5;
        return;
    };
    let target = if ctx.world.walkable(target.0, target.1) { target } else { nearest_walkable(ctx.world, target).unwrap_or(target) };
    v.target = target;
    if v.inside.is_some() {
        exit_building(v, ctx.world);
    }
    if !v.actor.path_to(ctx.world, ctx.pf, target, 30_000) {
        v.goal = Goal::Idle;
        v.think_cd = 1.0 + ctx.rng.f32() * 2.0;
    }
}

fn work_target(v: &mut Villager, ctx: &mut Ctx) -> Option<(i32, i32)> {
    let world = &*ctx.world;
    let vil = &world.villages[v.village];
    let (cx, cy) = (vil.cx, vil.cy);
    match v.job {
        Job::Farmer => {
            if vil.fields.is_empty() {
                return random_tile_near(world, ctx.rng, cx, cy, 8, true);
            }
            let (fx, fy, fw, fh) = vil.fields[ctx.rng.below(vil.fields.len())];
            let x = fx + ctx.rng.range(0, fw);
            let y = fy + ctx.rng.range(0, fh);
            // Stand beside the crop row.
            let idx = world.props.iter().position(|p| p.alive && p.kind == PropKind::Crop && p.x == x && p.y == y);
            v.work_prop = idx;
            Some((x, y))
        }
        Job::Woodcutter => {
            let p = find_prop_near(world, ctx.rng, cx, cy, 26, |k| k.is_tree())?;
            v.work_prop = Some(p);
            beside(world, world.props[p].x, world.props[p].y)
        }
        Job::Miner => {
            let p = find_prop_near(world, ctx.rng, cx, cy, 34, |k| matches!(k, PropKind::Rock | PropKind::Boulder | PropKind::OreRock))?;
            v.work_prop = Some(p);
            beside(world, world.props[p].x, world.props[p].y)
        }
        Job::Fisher => {
            for _ in 0..60 {
                let x = cx + ctx.rng.range(-24, 25);
                let y = cy + ctx.rng.range(-24, 25);
                if world.walkable(x, y) && [(0, 1), (1, 0), (-1, 0), (0, -1)].iter().any(|(dx, dy)| world.terrain(x + dx, y + dy).is_water() && world.level(x + dx, y + dy) == world.level(x, y)) {
                    return Some((x, y));
                }
            }
            random_tile_near(world, ctx.rng, cx, cy, 6, true)
        }
        Job::Smith => building_door(world, v.village, BuildingKind::Smithy),
        Job::Merchant => building_door(world, v.village, BuildingKind::Store),
        Job::Innkeeper => building_door(world, v.village, BuildingKind::Inn),
        Job::Priest => building_door(world, v.village, BuildingKind::Chapel),
        Job::Guard => {
            // Patrol roads and the plaza.
            for _ in 0..40 {
                let x = cx + ctx.rng.range(-vil.radius - 4, vil.radius + 5);
                let y = cy + ctx.rng.range(-vil.radius - 4, vil.radius + 5);
                if world.walkable(x, y) && world.flags(x, y) & (flag::ROAD | flag::PLAZA) != 0 {
                    return Some((x, y));
                }
            }
            random_tile_near(world, ctx.rng, cx, cy, 6, true)
        }
        Job::Adventurer => random_tile_near(world, ctx.rng, cx, cy, 28, false),
        _ => None,
    }
}

fn enter_building(v: &mut Villager, prop: usize, world: &World) {
    v.inside = Some(prop);
    v.actor.hidden = true;
    let d = door_of(world, prop);
    v.actor.x = (d.0 * TILE + TILE / 2) as f32;
    v.actor.y = (d.1 * TILE + TILE / 2) as f32;
}

pub fn exit_building(v: &mut Villager, world: &World) {
    if let Some(p) = v.inside.take() {
        let d = door_of(world, p);
        v.actor.x = (d.0 * TILE + TILE / 2) as f32;
        v.actor.y = (d.1 * TILE + TILE / 2 + 6) as f32;
    }
    v.actor.hidden = false;
}

fn building_at_door(world: &World, village: usize, tile: (i32, i32)) -> Option<usize> {
    let vil = world.villages.get(village)?;
    vil.buildings.iter().find(|b| b.door == tile).map(|b| b.prop)
}

/// Arrived: start performing the goal.
fn arrive(v: &mut Villager, ctx: &mut Ctx) {
    v.phase = Phase::Perform;
    let r = &mut *ctx.rng;
    match v.goal {
        Goal::Sleep => {
            let home = v.home;
            enter_building(v, home, ctx.world);
            v.timer = 60.0 * 10.0;
        }
        Goal::Eat => {
            if let Some(b) = building_at_door(ctx.world, v.village, v.target) {
                enter_building(v, b, ctx.world);
            } else {
                let home = v.home;
                enter_building(v, home, ctx.world);
            }
            v.timer = r.range_f(25.0, 50.0);
        }
        Goal::Work => {
            v.timer = r.range_f(70.0, 150.0);
            v.actor.activity = match v.job {
                Job::Farmer => Activity::Hoe,
                Job::Woodcutter => Activity::Chop,
                Job::Miner => Activity::Mine,
                Job::Fisher => Activity::Fish,
                Job::Smith => Activity::Smith,
                Job::Innkeeper => Activity::Sweep,
                Job::Priest => Activity::Pray,
                _ => Activity::None,
            };
            if let Some(p) = v.work_prop {
                let pr = &ctx.world.props[p];
                let (px, py) = ((pr.x * TILE + TILE / 2) as f32, (pr.y * TILE + TILE / 2) as f32);
                v.actor.face_toward(px, py);
            }
            if v.job == Job::Fisher {
                // Face the water.
                let (tx, ty) = v.actor.tile();
                for (dx, dy) in [(0, 1), (1, 0), (-1, 0), (0, -1)] {
                    if ctx.world.terrain(tx + dx, ty + dy).is_water() {
                        v.actor.face_toward(v.actor.x + dx as f32 * 10.0, v.actor.y + dy as f32 * 10.0);
                        break;
                    }
                }
            }
            if matches!(v.job, Job::Guard | Job::Adventurer | Job::Merchant) {
                v.timer = r.range_f(20.0, 60.0);
            }
        }
        Goal::Socialize(_) => {
            v.timer = r.range_f(18.0, 40.0);
            v.actor.activity = Activity::Talk;
        }
        Goal::Leisure => {
            v.timer = r.range_f(20.0, 70.0);
            if r.chance(0.3) {
                v.emote(Emote::Music, 3.0);
            }
        }
        Goal::Pray => {
            v.timer = r.range_f(20.0, 40.0);
            v.actor.activity = Activity::Pray;
            v.actor.facing = super::actor::Facing::Up;
            v.emote(Emote::Sparkle, 2.5);
        }
        Goal::Play => {
            v.timer = r.range_f(8.0, 20.0);
            v.actor.activity = Activity::Play;
        }
        Goal::Shop => {
            if let Some(b) = building_at_door(ctx.world, v.village, v.target) {
                enter_building(v, b, ctx.world);
            }
            v.timer = r.range_f(15.0, 30.0);
        }
        Goal::Visit(_) | Goal::Idle => {
            v.timer = 10.0;
        }
    }
}

/// Apply the result of a finished goal.
fn finish(v: &mut Villager, ctx: &mut Ctx) {
    let season = ctx.clock.season();
    let vid = v.village.min(ctx.stocks.len().saturating_sub(1));
    match v.goal {
        Goal::Eat => {
            let stock = &mut ctx.stocks[vid];
            if stock.food >= 1.0 {
                stock.food -= 1.0;
                v.needs.hunger = (v.needs.hunger + 60.0).min(100.0);
            } else {
                v.needs.hunger = (v.needs.hunger + 25.0).min(100.0);
                v.emote(Emote::Sad, 3.0);
            }
            if v.inside.map_or(false, |b| ctx.world.props[b].state == BuildingKind::Inn as u8) {
                v.gold -= 8;
                ctx.stocks[vid].gold += 8.0;
                v.needs.fun = (v.needs.fun + 20.0).min(100.0);
                v.needs.social = (v.needs.social + 15.0).min(100.0);
            }
        }
        Goal::Work => {
            let stock = &mut ctx.stocks[vid];
            let wage = 6 + (v.traits.diligent * 6.0) as i32;
            v.gold += wage;
            match v.job {
                Job::Farmer => {
                    stock.food += if season == 3 { 0.6 } else { 2.2 };
                    if let Some(p) = v.work_prop {
                        let pr = &mut ctx.world.props[p];
                        if pr.kind == PropKind::Crop {
                            if pr.state >= 3 {
                                pr.state = 0;
                                stock.food += 3.0;
                                v.emote(Emote::Sparkle, 2.0);
                            } else if season != 3 && ctx.rng.chance(0.5) {
                                pr.state += 1;
                            }
                            ctx.changed_props.push(p);
                        }
                    }
                }
                Job::Fisher => stock.food += 2.0,
                Job::Woodcutter => {
                    stock.wood += 1.0;
                    if let Some(p) = v.work_prop {
                        let pr = &mut ctx.world.props[p];
                        if pr.alive && pr.kind.is_tree() {
                            pr.hp = pr.hp.saturating_sub(2);
                            if pr.hp == 0 {
                                pr.kind = PropKind::Stump;
                                pr.hp = 2;
                                pr.state = 0;
                                stock.wood += 4.0;
                                ctx.changed_props.push(p);
                            }
                        }
                    }
                }
                Job::Miner => {
                    stock.ore += 1.5;
                }
                Job::Smith => {
                    if stock.ore >= 1.0 {
                        stock.ore -= 1.0;
                        stock.goods += 1.0;
                    }
                }
                Job::Merchant => {
                    if stock.goods >= 1.0 {
                        stock.goods -= 1.0;
                        stock.gold += 12.0;
                    }
                }
                _ => {}
            }
        }
        Goal::Leisure | Goal::Play => {
            v.needs.fun = (v.needs.fun + 25.0).min(100.0);
        }
        Goal::Socialize(other) => {
            ctx.talks.push((v.id, other));
        }
        Goal::Pray => {
            v.needs.fun = (v.needs.fun + 10.0).min(100.0);
        }
        Goal::Shop => {
            v.gold -= 10;
            ctx.stocks[vid].gold += 10.0;
            v.needs.fun = (v.needs.fun + 18.0).min(100.0);
            if ctx.stocks[vid].goods >= 0.5 {
                ctx.stocks[vid].goods -= 0.5;
            }
        }
        _ => {}
    }
    v.actor.activity = Activity::None;
    if v.inside.is_some() {
        exit_building(v, ctx.world);
    }
    v.goal = Goal::Idle;
    v.phase = Phase::Travel;
    v.think_cd = ctx.rng.range_f(0.2, 1.5);
}

/// Advance one villager. `dt` is real seconds scaled by game speed; `mins`
/// is in-game minutes elapsed.
pub fn update(v: &mut Villager, ctx: &mut Ctx, dt: f32, mins: f32) {
    // Needs decay.
    let sleeping = v.goal == Goal::Sleep && v.phase == Phase::Perform;
    v.needs.hunger = (v.needs.hunger - 0.055 * mins).max(0.0);
    v.needs.energy = if sleeping { (v.needs.energy + 0.25 * mins).min(100.0) } else { (v.needs.energy - 0.06 * mins).max(0.0) };
    v.needs.social = (v.needs.social - 0.045 * mins * (0.5 + v.traits.sociable)).max(0.0);
    v.needs.fun = (v.needs.fun - 0.035 * mins).max(0.0);
    if v.emote_t > 0.0 {
        v.emote_t -= dt;
        if v.emote_t <= 0.0 {
            v.emote = Emote::None;
        }
    }
    v.actor.update_anim(dt);

    match v.phase {
        Phase::Travel => {
            if v.goal == Goal::Idle {
                v.think_cd -= mins;
                if v.think_cd <= 0.0 {
                    decide(v, ctx);
                }
                return;
            }
            // Re-target moving conversation partners.
            if let Goal::Socialize(id) = v.goal {
                if let Some(s) = ctx.seen.iter().find(|s| s.id == id) {
                    let d = ((s.x - v.actor.x).powi(2) + (s.y - v.actor.y).powi(2)).sqrt();
                    if d < TILE as f32 * 1.4 {
                        v.actor.clear_path();
                        v.actor.face_toward(s.x, s.y);
                        ctx.invites.push((id, v.id));
                        arrive(v, ctx);
                        return;
                    }
                    if !s.available && d > TILE as f32 * 3.0 {
                        v.goal = Goal::Idle;
                        v.think_cd = 0.5;
                        v.actor.clear_path();
                        return;
                    }
                }
            }
            if v.actor.follow(dt) {
                arrive(v, ctx);
            }
        }
        Phase::Perform => {
            v.timer -= mins;
            match v.goal {
                Goal::Sleep => {
                    let h = ctx.clock.hour();
                    let wake = 5.5 + (1.0 - v.traits.diligent) * 2.0 + if v.job == Job::Child { 0.8 } else { 0.0 };
                    if (wake..wake + 4.0).contains(&h) && v.needs.energy > 70.0 {
                        v.timer = 0.0;
                    } else {
                        v.timer = 1.0;
                    }
                }
                Goal::Socialize(_) => {
                    v.needs.social = (v.needs.social + 0.9 * mins).min(100.0);
                    if v.emote == Emote::None && ctx.rng.chance(0.004 * mins.max(1.0)) {
                        v.emote(if ctx.rng.chance(0.75) { Emote::Chat } else { Emote::Music }, 2.0);
                    }
                }
                Goal::Play => {
                    // Kids dart about between short hops.
                    if !v.actor.has_path() && ctx.rng.chance(0.05) {
                        let (tx, ty) = v.actor.tile();
                        if let Some(t) = random_tile_near(ctx.world, ctx.rng, tx, ty, 3, true) {
                            v.actor.path_to(ctx.world, ctx.pf, t, 400);
                        }
                    }
                    v.actor.follow(dt);
                    if v.emote == Emote::None && ctx.rng.chance(0.002) {
                        v.emote(Emote::Music, 1.5);
                    }
                }
                Goal::Work if v.job == Job::Guard || v.job == Job::Adventurer => {
                    if !v.actor.has_path() && ctx.rng.chance(0.03) {
                        if let Some(t) = work_target(v, ctx) {
                            v.actor.path_to(ctx.world, ctx.pf, t, 4000);
                        }
                    }
                    v.actor.follow(dt);
                }
                _ => {}
            }
            if v.timer <= 0.0 {
                finish(v, ctx);
            }
        }
    }
}

/// Is this villager free to be pulled into a conversation?
pub fn available(v: &Villager) -> bool {
    v.alive
        && v.inside.is_none()
        && !matches!(v.goal, Goal::Sleep | Goal::Socialize(_))
        && !(v.goal == Goal::Work && v.phase == Phase::Perform && v.traits.diligent > 0.6)
}

/// Accept an invitation to chat.
pub fn accept_invite(v: &mut Villager, from: u32, fx: f32, fy: f32, rng: &mut Rng) {
    v.goal = Goal::Socialize(from);
    v.phase = Phase::Perform;
    v.timer = rng.range_f(18.0, 40.0);
    v.actor.clear_path();
    v.actor.activity = Activity::Talk;
    v.actor.face_toward(fx, fy);
    v.emote(Emote::Chat, 1.5);
}

pub fn is_water_edge(world: &World, x: i32, y: i32) -> bool {
    [(0, 1), (1, 0), (-1, 0), (0, -1)].iter().any(|(dx, dy)| world.terrain(x + dx, y + dy) == Terrain::Shallow)
}
