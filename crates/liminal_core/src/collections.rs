//! Hash collections whose iteration order is defined (spec §5.4): [`DetHashMap`] and
//! [`DetHashSet`] iterate in the order their keys or values were first inserted.
//!
//! Simulation state must not use collections whose iteration order depends on a random seed, on
//! hash values or on capacity (ADR-0009). In these two the order depends only on the sequence of
//! operations, so it is the same in every process and on every platform. Lookups hash with
//! [`FixedState`](crate::hash::FixedState) (XXH64, seed 0).
//!
//! Cost: `insert`, `get` and `contains` take expected constant time; `remove` takes time linear
//! in the length, because it keeps the order of the remaining entries.

mod index_table;
mod map;
mod set;

pub use map::DetHashMap;
pub use set::DetHashSet;
