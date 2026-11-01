//! Deterministic randomness. Everything in the world derives from one seed.

use serde::{Deserialize, Serialize};

/// PCG32 (XSH-RR). Small, fast, statistically solid.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rng {
    state: u64,
    inc: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self::with_stream(seed, 0xda3e_39cb_94b9_5bdb)
    }

    pub fn with_stream(seed: u64, stream: u64) -> Self {
        let mut r = Rng { state: 0, inc: (stream << 1) | 1 };
        r.next_u32();
        r.state = r.state.wrapping_add(seed);
        r.next_u32();
        r
    }

    /// Derive an independent generator for a named subsystem.
    pub fn fork(&mut self, salt: u64) -> Rng {
        Rng::with_stream(self.next_u64() ^ salt, salt.wrapping_mul(0x9e37_79b9_7f4a_7c15))
    }

    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old.wrapping_mul(6364136223846793005).wrapping_add(self.inc);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    pub fn next_u64(&mut self) -> u64 {
        ((self.next_u32() as u64) << 32) | self.next_u32() as u64
    }

    /// Uniform in [0, 1).
    pub fn f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / 16_777_216.0
    }

    pub fn range_f(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.f32()
    }

    /// Uniform integer in [lo, hi).
    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        lo + (self.next_u32() % (hi - lo) as u32) as i32
    }

    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        (self.next_u32() as usize) % n
    }

    pub fn chance(&mut self, p: f32) -> bool {
        self.f32() < p
    }

    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }

    /// Pick an index with probability proportional to its weight.
    pub fn weighted(&mut self, weights: &[f32]) -> usize {
        let total: f32 = weights.iter().sum();
        if total <= 0.0 {
            return 0;
        }
        let mut t = self.f32() * total;
        for (i, w) in weights.iter().enumerate() {
            if t < *w {
                return i;
            }
            t -= w;
        }
        weights.len() - 1
    }

    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i + 1);
            items.swap(i, j);
        }
    }
}

/// Stateless integer hash, used for per-pixel and per-tile detail so baking is
/// order-independent and chunks can be rebuilt identically at any time.
#[inline]
pub fn hash2(x: i32, y: i32, seed: u32) -> u32 {
    let mut h = seed ^ 0x27d4_eb2d;
    h = h.wrapping_add((x as u32).wrapping_mul(0x85eb_ca6b));
    h ^= h >> 15;
    h = h.wrapping_add((y as u32).wrapping_mul(0xc2b2_ae35));
    h ^= h >> 13;
    h = h.wrapping_mul(0x27d4_eb2f);
    h ^= h >> 16;
    h
}

#[inline]
pub fn hash2f(x: i32, y: i32, seed: u32) -> f32 {
    (hash2(x, y, seed) >> 8) as f32 / 16_777_216.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }

    #[test]
    fn range_bounds() {
        let mut r = Rng::new(7);
        for _ in 0..10_000 {
            let v = r.range(-3, 5);
            assert!((-3..5).contains(&v));
            let f = r.f32();
            assert!((0.0..1.0).contains(&f));
        }
    }
}
