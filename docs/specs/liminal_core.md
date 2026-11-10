---
crate: liminal_core
req_prefix: CORE
layer: 0
maturity: usable
spec_version: 4
last_reviewed: 2026-11-09
adrs: [ADR-0009, ADR-0017, ADR-0018, ADR-0024]
---

# Spec: `liminal_core`

## 1. Purpose

`liminal_core` holds the small set of types that every other engine crate needs and that must mean
the same thing everywhere: the error model with stable codes (ADR-0017), generational handles, the
state hasher that determinism tests compare (ADR-0024), collections with a defined iteration order
and seeded RNG streams for simulation (ADR-0009), and the tick-based time types of the fixed
timestep. It is a separate crate at layer 0 with no dependencies, so that every crate can use it
and so that one definition of each of these exists (two state hashers or two RNGs would make
recorded hashes disagree).

## 2. Scope

**In scope:**
- `error`: `ErrorCode` (`LIM-<AREA>-<NNNN>`), the `EngineError` trait every engine error
  implements, structured context, the one-line text form, the JSON form, and a contract checker
  other crates call from their tests.
- `handle`: generational handles (`RawHandle`, typed `Handle<T>`), the `HandleAllocator`, and the
  slot map `HandleMap<T>`.
- `hash`: the state hasher (`StateHasher`, `StateHash` trait; XXH64 with seed 0, little-endian
  encoding), and a fixed-seed `BuildHasher` (`FixedState`) for `std` collections.
- `collections`: `DetHashMap` and `DetHashSet`, hash collections whose iteration order is the
  insertion order.
- `rng`: `Rng` (xoshiro256\*\*), streams derived from a world seed and a `StreamKey`.
- `time`: `TickRate` (fixed `dt`), `Tick` (a simulation step index), `TickDuration` (a number of
  ticks).
- The crate's own error enum `CoreError` (codes `LIM-CORE-NNNN`, §8).

**Out of scope / non-goals:**
- Converting variable frame time into fixed steps (the accumulator, interpolation factor,
  spiral-of-death clamp): `liminal_app`. `liminal_core` never reads a clock.
- Hashing all simulation state of a world (`World::state_hash()`): `liminal_ecs`, built on
  `StateHasher` (ADR-0009 item 6). Reflection-driven hashing: `liminal_ecs` / `liminal_reflect`.
- Floating-point math, transcendental functions (`liminal_math::det`), vectors: `liminal_math`.
- Allocators: `liminal_memory`. Logging and spans: `liminal_telemetry`.
- General serialization of errors and other types (beyond the fixed error JSON of §5.1):
  `liminal_serial` (M2, OQ-002).
- Entity IDs: `liminal_ecs` decides whether `Entity` wraps `RawHandle` (open question 5).
- Cryptographic hashing or randomness. XXH64 and xoshiro256\*\* are not secure against an
  adversary; nothing in the engine may use them for security.
- `no_std`. The crate uses `std` (`String`, `Vec`, `std::error::Error`, `std::collections`).
- 32-bit targets. Supported targets are 64-bit (ADR-0005); `usize` is hashed as 64 bits.

## 3. Dependencies

- Engine crates: none (`architecture.toml`: layer 0, `same_layer_deps = []`).
- External crates: none at runtime (`external = []`). Dev-dependency `proptest` only
  (`dev_external`, ADR-0010).
- `unsafe = "forbid"` (ADR-0018): the crate root has `#![forbid(unsafe_code)]`.
- `deterministic = true`: the determinism lint configuration of CODE_STYLE.md (mechanism chosen by
  M1-T011) applies, and the crate meets §11.
- Structure: modules `error`, `handle`, `hash`, `collections`, `rng`, `time`. Every public item is
  public only at its module path (`liminal_core::rng::Rng`). The crate root declares the modules
  and re-exports nothing (CODE_STYLE.md "One obvious way").

## 4. Concepts

- **Error code:** `LIM-<AREA>-<NNNN>`. `AREA` is the `req_prefix` of the spec of the crate that
  defines the error (1 to 8 ASCII uppercase letters; `CORE` here). `NNNN` is exactly four decimal
  digits, `0001` to `9999`. A code is never reused or renumbered (ADR-0017).
- **Engine error:** a value of a type implementing `EngineError`: a code, a message (one or more
  sentences saying what happened), an optional help text (how to fix it, when knowable), and a
  context (an ordered list of `key = value` fields with the data the message talks about).
- **Error contract:** the rules every engine error meets (§5.1, CORE-R012). Each crate checks its
  own errors with `validate_error` in its tests.
- **Slot, index, generation:** a `HandleAllocator` owns slots numbered by a `u32` index. Each slot
  has a generation (`u32`, starting at 1). A handle names a slot and the generation it had when the
  handle was issued; it is *live* while the slot is occupied with that generation. Freeing a slot
  increments its generation, so all earlier handles to it become stale. A slot whose generation
  would pass `u32::MAX` is *retired* and never used again, so no handle value is issued twice
  (no ABA).

  ```
  slot 3:  gen 1 (live: 3v1) --free--> gen 2 (free) --allocate--> gen 2 (live: 3v2) --free--> gen 3 ...
           ... gen 4294967295 (live) --free--> retired (never reallocated)
  ```
- **State hash:** a `u64` computed by `StateHasher` over a canonical byte encoding of values
  (§5.3). Equal state gives equal hashes in every process and on every supported platform. Unequal
  state gives equal hashes with probability about 2^-64 per comparison (assumption: XXH64 behaves
  as a random function on these inputs; it is not collision-resistant against chosen inputs).
- **World seed, stream key, stream:** the world seed is a `u64` chosen by the game or test. A
  stream key is a `u64` that names one consumer of randomness (`StreamKey::from_name("boids.spawn")`).
  The pair (world seed, stream key) determines a stream: the sequence of an `Rng`'s outputs.
- **Tick, tick rate:** the simulation advances in ticks of fixed length `dt = 1 / rate` seconds.
  `Tick(n)` is the n-th step (`Tick::ZERO` is the first). Time inside the simulation is counted in
  ticks, which are integers, so it never accumulates rounding error.

## 5. Public API

Signatures are normative; bodies and private items are not. Derives listed are part of the API.

### 5.1 `liminal_core::error`

```rust
/// `LIM-<AREA>-<NNNN>`. Copy, at most 16 bytes.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ErrorCode { /* area: [u8; 8] + length, number: u16 */ }

impl ErrorCode {
    /// For constants: `const NOT_REGISTERED: ErrorCode = ErrorCode::new("ECS", 3);`
    /// Panics if `area` is not 1–8 ASCII uppercase letters or `number` is not 1..=9999
    /// (a compile error when evaluated in a const context).
    pub const fn new(area: &'static str, number: u16) -> ErrorCode;
    /// Parses `LIM-ECS-0003`. Errors: LIM-CORE-0001.
    pub fn parse(text: &str) -> Result<ErrorCode, CoreError>;
    pub fn area(&self) -> &str;
    pub fn number(&self) -> u16;
}
impl core::fmt::Display for ErrorCode { /* `LIM-ECS-0003` */ }

/// Implemented by every engine error type (ADR-0017). Dyn-compatible.
pub trait EngineError: std::error::Error {
    fn code(&self) -> ErrorCode;
    /// What happened. Non-empty. Does not contain the code or the help text.
    fn message(&self) -> String;
    /// How to fix it, if knowable. `Some` is never empty.
    fn help(&self) -> Option<String>;
    /// Structured fields, in a fixed order per variant.
    fn context(&self) -> ErrorContext;
}

/// Writes the text form: `{code}: {message}`, then ` help: {help}` if help is `Some`.
/// Every `EngineError` implementation's `Display` is exactly this (CORE-R006, CORE-R012).
pub fn write_display(error: &dyn EngineError, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result;

/// The JSON form (CORE-R008 … CORE-R011), one line, no trailing newline.
pub fn render_json(error: &dyn EngineError) -> String;

/// Checks the error contract (CORE-R012) for one error value; returns the first violation.
pub fn validate_error(error: &dyn EngineError, expected_area: &str) -> Result<(), CoreError>;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ErrorContext { /* Vec<(&'static str, ContextValue)> */ }

impl ErrorContext {
    pub fn new() -> ErrorContext;
    /// Appends a field. Key form and uniqueness are checked by `validate_error`, not here.
    pub fn with(self, key: &'static str, value: impl Into<ContextValue>) -> ErrorContext;
    /// First field with this key.
    pub fn get(&self, key: &str) -> Option<&ContextValue>;
    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &ContextValue)>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}

#[derive(Clone, Debug, PartialEq)]
pub enum ContextValue { Bool(bool), Int(i64), UInt(u64), Float(f64), Text(String) }
// From impls: bool; i8, i16, i32, i64, isize -> Int; u8, u16, u32, u64, usize -> UInt;
// f32, f64 -> Float (f32 widened exactly); &str, String -> Text; ErrorCode -> Text (its Display).

/// The crate's own errors (§8).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CoreError {
    InvalidErrorCode { text: String },                                  // LIM-CORE-0001
    InvalidContextKey { code: ErrorCode, key: String },                 // LIM-CORE-0002
    DuplicateContextKey { code: ErrorCode, key: String },               // LIM-CORE-0003
    EmptyMessage { code: ErrorCode },                                   // LIM-CORE-0004
    EmptyHelp { code: ErrorCode },                                      // LIM-CORE-0005
    DisplayMismatch { code: ErrorCode, expected: String, actual: String }, // LIM-CORE-0006
    AreaMismatch { code: ErrorCode, expected_area: String },           // LIM-CORE-0007
    StaleHandle { index: u32, generation: u32 },                        // LIM-CORE-0010
    HandleSlotsExhausted { slot_limit: u32 },                           // LIM-CORE-0011
    InvalidTickRate { ticks_per_second: u32 },                          // LIM-CORE-0020
    ZeroRngState,                                                       // LIM-CORE-0030
}
impl EngineError for CoreError { /* … */ }
impl std::error::Error for CoreError {}
impl core::fmt::Display for CoreError { /* write_display */ }
```

JSON form (`render_json`), shown here with whitespace for reading; the real output has none:

```json
{
  "code": "LIM-CORE-0010",
  "message": "handle `3v2` is not live.",
  "help": "the handle was freed already or was not issued by this allocator; check `is_live` before freeing.",
  "context": { "index": 3, "generation": 2 }
}
```

`help` is `null` when `help()` is `None`. `context` is `{}` when empty. `source()` (the
`std::error::Error` cause chain) is not rendered (open question 1).

### 5.2 `liminal_core::handle`

```rust
/// Untyped generational handle: `index` (u32) and `generation` (NonZeroU32). 8 bytes;
/// `Option<RawHandle>` is also 8 bytes.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct RawHandle { /* index: u32, generation: NonZeroU32 */ }

impl RawHandle {
    pub fn index(self) -> u32;
    pub fn generation(self) -> core::num::NonZeroU32;
    /// `generation × 2^32 + index`.
    pub fn to_bits(self) -> u64;
    /// `None` if the upper 32 bits (the generation) are zero.
    pub fn from_bits(bits: u64) -> Option<RawHandle>;
}
impl core::fmt::Display for RawHandle { /* `{index}v{generation}`, e.g. `3v2` */ }

/// Typed handle. Copy, Eq, Ord, Hash, Debug for every `T` (implemented by hand, no bounds on T).
pub struct Handle<T> { /* raw: RawHandle, PhantomData<fn() -> T> */ }

impl<T> Handle<T> {
    pub fn from_raw(raw: RawHandle) -> Handle<T>;
    pub fn raw(self) -> RawHandle;
}

/// Issues and frees raw handles (no values stored).
#[derive(Clone, Debug)]
pub struct HandleAllocator { /* … */ }

impl HandleAllocator {
    /// Slot limit u32::MAX.
    pub fn new() -> HandleAllocator;
    /// At most `slot_limit` slots (indices 0..slot_limit) are ever used.
    pub fn with_slot_limit(slot_limit: u32) -> HandleAllocator;
    /// Errors: LIM-CORE-0011.
    pub fn allocate(&mut self) -> Result<RawHandle, CoreError>;
    /// Errors: LIM-CORE-0010.
    pub fn free(&mut self, handle: RawHandle) -> Result<(), CoreError>;
    pub fn is_live(&self, handle: RawHandle) -> bool;
    pub fn live_count(&self) -> u32;
    /// Live handles in ascending index order.
    pub fn iter_live(&self) -> impl Iterator<Item = RawHandle> + '_;
}
/// Not derived: a derived `Default` would set the slot limit to 0 (CORE-R041).
impl Default for HandleAllocator { /* new() */ }

/// Slot map: stores one `T` per live handle.
#[derive(Clone, Debug)]
pub struct HandleMap<T> { /* … */ }

impl<T> HandleMap<T> {
    pub fn new() -> HandleMap<T>;
    pub fn with_slot_limit(slot_limit: u32) -> HandleMap<T>;
    /// Errors: LIM-CORE-0011 (the value is dropped).
    pub fn insert(&mut self, value: T) -> Result<Handle<T>, CoreError>;
    pub fn get(&self, handle: Handle<T>) -> Option<&T>;
    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T>;
    /// `None` (map unchanged) if the handle is not live.
    pub fn remove(&mut self, handle: Handle<T>) -> Option<T>;
    pub fn contains(&self, handle: Handle<T>) -> bool;
    pub fn len(&self) -> u32;
    pub fn is_empty(&self) -> bool;
    /// Ascending index order.
    pub fn iter(&self) -> impl Iterator<Item = (Handle<T>, &T)> + '_;
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Handle<T>, &mut T)> + '_;
}
impl<T> Default for HandleMap<T> { /* new() */ }
```

### 5.3 `liminal_core::hash`

```rust
/// XXH64, seed 0, streaming. Every write appends bytes to one sequence; `finish` hashes it.
#[derive(Clone, Debug)]
pub struct StateHasher { /* … */ }

impl StateHasher {
    pub fn new() -> StateHasher;
    /// Appends the bytes as they are (no length prefix).
    pub fn write_bytes(&mut self, bytes: &[u8]);
    pub fn write_u8(&mut self, value: u8);    // and u16, u32, u64, u128: little-endian, natural width
    pub fn write_i8(&mut self, value: i8);    // and i16, i32, i64, i128: little-endian two's complement
    pub fn write_usize(&mut self, value: usize); // as u64
    pub fn write_isize(&mut self, value: isize); // as i64
    pub fn write_bool(&mut self, value: bool);   // one byte, 0x00 or 0x01
    pub fn write_char(&mut self, value: char);   // as u32
    pub fn write_f32(&mut self, value: f32);     // to_bits() as u32 (no NaN or -0.0 canonicalization)
    pub fn write_f64(&mut self, value: f64);     // to_bits() as u64
    pub fn write_str(&mut self, value: &str);    // byte length as u64, then the UTF-8 bytes
    /// Hash of all bytes written so far. Does not change the hasher.
    pub fn finish(&self) -> u64;
}
impl Default for StateHasher { /* new() */ }

/// Canonical state encoding. Not `core::hash::Hash`: std `Hash` impls are not guaranteed
/// stable across Rust releases or platforms.
pub trait StateHash {
    fn state_hash(&self, hasher: &mut StateHasher);
}
// Implemented for: all primitive integers, bool, char, f32, f64, str, String, [T], Vec<T>,
// [T; N], Option<T>, &T, tuples of 1 to 4 elements, and the liminal_core types listed in
// CORE-R054.

/// `let mut h = StateHasher::new(); value.state_hash(&mut h); h.finish()`.
pub fn state_hash_of<T: StateHash + ?Sized>(value: &T) -> u64;

/// Fixed-seed `BuildHasher` for `std` collections. Iteration order of a `std` HashMap that
/// uses it is NOT defined (it depends on the std implementation); use it only where iteration
/// order cannot affect results, otherwise use `DetHashMap`.
#[derive(Clone, Copy, Debug, Default)]
pub struct FixedState;
impl core::hash::BuildHasher for FixedState { type Hasher = DetHasher; /* … */ }

/// `core::hash::Hasher` over XXH64 seed 0; integers written little-endian, usize/isize as 64 bits.
#[derive(Clone, Debug)]
pub struct DetHasher { /* … */ }
impl core::hash::Hasher for DetHasher { /* … */ }
```

### 5.4 `liminal_core::collections`

```rust
/// Hash map whose iteration order is the insertion order of its keys.
#[derive(Clone)]
pub struct DetHashMap<K, V> { /* … */ }
/// Writes `{key: value, …}` in iteration order (not derived: no internals in the output).
impl<K: Debug, V: Debug> Debug for DetHashMap<K, V> { /* … */ }

/// No bounds on `K` or `V`: what `Default` needs, and what every `K`, `V` can observe (CORE-R076).
impl<K, V> DetHashMap<K, V> {
    pub fn new() -> DetHashMap<K, V>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}

impl<K: core::hash::Hash + Eq, V> DetHashMap<K, V> {
    pub fn with_capacity(capacity: usize) -> DetHashMap<K, V>;
    /// New key: appended at the end. Existing key: value replaced in place, old value returned.
    pub fn insert(&mut self, key: K, value: V) -> Option<V>;
    pub fn get<Q>(&self, key: &Q) -> Option<&V> where K: Borrow<Q>, Q: Hash + Eq + ?Sized;
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V> where K: Borrow<Q>, Q: Hash + Eq + ?Sized;
    pub fn contains_key<Q>(&self, key: &Q) -> bool where K: Borrow<Q>, Q: Hash + Eq + ?Sized;
    /// Keeps the relative order of the remaining entries. O(len).
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V> where K: Borrow<Q>, Q: Hash + Eq + ?Sized;
    pub fn clear(&mut self);
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> + '_;
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> + '_;
    pub fn keys(&self) -> impl Iterator<Item = &K> + '_;
    pub fn values(&self) -> impl Iterator<Item = &V> + '_;
}
impl<K, V> Default for DetHashMap<K, V> { /* new() */ }

/// Hash set whose iteration order is the insertion order.
#[derive(Clone)]
pub struct DetHashSet<T> { /* … */ }
/// Writes `{value, …}` in iteration order.
impl<T: Debug> Debug for DetHashSet<T> { /* … */ }

/// No bounds on `T` (CORE-R076).
impl<T> DetHashSet<T> {
    pub fn new() -> DetHashSet<T>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}

impl<T: core::hash::Hash + Eq> DetHashSet<T> {
    pub fn with_capacity(capacity: usize) -> DetHashSet<T>;
    /// `true` if the value was not present (it is appended). Present: no change, `false`.
    pub fn insert(&mut self, value: T) -> bool;
    pub fn contains<Q>(&self, value: &Q) -> bool where T: Borrow<Q>, Q: Hash + Eq + ?Sized;
    /// Keeps the relative order of the remaining values. O(len).
    pub fn remove<Q>(&mut self, value: &Q) -> bool where T: Borrow<Q>, Q: Hash + Eq + ?Sized;
    pub fn clear(&mut self);
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_;
}
impl<T> Default for DetHashSet<T> { /* new() */ }
```

`Borrow` is `core::borrow::Borrow`; `Hash` is `core::hash::Hash`; `Debug` is `core::fmt::Debug`.
The `Debug` impls are hand-written (`spec_version` 4): they have the bounds a derive would have
and write only the entries, as `std` collections do.

`new`, `len` and `is_empty` of both collections have no trait bounds (`spec_version` 3), so they
and `Default::default()` exist for every `K`, `V` and `T`. Every other method requires
`K: Hash + Eq` / `T: Hash + Eq`.

### 5.5 `liminal_core::rng`

```rust
/// Names one consumer of randomness.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct StreamKey(u64);

impl StreamKey {
    pub const fn new(key: u64) -> StreamKey;
    /// XXH64 (seed 0) of the UTF-8 bytes of `name`.
    pub fn from_name(name: &str) -> StreamKey;
    pub fn value(self) -> u64;
}

/// xoshiro256** 1.0. No `Default`: every `Rng` comes from a seed and a stream.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rng { /* s: [u64; 4] */ }

impl Rng {
    pub fn new(world_seed: u64, stream: StreamKey) -> Rng;
    pub fn next_u64(&mut self) -> u64;
    /// Upper 32 bits of `next_u64`.
    pub fn next_u32(&mut self) -> u32;
    /// Uniform in 0..bound.
    pub fn below_u32(&mut self, bound: core::num::NonZeroU32) -> u32;
    pub fn below_u64(&mut self, bound: core::num::NonZeroU64) -> u64;
    /// Uniform in [0, 1), multiples of 2^-24.
    pub fn next_f32(&mut self) -> f32;
    /// Uniform in [0, 1), multiples of 2^-53.
    pub fn next_f64(&mut self) -> f64;
    /// The four state words, for save games and replays.
    pub fn state(&self) -> [u64; 4];
    /// Errors: LIM-CORE-0030 if all four words are zero.
    pub fn from_state(state: [u64; 4]) -> Result<Rng, CoreError>;
}
```

### 5.6 `liminal_core::time`

```rust
/// Ticks per second, 1..=1000.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct TickRate { /* NonZeroU32 */ }

impl TickRate {
    /// 60 ticks per second (ADR-0009 item 1).
    pub const DEFAULT: TickRate;
    /// Errors: LIM-CORE-0020 if `ticks_per_second` is 0 or above 1000.
    pub fn new(ticks_per_second: u32) -> Result<TickRate, CoreError>;
    pub fn ticks_per_second(self) -> u32;
    /// `1.0_f32 / ticks_per_second as f32`.
    pub fn dt_secs_f32(self) -> f32;
    /// `1.0_f64 / ticks_per_second as f64`.
    pub fn dt_secs_f64(self) -> f64;
    /// `duration.count() as f64 / ticks_per_second as f64`.
    pub fn ticks_to_secs_f64(self, duration: TickDuration) -> f64;
    /// Rounded to the nearest tick, ties up, in integer arithmetic.
    pub fn ticks_from_millis(self, millis: u64) -> TickDuration;
}
impl Default for TickRate { /* DEFAULT */ }

/// Index of a simulation step.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct Tick(u64);

impl Tick {
    pub const ZERO: Tick;
    pub const fn new(index: u64) -> Tick;
    pub fn index(self) -> u64;
    /// Panics on overflow (unreachable in practice: 2^64 ticks at 1000 Hz is 5.8 × 10^8 years).
    pub fn next(self) -> Tick;
    pub fn checked_add(self, duration: TickDuration) -> Option<Tick>;
    /// `Some(self - earlier)` if `self >= earlier`, else `None`.
    pub fn since(self, earlier: Tick) -> Option<TickDuration>;
}

/// A number of ticks.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct TickDuration(u64);

impl TickDuration {
    pub const ZERO: TickDuration;
    pub const fn new(count: u64) -> TickDuration;
    pub fn count(self) -> u64;
    pub fn checked_add(self, other: TickDuration) -> Option<TickDuration>;
}
```

## 6. Requirements

### Error model (R001–R019)

| ID | Level | Requirement |
|---|---|---|
| CORE-R001 | MUST | `ErrorCode::parse` returns `Ok` exactly for strings that consist of `LIM-`, 1 to 8 ASCII uppercase letters `A`–`Z`, `-`, and 4 ASCII digits whose value is 1 to 9999 (no surrounding whitespace, no other characters). Every other input returns `Err(CoreError::InvalidErrorCode)` (LIM-CORE-0001) whose `text` is the input. |
| CORE-R002 | MUST | `ErrorCode`'s `Display` writes `LIM-`, the area, `-`, and the number zero-padded to 4 digits. For every valid code `c`, `ErrorCode::parse(&c.to_string()) == Ok(c)`, and `area()` and `number()` return the parts. |
| CORE-R003 | MUST | `ErrorCode::new` is a `const fn`. With a valid area (1 to 8 ASCII uppercase letters) and number (1 to 9999) it returns the code equal to `parse` of its text form. With an invalid area or number it panics; used in a `const` item this is a compile error. |
| CORE-R004 | MUST | `ErrorCode` is `Copy` and `size_of::<ErrorCode>()` is at most 16 bytes. Its `Ord` orders by area (byte-wise lexicographic) and then by number. |
| CORE-R005 | MUST | `EngineError` has `std::error::Error` as supertrait and the methods `code`, `message`, `help`, `context` of §5.1, and is dyn-compatible: `&dyn EngineError` can be created from a `&CoreError` and passed to `write_display`, `render_json` and `validate_error`. |
| CORE-R006 | MUST | `write_display` writes exactly the code's text form, `: `, the message, and, if `help()` is `Some(h)`, ` help: ` followed by `h`. Nothing else (no trailing newline). |
| CORE-R007 | MUST | `ErrorContext::with` appends a field; `iter` yields the fields in the order they were appended, including duplicates; `get(key)` returns the value of the first field with that key, or `None`; `len` and `is_empty` count fields. The `From` conversions to `ContextValue` map types as listed in §5.1, with values unchanged (`f32` widened exactly to `f64`). |
| CORE-R008 | MUST | `render_json` returns one JSON object (RFC 8259) without whitespace outside strings and without a trailing newline, with exactly the members `code` (the code's text form), `message`, `help` (the help string, or `null` when `help()` is `None`) and `context` (an object with one member per context field, in `ErrorContext` order; `{}` when empty), in that order. |
| CORE-R009 | MUST | JSON strings written by `render_json` escape `"` as `\"`, `\` as `\\`, U+0008 as `\b`, U+000C as `\f`, U+000A as `\n`, U+000D as `\r`, U+0009 as `\t`, every other code point U+0000 to U+001F as `\u00` followed by two lowercase hex digits, and write every other code point as its UTF-8 bytes unchanged (including `/`, U+007F and non-ASCII). Decoding the output with a JSON string decoder yields the original string. |
| CORE-R010 | MUST | Context values render as: `Bool` as `true` or `false`; `Int` and `UInt` as decimal integers (a leading `-` only for negative values, no leading zeros, no `+`); finite `Float` as the output of Rust's `Display` for `f64` (the shortest decimal that parses back to the same bits, no exponent; `-0.0` renders as `-0`); non-finite `Float` (NaN, infinities) as `null`; `Text` as a JSON string (CORE-R009). |
| CORE-R011 | MUST | `render_json` and `write_display` are deterministic: for equal error values they produce identical bytes in every call, in every process and on every supported platform. |
| CORE-R012 | MUST | `validate_error(e, area)` checks, in this order, and returns the first violation as `Err`: the code's area equals `area` (else LIM-CORE-0007); `message()` is not empty (else LIM-CORE-0004); `help()` is not `Some("")` (else LIM-CORE-0005); every context key matches `^[a-z][a-z0-9_]*$` (else LIM-CORE-0002 naming the first bad key); no context key occurs twice (else LIM-CORE-0003 naming the first repeated key); `e.to_string()` equals the `write_display` output (else LIM-CORE-0006 with both strings). It returns `Ok(())` if all checks pass. |
| CORE-R013 | MUST | `CoreError` implements `EngineError`. Each variant has the code, message, help and context fields given in §8, and every variant value passes `validate_error(&e, "CORE")`. |

### Handles (R030–R049)

| ID | Level | Requirement |
|---|---|---|
| CORE-R030 | MUST | `size_of::<RawHandle>()`, `size_of::<Option<RawHandle>>()`, `size_of::<Handle<T>>()` and `size_of::<Option<Handle<T>>>()` are 8 bytes, for any `T` (tested with at least a zero-sized type and a 1 KiB type). |
| CORE-R031 | MUST | `RawHandle::to_bits` returns `generation × 2^32 + index`. `RawHandle::from_bits(b)` returns `None` if `b < 2^32`, else the handle with `index = b mod 2^32` and `generation = b / 2^32`. `from_bits(h.to_bits()) == Some(h)` for every handle `h`. `Display` writes `{index}v{generation}` in decimal. |
| CORE-R032 | MUST | `Handle<T>` is `Copy`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash` and `Debug` for every `T`, including types that implement none of these traits. Equality and order are those of the raw handles. `Handle::from_raw(r).raw() == r`. |
| CORE-R033 | MUST | `HandleAllocator::allocate` reuses the most recently freed non-retired slot if one exists (LIFO), otherwise takes the lowest never-used index. A never-used slot is issued with generation 1; a reused slot is issued with its current generation. A new allocator issues indices 0, 1, 2, … in order. |
| CORE-R034 | MUST | `free(h)` of a live handle makes `is_live(h)` false and increments the slot's generation by 1. No handle value is issued twice by one allocator: after any sequence of operations, every handle returned by `allocate` differs from every handle returned earlier. |
| CORE-R035 | MUST | Freeing a live handle whose generation is `u32::MAX` retires its slot: the slot is never issued again, and `allocate` takes another free slot or a never-used index instead. |
| CORE-R036 | MUST | `free(h)` of a handle that is not live (freed already, slot retired, index never used, or generation different from the slot's) returns `Err(CoreError::StaleHandle)` (LIM-CORE-0010) with the handle's index and generation, and leaves the allocator unchanged. |
| CORE-R037 | MUST | An allocator from `with_slot_limit(n)` never uses an index `>= n`. When no free slot exists and `n` indices have been used (or `n` is 0), `allocate` returns `Err(CoreError::HandleSlotsExhausted)` (LIM-CORE-0011) with `slot_limit = n`, and leaves the allocator unchanged. `new()` has the slot limit `u32::MAX`. |
| CORE-R038 | MUST | `is_live(h)` is true exactly for handles returned by `allocate` and not yet freed. `live_count` is the number of such handles. `iter_live` yields each of them once, in ascending index order. |
| CORE-R039 | MUST | `HandleMap<T>` issues handles by the rules of CORE-R033 to CORE-R035 and CORE-R037 (the same sequence of `insert` and live `remove` calls gives the same handle sequence as `allocate` and `free` on a `HandleAllocator` with the same slot limit). `get`, `get_mut` and `contains` return the stored value (or `true`) exactly for live handles and `None` (or `false`) otherwise. `remove` of a live handle returns its value and makes it stale; `remove` of a non-live handle returns `None` and leaves the map unchanged. `insert` on exhaustion returns LIM-CORE-0011 and leaves the map unchanged. `len` counts live entries; `iter` and `iter_mut` yield every live entry once, in ascending index order. |
| CORE-R040 | SHOULD | `allocate`, `free`, `is_live`, `HandleMap::insert`, `get`, `get_mut`, `contains` and `remove` run in amortized constant time: they perform no scan over slots or live handles. Checked by review of the implementation. |
| CORE-R041 | MUST | `HandleAllocator::default()` and `HandleMap::<T>::default()` return a value in the same state as `new()`: slot limit `u32::MAX` (not 0), no slot used, `live_count()` / `len()` 0. The first `allocate` / `insert` on it returns `Ok` with the handle `0v1`, and its `Debug` output equals that of `new()` (for `HandleMap<T>`, with `T: Debug`). `HandleMap<T>: Default` holds for every `T`, including types that do not implement `Default`. |

### State hasher and fixed-seed hasher (R050–R069)

| ID | Level | Requirement |
|---|---|---|
| CORE-R050 | MUST | `StateHasher::finish` returns XXH64 with seed 0 (xxHash specification, `doc/xxhash_spec.md` in the xxHash repository, algorithm XXH64) of the concatenation of all bytes written since `StateHasher::new()`. A new hasher with nothing written returns XXH64 of the empty input. |
| CORE-R051 | MUST | Splitting a byte sequence into any number of `write_bytes` calls of any lengths (including empty calls) gives the same `finish` value as writing it in one call. |
| CORE-R052 | MUST | `finish` does not change the hasher: calling it twice returns the same value, and writing after `finish` continues the same byte sequence. |
| CORE-R053 | MUST | The typed write methods append exactly these bytes: `u8` to `u128` and `i8` to `i128` their little-endian two's-complement bytes of their natural width; `usize` as `u64` and `isize` as `i64` (8 bytes); `bool` one byte `0x00` or `0x01`; `char` its scalar value as `u32`; `f32` and `f64` the little-endian bytes of `to_bits()`, without canonicalizing NaN payloads or `-0.0`; `str` its length in bytes as `u64` followed by its UTF-8 bytes. |
| CORE-R054 | MUST | `StateHash` encodings: integers, `bool`, `char`, `f32`, `f64` and `str` as in CORE-R053; `String` as `str`; `[T]`, `Vec<T>` and `[T; N]` their length as `u64` followed by each element in order; `Option<T>` `0x00` for `None` and `0x01` followed by the value for `Some`; `&T` as `T`; tuples of 1 to 4 elements each element in order with no prefix; `RawHandle` and `Handle<T>` their `to_bits()` as `u64`; `DetHashMap` its length as `u64` followed by each key and value in iteration order; `DetHashSet` its length as `u64` followed by each value in iteration order; `Rng` its four state words as `u64` in order; `Tick` and `TickDuration` their count as `u64`; `TickRate` its ticks per second as `u32`; `StreamKey` its value as `u64`. |
| CORE-R055 | MUST | `state_hash_of(v)` equals `finish()` of a new `StateHasher` after `v.state_hash(&mut hasher)`. |
| CORE-R056 | MUST | `StateHasher` results are identical on every supported platform: a known-answer test with recorded literals (CORE-R050 vectors and one value per `StateHash` impl of CORE-R054) passes on the macOS arm64, Windows x86_64 and Linux x86_64 CI jobs. |
| CORE-R057 | MUST | `StateHasher::default()` returns a hasher in the same state as `StateHasher::new()`: nothing written, `finish` returns XXH64 (seed 0) of the empty input, and after the same writes both give the same `finish` value. |
| CORE-R060 | MUST | Every `DetHasher` built by `FixedState::build_hasher` starts in the same state (XXH64 seed 0, nothing written), in every process and on every supported platform: with nothing written its `finish` returns XXH64 (seed 0) of the empty input, and the same sequence of `Hasher` calls made on it gives the same `finish` (CORE-R061). Therefore, within one process, two `FixedState` instances give the same `hash_one(&v)` for any value `v: Hash`. The value of `hash_one(&v)` is not part of this contract across Rust releases: which `Hasher` calls a `Hash` impl makes is decided by that impl, and std's impls are not guaranteed stable (§5.3). Tests compare `hash_one` results only with each other in the same process and record hash literals only for call sequences the test makes on a `DetHasher` directly. |
| CORE-R061 | MUST | `DetHasher` overrides the integer write methods of `core::hash::Hasher` to append little-endian bytes, with `write_usize` and `write_isize` appending 8 bytes, so the same sequence of `Hasher` calls gives the same `finish` on every supported platform; `finish` is XXH64 seed 0 of the appended bytes. |

### Collections (R070–R089)

| ID | Level | Requirement |
|---|---|---|
| CORE-R070 | MUST | `DetHashMap` iteration (`iter`, `iter_mut`, `keys`, `values`) yields entries in the order their keys were first inserted. `insert` of a key that is present replaces the value in place, keeps the entry's position, and returns the old value; `insert` of a new key appends the entry and returns `None`. |
| CORE-R071 | MUST | `DetHashMap::remove` of a present key returns its value and keeps the relative order of the remaining entries; a later `insert` of that key appends it at the end. `remove` of an absent key returns `None` and changes nothing. `clear` removes all entries. |
| CORE-R072 | MUST | For every sequence of `insert`, `remove`, `get`, `get_mut`, `contains_key`, `len`, `is_empty`, `clear` and iteration, `DetHashMap` gives the same results as a reference model that stores entries in a `Vec<(K, V)>` and finds keys by linear search (property test, differential). |
| CORE-R073 | MUST | `DetHashMap` and `DetHashSet` iteration order depends only on the sequence of operations, not on capacity, `with_capacity`, hash values or the process. |
| CORE-R074 | MUST | `DetHashSet` follows CORE-R070 to CORE-R073 for values: iteration in first-insertion order; `insert` of a present value returns `false` and changes nothing; `remove` keeps the order of the rest; same results as a `Vec<T>` linear-search reference model. |
| CORE-R075 | SHOULD | `DetHashMap::insert`, `get`, `get_mut`, `contains_key` and `DetHashSet::insert`, `contains` take expected constant time (hash lookup, no linear scan). `remove` takes time linear in `len`. Checked by review. |
| CORE-R076 | MUST | `DetHashMap::<K, V>::default()` and `DetHashSet::<T>::default()` return an empty collection in the same state as `new()`. Two cases, by what §5.4 lets a caller observe. (a) For every `K`, `V` and `T`, including types that implement none of `Default`, `Hash`, `Eq` and `Debug`: `Default::default()` and `new()` compile, and on the result of each `len()` is 0 and `is_empty()` is `true`. (b) For `K: Hash + Eq` / `T: Hash + Eq` in addition: iteration over the result of `default()` yields nothing, and after the same sequence of operations a collection from `default()` and one from `new()` give the same results, the same `len()` and iterate in the same order with the same contents. |

### RNG streams (R090–R109)

| ID | Level | Requirement |
|---|---|---|
| CORE-R090 | MUST | `Rng::next_u64` is xoshiro256\*\* 1.0 (Blackman and Vigna, reference `xoshiro256starstar.c`): `result = rotl(s1 × 5, 7) × 9` (wrapping), then `t = s1 << 17; s2 ^= s0; s3 ^= s1; s1 ^= s2; s0 ^= s3; s2 ^= t; s3 = rotl(s3, 45)`. For the same state it returns the same values as the reference C implementation. |
| CORE-R091 | MUST | `Rng::new(world_seed, stream)` sets `x` to XXH64 (seed 0) of the 16 bytes `world_seed` little-endian followed by `stream.value()` little-endian, then sets `s0`, `s1`, `s2`, `s3` to four consecutive outputs of SplitMix64 (reference `splitmix64.c`: `x += 0x9E3779B97F4A7C15; z = x; z = (z ^ (z >> 30)) × 0xBF58476D1CE4E5B9; z = (z ^ (z >> 27)) × 0x94D049BB133111EB; z ^ (z >> 31)`, wrapping) started from `x`. |
| CORE-R092 | MUST | `StreamKey::from_name(name)` equals `StreamKey::new(h)` where `h` is XXH64 (seed 0) of the UTF-8 bytes of `name`. `StreamKey::new(k).value() == k`. |
| CORE-R093 | MUST | For world seeds 0 to 31 and stream keys `StreamKey::new(0)` to `StreamKey::new(31)` (1,024 pairs), the first `next_u64` outputs of the 1,024 streams are pairwise distinct. |
| CORE-R094 | MUST | `next_u32` returns the upper 32 bits of one `next_u64` call. |
| CORE-R095 | MUST | `below_u32(bound)` with `s = bound.get()`: draw `x = next_u32()`, `m = x × s` as `u64`, `l = m mod 2^32`; if `l < s`, compute `t = (2^32 − s) mod s` and, while `l < t`, draw again and recompute `m` and `l`; return `m / 2^32`. `below_u64` is the same with `next_u64`, `u128` products and `2^64`. The result is in `0..bound` and every value in `0..bound` has probability exactly `1 / bound` (Lemire 2019, "Fast Random Integer Generation in an Interval"). |
| CORE-R096 | MUST | `next_f64` returns `(next_u64() >> 11) as f64 × 2^-53` and `next_f32` returns `(next_u64() >> 40) as f32 × 2^-24`; both are in `[0, 1)` and computed exactly (no rounding). |
| CORE-R097 | MUST | `Rng::from_state(r.state())` returns an `Rng` that produces the same sequence as `r` from that point on. `from_state([0; 4])` returns `Err(CoreError::ZeroRngState)` (LIM-CORE-0030). A cloned `Rng` produces the same sequence as the original, and advancing one does not advance the other. |
| CORE-R098 | MUST | `Rng` outputs (all methods) are identical on every supported platform: a known-answer test with recorded literals for fixed seeds and stream keys passes on the macOS arm64, Windows x86_64 and Linux x86_64 CI jobs. |

### Time (R110–R129)

| ID | Level | Requirement |
|---|---|---|
| CORE-R110 | MUST | `TickRate::new(n)` returns `Ok` for `n` in 1..=1000 and `Err(CoreError::InvalidTickRate)` (LIM-CORE-0020) with `ticks_per_second = n` otherwise. `TickRate::DEFAULT` and `TickRate::default()` are 60 ticks per second. |
| CORE-R111 | MUST | `dt_secs_f32` returns `1.0_f32 / n as f32` and `dt_secs_f64` returns `1.0_f64 / n as f64`, each one IEEE 754 division (correctly rounded), for tick rate `n`. |
| CORE-R112 | MUST | `ticks_to_secs_f64(d)` returns `d.count() as f64 / n as f64` (Rust `as` conversion, round-to-nearest-even for counts above 2^53, then one IEEE 754 division). |
| CORE-R113 | MUST | `ticks_from_millis(ms)` returns `(ms × n + 500) / 1000` computed in `u128` integer arithmetic (rounding to nearest, ties up). It does not overflow or panic for any `u64` input. |
| CORE-R114 | MUST | `Tick::next` returns the tick with index + 1 and panics if the index is `u64::MAX`. `Tick::checked_add(d)` returns `None` on overflow. `t.since(e)` returns `Some(TickDuration::new(t.index() − e.index()))` if `t >= e`, else `None`. `Tick` and `TickDuration` order and compare by their counts. `TickDuration::checked_add` returns `None` on overflow. `Tick::ZERO` and `TickDuration::ZERO` have count 0, and so have `Tick::default()` and `TickDuration::default()`. |

### Determinism (R130–R139)

| ID | Level | Requirement |
|---|---|---|
| CORE-R130 | MUST | Workload `rng_streams` in `crates/liminal_core/tests/determinism.rs` (§11) gives the same state hash in two runs in one process and equals its recorded hash literal on every supported platform. |
| CORE-R131 | MUST | Workload `handle_churn` (§11) gives the same state hash in two runs in one process and equals its recorded hash literal on every supported platform. |
| CORE-R132 | MUST | Workload `det_map_churn` (§11) gives the same state hash in two runs in one process and equals its recorded hash literal on every supported platform. |
| CORE-R133 | MUST | Workload `time_conversions` (§11) gives the same state hash in two runs in one process and equals its recorded hash literal on every supported platform. |

### Assignment to implementation tasks

Every requirement is assigned to exactly one task; the IDs are in the tasks' `requirements`
frontmatter.

| Task | Requirements |
|---|---|
| M1-T013 (error model, handles) | CORE-R001 to CORE-R013, CORE-R030 to CORE-R040 |
| M1-T059 (`Default` tests for the handle types) | CORE-R041 |
| M1-T014 (hashing, RNG, collections, time, determinism) | CORE-R050 to CORE-R057, CORE-R060, CORE-R061, CORE-R070 to CORE-R076, CORE-R090 to CORE-R098, CORE-R110 to CORE-R114, CORE-R130 to CORE-R133 |

M1-T013 implements all of `CoreError` (CORE-R013), including the variants whose producers come in
M1-T014. M1-T014 implements the `StateHash` impls for handles (CORE-R054). The code for CORE-R041 is
M1-T013's (written against `spec_version` 1, before the requirement existed); M1-T059 adds its
`Verifies:` tests and owns the requirement.

## 7. Invariants

- INV-1: A live handle's generation equals its slot's current generation; a free slot is either in
  the free list or retired, never both; a retired slot has generation `u32::MAX` at the time it was
  freed and is never in the free list. (`debug_assert!` in `allocate` and `free`; property tests
  of CORE-R034, CORE-R035.)
- INV-2: `HandleAllocator::live_count()` equals the number of occupied slots, and `HandleMap::len()`
  equals the number of stored values. (`debug_assert!`; model-based property test.)
- INV-3: In `DetHashMap` and `DetHashSet`, the lookup structure and the ordered entry list contain
  the same keys, each once. (`debug_assert!` after `insert` and `remove`; differential property
  test CORE-R072.)
- INV-4: An `Rng` state is never all zeros (xoshiro256\*\* would output 0 forever). Guaranteed by
  construction: SplitMix64 outputs four consecutive distinct values, and `from_state` rejects zero.
- INV-5: `TickRate` is in 1..=1000 (type invariant; only `new` and `DEFAULT` create it).
- INV-6: `ErrorCode` always holds a valid area and number (type invariant; only `new` and `parse`
  create it).

## 8. Error codes

`liminal_core` has recoverable errors: invalid input to `ErrorCode::parse`, `TickRate::new` and
`Rng::from_state`, stale handles, exhausted handle slots, and error-contract violations found by
`validate_error`. Messages and help below are normative; `{x}` is the value of context field `x`, and the
backticks shown are part of the text.

| Code | Variant | Meaning (message) | Help | Context |
|---|---|---|---|---|
| LIM-CORE-0001 | `InvalidErrorCode` | error code text `{text}` does not have the form LIM-AREA-NNNN. | write `LIM-`, the crate's requirement prefix (1 to 8 uppercase ASCII letters), `-`, and 4 digits from 0001 to 9999, for example `LIM-ECS-0007`. | `text` |
| LIM-CORE-0002 | `InvalidContextKey` | error `{code}` has context key `{key}`, which is not snake_case. | use keys that match `^[a-z][a-z0-9_]*$`. | `code`, `key` |
| LIM-CORE-0003 | `DuplicateContextKey` | error `{code}` has context key `{key}` more than once. | give each context field a distinct key. | `code`, `key` |
| LIM-CORE-0004 | `EmptyMessage` | error `{code}` has an empty message. | return a message that says what happened. | `code` |
| LIM-CORE-0005 | `EmptyHelp` | error `{code}` has an empty help text. | return `None` from `help()` when no fix is known, otherwise a non-empty hint. | `code` |
| LIM-CORE-0006 | `DisplayMismatch` | error `{code}` displays as `{actual}` instead of `{expected}`. | implement `Display` by calling `liminal_core::error::write_display`. | `code`, `expected`, `actual` |
| LIM-CORE-0007 | `AreaMismatch` | error `{code}` does not use the area `{expected_area}`. | use the `req_prefix` of the crate's spec as the code's area. | `code`, `expected_area` |
| LIM-CORE-0010 | `StaleHandle` | handle `{index}v{generation}` is not live. | the handle was freed already or was not issued by this allocator; check `is_live` before freeing. | `index`, `generation` |
| LIM-CORE-0011 | `HandleSlotsExhausted` | all `{slot_limit}` handle slots are in use or retired. | free handles that are no longer needed, or create the allocator with a higher slot limit. | `slot_limit` |
| LIM-CORE-0020 | `InvalidTickRate` | tick rate `{ticks_per_second}` is outside 1 to 1000 ticks per second. | use a tick rate from 1 to 1000; the default is 60 (`TickRate::DEFAULT`). | `ticks_per_second` |
| LIM-CORE-0030 | `ZeroRngState` | the RNG state is all zeros, which the generator cannot use. | restore a state saved with `Rng::state`, or create the RNG with `Rng::new`. | none |

Each message ends with `.`; context values of type `ErrorCode` are rendered as their text form
(`Text`), the others with their natural `ContextValue` variant (`UInt` for `u32`). Codes 0008,
0009, 0012 to 0019, 0021 to 0029 and 0031 onward are unassigned; new codes take the next free
number in their group.

## 9. Performance budgets

None yet: set at `stable` from baselines. `liminal_core` has no optimized paths in `spec_version`
4. The scenarios below are measured on the reference machine (PERFORMANCE.md) once the harness
exists; no number is set before that measurement.

| Scenario | Budget | Machine | Source of number |
|---|---|---|---|
| `StateHasher::write_bytes` over a 1 MiB buffer, then `finish` (throughput, GiB/s) | none yet | Reference | Baseline `ci/perf/<machine-id>/liminal_core.json` from M1-T016 / M1-T017 |
| 1,000,000 × `Rng::next_u64` on one stream | none yet | Reference | Same baseline (M1-T016 / M1-T017) |
| 1,000,000 × `Rng::below_u32(1000)` | none yet | Reference | Same baseline (M1-T016 / M1-T017) |

The table lists only scenarios that a task measures. `HandleAllocator` and `DetHashMap` have no
scenario row: no existing task adds a benchmark for them (M1-T016's bench target covers the hasher
and the RNG only). The task that prepares the `stable` promotion of `liminal_core` (not written
yet) adds their scenarios and rows by a `spec_version` bump (open question 6).

## 10. Test plan

No path in `spec_version` 4 is an optimized variant of another; each algorithm has one
implementation, checked against an external reference. If a later version adds an optimized path
(SIMD XXH64, a faster map), the current implementation stays as its reference and a differential
test is added (AGENTS.md rule 4).

| Area | Test kinds | Oracle |
|---|---|---|
| `ErrorCode` (R001–R004) | unit, property (round trip over generated valid codes; generated invalid strings rejected), doc test | Test-local predicate written from the CORE-R001 grammar |
| `write_display`, `render_json` (R005–R011) | unit, snapshot (`crates/liminal_core/tests/snapshots/core_errors.txt`: Display and JSON of one value of every `CoreError` variant), property (string escaping round trip over generated strings including all of U+0000–U+001F) | Approved snapshot; test-local JSON string decoder in `tests/common/` |
| `validate_error`, `CoreError` (R012, R013) | unit (one test error type per violation), integration | Hand-written expected codes |
| Handles (R030–R041) | unit, property (random operation sequences), integration, doc test | Model in `tests/common/`: list of every issued handle plus the set of live handles, checked by linear search. Generation wrap (R035) is tested in a unit test inside the module that sets a slot's generation to `u32::MAX - 1` directly, since 2^32 frees are too slow for a test |
| `StateHasher` (R050–R057) | unit known-answer, property (R051 splits) | XXH64 reference vectors from the xxHash repository: the seed-0 rows of `XSUM_XXH64_testdata` (`tests/sanity_test_vectors.h`, tag v0.8.3) for 121 lengths from 0 to 4160, copied into `tests/common/xxh64_vectors.rs`; for R053/R054, the expected byte sequence written by hand and hashed with `write_bytes`; for the R056 literals, XXH64 of those byte sequences computed by the independent Python implementation `tests/reference/kat.py` |
| `FixedState`, `DetHasher` (R060, R061) | unit, integration | XXH64 of hand-built little-endian byte sequences, as recorded literals, for fixed sequences of `Hasher` calls made directly on a `DetHasher` from `FixedState::build_hasher` (a literal recorded in an earlier process covers "in every process"; the first sequence is empty). `hash_one(&v)` of two `FixedState` instances is compared in-process only, never with a recorded literal, because the calls a std `Hash` impl makes may change with the Rust toolchain |
| `DetHashMap`, `DetHashSet` (R070–R076) | property (differential), unit | `Vec<(K, V)>` / `Vec<T>` linear-search reference model. CORE-R076 case (a): a unit test with a key, value and element type that implements no trait (it fails to compile if `Default`, `new`, `len` or `is_empty` gains a bound); case (b): the same operation sequence applied to `default()` and `new()` |
| `Rng`, `StreamKey` (R090–R098) | unit known-answer, property (range of `below_*`, `[0, 1)` of floats, seeding against a test-local `SplitMix64`) | `next_u64` and `SplitMix64`: the published outputs of the reference C implementations `xoshiro256starstar.c` and `splitmix64.c` (prng.di.unimi.it) as recorded in the `reference` tests of `rand_xoshiro` (github.com/rust-random/rngs, commit 84683fca). Seeding (R091), `below_*` (R095), floats (R096) and the R098 literals: values computed by `tests/reference/kat.py`, an independent Python implementation of XXH64, `SplitMix64`, xoshiro256\*\* and Lemire's method that checks itself against the published vectors; the command is recorded at each literal. For power-of-two bounds `below_u32(2^k)` must equal `next_u32() >> (32 − k)` (no rejection occurs) |
| Time (R110–R114) | unit, property (`ticks_from_millis` monotone; exact for whole seconds) | Hand-computed values; float bit patterns and the `u64::MAX` cases computed with Python (`kat.py time`) |
| Determinism (R130–R133) | determinism (`tests/determinism.rs`, ADR-0024) | Run twice in-process; recorded hash literals (the `rng_streams` literal is also reproduced by `kat.py rng-streams`) |

`tests/reference/kat.py` is not run by any test or by CI; it documents how the literals were
produced and lets a reviewer reproduce them (`python3`, standard library only). The reference C
files were not compiled for `spec_version` 4: the implementing session was not permitted to
run downloaded code, so their outputs are taken from the published `rand_xoshiro` tests.

Every requirement's verifying test carries `/// Verifies: CORE-Rnnn`. Primary entry points with
`# Examples`: `ErrorCode`, `EngineError` (a complete small error enum), `render_json`,
`HandleAllocator`, `HandleMap`, `StateHasher`, `state_hash_of`, `DetHashMap`, `Rng`, `TickRate`.

## 11. Determinism

`liminal_core` is `deterministic = true` (ADR-0009):

- It reads no clock, no environment and no OS randomness. It has no API that does.
- Its only hasher is XXH64 with seed 0; no type uses a randomly seeded hasher. `DetHashMap` and
  `DetHashSet` iteration order is defined by the operation sequence (CORE-R073).
- It uses no floating-point operation other than integer-to-float conversion, multiplication by a
  power of two and one division per call (CORE-R096, CORE-R111, CORE-R112). These are exactly
  specified by IEEE 754 and Rust; all supported targets (aarch64, x86_64 with SSE2) implement them
  without extended precision. No transcendental functions.
- It is single-threaded: no type has interior mutability or shared state.
- The determinism lint configuration of CODE_STYLE.md applies (M1-T011, configured by M1-T013).

**Workloads of `crates/liminal_core/tests/determinism.rs`** (ADR-0024). Each workload runs at least
twice in the same process, feeds its results into a `StateHasher`, and compares the final hashes
with each other and with a hash literal recorded in the test. Exact parameters (seeds, counts,
operation mix) are fixed in the test; changing them changes the literal and is an oracle change
(AGENTS.md rule 5).

| Workload | What it does | Cross-platform bit-identity claimed |
|---|---|---|
| `rng_streams` (CORE-R130) | For 3 world seeds × 3 stream keys (including one from `StreamKey::from_name`), 1,000 rounds of every `Rng` output method, each result hashed (floats by bits), then the generator's state | Yes (integer arithmetic; exact float conversions) |
| `handle_churn` (CORE-R131) | 10,000 seeded insert/remove operations on a `HandleMap<u64>` with a slot limit of 48 (exhaustion and stale removes included; the workload asserts that each occurs more than 100 times); every returned handle, value and error code hashed, then the final map | Yes (integer arithmetic) |
| `det_map_churn` (CORE-R132) | 10,000 seeded insert/remove/get operations on `DetHashMap<u64, u64>` over a key space of 300 keys; every result hashed, and the map's `StateHash` every 100 operations | Yes (iteration order does not depend on hash values, CORE-R073) |
| `time_conversions` (CORE-R133) | `dt_secs_f32`, `dt_secs_f64` for every tick rate 1..=1000; `ticks_to_secs_f64` and `ticks_from_millis` for 10 fixed counts including 0, 2^53 + 1 and `u64::MAX`; results hashed by bits | Yes (one correctly rounded IEEE 754 division per value) |

Cross-platform bit-identity is claimed for all four workloads and **verified** for the recorded
literals on the three CI operating systems: GitHub Actions run 42882724790 (commit `9760608`,
2026-11-10) passed `cargo xtask ci`, which runs `tests/determinism.rs` and the known-answer tests
of CORE-R056 and CORE-R098, on `macos-latest` (arm64), `windows-latest` (x86_64) and
`ubuntu-latest` (x86_64). Every later CI run repeats the check. Not verified: any other target
(x86_64 macOS, aarch64 Linux or Windows).

## 12. Open questions

1. **Cause chains in the error JSON.** `render_json` omits `std::error::Error::source()`. Errors
   that wrap others (I/O, asset loading) may need a `cause` member. Resolve in the first spec that
   wraps errors (`liminal_assets`, M4, or `liminal_probe`) by a `spec_version` bump here.
2. **Error JSON once `liminal_serial` exists.** `render_json` is hand-written because no serializer
   exists in M1. When `liminal_serial` is specified (M2, OQ-002), its spec decides whether it
   produces the error JSON; the format of CORE-R008 to CORE-R010 stays as it is either way.
3. **Checking codes against spec tables.** ADR-0017 "Verification" says `check-docs` (from M1)
   checks that every code used in a crate is listed in its spec and vice versa. No task implements
   that check yet. Until one does, `validate_error` tests and review are the only check.
   Resolve: a task in M1 (to be written; reported in M1-T012's handoff).
4. **Area names in other task files.** Resolved (M1-T055, `spec_version` 2). M1-T027 and M1-T047
   named the error areas `TEL` and `TK`, while `docs/specs/README.md` reserves the prefixes `TELE`
   and `TKIT`, and CORE-R012 requires the area to equal the spec's `req_prefix`. The
   `liminal_telemetry` spec written by M1-T027 uses `TELE` (scope note in that task); the M1-T047
   task file now says `TKIT`.
5. **Entities and handles.** Whether `liminal_ecs` builds `Entity` on `RawHandle` and
   `HandleAllocator` (including the LIFO reuse and retirement rules) or defines its own allocator
   is decided by the ECS spec (M1-T037). If ECS needs a different reuse order, this spec is not
   changed silently: either ECS owns its allocator or this spec gets a new requirement.
6. **Order-preserving `remove` cost.** `DetHashMap::remove` is O(len) to keep insertion order. If a
   benchmark shows this matters, a `swap_remove` with its own defined order may be added. Resolve
   at the `stable` promotion. No benchmark of `DetHashMap` or `HandleAllocator` is scheduled; the
   task that prepares that promotion adds their scenarios to §9 first.
7. **Tick rate ceiling.** 1000 Hz is an assumption about the fastest fixed step any system needs
   (physics substeps are expected to run inside a tick). Revisit in the physics spec (M6, OQ-008).

## 13. Maturity

Current: `usable` (promoted from `planned` via `experimental` by the M1-T014 review,
2026-11-10: every gate below re-checked by the reviewer, see the "Review" note in
`docs/tasks/M1-T014-implement-liminal-core-hashing-rng-collections-t.md`). Next target: `stable`.
Evidence for `usable` prepared by M1-T014 (2026-11-10, measured on macOS 27.2 arm64, Rust 1.99.1).

- [x] Spec follows `docs/specs/_TEMPLATE.md` and passed a spec review (REVIEW.md §2) (evidence:
      review of M1-T012, 2026-11-09, "Review" note in
      `docs/tasks/M1-T012-write-the-spec-for-liminal-core.md`; versions 2 and 3 reviewed with
      M1-T055 and M1-T059; version 4 is reviewed with M1-T014).
- [x] Compiles, all lints pass, determinism lint configuration accepted by `check-arch`,
      registered in `ci/coverage.toml` (evidence: `cargo xtask ci` passes, 11 steps, none skipped;
      `crates/liminal_core/clippy.toml`; floor `liminal_core = 99.3`).
- [x] All MUST requirements traced (evidence: `cargo xtask trace --crate liminal_core` on a
      scratch copy of the repository with `maturity: usable` in this file: `tested: MUST 58/58,
      SHOULD 1/2`, `trace: ok`. The untested SHOULD is CORE-R075, checked by review).
- [x] Line coverage ≥ 85 % (layer 0 pure logic, TESTING.md) (evidence: `cargo xtask coverage`:
      99.37 %, 1753 of 1764 lines. The 11 uncovered lines are in `error` and `handle` (panic
      arms of test helpers and assertion messages); `hash`, `collections`, `rng` and `time` have
      no uncovered line).
- [x] Every public item documented; the primary entry points of §10 have runnable `# Examples`
      (evidence: `missing_docs = "deny"` passes; `cargo test --doc -p liminal_core` runs 24 doc
      tests, among them `StateHasher`, `state_hash_of`, `StateHash`, `FixedState`, `DetHasher`,
      `DetHashMap`, `DetHashSet`, `Rng`, `StreamKey`, `TickRate`, `Tick`, `TickDuration`).
- [x] Integration tests in `crates/liminal_core/tests/` use only the public API (evidence:
      `error_model.rs`, `handles.rs`, `hashing.rs`, `collections.rs`, `rng.rs`, `time.rs`,
      `determinism.rs`; 99 tests. Three behaviours need private access and are unit tests inside
      the crate: the generation wrap, the private `splitmix64`, the lookup table).
- [x] `tests/determinism.rs` with the four workloads of §11 and recorded literals, passing on the
      three CI operating systems (evidence: GitHub Actions run 42882724790 on `9760608`, §11).
- [x] At least one example or dependent crate uses it (evidence:
      `crates/liminal_core/examples/seeded_world.rs`, run with
      `cargo run -p liminal_core --example seeded_world`; it uses `handle`, `hash`,
      `collections`, `rng`, `time` and `error` and fails if two runs with one seed differ.
      `cargo test` compiles it; no CI step runs it. `liminal_math` (M1-T020) will be the first
      dependent crate).

## 14. History

| spec_version | Date | Change | By task |
|---|---|---|---|
| 1 | 2026-11-09 | Initial spec | M1-T012 |
| 2 | 2026-11-10 | §5.2: `HandleAllocator` `Default` is a hand-written impl equal to `new()`, no longer derived (a derived one has slot limit 0). New CORE-R041, CORE-R057, CORE-R076: `Default::default()` equals `new()` for `HandleAllocator`, `HandleMap`, `StateHasher`, `DetHashMap`, `DetHashSet`. CORE-R060 and §10: recorded literals only for `Hasher` call sequences made on a `DetHasher` directly; `hash_one` values are not fixed across Rust releases. §9: the two scenario rows without a measuring task removed. §12 item 4 resolved; item 6 updated | M1-T055 |
| 3 | 2026-11-10 | §5.4: `new`, `len` and `is_empty` of `DetHashMap` and `DetHashSet` moved into `impl` blocks without trait bounds (were `K: Hash + Eq` / `T: Hash + Eq`). CORE-R076 reworded into two cases, each naming what a test observes: every `K`, `V`, `T` (`default()` and `new()` compile, `len()` 0, `is_empty()`), and `Hash + Eq` types (iteration, same results after the same operations). §6 assignment: CORE-R041 moves from M1-T013 to M1-T059, which writes its `Verifies:` tests. §10: oracle for CORE-R076 | M1-T059 |
| 4 | 2026-11-10 | §5.4: `Debug` of `DetHashMap` and `DetHashSet` is a hand-written impl that writes the entries in iteration order (was derived). CORE-R114: `Tick::default()` and `TickDuration::default()` have count 0. §10: oracles as implemented (xxHash sanity vectors; published xoshiro256\*\* and `SplitMix64` outputs; the independent Python implementation `tests/reference/kat.py` instead of compiled reference C). §11: workload parameters and verification state. §13: evidence for the `usable` gates | M1-T014 |
