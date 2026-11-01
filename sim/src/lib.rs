//! Pixel Isekai simulation core. Runs natively for tests and tooling and as
//! WebAssembly inside a Web Worker in the browser.

pub mod art;
pub mod game;
pub mod names;
pub mod noise;
pub mod palette;
pub mod rng;
pub mod world;

#[cfg(target_arch = "wasm32")]
pub mod api;
