//! Streaming XXH64 with seed 0 (xxHash specification, `doc/xxhash_spec.md` in the xxHash
//! repository, algorithm XXH64). The only hash function of the crate; [`StateHasher`] and
//! [`DetHasher`] wrap it.
//!
//! [`StateHasher`]: super::StateHasher
//! [`DetHasher`]: super::DetHasher

const PRIME_1: u64 = 0x9E37_79B1_85EB_CA87;
const PRIME_2: u64 = 0xC2B2_AE3D_27D4_EB4F;
const PRIME_3: u64 = 0x1656_67B1_9E37_79F9;
const PRIME_4: u64 = 0x85EB_CA77_C2B2_AE63;
const PRIME_5: u64 = 0x27D4_EB2F_1656_67C5;

/// Bytes per stripe: four 8-byte lanes, one per accumulator.
const STRIPE_LEN: usize = 32;

/// XXH64 state for seed 0. `write` appends bytes to one sequence; `finish` hashes the sequence.
#[derive(Clone, Debug)]
pub(crate) struct Xxh64 {
    /// The four accumulators, updated once per complete stripe.
    accumulators: [u64; 4],
    /// Bytes written since the last complete stripe; only `buffer[..buffered]` is meaningful.
    buffer: [u8; STRIPE_LEN],
    /// Number of meaningful bytes in `buffer`, always below `STRIPE_LEN`.
    buffered: usize,
    /// Total number of bytes written, modulo 2^64 (as the specification defines).
    total_len: u64,
}

impl Xxh64 {
    /// The state with nothing written.
    pub(crate) fn new() -> Xxh64 {
        Xxh64 {
            // Seed 0: seed + PRIME_1 + PRIME_2, seed + PRIME_2, seed, seed - PRIME_1.
            accumulators: [
                PRIME_1.wrapping_add(PRIME_2),
                PRIME_2,
                0,
                0_u64.wrapping_sub(PRIME_1),
            ],
            buffer: [0; STRIPE_LEN],
            buffered: 0,
            total_len: 0,
        }
    }

    /// Appends `bytes` to the sequence.
    pub(crate) fn write(&mut self, bytes: &[u8]) {
        self.total_len = self.total_len.wrapping_add(len_u64(bytes.len()));
        let mut rest = bytes;
        if self.buffered > 0 {
            let take = rest.len().min(STRIPE_LEN - self.buffered);
            let (head, tail) = rest.split_at(take);
            self.buffer[self.buffered..self.buffered + take].copy_from_slice(head);
            self.buffered += take;
            rest = tail;
            if self.buffered < STRIPE_LEN {
                return;
            }
            self.accumulators = consume_stripe(self.accumulators, &self.buffer);
            self.buffered = 0;
        }
        let (stripes, tail) = rest.as_chunks::<STRIPE_LEN>();
        for stripe in stripes {
            self.accumulators = consume_stripe(self.accumulators, stripe);
        }
        self.buffer[..tail.len()].copy_from_slice(tail);
        self.buffered = tail.len();
    }

    /// XXH64 of all bytes written so far. Does not change the state.
    pub(crate) fn finish(&self) -> u64 {
        let [a, b, c, d] = self.accumulators;
        let mut hash = if self.total_len >= len_u64(STRIPE_LEN) {
            let mut merged = a
                .rotate_left(1)
                .wrapping_add(b.rotate_left(7))
                .wrapping_add(c.rotate_left(12))
                .wrapping_add(d.rotate_left(18));
            for accumulator in self.accumulators {
                merged = (merged ^ round(0, accumulator))
                    .wrapping_mul(PRIME_1)
                    .wrapping_add(PRIME_4);
            }
            merged
        } else {
            // Fewer than 32 bytes in total: seed + PRIME_5; the third accumulator is the seed.
            c.wrapping_add(PRIME_5)
        };
        hash = hash.wrapping_add(self.total_len);

        let (lanes, rest) = self.buffer[..self.buffered].as_chunks::<8>();
        for lane in lanes {
            hash = (hash ^ round(0, u64::from_le_bytes(*lane)))
                .rotate_left(27)
                .wrapping_mul(PRIME_1)
                .wrapping_add(PRIME_4);
        }
        let (words, rest) = rest.as_chunks::<4>();
        for word in words {
            hash = (hash ^ u64::from(u32::from_le_bytes(*word)).wrapping_mul(PRIME_1))
                .rotate_left(23)
                .wrapping_mul(PRIME_2)
                .wrapping_add(PRIME_3);
        }
        for byte in rest {
            hash = (hash ^ u64::from(*byte).wrapping_mul(PRIME_5))
                .rotate_left(11)
                .wrapping_mul(PRIME_1);
        }

        // Final avalanche.
        hash ^= hash >> 33;
        hash = hash.wrapping_mul(PRIME_2);
        hash ^= hash >> 29;
        hash = hash.wrapping_mul(PRIME_3);
        hash ^ (hash >> 32)
    }
}

/// One accumulator round: mixes one 8-byte lane into an accumulator.
fn round(accumulator: u64, lane: u64) -> u64 {
    accumulator
        .wrapping_add(lane.wrapping_mul(PRIME_2))
        .rotate_left(31)
        .wrapping_mul(PRIME_1)
}

/// Feeds one 32-byte stripe to the four accumulators, lane `i` to accumulator `i`.
fn consume_stripe(accumulators: [u64; 4], stripe: &[u8; STRIPE_LEN]) -> [u64; 4] {
    let (lanes, _) = stripe.as_chunks::<8>();
    let mut next = accumulators;
    for (accumulator, lane) in next.iter_mut().zip(lanes) {
        *accumulator = round(*accumulator, u64::from_le_bytes(*lane));
    }
    next
}

/// A length as `u64`. Supported targets are 64-bit (spec §2), so the conversion is lossless.
pub(crate) fn len_u64(len: usize) -> u64 {
    u64::try_from(len).expect("invariant: usize is at most 64 bits")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies: CORE-R050
    #[test]
    fn the_empty_input_hashes_to_the_published_value() {
        // XSUM_XXH64_testdata[0] of tests/sanity_test_vectors.h, xxHash v0.8.3 (length 0, seed 0).
        assert_eq!(Xxh64::new().finish(), 0xEF46_DB37_51D8_E999);
    }

    /// Verifies: CORE-R051
    #[test]
    fn a_write_that_exactly_fills_the_buffer_consumes_one_stripe() {
        let bytes: Vec<u8> = (0_u8..96).collect();
        let mut whole = Xxh64::new();
        whole.write(&bytes);
        let mut parts = Xxh64::new();
        parts.write(&bytes[..5]);
        parts.write(&bytes[5..32]);
        assert_eq!(parts.buffered, 0);
        parts.write(&bytes[32..]);
        assert_eq!(parts.finish(), whole.finish());
    }
}
