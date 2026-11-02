//! Render speed and a pixel fingerprint of the roster plus random looks
//! (refactors must keep the fingerprint identical).
//! cargo run --release --example bench
use sim::art::people::{self, Look};
use sim::rng::Rng;

fn main() {
    let mut looks: Vec<Look> = people::cast().into_iter().map(|(_, l)| l).collect();
    let mut rng = Rng::new(2024);
    looks.extend((0..200).map(|_| Look::random_any(&mut rng)));
    let mut hash: u64 = 0xcbf29ce484222325;
    let mut frames = 0u32;
    let t = std::time::Instant::now();
    for l in &looks {
        for d in 0..3u8 {
            for f in 0..people::FRAMES_PER_DIR {
                let img = people::frame(l, d, f);
                for &b in &img.px {
                    hash = (hash ^ b as u64).wrapping_mul(0x100000001b3);
                }
                frames += 1;
            }
        }
    }
    let el = t.elapsed();
    println!("{frames} frames in {:.0} ms = {:.1} us/frame, fingerprint {hash:016x}", el.as_secs_f64() * 1e3, el.as_secs_f64() * 1e6 / frames as f64);
}
