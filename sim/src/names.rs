//! Syllable-based name generation. Each culture has its own phonetic flavour.

use crate::rng::Rng;

pub const CULTURES: usize = 4;

struct Culture {
    town_pre: &'static [&'static str],
    town_suf: &'static [&'static str],
    first_a: &'static [&'static str],
    first_b: &'static [&'static str],
    fem_end: &'static [&'static str],
    masc_end: &'static [&'static str],
    family: &'static [&'static str],
}

const C: [Culture; CULTURES] = [
    // Meadowland: soft and pastoral
    Culture {
        town_pre: &["Bel", "Wil", "Hal", "Elder", "Mead", "Fen", "Ash", "Brook", "Clover", "Hollow"],
        town_suf: &["dale", "brook", "ford", "field", "mere", "wick", "vale", "hollow"],
        first_a: &["El", "Ro", "Mi", "Lu", "Ber", "Wen", "Fin", "Ma", "Tam", "Is", "Cor", "Al"],
        first_b: &["", "ri", "an", "el", "ly", "wy", "o", "ra", "dre"],
        fem_end: &["a", "ie", "wen", "lyn", "ette", "ra"],
        masc_end: &["n", "ric", "win", "as", "o", "den"],
        family: &["Thistle", "Brook", "Fairweather", "Hawthorn", "Miller", "Ashby", "Meadows", "Pell", "Greenhill", "Oakes"],
    },
    // Highland: stony and hard
    Culture {
        town_pre: &["Kar", "Dun", "Grim", "Har", "Stor", "Bran", "Kel", "Tor", "Ulf", "Dorn"],
        town_suf: &["holm", "stead", "hold", "gard", "crag", "berg", "fell", "keep"],
        first_a: &["Bjor", "Ka", "Ha", "Sig", "Ul", "Ra", "Ing", "Tor", "Ey", "Gun", "Ast", "Vi"],
        first_b: &["", "r", "ld", "ga", "mun", "di", "va"],
        fem_end: &["a", "rid", "hild", "dis", "ja"],
        masc_end: &["n", "ar", "ulf", "rik", "mund", "i"],
        family: &["Stonehand", "Ironside", "Frostbeard", "Hallström", "Graves", "Kettil", "Ravenmoor", "Holt", "Brandt", "Crag"],
    },
    // Coastal: salty and open
    Culture {
        town_pre: &["Mar", "Sel", "Cor", "Port", "Gull", "Tide", "Sal", "Wave", "Coral", "Shell"],
        town_suf: &["port", "haven", "cove", "mouth", "bay", "strand", "reach", "point"],
        first_a: &["Ma", "Ner", "Sa", "Ca", "Lo", "Pel", "Ori", "Del", "Mer", "Bri", "Tes", "Ju"],
        first_b: &["", "ri", "ne", "lo", "sa", "ve", "an"],
        fem_end: &["a", "ine", "elle", "ia", "ssa"],
        masc_end: &["o", "an", "us", "en", "is"],
        family: &["Saltwind", "Driftwood", "Marlowe", "Netter", "Seabright", "Corvin", "Tidewell", "Gale", "Harbor", "Pike"],
    },
    // Old forest: lilting, faintly elvish
    Culture {
        town_pre: &["Syl", "Aer", "Ithil", "Lae", "Fae", "Thal", "Eryn", "Myr", "Vael", "Lune"],
        town_suf: &["wood", "glade", "thal", "shade", "dell", "grove", "lume", "ara"],
        first_a: &["Ae", "Li", "Sy", "Tha", "Ere", "Nai", "Ili", "Fae", "Ela", "Ny", "Ori", "Ve"],
        first_b: &["", "la", "ri", "th", "an", "lia", "ndr"],
        fem_end: &["a", "iel", "wyn", "ae", "ith"],
        masc_end: &["on", "ar", "il", "ion", "eth"],
        family: &["Moonwhisper", "Leafsong", "Starbrook", "Willowmere", "Dawnpetal", "Silverbough", "Fernshade", "Lumen", "Mistvale", "Aelor"],
    },
];

fn cap(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

pub fn town(rng: &mut Rng, culture: usize) -> String {
    let c = &C[culture % CULTURES];
    let pre = *rng.pick(c.town_pre);
    let suf = *rng.pick(c.town_suf);
    cap(&format!("{}{}", pre, suf))
}

pub fn first(rng: &mut Rng, culture: usize, female: bool) -> String {
    let c = &C[culture % CULTURES];
    let a = *rng.pick(c.first_a);
    let b = *rng.pick(c.first_b);
    let e = if female { *rng.pick(c.fem_end) } else { *rng.pick(c.masc_end) };
    let mut s = format!("{}{}{}", a, b, e);
    // Collapse awkward doubled vowels at the joins.
    for pair in ["aa", "ee", "ii", "oo", "uu", "aea", "yy"] {
        if s.contains(pair) {
            s = s.replacen(pair, &pair[..1], 1);
        }
    }
    cap(&s)
}

pub fn family(rng: &mut Rng, culture: usize) -> String {
    let c = &C[culture % CULTURES];
    rng.pick(c.family).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_reasonable() {
        let mut r = Rng::new(5);
        for c in 0..CULTURES {
            for _ in 0..50 {
                let t = town(&mut r, c);
                let fem = r.chance(0.5);
                let f = first(&mut r, c, fem);
                assert!(t.len() >= 4 && t.len() <= 16, "{t}");
                assert!(f.len() >= 2 && f.len() <= 14, "{f}");
                assert!(f.chars().next().unwrap().is_uppercase());
            }
        }
    }
}
