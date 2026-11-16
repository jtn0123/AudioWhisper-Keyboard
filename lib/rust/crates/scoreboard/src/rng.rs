//! A small seeded random number generator, so every run of the scoreboard draws the same taps and
//! swipes. Each word gets its own stream ([`Rng::for_item`]), so the results don't depend on how
//! the work is split between threads.

/// SplitMix64: tiny, fast and good enough for simulated touches.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed)
    }

    /// The stream for one item, such as a word of a sentence, under `seed`.
    pub fn for_item(seed: u64, a: u64, b: u64) -> Self {
        let mut rng = Rng(seed ^ a.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        rng.0 ^= rng.next_u64() ^ b.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
        rng
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in [0, 1).
    pub fn uniform(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// A standard normal draw (Box-Muller).
    pub fn gaussian(&mut self) -> f64 {
        let u1 = 1.0 - self.uniform();
        let u2 = self.uniform();
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_seed_gives_the_same_draws() {
        let mut a = Rng::for_item(7, 1, 2);
        let mut b = Rng::for_item(7, 1, 2);
        let mut c = Rng::for_item(7, 2, 1);
        let (x, y, z) = (a.next_u64(), b.next_u64(), c.next_u64());
        assert_eq!(x, y);
        assert_ne!(x, z);
    }

    #[test]
    fn gaussian_draws_have_unit_spread() {
        let mut rng = Rng::new(20261116);
        let n = 20_000;
        let draws: Vec<f64> = (0..n).map(|_| rng.gaussian()).collect();
        let mean = draws.iter().sum::<f64>() / n as f64;
        let var = draws.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / n as f64;
        assert!(mean.abs() < 0.03, "{mean}");
        assert!((var - 1.0).abs() < 0.05, "{var}");
        assert!((0..1000).all(|_| (0.0..1.0).contains(&rng.uniform())));
    }
}
