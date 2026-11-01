//! Villagers: identity, personality, needs, family and relationships.

use serde::{Deserialize, Serialize};

use super::actor::Actor;
use crate::art::people::{self, held, outfit, Look};
use crate::names;
use crate::rng::Rng;
use crate::world::villages::BuildingKind;
use crate::world::{World, TILE};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Job {
    Farmer,
    Woodcutter,
    Miner,
    Fisher,
    Smith,
    Merchant,
    Innkeeper,
    Guard,
    Priest,
    Adventurer,
    Healer,
    Scholar,
    Mage,
    Cook,
    Child,
    Retired,
}

impl Job {
    /// Working professions (everyone but children and the retired).
    pub const WORKING: [Job; 14] = [
        Job::Farmer,
        Job::Woodcutter,
        Job::Miner,
        Job::Fisher,
        Job::Smith,
        Job::Merchant,
        Job::Innkeeper,
        Job::Guard,
        Job::Priest,
        Job::Adventurer,
        Job::Healer,
        Job::Scholar,
        Job::Mage,
        Job::Cook,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Job::Farmer => "Farmer",
            Job::Woodcutter => "Woodcutter",
            Job::Miner => "Miner",
            Job::Fisher => "Fisher",
            Job::Smith => "Blacksmith",
            Job::Merchant => "Merchant",
            Job::Innkeeper => "Innkeeper",
            Job::Guard => "Guard",
            Job::Priest => "Priest",
            Job::Adventurer => "Adventurer",
            Job::Healer => "Healer",
            Job::Scholar => "Scholar",
            Job::Mage => "Mage",
            Job::Cook => "Cook",
            Job::Child => "Child",
            Job::Retired => "Retired",
        }
    }
    /// Outfit and hat that make the job readable at a glance.
    /// Outfit, hat and held item that make the job readable at a glance.
    /// Each profession's signature look: outfit, headwear, palette and tool
    /// chosen so the job reads at a glance, with a little personal variety.
    pub fn dress(self, look: &mut Look, rng: &mut Rng) {
        look.held = held::NONE;
        look.glasses = false;
        if look.hat != 7 && look.hat != 9 {
            look.hat = 0;
        }
        match self {
            Job::Farmer => {
                look.hat = if rng.chance(0.8) { 1 } else { 0 };
                look.outfit = if look.female { outfit::DRESS } else { *rng.pick(&[outfit::VEST, outfit::TUNIC]) };
                look.top = *rng.pick(&[2, 3, 0]);
                look.bottom = *rng.pick(&[3, 0]);
                look.held = if rng.chance(0.7) { held::HOE } else { held::BASKET };
            }
            Job::Woodcutter => {
                look.outfit = outfit::TUNIC;
                look.top = 1;
                look.bottom = 3;
                if rng.chance(0.4) {
                    look.hat = 2;
                    look.hat_color = 2;
                }
                look.held = held::AXE;
            }
            Job::Miner => {
                look.outfit = outfit::TUNIC;
                look.hat = 11;
                look.top = 8;
                look.bottom = 3;
                look.held = held::PICKAXE;
            }
            Job::Fisher => {
                look.outfit = *rng.pick(&[outfit::TUNIC, outfit::VEST]);
                look.hat = 1;
                look.top = *rng.pick(&[9, 0]);
                look.bottom = *rng.pick(&[3, 8]);
                look.held = held::ROD;
            }
            Job::Smith => {
                look.outfit = outfit::SMITH;
                look.bottom = 8;
                look.held = held::HAMMER;
                if rng.chance(0.5) {
                    look.hat = 2;
                    look.hat_color = 1;
                }
            }
            Job::Merchant => {
                look.outfit = outfit::APRON;
                look.hat = 5;
                look.hat_color = 6;
                look.glasses = rng.chance(0.4);
                look.held = held::BASKET;
            }
            Job::Innkeeper => {
                if look.female {
                    look.outfit = outfit::DIRNDL;
                    look.top = *rng.pick(&[1, 0, 2]);
                    look.bottom = *rng.pick(&[2, 0, 6]);
                    look.hat = 10;
                } else {
                    look.outfit = outfit::APRON;
                }
                look.held = held::MUG;
            }
            Job::Guard => {
                look.outfit = outfit::ARMOR;
                look.hat = if look.female && rng.chance(0.5) { 0 } else { 3 };
                look.hat_color = 1;
                look.held = held::SPEAR;
            }
            Job::Priest => {
                look.outfit = outfit::ROBE;
                look.top = 7;
                look.held = held::BOOK;
                if rng.chance(0.5) {
                    look.hat = 4;
                    look.hat_color = 7;
                }
            }
            Job::Adventurer => {
                look.outfit = if look.female && rng.chance(0.4) { outfit::VEST } else { outfit::CLOAK };
                look.hat_color = *rng.pick(&[1, 2, 5, 9]);
                look.held = held::SWORD;
                if rng.chance(0.25) {
                    look.hat = 4;
                }
            }
            Job::Healer => {
                look.outfit = outfit::ROBE;
                look.top = 7;
                look.held = held::STAFF;
                if rng.chance(0.4) {
                    look.hat = 4;
                    look.hat_color = 7;
                }
            }
            Job::Scholar => {
                look.outfit = outfit::VEST;
                look.glasses = true;
                look.held = held::BOOK;
            }
            Job::Mage => {
                look.outfit = outfit::ROBE;
                look.top = *rng.pick(&[5, 0, 8]);
                look.hat = 6;
                look.hat_color = look.top;
                look.held = held::STAFF;
            }
            Job::Cook => {
                look.outfit = outfit::APRON;
                look.hat = 8;
                look.held = held::LADLE;
            }
            Job::Child => {
                if look.female && rng.chance(0.5) {
                    look.hat = if rng.chance(0.5) { 7 } else { 9 };
                }
            }
            Job::Retired => {}
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Traits {
    /// 0..1 each.
    pub sociable: f32,
    pub diligent: f32,
    pub kind: f32,
    pub temper: f32,
    pub curious: f32,
    pub devout: f32,
}

impl Traits {
    pub fn random(rng: &mut Rng) -> Traits {
        let mut t = || (rng.f32() + rng.f32()) * 0.5;
        Traits { sociable: t(), diligent: t(), kind: t(), temper: t(), curious: t(), devout: t() }
    }
    pub fn blend(a: &Traits, b: &Traits, rng: &mut Rng) -> Traits {
        let mut m = |x: f32, y: f32| ((x + y) * 0.5 + (rng.f32() - 0.5) * 0.3).clamp(0.0, 1.0);
        Traits {
            sociable: m(a.sociable, b.sociable),
            diligent: m(a.diligent, b.diligent),
            kind: m(a.kind, b.kind),
            temper: m(a.temper, b.temper),
            curious: m(a.curious, b.curious),
            devout: m(a.devout, b.devout),
        }
    }
    /// Similar personalities get along; kindness smooths everything.
    pub fn compatibility(&self, o: &Traits) -> f32 {
        let d = (self.sociable - o.sociable).abs() + (self.diligent - o.diligent).abs() + (self.curious - o.curious).abs() + (self.devout - o.devout).abs();
        1.0 - d / 2.0 + (self.kind + o.kind - 1.0) * 0.5 - (self.temper + o.temper - 1.0) * 0.4
    }
    pub fn describe(&self) -> Vec<&'static str> {
        let mut v = Vec::new();
        if self.sociable > 0.65 { v.push("Outgoing") } else if self.sociable < 0.35 { v.push("Shy") }
        if self.diligent > 0.65 { v.push("Hardworking") } else if self.diligent < 0.35 { v.push("Laid-back") }
        if self.kind > 0.65 { v.push("Kind") } else if self.kind < 0.35 { v.push("Blunt") }
        if self.temper > 0.65 { v.push("Hot-headed") } else if self.temper < 0.3 { v.push("Calm") }
        if self.curious > 0.65 { v.push("Curious") }
        if self.devout > 0.7 { v.push("Devout") }
        if v.is_empty() {
            v.push("Easygoing");
        }
        v
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Needs {
    /// 100 = fully satisfied, 0 = desperate.
    pub hunger: f32,
    pub energy: f32,
    pub social: f32,
    pub fun: f32,
}

impl Needs {
    pub fn full(rng: &mut Rng) -> Needs {
        Needs { hunger: rng.range_f(60.0, 95.0), energy: rng.range_f(70.0, 100.0), social: rng.range_f(40.0, 90.0), fun: rng.range_f(40.0, 90.0) }
    }
    pub fn mood(&self) -> f32 {
        (self.hunger * 1.2 + self.energy + self.social * 0.8 + self.fun * 0.6) / 3.6
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Emote {
    None = 0,
    Heart = 1,
    Chat = 2,
    Music = 3,
    Zzz = 4,
    Anger = 5,
    Sweat = 6,
    Exclaim = 7,
    Question = 8,
    Sparkle = 9,
    Food = 10,
    Sad = 11,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum Goal {
    Idle,
    Sleep,
    Eat,
    Work,
    Socialize(u32),
    Leisure,
    Pray,
    Play,
    Shop,
    Visit(usize),
}

impl Goal {
    pub fn label(&self) -> &'static str {
        match self {
            Goal::Idle => "Thinking",
            Goal::Sleep => "Sleeping",
            Goal::Eat => "Having a meal",
            Goal::Work => "Working",
            Goal::Socialize(_) => "Chatting",
            Goal::Leisure => "Taking a stroll",
            Goal::Pray => "Praying",
            Goal::Play => "Playing",
            Goal::Shop => "Shopping",
            Goal::Visit(_) => "Travelling",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum Phase {
    Travel,
    Perform,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Villager {
    pub id: u32,
    pub first: String,
    pub family: String,
    pub female: bool,
    #[serde(default)]
    pub race: u8,
    pub age: f32,
    pub look: Look,
    pub traits: Traits,
    pub needs: Needs,
    pub job: Job,
    pub village: usize,
    /// Prop index of the home building.
    pub home: usize,
    pub actor: Actor,
    pub goal: Goal,
    pub phase: Phase,
    pub timer: f32,
    /// Target world tile for the current goal.
    pub target: (i32, i32),
    /// Prop being worked (tree, rock, crop) if any.
    pub work_prop: Option<usize>,
    pub partner: Option<u32>,
    pub dating: Option<u32>,
    pub parents: Vec<u32>,
    pub children: Vec<u32>,
    /// Opinions of others, -100..100.
    pub opinions: Vec<(u32, f32)>,
    pub player_opinion: f32,
    pub emote: Emote,
    pub emote_t: f32,
    pub inside: Option<usize>,
    pub pregnant: f32,
    pub gold: i32,
    pub think_cd: f32,
    pub alive: bool,
    /// Short memory of notable recent events, newest last.
    pub memories: Vec<String>,
    pub talked_today: bool,
    pub gifted_today: bool,
}

impl Villager {
    pub fn name(&self) -> String {
        format!("{} {}", self.first, self.family)
    }

    pub fn stage(&self) -> &'static str {
        let a = self.life_age();
        if a < 13.0 {
            "Child"
        } else if a < 18.0 {
            "Teen"
        } else if a < 60.0 {
            "Adult"
        } else {
            "Elder"
        }
    }

    /// Years of life per human-equivalent year: elves and demons are long-lived.
    pub fn life_mult(&self) -> f32 {
        race_life_mult(self.race)
    }

    /// Age in human-equivalent years (drives life stages).
    pub fn life_age(&self) -> f32 {
        self.age / self.life_mult()
    }

    pub fn is_adult(&self) -> bool {
        self.life_age() >= 18.0
    }

    pub fn opinion_of(&self, id: u32) -> f32 {
        self.opinions.iter().find(|(o, _)| *o == id).map(|(_, v)| *v).unwrap_or(0.0)
    }

    pub fn adjust_opinion(&mut self, id: u32, delta: f32) {
        if let Some(e) = self.opinions.iter_mut().find(|(o, _)| *o == id) {
            e.1 = (e.1 + delta).clamp(-100.0, 100.0);
        } else {
            self.opinions.push((id, delta.clamp(-100.0, 100.0)));
        }
    }

    pub fn emote(&mut self, e: Emote, secs: f32) {
        self.emote = e;
        self.emote_t = secs;
    }

    pub fn remember(&mut self, s: String) {
        self.memories.push(s);
        if self.memories.len() > 6 {
            self.memories.remove(0);
        }
    }
}

pub fn race_life_mult(race: u8) -> f32 {
    match race {
        people::RACE_ELF => 4.0,
        people::RACE_DWARF => 2.5,
        people::RACE_DEMON => 5.0,
        people::RACE_BEAST => 0.9,
        _ => 1.0,
    }
}

/// Pick a race for someone born in a village of this culture.
pub fn pick_race(rng: &mut Rng, culture: usize) -> u8 {
    let w: [f32; 5] = match culture {
        0 => [0.7, 0.08, 0.06, 0.13, 0.03],
        1 => [0.35, 0.03, 0.5, 0.09, 0.03],
        2 => [0.55, 0.05, 0.06, 0.3, 0.04],
        _ => [0.25, 0.6, 0.03, 0.1, 0.02],
    };
    rng.weighted(&w) as u8
}

/// Make a new villager of a given age. Sheet is assigned by the caller.
#[allow(clippy::too_many_arguments)]
pub fn make(id: u32, rng: &mut Rng, culture: usize, family: &str, female: bool, age: f32, job: Job, village: usize, home: usize, x: f32, y: f32) -> Villager {
    let race = pick_race(rng, culture);
    make_race(id, rng, culture, family, female, age, job, village, home, x, y, race)
}

/// `age` is in human-equivalent years; it's scaled by the race's lifespan.
#[allow(clippy::too_many_arguments)]
pub fn make_race(id: u32, rng: &mut Rng, culture: usize, family: &str, female: bool, age: f32, job: Job, village: usize, home: usize, x: f32, y: f32, race: u8) -> Villager {
    let stage = if age < 13.0 { 0 } else if age >= 60.0 { 2 } else { 1 };
    let mut look = Look::random(rng, female, stage);
    look.mature = age >= 18.0;
    look.race = race;
    match race {
        people::RACE_BEAST => look.beast = rng.below(4) as u8,
        people::RACE_DEMON => look.skin = rng.below(3) as u8,
        people::RACE_DWARF => {
            look.beard = !female && stage > 0 && rng.chance(0.9);
        }
        people::RACE_ELF => {
            if rng.chance(0.4) {
                look.hair = *rng.pick(&[3, 5, 7]);
            }
        }
        _ => {}
    }
    let age = age * race_life_mult(race);
    job.dress(&mut look, rng);
    // Elves take forest names, dwarves highland names.
    let name_culture = match race {
        people::RACE_ELF => 3,
        people::RACE_DWARF => 1,
        _ => culture,
    };
    let speed = if stage == 0 { 2.4 } else if stage == 2 { 1.8 } else { 2.2 };
    Villager {
        id,
        first: names::first(rng, name_culture, female),
        family: family.to_string(),
        female,
        race,
        age,
        look,
        traits: Traits::random(rng),
        needs: Needs::full(rng),
        job,
        village,
        home,
        actor: Actor::new(x, y, 0, speed),
        goal: Goal::Idle,
        phase: Phase::Travel,
        timer: 0.0,
        target: ((x / TILE as f32) as i32, (y / TILE as f32) as i32),
        work_prop: None,
        partner: None,
        dating: None,
        parents: Vec::new(),
        children: Vec::new(),
        opinions: Vec::new(),
        player_opinion: 0.0,
        emote: Emote::None,
        emote_t: 0.0,
        inside: None,
        pregnant: 0.0,
        gold: rng.range(5, 60),
        think_cd: rng.range_f(0.0, 2.0),
        alive: true,
        memories: Vec::new(),
        talked_today: false,
        gifted_today: false,
    }
}

/// Populate every village: households in homes, specialists for the inn,
/// store, smithy and chapel, and a working population matched to geography.
pub fn populate(world: &World, rng: &mut Rng) -> Vec<Villager> {
    let mut out: Vec<Villager> = Vec::new();
    let mut next_id = 1u32;
    for v in &world.villages {
        let culture = v.culture as usize;
        let homes: Vec<usize> = v.buildings.iter().filter(|b| b.kind == BuildingKind::Home).map(|b| b.prop).collect();
        if homes.is_empty() {
            continue;
        }
        // Jobs this village needs, specialists first.
        let mut jobs: Vec<Job> = Vec::new();
        for b in &v.buildings {
            match b.kind {
                BuildingKind::Inn => jobs.push(Job::Innkeeper),
                BuildingKind::Store => jobs.push(Job::Merchant),
                BuildingKind::Smithy => jobs.push(Job::Smith),
                BuildingKind::Chapel => jobs.push(Job::Priest),
                BuildingKind::Home => {}
            }
        }
        jobs.push(Job::Guard);
        jobs.push(Job::Healer);
        if rng.chance(0.6) {
            jobs.push(Job::Cook);
        }
        if rng.chance(0.5) {
            jobs.push(Job::Scholar);
        }
        if rng.chance(0.5) {
            jobs.push(Job::Mage);
        }
        let near_water = (0..24).any(|d| world.terrain(v.cx + d, v.cy).is_water() || world.terrain(v.cx - d, v.cy).is_water() || world.terrain(v.cx, v.cy + d).is_water());
        let highland = world.level(v.cx, v.cy) >= 2;
        let mut job_iter = 0;
        let mut next_job = |rng: &mut Rng| -> Job {
            job_iter += 1;
            if let Some(j) = jobs.get(job_iter - 1) {
                return *j;
            }
            let w = [
                0.38,
                0.18,
                if highland { 0.2 } else { 0.06 },
                if near_water { 0.18 } else { 0.0 },
                0.08,
                0.06,
            ];
            [Job::Farmer, Job::Woodcutter, Job::Miner, Job::Fisher, Job::Guard, Job::Adventurer][rng.weighted(&w)]
        };
        for &home in &homes {
            let hp = &world.props[home];
            let door = ((hp.x + hp.fw as i32 / 2) * TILE + TILE / 2, (hp.y + 1) * TILE + TILE / 2);
            let family = names::family(rng, culture);
            let couple = rng.chance(0.72);
            let a_female = rng.chance(0.5);
            let age_a = rng.range_f(20.0, 58.0);
            let mut members: Vec<usize> = Vec::new();
            let ja = next_job(rng);
            let jitter = |rng: &mut Rng| rng.range_f(-20.0, 20.0);
            let ax = door.0 as f32 + jitter(rng);
            let ay = door.1 as f32 + 8.0;
            out.push(make(next_id, rng, culture, &family, a_female, age_a, ja, v.id, home, ax, ay));
            members.push(out.len() - 1);
            next_id += 1;
            if couple {
                // Mostly mixed couples, some same-sex.
                let b_female = if rng.chance(0.88) { !a_female } else { a_female };
                let age_b = (age_a + rng.range_f(-6.0, 6.0)).clamp(19.0, 62.0);
                let jb = next_job(rng);
                let bx = door.0 as f32 + jitter(rng);
                out.push(make(next_id, rng, culture, &family, b_female, age_b, jb, v.id, home, bx, ay));
                next_id += 1;
                let (ia, ib) = (members[0], out.len() - 1);
                let (ida, idb) = (out[ia].id, out[ib].id);
                out[ia].partner = Some(idb);
                out[ib].partner = Some(ida);
                members.push(ib);
                // Only grown children (no child characters in the game).
                if age_a >= 38.0 {
                    let kids = rng.range(0, 3);
                    for _ in 0..kids {
                        let age_k = rng.range_f(18.0, (age_a - 18.0).max(18.5));
                        let job = next_job(rng);
                        let kx = door.0 as f32 + jitter(rng);
                        let kf = rng.chance(0.5);
                        let mut kid = make(next_id, rng, culture, &family, kf, age_k, job, v.id, home, kx, ay + 6.0);
                        kid.parents = vec![ida, idb];
                        kid.traits = Traits::blend(&out[ia].traits, &out[ib].traits, rng);
                        // Children inherit a parent's hair or skin.
                        kid.look.skin = if rng.chance(0.5) { out[ia].look.skin } else { out[ib].look.skin };
                        if out[ia].look.hair < 6 && rng.chance(0.6) {
                            kid.look.hair = out[ia].look.hair;
                        }
                        let kid_id = kid.id;
                        out.push(kid);
                        next_id += 1;
                        out[ia].children.push(kid_id);
                        out[ib].children.push(kid_id);
                        members.push(out.len() - 1);
                    }
                }
            }
            if rng.chance(0.18) {
                let ex = door.0 as f32 + jitter(rng);
                let (ef, ea) = (rng.chance(0.55), rng.range_f(61.0, 78.0));
                out.push(make(next_id, rng, culture, &family, ef, ea, Job::Retired, v.id, home, ex, ay));
                next_id += 1;
                members.push(out.len() - 1);
            }
            // Families love each other.
            for &i in &members {
                for &j in &members {
                    if i != j {
                        let id = out[j].id;
                        let base = rng.range_f(55.0, 90.0);
                        out[i].adjust_opinion(id, base);
                    }
                }
            }
        }
    }
    // Neighbours know each other a little.
    let n = out.len();
    for i in 0..n {
        for j in 0..n {
            if i == j || out[i].village != out[j].village || out[i].opinion_of(out[j].id) != 0.0 {
                continue;
            }
            if rng.chance(0.5) {
                let compat = out[i].traits.compatibility(&out[j].traits);
                let id = out[j].id;
                out[i].adjust_opinion(id, compat * 25.0 + rng.range_f(-10.0, 15.0));
            }
        }
    }
    out
}
