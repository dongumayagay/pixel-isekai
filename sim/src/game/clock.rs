//! Calendar and weather.

use serde::{Deserialize, Serialize};

use crate::rng::Rng;

/// Real seconds per in-game minute at 1× speed. A full day is ~14 minutes.
pub const SEC_PER_MIN: f32 = 0.6;
pub const MINUTES_PER_DAY: f64 = 1440.0;
pub const DAYS_PER_SEASON: u32 = 12;
/// Villagers age half a year per day, so generations turn over in a long save.
pub const YEARS_PER_DAY: f32 = 0.5;

pub const SEASON_NAMES: [&str; 4] = ["Spring", "Summer", "Autumn", "Winter"];
/// Season index of winter: nothing grows and snow covers the ground.
pub const WINTER: u8 = 3;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Weather {
    Clear = 0,
    Cloudy = 1,
    Rain = 2,
    Storm = 3,
    Snow = 4,
    Fog = 5,
}

impl Weather {
    /// Inverse of `weather as u8`; out-of-range values clamp to fog.
    pub fn from_index(i: usize) -> Weather {
        [Weather::Clear, Weather::Cloudy, Weather::Rain, Weather::Storm, Weather::Snow, Weather::Fog][i.min(5)]
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Clock {
    /// Total in-game minutes since the world began.
    pub minutes: f64,
    pub weather: Weather,
    /// 0..1, eases toward its target for smooth transitions.
    pub intensity: f32,
    target_intensity: f32,
    next_change: f64,
    pub wind: f32,
}

impl Clock {
    pub fn new() -> Clock {
        // Start on the morning of Spring 1.
        Clock { minutes: 7.0 * 60.0, weather: Weather::Clear, intensity: 0.0, target_intensity: 0.0, next_change: 10.0 * 60.0, wind: 0.3 }
    }

    pub fn day(&self) -> u32 {
        (self.minutes / MINUTES_PER_DAY) as u32
    }
    pub fn minute_of_day(&self) -> f32 {
        (self.minutes % MINUTES_PER_DAY) as f32
    }
    pub fn hour(&self) -> f32 {
        self.minute_of_day() / 60.0
    }
    pub fn season(&self) -> u8 {
        ((self.day() / DAYS_PER_SEASON) % 4) as u8
    }
    pub fn day_of_season(&self) -> u32 {
        self.day() % DAYS_PER_SEASON + 1
    }
    pub fn year(&self) -> u32 {
        self.day() / (DAYS_PER_SEASON * 4) + 1
    }
    pub fn is_night(&self) -> bool {
        let h = self.hour();
        !(5.5..20.0).contains(&h)
    }
    pub fn date_label(&self) -> String {
        format!("{} {}, Year {}", SEASON_NAMES[self.season() as usize], self.day_of_season(), self.year())
    }

    /// Advance by `mins` in-game minutes. Returns true when a new day began.
    pub fn advance(&mut self, mins: f64, rng: &mut Rng) -> bool {
        let before = self.day();
        self.minutes += mins;
        if self.minutes >= self.next_change {
            self.roll_weather(rng);
        }
        let k = (mins as f32 * 0.02).min(1.0);
        self.intensity += (self.target_intensity - self.intensity) * k;
        self.wind += (rng.f32() - 0.5) * 0.01 * mins as f32;
        self.wind = self.wind.clamp(0.1, 1.0);
        self.day() != before
    }

    fn roll_weather(&mut self, rng: &mut Rng) {
        // Weights: clear, cloudy, rain, storm, snow, fog
        let w: [f32; 6] = match self.season() {
            0 => [0.42, 0.22, 0.24, 0.04, 0.0, 0.08],
            1 => [0.58, 0.18, 0.1, 0.1, 0.0, 0.04],
            2 => [0.36, 0.26, 0.22, 0.06, 0.0, 0.1],
            _ => [0.34, 0.22, 0.0, 0.0, 0.34, 0.1],
        };
        self.weather = Weather::from_index(rng.weighted(&w));
        self.target_intensity = match self.weather {
            Weather::Clear => 0.0,
            Weather::Cloudy => 0.35,
            _ => rng.range_f(0.5, 1.0),
        };
        self.next_change = self.minutes + rng.range_f(180.0, 720.0) as f64;
    }

    /// Force the weather (debug/testing and festival days).
    pub fn set_weather(&mut self, w: Weather, intensity: f32) {
        self.weather = w;
        self.target_intensity = intensity;
        self.intensity = intensity;
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}
