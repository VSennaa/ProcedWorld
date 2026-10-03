//! Versioned xoshiro256** pseudo-random number generator.

use crate::hash::fnv1a;

/// Bump whenever the algorithm or seed expansion changes.
pub const RNG_VERSION: u32 = 1;

/// xoshiro256** seeded through splitmix64.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rng {
    state: [u64; 4],
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        let mut seed_state = seed;
        Self {
            state: [
                splitmix64(&mut seed_state),
                splitmix64(&mut seed_state),
                splitmix64(&mut seed_state),
                splitmix64(&mut seed_state),
            ],
        }
    }

    /// Creates an independent named stream from a world seed.
    pub fn derive(seed: u64, stage: &str) -> Self {
        Self::new(seed ^ fnv1a(stage.as_bytes()))
    }

    pub fn next_u64(&mut self) -> u64 {
        let result = self.state[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let temporary = self.state[1] << 17;

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];
        self.state[2] ^= temporary;
        self.state[3] = self.state[3].rotate_left(45);
        result
    }

    /// Returns a uniform number in `0..n` using Lemire's multiply-and-reject method.
    ///
    /// `n` must be non-zero.
    pub fn below(&mut self, n: u32) -> u32 {
        assert!(n != 0, "Rng::below requires a non-zero bound");
        let mut product = (self.next_u64() >> 32) * u64::from(n);
        if (product as u32) < n {
            let threshold = n.wrapping_neg() % n;
            while (product as u32) < threshold {
                product = (self.next_u64() >> 32) * u64::from(n);
            }
        }
        (product >> 32) as u32
    }
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut value = *state;
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rng_is_reproducible() {
        let mut left = Rng::new(42);
        let mut right = Rng::new(42);
        for _ in 0..1_000 {
            assert_eq!(left.next_u64(), right.next_u64());
        }
    }

    #[test]
    fn derived_stages_are_reproducible_and_independent() {
        let mut first = Rng::derive(9, "elevation");
        let mut second = Rng::derive(9, "elevation");
        assert_eq!(first.next_u64(), second.next_u64());
        assert_ne!(Rng::derive(9, "elevation"), Rng::derive(9, "fertility"));
    }

    #[test]
    fn below_stays_in_range() {
        let mut rng = Rng::new(7);
        for bound in [1_u32, 2, 3, 6, 1_000, u32::MAX] {
            for _ in 0..2_000 {
                assert!(rng.below(bound) < bound);
            }
        }
    }
}
