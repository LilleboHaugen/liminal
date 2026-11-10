//! State hashing (spec §5.3): one fixed hash function, XXH64 with seed 0 over a canonical
//! little-endian byte encoding, so equal values give equal hashes in every process and on every
//! supported platform (ADR-0009, ADR-0024).
//!
//! - [`StateHasher`]: the streaming hasher that determinism tests compare.
//! - [`StateHash`]: the canonical encoding of a value; [`state_hash_of`] hashes one value.
//! - [`FixedState`] and [`DetHasher`]: the same hash function as a `core::hash::BuildHasher`, for
//!   hash collections that must not be randomly seeded.
//!
//! XXH64 is not a cryptographic hash: an adversary can construct collisions. Nothing may use it
//! for security. For inputs that are not chosen by an adversary, two unequal states have equal
//! hashes with probability about 2^-64 (assumption: XXH64 behaves as a random function there).
//!
//! No other crate defines its own state hasher (ADR-0024): changing the algorithm or an encoding
//! changes every recorded hash in the repository.

mod fixed;
mod state_hash;
mod state_hasher;
mod xxh64;

pub use fixed::{DetHasher, FixedState};
pub use state_hash::{StateHash, state_hash_of};
pub use state_hasher::StateHasher;
