//! Procedural dialogue. Lines are assembled from a villager's personality,
//! mood, job, relationships, recent memories and village gossip, so the
//! same person says different things from day to day.

use super::villager::{Goal, Job};
use super::Game;

impl Game {
    pub fn dialogue_for(&mut self, i: usize) -> Vec<String> {
        let v = self.villagers[i].clone();
        let h = self.clock.hour();
        let mut lines: Vec<String> = Vec::new();
        let r = &mut self.rng;
        let warm = v.player_opinion > 35.0;
        let first_meet = v.player_opinion < 1.0 && !v.talked_today;

        // Greeting
        let greet = if h < 11.0 {
            "Good morning"
        } else if h < 17.0 {
            "Good afternoon"
        } else {
            "Good evening"
        };
        if first_meet {
            let intro = match v.job {
                Job::Child => format!("Hi! I'm {}. Are you the stranger everyone's talking about?", v.first),
                _ => format!("{greet}. I'm {}, the {} here in {}.", v.first, v.job.label().to_lowercase(), self.world.villages[v.village].name),
            };
            lines.push(intro);
            if v.traits.curious > 0.6 {
                lines.push("They say you fell from the sky in a flash of light. Is it true you came from another world?".into());
            } else if v.traits.kind > 0.6 {
                lines.push("If you need anything, just ask. We look after each other here.".into());
            } else {
                lines.push("Strangers aren't common around here. Mind your manners.".into());
            }
            return lines;
        }
        let opener = if warm {
            *r.pick(&["Oh, it's you! ", "Hey, friend! ", "Good to see you. ", ""])
        } else if v.traits.sociable < 0.35 {
            *r.pick(&["...Oh. Hello. ", "Hm? ", ""])
        } else {
            *r.pick(&["Hello there. ", "Ah, hello. ", ""])
        };

        // Mood and needs
        let n = &v.needs;
        let mood_line = if n.hunger < 25.0 {
            "My stomach hasn't stopped growling all day."
        } else if n.energy < 25.0 {
            "I can barely keep my eyes open..."
        } else if n.social < 20.0 {
            "It's been ages since anyone stopped to talk with me. Thank you."
        } else if n.fun < 20.0 {
            "Every day is the same. I need something exciting to happen."
        } else if n.mood() > 75.0 {
            *r.pick(&["What a lovely day!", "I feel wonderful today.", "Life's been kind lately."])
        } else {
            ""
        };
        if !mood_line.is_empty() {
            lines.push(format!("{opener}{mood_line}"));
        }

        // Heart topics: romance, family, grief.
        let name_of = |g: &Game, id: u32| g.villagers.iter().find(|o| o.id == id).map(|o| o.first.clone()).unwrap_or_default();
        if let Some(d) = v.dating {
            let nm = name_of(self, d);
            lines.push(format!("Can I tell you a secret? I think I'm falling for {nm}."));
        } else if let Some(p) = v.partner {
            let nm = name_of(self, p);
            if self.rng.chance(0.4) {
                lines.push(format!("{nm} and I are doing well. I'm lucky to have them."));
            }
        }
        if v.pregnant > 0.0 {
            lines.push("We're expecting a little one soon! I'm so nervous.".into());
        }
        if let Some(m) = v.memories.last() {
            if m.starts_with("Misses") || m.starts_with("Mourning") {
                lines.push("I keep thinking about the ones we've lost. The village feels quieter.".into());
            } else if m.starts_with("Argued") {
                let who = m.trim_start_matches("Argued with ").to_string();
                lines.push(format!("Ugh, don't get me started on {who}."));
            } else if m.starts_with("Welcomed") {
                lines.push("Have you seen our baby? Sleepless nights, but worth it.".into());
            }
        }

        // Job talk
        let season = self.clock.season();
        let job_line = match v.job {
            Job::Farmer => match season {
                0 => "Spring's the time to sow. The soil smells sweet this year.",
                1 => "The crops are drinking up the sun. Harvest will be good.",
                2 => "Harvest season! My back aches but the barns are full.",
                _ => "Nothing grows in winter. I mend fences and wait.",
            },
            Job::Woodcutter => "These woods give us everything, so I only take what we need.",
            Job::Miner => "There's ore in those hills. Copper, iron... maybe even crystal.",
            Job::Fisher => "The fish bite best at dawn. Patience is the whole trick.",
            Job::Smith => "Bring me ore and I'll make you something that lasts a lifetime.",
            Job::Merchant => "Goods from all the villages pass through my hands. Business is business!",
            Job::Innkeeper => "Come by the inn tonight. Warm stew, and the gossip is free.",
            Job::Guard => {
                if self.clock.is_night() {
                    "Stay close to the lamps. Things wander out of the forest at night."
                } else {
                    "Quiet day. Just how I like it."
                }
            }
            Job::Priest => "The Goddess watches over every soul, even ones from far away.",
            Job::Adventurer => "I've seen ruins out east that no map shows. One day I'll go inside.",
            Job::Child => "Wanna play tag? You're it!",
            Job::Retired => "In my day, winters were twice as long and we liked it.",
        };
        if self.rng.chance(0.6) || lines.is_empty() {
            lines.push(job_line.into());
        }

        // Gossip about something that recently happened.
        if v.traits.sociable > 0.45 {
            let recent: Vec<_> = self.chronicle.iter().rev().take(12).filter(|e| !e.who.contains(&v.id) && matches!(e.kind, "romance" | "wedding" | "birth" | "quarrel" | "arrival" | "death")).collect();
            if !recent.is_empty() {
                let e = recent[self.rng.below(recent.len())];
                let pre = *self.rng.pick(&["Did you hear? ", "Word is going around: ", "Between you and me... "]);
                lines.push(format!("{pre}{}", e.text));
            }
        }

        // Weather remark
        use super::clock::Weather;
        let wl = match self.clock.weather {
            Weather::Rain => "This rain! At least the fields are happy.",
            Weather::Storm => "Listen to that thunder. Best stay indoors.",
            Weather::Snow => "Snow again. The children are delighted, at least.",
            Weather::Fog => "Can't see past the well in this fog.",
            _ => "",
        };
        if !wl.is_empty() && self.rng.chance(0.5) {
            lines.push(wl.into());
        }
        if matches!(v.goal, Goal::Work) && v.traits.diligent > 0.7 && self.rng.chance(0.5) {
            lines.push("Anyway, I'd better get back to work.".into());
        }
        lines.truncate(4);
        lines
    }
}
