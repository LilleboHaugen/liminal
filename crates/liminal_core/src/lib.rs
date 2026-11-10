//! Foundation types that every Liminal engine crate shares and that must mean the same thing
//! everywhere: the error model with stable codes ([`error`], ADR-0017) and generational handles
//! ([`handle`]).
//!
//! Specification: `docs/specs/liminal_core.md`. The modules `hash`, `collections`, `rng` and
//! `time` of that spec are not implemented yet (task M1-T014).
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
//! # Ok::<(), CoreError>(())
//! ```

#![forbid(unsafe_code)]

pub mod error;
pub mod handle;
