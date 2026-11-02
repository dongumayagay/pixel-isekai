//! Which traits go together, as data: one table of headwear metadata and
//! one of lock rules. Modelled on the LPC spritesheet generator, where every
//! item lists the body types and other items it is drawn for (the creator
//! greys out everything else) instead of fixing clashes inside the art.
//!
//! [`Look::normalized`] applies every rule in order, so anything that
//! renders has already been made consistent; `look_options` in the wasm
//! API asks the same table which option values a creator should disable.

use super::*;

pub const HAT_NONE: u8 = 0;
pub const HAT_STRAW: u8 = 1;
pub const HAT_BANDANA: u8 = 2;
pub const HAT_HELMET: u8 = 3;
pub const HAT_HOOD: u8 = 4;
pub const HAT_CAP: u8 = 5;
pub const HAT_WITCH: u8 = 6;
pub const HAT_FLOWER: u8 = 7;
pub const HAT_CHEF: u8 = 8;
pub const HAT_RIBBON: u8 = 9;
pub const HAT_MAID_BAND: u8 = 10;
pub const HAT_MINER: u8 = 11;

/// Headwear metadata: how it is drawn and what it refuses to be drawn with.
pub struct Hat {
    pub name: &'static str,
    /// Sits over the crown: hair that would rise above it (ahoge) is not drawn.
    pub covers_crown: bool,
    /// Mirror-exact in the front and back views.
    pub symmetric: bool,
    /// Hair styles that poke through it (buns, spikes, top-knots, tails).
    pub excluded_hair: &'static [u8],
    /// Races whose ears or horns pierce it.
    pub excluded_races: &'static [u8],
    /// Outfits it looks absurd with.
    pub excluded_outfits: &'static [u8],
}

const BUN: u8 = 3;
const SPIKY: u8 = 4;
const TWIN_TAILS: u8 = 6;
const SAMURAI: u8 = 17;
const BALD: u8 = 18;
const HORNED: &[u8] = &[RACE_BEAST, RACE_DEMON];
const BEACH: &[u8] = &[outfit::SWIM, outfit::MICRO];

macro_rules! hat {
    ($name:literal, crown: $crown:literal, sym: $sym:literal, hair: $hair:expr, races: $races:expr, outfits: $outfits:expr) => {
        Hat { name: $name, covers_crown: $crown, symmetric: $sym, excluded_hair: $hair, excluded_races: $races, excluded_outfits: $outfits }
    };
}

/// Indexed by `Look::hat`. Exclusions were settled by rendering every
/// pairing (`cargo run --example combos`): close-fitting hats read as having
/// ear and horn holes, brimmed and soft ones get pierced.
pub const HAT_TABLE: [Hat; HATS as usize] = [
    hat!("None", crown: false, sym: true, hair: &[], races: &[], outfits: &[]),
    hat!("Straw hat", crown: true, sym: false, hair: &[BUN, SAMURAI], races: HORNED, outfits: &[]),
    hat!("Bandana", crown: true, sym: true, hair: &[BUN, SPIKY, SAMURAI], races: &[], outfits: &[]),
    hat!("Helmet", crown: true, sym: true, hair: &[], races: &[], outfits: BEACH),
    hat!("Hood", crown: true, sym: true, hair: &[BUN, SPIKY, TWIN_TAILS, SAMURAI], races: &[RACE_ELF], outfits: BEACH),
    hat!("Cap", crown: true, sym: true, hair: &[BUN, SPIKY, SAMURAI], races: &[], outfits: &[]),
    hat!("Witch hat", crown: false, sym: false, hair: &[], races: HORNED, outfits: &[]),
    hat!("Flower", crown: false, sym: false, hair: &[BALD], races: &[], outfits: &[]),
    hat!("Chef", crown: true, sym: true, hair: &[], races: HORNED, outfits: &[]),
    hat!("Ribbon", crown: false, sym: false, hair: &[BALD], races: &[], outfits: &[]),
    hat!("Maid band", crown: false, sym: true, hair: &[BALD], races: &[], outfits: &[]),
    hat!("Miner helmet", crown: true, sym: false, hair: &[BUN, SPIKY, SAMURAI], races: HORNED, outfits: BEACH),
];

/// The nearest style a hat can be pulled over, for hair that pokes through
/// it: a job hat stays on and the hair goes under it.
pub const TUCKED: [(u8, u8); 4] = [(BUN, 2), (SPIKY, 0), (TWIN_TAILS, 10), (SAMURAI, 14)];

pub const HELD_NAMES: [&str; held::COUNT as usize] = ["None", "Staff", "Hammer", "Mug", "Book", "Sword", "Basket", "Axe", "Pickaxe", "Hoe", "Fishing rod", "Spear", "Ladle"];

/// Beastfolk kinds; demons use 0 for short horns and 1 for curled ones.
pub const BEAST_NAMES: [&str; 4] = ["Cat", "Fox", "Wolf", "Rabbit"];

pub fn hat_names() -> Vec<&'static str> {
    HAT_TABLE.iter().map(|h| h.name).collect()
}

/// One rule: when `clash` holds, `fix` changes the look until it doesn't.
pub struct Lock {
    pub why: &'static str,
    pub clash: fn(&Look) -> bool,
    pub fix: fn(&mut Look),
    /// A consequence of the body rather than a choice (facial hair on a
    /// woman is simply not drawn): the creator hides or ignores these
    /// instead of greying out the option that triggers them.
    pub derived: bool,
}

const fn lock(why: &'static str, clash: fn(&Look) -> bool, fix: fn(&mut Look)) -> Lock {
    Lock { why, clash, fix, derived: false }
}

const fn derived(why: &'static str, clash: fn(&Look) -> bool, fix: fn(&mut Look)) -> Lock {
    Lock { why, clash, fix, derived: true }
}

fn beast_kinds(l: &Look) -> u8 {
    match l.race {
        RACE_BEAST => BEAST_NAMES.len() as u8,
        RACE_DEMON => 2,
        _ => 1,
    }
}

/// Applied in order by [`Look::normalized`]; later rules see earlier fixes.
pub const LOCKS: &[Lock] = &[
    lock(
        "hair pokes through the hat",
        |l| l.hat_meta().excluded_hair.contains(&l.hair_style),
        |l| match TUCKED.iter().find(|(s, _)| *s == l.hair_style) {
            Some(&(_, under)) if !l.hat_meta().excluded_hair.contains(&under) => l.hair_style = under,
            _ => l.hat = HAT_NONE,
        },
    ),
    lock("ears or horns pierce the hat", |l| l.hat_meta().excluded_races.contains(&l.race), |l| l.hat = HAT_NONE),
    lock("hard hats and hoods with swimwear", |l| l.hat_meta().excluded_outfits.contains(&l.outfit), |l| l.hat = HAT_NONE),
    lock("beast kind means nothing on other races", |l| l.beast >= beast_kinds(l), |l| l.beast = 0),
    derived("children are never mature", |l| l.age == 0 && l.mature, |l| l.mature = false),
    derived("facial hair only on grown men", |l| (l.female || !l.mature) && l.facial != 0, |l| l.facial = 0),
    derived("figure only on grown women", |l| (!l.female || !l.mature) && l.figure != default_figure(), |l| l.figure = default_figure()),
    derived("skin wraps to the race's palette", |l| l.skin >= skin_tones(l), |l| l.skin %= skin_tones(l)),
];

fn skin_tones(l: &Look) -> u8 {
    if l.race == RACE_DEMON { pal::DEMON_SKIN.len() as u8 } else { pal::SKIN.len() as u8 }
}

impl Look {
    pub fn hat_meta(&self) -> &'static Hat {
        &HAT_TABLE[self.hat as usize % HAT_TABLE.len()]
    }

    /// The first rule this look breaks, if any.
    pub fn clash(&self) -> Option<&'static Lock> {
        LOCKS.iter().find(|k| (k.clash)(self))
    }

    /// Breaks no rule: `normalized()` would return it unchanged.
    pub fn compatible(&self) -> bool {
        self.clash().is_none()
    }

    /// Whether `field` may take `value` without any non-derived rule
    /// firing on the result; `patch` sets it on a copy.
    pub fn allows(&self, patch: impl FnOnce(&mut Look)) -> bool {
        let mut l = *self;
        patch(&mut l);
        !LOCKS.iter().any(|k| !k.derived && (k.clash)(&l))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_hat_has_names_and_no_self_contradiction() {
        assert_eq!(hat_names().len(), HATS as usize);
        for (style, under) in TUCKED {
            assert_ne!(style, under);
            for h in &HAT_TABLE {
                // A tuck target must itself fit under any hat that excludes the style.
                if h.excluded_hair.contains(&style) {
                    assert!(!h.excluded_hair.contains(&under), "{}: tuck {style}->{under} still pokes through", h.name);
                }
            }
        }
    }

    #[test]
    fn locks_match_the_rendered_review() {
        let base = cast()[1].1;
        let hooded_elf = Look { race: RACE_ELF, hat: HAT_HOOD, ..base }.normalized();
        assert_eq!(hooded_elf.hat, HAT_NONE);
        let hooded_bun = Look { hair_style: BUN, hat: HAT_HOOD, ..base }.normalized();
        assert_eq!((hooded_bun.hat, hooded_bun.hair_style), (HAT_HOOD, 2));
        let bald_ribbon = Look { hair_style: BALD, hat: HAT_RIBBON, ..base }.normalized();
        assert_eq!(bald_ribbon.hat, HAT_NONE);
        let cat_cap = Look { race: RACE_BEAST, hat: HAT_CAP, ..base }.normalized();
        assert_eq!(cat_cap.hat, HAT_CAP);
        assert!(Look { hair_style: BUN, ..base }.allows(|l| l.hat = HAT_WITCH));
        assert!(!Look { hair_style: BUN, ..base }.allows(|l| l.hat = HAT_HOOD));
    }
}
