//! Foundation types that every Liminal engine crate shares and that must mean the same thing
//! everywhere:
//!
//! - [`error`]: the error model with stable codes (ADR-0017).
//! - [`handle`]: generational handles and the slot map.
//! - [`hash`]: the state hasher that determinism tests compare (XXH64, seed 0; ADR-0024) and a
//!   fixed-seed `BuildHasher`.
//! - [`collections`]: hash map and set that iterate in insertion order (ADR-0009).
//! - [`rng`]: seeded random number streams for simulation (xoshiro256\*\*; ADR-0009).
//! - [`time`]: tick rate, tick index and tick duration of the fixed timestep.
//!
//! Specification: `docs/specs/liminal_core.md`.
//!
//! Every public item is public only at its module path (`liminal_core::error::ErrorCode`); the
//! crate root re-exports nothing. The crate has no feature flags, no dependencies and no `unsafe`
//! code, and it reads no clock, environment or OS randomness (ADR-0009).
//!
//! # Examples
//!
//! ```
//! use liminal_core::error::{CoreError, EngineError, render_json};
//! use liminal_core::handle::HandleMap;
//!
//! let mut names: HandleMap<&str> = HandleMap::new();
//! let hero = names.insert("hero")?;
//! assert_eq!(names.get(hero), Some(&"hero"));
//!
//! // A removed handle is stale: lookups fail instead of reaching a reused slot.
//! assert_eq!(names.remove(hero), Some("hero"));
//! assert_eq!(names.get(hero), None);
//!
//! // Errors carry a stable code, a message, a help text and structured context.
//! let error = CoreError::StaleHandle { index: 3, generation: 2 };
//! assert_eq!(error.code().to_string(), "LIM-CORE-0010");
//! assert!(render_json(&error).starts_with(r#"{"code":"LIM-CORE-0010","#));
//!
//! // Simulation randomness is a stream named by a world seed and a key; state is compared by
//! // its hash. Both are the same in every process and on every supported platform.
//! use liminal_core::hash::state_hash_of;
//! use liminal_core::rng::{Rng, StreamKey};
//!
//! let mut rng = Rng::new(42, StreamKey::from_name("example"));
//! let rolls: Vec<u64> = (0..3).map(|_| rng.next_u64()).collect();
//! let mut again = Rng::new(42, StreamKey::from_name("example"));
//! let same: Vec<u64> = (0..3).map(|_| again.next_u64()).collect();
//! assert_eq!(state_hash_of(&rolls), state_hash_of(&same));
//! # Ok::<(), CoreError>(())
//! ```

#![forbid(unsafe_code)]

pub mod collections;
pub mod error;
pub mod handle;
pub mod hash;
pub mod rng;
pub mod time;
