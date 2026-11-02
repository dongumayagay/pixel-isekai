//! The hand-tuned core roster.

use super::*;

/// The hand-tuned core cast: the faces a typical isekai story is built on.
/// Each is curated for quality first; procedural villagers come later.
pub fn cast() -> Vec<(&'static str, Look)> {
    let base = Look { skin: 1, hair: 0, hair_style: 1, primary: 0, secondary: 0, outfit: outfit::PEASANT, hat: 0, accent: 1, female: true, age: 1, beard: false, eyes: 0, mature: true, race: RACE_HUMAN, beast: 0, held: held::NONE, glasses: false, figure: 2, build: 1, height: 1, facial: 0, cut: 0 };
    vec![
        ("Reincarnated Hero", Look { build: 1, female: false, hair: 0, hair_style: 13, outfit: outfit::HERO, primary: 11, secondary: 4, accent: 9, eyes: 2, skin: 1, held: held::SWORD, height: 1, ..base }),
        ("Guild Receptionist", Look { figure: 3, build: 1, hair: 1, hair_style: 1, outfit: outfit::GUILD, primary: 13, secondary: 10, accent: 1, hat: 9, eyes: 1, skin: 1, held: held::BOOK, ..base }),
        ("Elf Mage", Look { figure: 1, build: 0, race: RACE_ELF, hair: 5, hair_style: 8, outfit: outfit::MAGE, primary: 10, secondary: 2, accent: 6, eyes: 6, skin: 0, held: held::STAFF, ..base }),
        ("Cat-girl Adventurer", Look { figure: 2, build: 2, race: RACE_BEAST, beast: 0, hair: 2, hair_style: 5, outfit: outfit::ADVENTURER, primary: 7, secondary: 3, accent: 1, eyes: 5, skin: 2, held: held::SWORD, ..base }),
        ("Female Knight", Look { figure: 3, build: 2, hair: 3, hair_style: 2, outfit: outfit::KNIGHT, primary: 1, secondary: 13, accent: 6, eyes: 0, skin: 1, held: held::SWORD, ..base }),
        ("Healer", Look { figure: 3, build: 1, hair: 3, hair_style: 11, outfit: outfit::PRIEST, primary: 10, secondary: 1, accent: 6, eyes: 0, skin: 0, held: held::STAFF, hat: 7, ..base }),
        ("Dwarf Smith", Look { facial: 6, build: 2, female: false, race: RACE_DWARF, hair: 4, hair_style: 0, outfit: outfit::SMITH, primary: 3, secondary: 8, accent: 1, beard: true, hat: 2, eyes: 2, skin: 3, held: held::HAMMER, ..base }),
        ("Demon Lord", Look { figure: 4, build: 2, race: RACE_DEMON, skin: 3, beast: 1, hair: 0, hair_style: 1, outfit: outfit::DEMON, primary: 11, secondary: 1, accent: 5, eyes: 4, height: 2, ..base }),
        ("Barmaid", Look { figure: 4, build: 3, hair: 3, hair_style: 10, outfit: outfit::INNKEEPER, primary: 1, secondary: 2, accent: 6, hat: 10, eyes: 1, skin: 2, held: held::MUG, ..base }),
        ("Fox Merchant", Look { figure: 2, build: 2, race: RACE_BEAST, beast: 1, hair: 4, hair_style: 7, outfit: outfit::MERCHANT, primary: 5, secondary: 7, accent: 6, eyes: 5, skin: 1, held: held::BASKET, ..base }),
        ("Village Elder", Look { facial: 6, build: 0, female: false, age: 2, hair: 5, hair_style: 18, outfit: outfit::SCHOLAR, primary: 2, secondary: 7, accent: 6, beard: true, eyes: 2, skin: 3, held: held::STAFF, ..base }),
        ("Shopkeeper", Look { facial: 2, build: 3, female: false, hair: 1, hair_style: 0, outfit: outfit::INNKEEPER, primary: 9, secondary: 3, accent: 6, hat: 5, glasses: true, eyes: 2, skin: 2, held: held::BASKET, ..base }),
        ("Noble Lady", Look { figure: 3, build: 1, hair: 3, hair_style: 9, outfit: outfit::NOBLE, primary: 12, secondary: 10, accent: 6, eyes: 0, skin: 0, cut: 1, ..base }),
        ("Rival Noble", Look { female: false, build: 1, hair: 6, hair_style: 15, outfit: outfit::NOBLE, primary: 13, secondary: 10, accent: 1, eyes: 4, skin: 0, held: held::SWORD, height: 2, ..base }),
        ("Maid", Look { figure: 3, build: 1, hair: 0, hair_style: 6, outfit: outfit::MAID, primary: 11, secondary: 10, accent: 1, hat: 10, eyes: 3, skin: 1, ..base }),
        ("Butler", Look { female: false, build: 0, hair: 5, hair_style: 14, outfit: outfit::MAID, primary: 11, secondary: 10, accent: 1, eyes: 1, skin: 1, height: 2, ..base }),
        ("Witch", Look { figure: 2, build: 1, hair: 7, hair_style: 6, outfit: outfit::MAGE, primary: 5, secondary: 11, accent: 6, hat: 6, eyes: 4, skin: 1, held: held::STAFF, cut: 1, ..base }),
        ("Rogue", Look { female: false, build: 0, hair: 0, hair_style: 13, outfit: outfit::ROGUE, primary: 8, secondary: 11, accent: 6, hat: 4, eyes: 5, skin: 2, held: held::SWORD, ..base }),
        ("Elf Ranger", Look { female: false, build: 0, race: RACE_ELF, hair: 3, hair_style: 14, outfit: outfit::RANGER, primary: 2, secondary: 3, accent: 6, eyes: 1, skin: 0, height: 2, ..base }),
        ("Monk", Look { female: false, build: 2, hair: 0, hair_style: 16, outfit: outfit::MONK, primary: 6, secondary: 3, accent: 1, eyes: 2, skin: 3, ..base }),
        ("Beach Heroine", Look { figure: 3, build: 2, hair: 2, hair_style: 2, outfit: outfit::SWIM, primary: 12, secondary: 10, accent: 10, eyes: 1, skin: 2, ..base }),
    ]
}
