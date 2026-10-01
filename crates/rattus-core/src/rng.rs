//! A small deterministic PRNG.
//!
//! xoshiro256++ seeded through SplitMix64. It is fast, gives the same sequence on every
//! platform and serializes as four integers, so a saved experiment continues exactly where
//! it stopped.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rng {
    s: [u64; 4],
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        let mut state = seed;
        let mut next = || {
            state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        };
        Self {
            s: [next(), next(), next(), next()],
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let s = &mut self.s;
        let result = s[0].wrapping_add(s[3]).rotate_left(23).wrapping_add(s[0]);
        let t = s[1] << 17;
        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(45);
        result
    }

    /// Uniform in `[0, 1)`.
    pub fn f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// Uniform in `[lo, hi)`.
    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.f64()
    }

    pub fn chance(&mut self, p: f64) -> bool {
        self.f64() < p
    }

    /// Uniform integer in `[lo, hi]`.
    pub fn int_inclusive(&mut self, lo: u32, hi: u32) -> u32 {
        debug_assert!(lo <= hi);
        lo + (self.next_u64() % (u64::from(hi - lo) + 1)) as u32
    }

    /// Picks an item with probability proportional to its weight. Non-positive weights are skipped.
    pub fn pick_weighted<T: Copy>(&mut self, items: &[(T, f64)]) -> Option<T> {
        let total: f64 = items.iter().map(|(_, w)| w.max(0.0)).sum();
        if total <= 0.0 {
            return None;
        }
        let mut target = self.f64() * total;
        for &(item, w) in items {
            let w = w.max(0.0);
            if target < w {
                return Some(item);
            }
            target -= w;
        }
        items
            .iter()
            .rev()
            .find(|(_, w)| *w > 0.0)
            .map(|(item, _)| *item)
    }

    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.int_inclusive(0, i as u32) as usize;
            items.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn f64_in_unit_interval_with_sane_mean() {
        let mut rng = Rng::new(7);
        let n = 100_000;
        let mut sum = 0.0;
        for _ in 0..n {
            let x = rng.f64();
            assert!((0.0..1.0).contains(&x));
            sum += x;
        }
        assert!((sum / n as f64 - 0.5).abs() < 0.01);
    }

    #[test]
    fn int_inclusive_covers_bounds() {
        let mut rng = Rng::new(1);
        let mut seen = [false; 5];
        for _ in 0..1000 {
            seen[rng.int_inclusive(2, 6) as usize - 2] = true;
        }
        assert!(seen.iter().all(|s| *s));
    }

    #[test]
    fn weighted_pick_respects_weights() {
        let mut rng = Rng::new(3);
        let items = [('a', 1.0), ('b', 3.0), ('c', 0.0)];
        let mut counts = [0; 3];
        for _ in 0..40_000 {
            match rng.pick_weighted(&items).unwrap() {
                'a' => counts[0] += 1,
                'b' => counts[1] += 1,
                _ => counts[2] += 1,
            }
        }
        assert_eq!(counts[2], 0);
        let ratio = counts[1] as f64 / counts[0] as f64;
        assert!((ratio - 3.0).abs() < 0.2, "ratio {ratio}");
    }

    #[test]
    fn survives_serde_round_trip() {
        let mut rng = Rng::new(99);
        rng.next_u64();
        let json = serde_json::to_string(&rng).unwrap();
        let mut back: Rng = serde_json::from_str(&json).unwrap();
        assert_eq!(rng.next_u64(), back.next_u64());
    }
}
