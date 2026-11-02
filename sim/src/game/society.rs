//! Social life: conversation outcomes, romance, weddings, births, aging and
//! death, newcomers, and the village economy's daily cycle.

use super::actor::{nearest_walkable, stand_pos};
use super::ai::Stock;
use super::clock::{SEASON_NAMES, WINTER, YEARS_PER_DAY};
use super::villager::{self, look_stage, top_up, Emote, Goal, Job, Phase, Traits, Villager, ADULT_AGE, ELDER_AGE, TEEN_AGE};
use super::Game;
use crate::art::people::Look;
use crate::names;
use crate::world::PropKind;

/// Child characters are disabled for now: no births, households start with
/// grown children only, and newcomers keep villages growing.
const CHILDREN: bool = false;

/// How many chronicle entries are kept; older ones drop off.
const CHRONICLE_LEN: usize = 400;

/// A notable happening, shown in the chronicle and as a UI notification.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Event {
    pub day: u32,
    pub minute: u32,
    pub kind: &'static str,
    pub text: String,
    pub who: Vec<u32>,
}

impl Game {
    pub fn chronicle(&mut self, kind: &'static str, text: String, who: Vec<u32>) {
        let e = Event { day: self.clock.day(), minute: self.clock.minute_of_day() as u32, kind, text, who };
        self.events_out.push(e.clone());
        self.chronicle.push(e);
        if self.chronicle.len() > CHRONICLE_LEN {
            self.chronicle.remove(0);
        }
    }

    /// Resolve finished conversations between pairs of villagers.
    pub fn resolve_talks(&mut self, talks: Vec<(u32, u32)>) {
        for (a, b) in talks {
            let (Some(ia), Some(ib)) = (self.index_of(a), self.index_of(b)) else { continue };
            // Each pair is resolved once (from the lower id's side).
            if a > b && self.villagers[ib].goal == Goal::Socialize(a) {
                continue;
            }
            let ta: Traits = self.villagers[ia].traits;
            let tb: Traits = self.villagers[ib].traits;
            let compat = ta.compatibility(&tb);
            let roll = self.rng.f32();
            let argue = compat < 0.15 && roll < (ta.temper + tb.temper) * 0.35;
            if argue {
                self.villagers[ia].adjust_opinion(b, -9.0);
                self.villagers[ib].adjust_opinion(a, -9.0);
                self.villagers[ia].emote(Emote::Anger, 3.0);
                self.villagers[ib].emote(Emote::Anger, 3.0);
                let (na, nb) = (self.villagers[ia].name(), self.villagers[ib].name());
                if self.rng.chance(0.35) {
                    let topic = *self.rng.pick(&["the price of bread", "whose turn it was at the well", "a borrowed axe", "the noise at the inn", "the harvest", "an old rumour"]);
                    self.chronicle("quarrel", format!("{na} and {nb} argued about {topic}."), vec![a, b]);
                }
                self.villagers[ia].remember(format!("Argued with {nb}"));
                self.villagers[ib].remember(format!("Argued with {na}"));
                continue;
            }
            let gain = 2.5 + compat * 4.0 + roll * 3.0;
            self.villagers[ia].adjust_opinion(b, gain);
            self.villagers[ib].adjust_opinion(a, gain);
            let (oa, ob) = (self.villagers[ia].opinion_of(b), self.villagers[ib].opinion_of(a));
            let (va, vb) = (&self.villagers[ia], &self.villagers[ib]);
            let single = |v: &Villager| v.partner.is_none() && v.is_adult();
            let related = va.parents.iter().any(|p| vb.parents.contains(p)) || va.parents.contains(&b) || vb.parents.contains(&a);
            // Romance blossoms between compatible singles who like each other.
            if single(va) && single(vb) && !related && (va.age - vb.age).abs() < 15.0 && oa > 62.0 && ob > 62.0 {
                let attracted = va.female != vb.female || self.rng.chance(0.25);
                if attracted && va.dating.is_none() && vb.dating.is_none() {
                    self.villagers[ia].dating = Some(b);
                    self.villagers[ib].dating = Some(a);
                    self.villagers[ia].emote(Emote::Heart, 4.0);
                    self.villagers[ib].emote(Emote::Heart, 4.0);
                    let (na, nb) = (self.villagers[ia].name(), self.villagers[ib].name());
                    self.chronicle("romance", format!("{na} and {nb} have started courting."), vec![a, b]);
                } else if self.villagers[ia].dating == Some(b) {
                    self.villagers[ia].emote(Emote::Heart, 3.0);
                    self.villagers[ib].emote(Emote::Heart, 3.0);
                    if oa > 88.0 && ob > 88.0 && self.rng.chance(0.3) {
                        self.wed(ia, ib);
                    }
                }
            } else if oa > 75.0 && self.rng.chance(0.15) {
                self.villagers[ia].emote(Emote::Heart, 2.0);
            }
        }
    }

    fn wed(&mut self, ia: usize, ib: usize) {
        let (a, b) = (self.villagers[ia].id, self.villagers[ib].id);
        self.villagers[ia].partner = Some(b);
        self.villagers[ib].partner = Some(a);
        self.villagers[ia].dating = None;
        self.villagers[ib].dating = None;
        // The younger moves into the elder's home and takes their name.
        let (stay, mov) = if self.villagers[ia].age >= self.villagers[ib].age { (ia, ib) } else { (ib, ia) };
        let home = self.villagers[stay].home;
        let fam = self.villagers[stay].family.clone();
        let village = self.villagers[stay].village;
        self.villagers[mov].home = home;
        self.villagers[mov].village = village;
        self.villagers[mov].family = fam;
        for i in [ia, ib] {
            self.villagers[i].emote(Emote::Heart, 6.0);
            self.villagers[i].remember("Got married".into());
        }
        let (na, nb) = (self.villagers[ia].first.clone(), self.villagers[ib].first.clone());
        let vname = self.world.villages[village].name.clone();
        self.chronicle("wedding", format!("Wedding bells in {vname}! {na} and {nb} are now married."), vec![a, b]);
        // The whole village is happy.
        for v in self.villagers.iter_mut().filter(|v| v.village == village && v.alive && v.inside.is_none()) {
            if self.rng.chance(0.5) {
                v.emote(Emote::Sparkle, 3.0);
            }
            top_up(&mut v.needs.fun, 15.0);
        }
    }

    /// Daily tick: aging, births, deaths, newcomers, regrowth, economy.
    pub fn new_day(&mut self) {
        let day = self.clock.day();
        let season = self.clock.season();
        for v in self.villagers.iter_mut() {
            v.talked_today = false;
        }
        // Aging and life stages.
        let n = self.villagers.len();
        let mut grown_up: Vec<usize> = Vec::new();
        let mut deaths: Vec<usize> = Vec::new();
        let mut births: Vec<usize> = Vec::new();
        for i in 0..n {
            let v = &mut self.villagers[i];
            if !v.alive {
                continue;
            }
            let before = v.age;
            v.age += YEARS_PER_DAY;
            let reached = |threshold: f32| before < threshold && v.age >= threshold;
            // Children take up a trade at fifteen.
            if reached(15.0) && v.job == Job::Child {
                grown_up.push(i);
            }
            if reached(ADULT_AGE) {
                v.look.mature = true;
                self.dirty_looks.push(v.id);
            }
            if reached(TEEN_AGE) || reached(ELDER_AGE) {
                // Re-draw the sprite for the new life stage; elders go silver.
                v.look.age = look_stage(v.age);
                if v.age >= ELDER_AGE {
                    v.look.hair = 5;
                }
                self.dirty_looks.push(v.id);
            }
            if v.age > 62.0 {
                let p = ((v.age - 62.0) * 0.004).min(0.25);
                if self.rng.chance(p) {
                    deaths.push(i);
                }
            }
            if v.pregnant > 0.0 {
                v.pregnant -= 1.0;
                if v.pregnant <= 0.0 {
                    births.push(i);
                }
            } else if CHILDREN && v.female && v.partner.is_some() && v.age < 44.0 && v.children.len() < 4 {
                let partner_male = v.partner.and_then(|p| self.villagers.iter().find(|o| o.id == p)).is_some_and(|o| !o.female);
                if partner_male && self.rng.chance(0.05) {
                    self.villagers[i].pregnant = 6.0;
                }
            }
        }
        for i in grown_up {
            let job = [Job::Farmer, Job::Woodcutter, Job::Fisher, Job::Miner, Job::Adventurer, Job::Guard][self.rng.below(6)];
            let v = &mut self.villagers[i];
            v.job = job;
            let mut rng = self.rng.fork(v.id as u64);
            job.dress(&mut v.look, &mut rng);
            let (name, jl) = (v.name(), job.label());
            let id = v.id;
            self.dirty_looks.push(id);
            self.chronicle("growth", format!("{name} came of age and became a {jl}."), vec![id]);
        }
        for i in births {
            self.birth(i);
        }
        for i in deaths {
            self.death(i);
        }
        // Newcomers keep small villages alive; with no births they are how
        // villages grow at all.
        for vid in 0..self.world.villages.len() {
            let pop = self.villagers.iter().filter(|v| v.alive && v.village == vid).count();
            if (pop < 10 && self.rng.chance(0.4)) || self.rng.chance(0.06) {
                self.newcomer(vid);
            }
        }
        // Stumps regrow; crops grow on their own in the growing seasons.
        for (i, p) in self.world.props.iter_mut().enumerate() {
            match p.kind {
                PropKind::Stump if p.alive => {
                    p.state += 1;
                    if p.state > 8 && season != WINTER {
                        p.kind = PropKind::Oak;
                        p.hp = 5;
                        p.state = 0;
                        self.changed_props.push(i);
                    }
                }
                PropKind::Crop if season != WINTER && p.state < 3 && self.rng.chance(0.35) => {
                    p.state += 1;
                    self.changed_props.push(i);
                }
                _ => {}
            }
        }
        // Economy: spoilage and a trickle of trade.
        for st in self.stocks.iter_mut() {
            st.food *= 0.97;
            st.goods *= 0.99;
        }
        if self.clock.day_of_season() == 1 && day > 0 {
            let s = SEASON_NAMES[season as usize];
            self.chronicle("season", format!("{s} has arrived."), vec![]);
            self.season_changed = true;
        }
    }

    fn birth(&mut self, mi: usize) {
        let mother = self.villagers[mi].clone();
        let Some(father) = mother.partner.and_then(|p| self.villagers.iter().find(|o| o.id == p).cloned()) else { return };
        let culture = self.world.villages[mother.village].culture as usize;
        let id = self.next_id;
        self.next_id += 1;
        let mut rng = self.rng.fork(id as u64);
        let female = rng.chance(0.5);
        let mut kid = villager::make(id, &mut rng, culture, &mother.family, female, 0.0, Job::Child, mother.village, mother.home, mother.actor.x, mother.actor.y);
        kid.parents = vec![mother.id, father.id];
        kid.traits = Traits::blend(&mother.traits, &father.traits, &mut rng);
        kid.look = Look {
            skin: if rng.chance(0.5) { mother.look.skin } else { father.look.skin },
            hair: if rng.chance(0.5) { mother.look.hair } else { father.look.hair },
            age: 0,
            mature: false,
            ..kid.look
        };
        // Babies aren't born silver-haired.
        if kid.look.hair == 5 {
            kid.look.hair = rng.below(5) as u8;
        }
        kid.adjust_opinion(mother.id, 90.0);
        kid.adjust_opinion(father.id, 90.0);
        kid.actor.hidden = true;
        kid.inside = Some(mother.home);
        kid.goal = Goal::Sleep;
        kid.phase = Phase::Perform;
        kid.timer = 60.0;
        kid.actor.sheet = self.sprites.add_sheet(&kid.look);
        let name = kid.first.clone();
        for v in self.villagers.iter_mut() {
            if v.id == mother.id || v.id == father.id {
                v.children.push(id);
                v.adjust_opinion(id, 95.0);
                v.emote(Emote::Heart, 6.0);
                v.remember(format!("Welcomed baby {name}"));
            }
        }
        let vname = self.world.villages[mother.village].name.clone();
        let fam = mother.family.clone();
        self.villagers.push(kid);
        self.chronicle("birth", format!("A baby was born in {vname}: {name} {fam}, child of {} and {}.", mother.first, father.first), vec![id, mother.id, father.id]);
    }

    fn death(&mut self, i: usize) {
        let v = self.villagers[i].clone();
        self.villagers[i].alive = false;
        self.villagers[i].actor.hidden = true;
        let age = v.age as u32;
        let vname = self.world.villages[v.village].name.clone();
        self.chronicle("death", format!("{} of {vname} passed away peacefully at {age}.", v.name()), vec![v.id]);
        for o in self.villagers.iter_mut() {
            if !o.alive {
                continue;
            }
            if o.partner == Some(v.id) {
                o.partner = None;
                o.remember(format!("Mourning {}", v.first));
                o.needs.fun = 10.0;
            }
            if o.opinion_of(v.id) > 50.0 {
                o.emote(Emote::Sad, 6.0);
                o.remember(format!("Misses {}", v.first));
            }
        }
    }

    fn newcomer(&mut self, vid: usize) {
        let homes = self.world.villages[vid].homes();
        if homes.is_empty() {
            return;
        }
        let home = homes[self.rng.below(homes.len())];
        let culture = self.rng.below(names::CULTURES);
        let id = self.next_id;
        self.next_id += 1;
        let mut rng = self.rng.fork(id as u64 ^ 0xabc);
        let fam = names::family(&mut rng, culture);
        let job = [Job::Farmer, Job::Adventurer, Job::Woodcutter, Job::Fisher][rng.below(4)];
        // They walk in along the road from the village edge.
        let vil = &self.world.villages[vid];
        let edge = (vil.cx + vil.radius + 6, vil.cy + 2);
        let (x, y) = stand_pos(nearest_walkable(&self.world, edge).unwrap_or((vil.cx, vil.cy + 2)));
        let (fem, age) = (rng.chance(0.5), rng.range_f(19.0, 35.0));
        let mut v = villager::make(id, &mut rng, culture, &fam, fem, age, job, vid, home, x, y);
        v.actor.sheet = self.sprites.add_sheet(&v.look);
        let (name, jl, vname) = (v.name(), job.label(), vil.name.clone());
        self.villagers.push(v);
        self.chronicle("arrival", format!("{name}, a wandering {jl}, settled in {vname}."), vec![id]);
    }

    pub fn starting_stocks(&self) -> Vec<Stock> {
        self.world
            .villages
            .iter()
            .map(|v| {
                let pop = self.villagers.iter().filter(|p| p.village == v.id).count() as f32;
                Stock { food: pop * 3.0, wood: 10.0, ore: 4.0, goods: 4.0, gold: 100.0 }
            })
            .collect()
    }
}
