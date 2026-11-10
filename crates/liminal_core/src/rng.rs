//! Seeded random number streams for simulation (spec §5.5, ADR-0009 item 2).
//!
//! All randomness in deterministic crates comes from an [`Rng`]. An `Rng` is one *stream*: the
//! sequence of outputs determined by a world seed (a `u64` chosen by the game or test) and a
//! [`StreamKey`] (a `u64` that names one consumer of randomness). Give every consumer its own
//! key, so that adding draws in one system does not change the numbers another system gets.
//!
//! The generator is xoshiro256\*\* 1.0 (Blackman and Vigna). Its state is seeded with `SplitMix64`
//! from XXH64 of the world seed and the stream key. All outputs are integer arithmetic or exact
//! float conversions, so they are identical on every supported platform.
//!
//! Not cryptographically secure: outputs are predictable from a few earlier outputs. Nothing may
//! use it for security.

use core::num::{NonZeroU32, NonZeroU64};

use crate::error::CoreError;
use crate::hash::{StateHash, StateHasher};

/// Names one consumer of randomness. Together with the world seed it selects a stream
/// ([`Rng::new`]).
///
/// # Examples
///
/// ```
/// use liminal_core::rng::StreamKey;
///
/// let by_name = StreamKey::from_name("boids.spawn");
/// assert_eq!(by_name, StreamKey::from_name("boids.spawn"));
/// assert_ne!(by_name, StreamKey::from_name("boids.steer"));
///
/// // An explicit key, for example an entity index.
/// assert_eq!(StreamKey::new(17).value(), 17);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct StreamKey(u64);

impl StreamKey {
    /// The key with this value.
    #[must_use]
    pub const fn new(key: u64) -> StreamKey {
        StreamKey(key)
    }

    /// The key whose value is XXH64 (seed 0) of the UTF-8 bytes of `name`. The same name gives
    /// the same key in every process and on every platform. Names are compared by their bytes:
    /// `"a.b"` and `"A.B"` are different streams.
    #[must_use]
    pub fn from_name(name: &str) -> StreamKey {
        let mut hasher = StateHasher::new();
        hasher.write_bytes(name.as_bytes());
        StreamKey(hasher.finish())
    }

    /// The key's value.
    #[must_use]
    pub fn value(self) -> u64 {
        self.0
    }
}

/// The value as `u64`.
impl StateHash for StreamKey {
    fn state_hash(&self, hasher: &mut StateHasher) {
        hasher.write_u64(self.0);
    }
}

/// One random number stream: xoshiro256\*\* 1.0 (spec CORE-R090 to CORE-R098).
///
/// The same world seed and stream key give the same sequence in every process and on every
/// supported platform. There is no `Default` and no way to seed from a clock or the OS: every
/// `Rng` comes from a seed and a stream, or from a saved state.
///
/// A clone continues the same sequence independently of the original. Two `Rng`s are equal if
/// their states are equal, that is, if they will produce the same sequence.
///
/// Period 2^256 − 1. Each method documents how many `next_u64` steps it takes.
///
/// # Examples
///
/// ```
/// use core::num::NonZeroU32;
///
/// use liminal_core::error::CoreError;
/// use liminal_core::rng::{Rng, StreamKey};
///
/// let world_seed = 42;
/// let mut spawn = Rng::new(world_seed, StreamKey::from_name("enemies.spawn"));
///
/// let six = NonZeroU32::new(6).expect("6 is not zero");
/// let die = spawn.below_u32(six) + 1; // 1..=6, each with probability exactly 1/6
/// assert!((1..=6).contains(&die));
/// let fraction = spawn.next_f32(); // in [0, 1)
/// assert!((0.0..1.0).contains(&fraction));
///
/// // The same seed and key give the same stream; another key gives another stream.
/// let mut again = Rng::new(world_seed, StreamKey::from_name("enemies.spawn"));
/// assert_eq!(again.below_u32(six) + 1, die);
/// let loot = Rng::new(world_seed, StreamKey::from_name("loot"));
/// assert_ne!(loot, Rng::new(world_seed, StreamKey::from_name("enemies.spawn")));
///
/// // Save and restore: the restored generator continues the sequence.
/// let mut restored = Rng::from_state(spawn.state())?;
/// assert_eq!(restored.next_u64(), spawn.next_u64());
/// # Ok::<(), CoreError>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rng {
    /// The xoshiro256** state words `s0` to `s3`. Never all zero (INV-4).
    state: [u64; 4],
}

impl Rng {
    /// The stream of `world_seed` and `stream`.
    ///
    /// Seeding (CORE-R091): `x` is XXH64 (seed 0) of the 16 bytes `world_seed` little-endian
    /// followed by `stream.value()` little-endian; the four state words are four consecutive
    /// `SplitMix64` outputs started from `x`. They are pairwise distinct, so the state is never
    /// all zero.
    #[must_use]
    pub fn new(world_seed: u64, stream: StreamKey) -> Rng {
        let mut hasher = StateHasher::new();
        hasher.write_u64(world_seed);
        hasher.write_u64(stream.value());
        let mut x = hasher.finish();
        Rng {
            state: core::array::from_fn(|_| splitmix64(&mut x)),
        }
    }

    /// The next 64 random bits. One xoshiro256\*\* step.
    pub fn next_u64(&mut self) -> u64 {
        let [s0, s1, s2, s3] = &mut self.state;
        let result = s1.wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = *s1 << 17;
        *s2 ^= *s0;
        *s3 ^= *s1;
        *s1 ^= *s2;
        *s0 ^= *s3;
        *s2 ^= t;
        *s3 = s3.rotate_left(45);
        result
    }

    /// The next 32 random bits: the upper 32 bits of one [`next_u64`](Self::next_u64) call.
    pub fn next_u32(&mut self) -> u32 {
        high_u32(self.next_u64())
    }

    /// A value in `0..bound`, each with probability exactly `1 / bound` (Lemire 2019, "Fast
    /// Random Integer Generation in an Interval").
    ///
    /// Takes one [`next_u32`](Self::next_u32) call, and with probability below `bound / 2^32`
    /// one or more further calls (rejection). For a `bound` that is a power of two `2^k`, no
    /// rejection occurs and the result is `next_u32() >> (32 − k)`.
    pub fn below_u32(&mut self, bound: NonZeroU32) -> u32 {
        let range = bound.get();
        let mut product = u64::from(self.next_u32()) * u64::from(range);
        if low_u32(product) < range {
            // (2^32 − range) mod range: the number of low halves to reject.
            let threshold = range.wrapping_neg() % range;
            while low_u32(product) < threshold {
                product = u64::from(self.next_u32()) * u64::from(range);
            }
        }
        high_u32(product)
    }

    /// A value in `0..bound`, each with probability exactly `1 / bound`: as
    /// [`below_u32`](Self::below_u32), with [`next_u64`](Self::next_u64), 128-bit products and
    /// `2^64`.
    pub fn below_u64(&mut self, bound: NonZeroU64) -> u64 {
        let range = bound.get();
        let mut product = u128::from(self.next_u64()) * u128::from(range);
        if low_u64(product) < range {
            // (2^64 − range) mod range.
            let threshold = range.wrapping_neg() % range;
            while low_u64(product) < threshold {
                product = u128::from(self.next_u64()) * u128::from(range);
            }
        }
        high_u64(product)
    }

    /// A value in `[0, 1)`: a multiple of 2^-24, each of the 2^24 values with equal probability.
    /// `(next_u64() >> 40) as f32 × 2^-24`, computed exactly. One `next_u64` step.
    pub fn next_f32(&mut self) -> f32 {
        /// 2^-24.
        const SCALE: f32 = 1.0 / 16_777_216.0;
        // A 24-bit integer: exactly representable in the 24-bit significand of an `f32`.
        let mantissa = self.next_u64() >> 40;
        #[expect(
            clippy::cast_precision_loss,
            reason = "the value is below 2^24, so the conversion is exact"
        )]
        let value = mantissa as f32;
        value * SCALE
    }

    /// A value in `[0, 1)`: a multiple of 2^-53, each of the 2^53 values with equal probability.
    /// `(next_u64() >> 11) as f64 × 2^-53`, computed exactly. One `next_u64` step.
    pub fn next_f64(&mut self) -> f64 {
        /// 2^-53.
        const SCALE: f64 = 1.0 / 9_007_199_254_740_992.0;
        // A 53-bit integer: exactly representable in the 53-bit significand of an `f64`.
        let mantissa = self.next_u64() >> 11;
        #[expect(
            clippy::cast_precision_loss,
            reason = "the value is below 2^53, so the conversion is exact"
        )]
        let value = mantissa as f64;
        value * SCALE
    }

    /// The four state words, for save games and replays. [`Rng::from_state`] restores them.
    #[must_use]
    pub fn state(&self) -> [u64; 4] {
        self.state
    }

    /// The generator with these state words. `Rng::from_state(r.state())` continues the
    /// sequence of `r`.
    ///
    /// # Errors
    ///
    /// [`CoreError::ZeroRngState`] (LIM-CORE-0030) if all four words are zero: the generator
    /// would output 0 forever.
    pub fn from_state(state: [u64; 4]) -> Result<Rng, CoreError> {
        if state == [0; 4] {
            return Err(CoreError::ZeroRngState);
        }
        Ok(Rng { state })
    }
}

/// The four state words as `u64`, in order.
impl StateHash for Rng {
    fn state_hash(&self, hasher: &mut StateHasher) {
        for word in self.state {
            hasher.write_u64(word);
        }
    }
}

/// One `SplitMix64` step (reference `splitmix64.c`): advances `x` and returns the output.
fn splitmix64(x: &mut u64) -> u64 {
    *x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// `value mod 2^32`.
#[expect(
    clippy::cast_possible_truncation,
    reason = "taking the low 32 bits is the operation"
)]
fn low_u32(value: u64) -> u32 {
    value as u32
}

/// `value / 2^32`.
fn high_u32(value: u64) -> u32 {
    low_u32(value >> 32)
}

/// `value mod 2^64`.
#[expect(
    clippy::cast_possible_truncation,
    reason = "taking the low 64 bits is the operation"
)]
fn low_u64(value: u128) -> u64 {
    value as u64
}

/// `value / 2^64`.
fn high_u64(value: u128) -> u64 {
    low_u64(value >> 64)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies: CORE-R091
    #[test]
    fn splitmix64_matches_the_published_reference_outputs() {
        // The first 12 of the 50 outputs for seed 1477776061723855037 in the `reference` test of
        // rand_xoshiro/src/splitmix64.rs (github.com/rust-random/rngs, commit 84683fca), which
        // states: "These values were produced with the reference implementation:
        // http://xoshiro.di.unimi.it/splitmix64.c".
        let expected: [u64; 12] = [
            1_985_237_415_132_408_290,
            2_979_275_885_539_914_483,
            13_511_426_838_097_143_398,
            8_488_337_342_461_049_707,
            15_141_737_807_933_549_159,
            17_093_170_987_380_407_015,
            16_389_528_042_912_955_399,
            13_177_319_091_862_933_652,
            10_841_969_400_225_389_492,
            17_094_824_097_954_834_098,
            3_336_622_647_361_835_228,
            9_678_412_372_263_018_368,
        ];
        let mut x = 1_477_776_061_723_855_037;
        for value in expected {
            assert_eq!(splitmix64(&mut x), value);
        }
    }
}
