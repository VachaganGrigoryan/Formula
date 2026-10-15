//! A tiny seeded random number generator (xorshift64*).
//!
//! Visualizations need repeatable "random" layouts and data, not
//! cryptographic quality, so this avoids pulling in a crate.

#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        // SplitMix64's mix is a bijection, so distinct seeds give distinct
        // states and neighbouring seeds land far apart. A zero state would
        // stay zero forever.
        let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        Self(if z == 0 { 0x9E37_79B9_7F4A_7C15 } else { z })
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform in `[0, 1)`.
    pub fn f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Uniform in `[lo, hi)`.
    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.f32()
    }

    /// Uniform index in `0..n`. `n` must be non-zero.
    pub fn index(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_in_range() {
        let (mut a, mut b) = (Rng::new(7), Rng::new(7));
        for _ in 0..1000 {
            let x = a.f32();
            assert_eq!(x, b.f32());
            assert!((0.0..1.0).contains(&x));
            assert!(a.index(5) < 5 && b.index(5) < 5);
        }
    }

    /// Every control that reseeds by adding 1 relies on neighbouring seeds
    /// giving different sequences.
    #[test]
    fn neighbouring_seeds_give_different_sequences() {
        for s in 0..1000 {
            assert_ne!(
                Rng::new(s).next_u64(),
                Rng::new(s + 1).next_u64(),
                "seeds {s} and {}",
                s + 1
            );
        }
    }

    #[test]
    fn distinct_seeds_give_distinct_states() {
        let firsts: std::collections::HashSet<u64> =
            (0..10_000).map(|s| Rng::new(s).next_u64()).collect();
        assert_eq!(firsts.len(), 10_000);
    }

    #[test]
    fn zero_seed_is_not_stuck() {
        let mut r = Rng::new(0x9E37_79B9_7F4A_7C15);
        let first = r.next_u64();
        assert_ne!(first, r.next_u64());
    }
}
