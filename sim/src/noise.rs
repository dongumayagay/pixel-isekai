//! Seeded 2D simplex noise with fBm and domain warping.

const GRAD: [(f32, f32); 12] = [
    (1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0),
    (1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0),
    (0.7071, 0.7071), (-0.7071, 0.7071), (0.7071, -0.7071), (-0.7071, -0.7071),
];

#[derive(Clone)]
pub struct Noise {
    perm: [u8; 512],
}

impl Noise {
    pub fn new(seed: u64) -> Self {
        let mut p: [u8; 256] = [0; 256];
        for (i, v) in p.iter_mut().enumerate() {
            *v = i as u8;
        }
        let mut rng = crate::rng::Rng::new(seed);
        rng.shuffle(&mut p);
        let mut perm = [0u8; 512];
        for i in 0..512 {
            perm[i] = p[i & 255];
        }
        Noise { perm }
    }

    /// Simplex noise in roughly [-1, 1].
    pub fn get(&self, x: f32, y: f32) -> f32 {
        const F2: f32 = 0.366_025_42; // (sqrt(3)-1)/2
        const G2: f32 = 0.211_324_87; // (3-sqrt(3))/6
        let s = (x + y) * F2;
        let i = (x + s).floor();
        let j = (y + s).floor();
        let t = (i + j) * G2;
        let x0 = x - (i - t);
        let y0 = y - (j - t);
        let (i1, j1) = if x0 > y0 { (1.0, 0.0) } else { (0.0, 1.0) };
        let x1 = x0 - i1 + G2;
        let y1 = y0 - j1 + G2;
        let x2 = x0 - 1.0 + 2.0 * G2;
        let y2 = y0 - 1.0 + 2.0 * G2;
        let ii = (i as i32 & 255) as usize;
        let jj = (j as i32 & 255) as usize;
        let p = &self.perm;
        let g0 = p[ii + p[jj] as usize] as usize % 12;
        let g1 = p[ii + i1 as usize + p[jj + j1 as usize] as usize] as usize % 12;
        let g2 = p[ii + 1 + p[jj + 1] as usize] as usize % 12;
        let mut n = 0.0;
        let mut corner = |gx: usize, dx: f32, dy: f32| {
            let t = 0.5 - dx * dx - dy * dy;
            if t > 0.0 {
                let t2 = t * t;
                n += t2 * t2 * (GRAD[gx].0 * dx + GRAD[gx].1 * dy);
            }
        };
        corner(g0, x0, y0);
        corner(g1, x1, y1);
        corner(g2, x2, y2);
        70.0 * n
    }

    /// Fractal Brownian motion, normalised to roughly [-1, 1].
    pub fn fbm(&self, x: f32, y: f32, octaves: u32, lacunarity: f32, gain: f32) -> f32 {
        let mut amp = 1.0;
        let mut freq = 1.0;
        let mut sum = 0.0;
        let mut norm = 0.0;
        for o in 0..octaves {
            // Offset each octave so the lattice artefacts don't line up.
            let off = o as f32 * 17.31;
            sum += amp * self.get(x * freq + off, y * freq - off);
            norm += amp;
            amp *= gain;
            freq *= lacunarity;
        }
        sum / norm
    }

    /// Ridged multifractal, in [0, 1]. Good for mountain spines.
    pub fn ridged(&self, x: f32, y: f32, octaves: u32) -> f32 {
        let mut amp = 0.5;
        let mut freq = 1.0;
        let mut sum = 0.0;
        let mut norm = 0.0;
        for o in 0..octaves {
            let off = o as f32 * 9.7;
            let v = 1.0 - self.get(x * freq + off, y * freq + off).abs();
            sum += amp * v * v;
            norm += amp;
            amp *= 0.5;
            freq *= 2.0;
        }
        sum / norm
    }

    /// fBm sampled through a warped domain, which gives organic, flowing shapes.
    pub fn warped(&self, x: f32, y: f32, warp: f32, octaves: u32) -> f32 {
        let qx = self.fbm(x + 5.2, y + 1.3, 3, 2.0, 0.5);
        let qy = self.fbm(x - 3.7, y + 8.1, 3, 2.0, 0.5);
        self.fbm(x + warp * qx, y + warp * qy, octaves, 2.0, 0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_and_deterministic() {
        let a = Noise::new(1);
        let b = Noise::new(1);
        for i in 0..1000 {
            let x = i as f32 * 0.37;
            let y = i as f32 * 0.11;
            let v = a.fbm(x, y, 5, 2.0, 0.5);
            assert_eq!(v, b.fbm(x, y, 5, 2.0, 0.5));
            assert!(v.abs() <= 1.2, "{v}");
        }
    }
}
