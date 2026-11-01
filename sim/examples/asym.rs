//! List the pixels that break mirror symmetry in a look's front frame.
//! cargo run --release --example asym -- '<look json>' [dir]
use sim::art::people::{self, Look};
fn main() {
    let mut a = std::env::args().skip(1);
    let look: Look = serde_json::from_str(&a.next().unwrap()).unwrap();
    let dir: u8 = a.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let f = people::frame(&look, dir, 0);
    for y in 0..f.h {
        for x in 0..f.w / 2 {
            let (l, r) = (f.get(x, y), f.get(f.w - 1 - x, y));
            if l != r {
                println!("({x},{y}) {:?} vs ({},{y}) {:?}", l, f.w - 1 - x, r);
            }
        }
    }
}
