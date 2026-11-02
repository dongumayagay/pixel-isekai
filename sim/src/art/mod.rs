//! Sprite generation. All art is produced procedurally at startup.

pub mod atlas;
pub mod buildings;
pub mod canvas;
pub mod flora;
pub mod fx;
pub mod people;

use crate::palette::{ramp_at, Ramp, Rgb};

/// Pick the ramp step for a brightness `v` in [0, 1]. No dithering: the
/// hard bands between steps are what make shaded forms read as pixel art.
#[inline]
pub(crate) fn shade(ramp: Ramp, v: f32) -> Rgb {
    let n = ramp.len() as f32;
    ramp_at(ramp, (v.clamp(0.0, 0.999) * n) as i32)
}
