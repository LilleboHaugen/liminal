---
crate: liminal_telemetry
req_prefix: TELE
layer: 0
maturity: planned
spec_version: 2
last_reviewed: 2026-11-10
adrs: [ADR-0016, ADR-0017, ADR-0018, ADR-0023]
---

# Spec: `liminal_telemetry`

## 1. Purpose

`liminal_telemetry` is how engine and game code report what it is doing: timed spans, counters,
gauges and structured log events, collected from every thread without locks on the recording
path, grouped into one record per frame, and exported as Chrome-trace JSON (for Perfetto) and as
a compact JSON summary for agents (ADR-0016). It is a separate crate at layer 0 so that every
engine crate, including the deterministic ones, can record into it, and so that one Cargo feature
(`telemetry`) removes all recording from a build.

## 2. Scope

**In scope:**
- The recording macros `span!`, `counter!`, `gauge!` and `log!`, the span guard, and log levels.
- The compile-out feature `telemetry` and what "near-zero cost when disabled" means (§4, TELE-R002).
- Per-thread record buffers (bounded, single-producer single-consumer, lock-free) and their
  hand-off to the collector once per frame.
- The `Collector`: the one object per process that turns recording on, drains all thread buffers
  and returns one `FrameRecord` per frame.
- Frame records: spans, counter totals, gauge values, log events, threads, dropped-record count,
  frame timing, and the number of simulation steps the caller reports.
- Export of frame records to Chrome-trace JSON and to a compact JSON summary, written by hand
  without `serde` (`serde` is not approved for runtime crates; OQ-002 is open until M2).
- The crate's error enum `TelemetryError` (codes `LIM-TELE-NNNN`, §8).

**Out of scope / non-goals:**
- Calling `Collector::end_frame` from the main loop, and deciding which schedules and systems get
  spans: `liminal_app` (M1-T044) and `liminal_ecs` (M1-T037). `liminal_jobs` names its spans in
  its own spec (M1-T034).
- Allocator statistics, entity counts and GPU timings as frame data: the crates that own those
  numbers report them through `counter!` and `gauge!` (`liminal_memory` cannot be read from here,
  see `docs/specs/liminal_memory.md` §2 on its branch; GPU timings: open question 6).
- Printing log events to a terminal. This crate never writes to stdout or stderr
  (CODE_STYLE.md lints); which binary prints logs, and how, is decided by the `liminal_app` and
  `liminal_cli` specs (open question 9).
- The probe methods `telemetry.frames` and `log.tail` and their JSON (ADR-0019, M2):
  `liminal_probe` (open question 5).
- Bridging the `log` crate used by external dependencies such as `winit`: `liminal_platform`, if
  needed, through an ADR (ADR-0016 Consequences; open question 8).
- A counting `#[global_allocator]` for other crates' "no allocation after warm-up" tests. This
  crate uses one only inside its own tests (open question 7).
- Sampling profilers, CPU performance counters, GPU timeline capture, network export, files on
  disk. Exporters return `String`; the caller writes it where it wants.
- Determinism of recorded values. Timestamps come from the OS clock; the crate is
  `deterministic = false` (§11).

## 3. Dependencies

- Engine crates: `liminal_core` (`architecture.toml`: layer 0, `same_layer_deps =
  ["liminal_core"]`). Used items: `liminal_core::error::{ErrorCode, EngineError, ErrorContext,
  ContextValue, write_display}` (the error model, ADR-0017) and `liminal_core::time::Tick`. These
  are specified in [`docs/specs/liminal_core.md`](liminal_core.md) (M1-T012).
- External crates: none at runtime (`external = []`). Dev-dependencies `proptest` and `loom`
  (`dev_external`, ADR-0010, ADR-0018).
- `unsafe = "allow"` (ADR-0018): `unsafe` is allowed only in the per-thread ring buffer (module
  `buffer`, private). Every block has a `// SAFETY:` comment; the module is tested under Miri and
  loom (§10). Every other module declares `#![forbid(unsafe_code)]` at its top. Why ADR-0018's
  benchmark clause asks for no benchmark of the ring against a safe buffer: §10 "Ring buffer and
  its reference".
- `deterministic = false`: the crate reads `std::time::Instant`. The determinism lint
  configuration of CODE_STYLE.md does not apply to it.
- Cargo features: `telemetry` (off by default, TELE-R001). Engine crates depend on
  `liminal_telemetry` without the feature; the facade (`liminal_engine`, M2) or a game enables it.
  Cargo unifies features, so enabling it anywhere in the build enables it for every crate.
- Structure: crate root (the four macros, `Level`, `SpanGuard`, `THREAD_BUFFER_CAPACITY`),
  modules `collect`, `export`, `error`, and the hidden support module `__private` (§5.5). Every
  public item is public at exactly one path; nothing is re-exported under a second name, except
  the macro selection of §5.5.

## 4. Concepts

- **Record:** one entry in a thread buffer: a span record, a counter record, a gauge record or a
  log record. A record has a fixed size of at most 48 bytes (TELE-R056). Span, counter and gauge
  records are stored inline. A log record is stored as one pointer to one heap allocation (a
  `Box`) that holds the whole log payload: level, target, message, time and the field list. In
  the buffer it therefore occupies one pointer plus the record tag, whatever its fields are.
  (Storing `target`, `message`, `time_ns` and `level` inline would already need more than 48
  bytes.) Whoever discards a log record frees its payload (TELE-R052, TELE-R061).
- **Active:** recording is active while a `Collector` is installed in the process (from `install`
  returning `Ok` until that collector is dropped). While inactive, the macros record nothing and
  evaluate none of their arguments (TELE-R010). The check is one relaxed atomic load.
- **Collector identity:** every successful `install` takes the next value of a process-wide
  `u64` counter that starts at 1 and is never reused; 0 stands for "no collector". The value the
  active check loads is the identity of the installed collector (or 0), so the check stays one
  relaxed load. A recording span guard keeps the identity it loaded in `span!` and records only
  if the same identity is installed when it is dropped (TELE-R011).
- **Thread buffer:** a ring of `THREAD_BUFFER_CAPACITY` (16,384) records owned by one thread
  (the producer) and drained by the collector (the consumer). A thread gets its buffer on its
  first record while active. It is registered once in a process-wide registry (a mutex taken
  only at registration and by the collector), never on the recording path. The recording path
  is: one relaxed load (active?), one thread-local access, one acquire load of the consumer
  position (only when the cached one says the ring is full), the slot write, one release store
  of the producer position. No lock, no read-modify-write atomic, no allocation, except a
  relaxed `fetch_add` on the dropped counter when the ring is full.

  ```
  thread A: span!/counter!/... --push--> [ring A: head .. tail] --+
  thread B: span!/counter!/... --push--> [ring B: head .. tail] --+--> Collector::end_frame
  thread C (exited)            ---------> [ring C, closed]      --+       |
                                                                         v
                                                              FrameRecord { spans, counters,
                                                                gauges, logs, threads, ... }
  ```
- **Thread index:** a `u32` given to a thread when its buffer is registered: 1 for the first
  thread in the process, 2 for the next, and so on. Indices are never reused, also not across
  collectors. Index 0 is never given to a thread; the Chrome-trace export uses it for the frame
  track.
- **Telemetry epoch and nanosecond timestamps:** the first clock read of the crate in a process
  (from `install` or a recording call) stores an `Instant`, the epoch. Every timestamp is the
  number of nanoseconds from the epoch to an `Instant::now()` reading, as `u64` (enough for 584
  years). Assumption (unverified): `Instant` readings on different threads are comparable (the
  macOS, Windows and Linux monotonic clocks are system-wide). Nothing in this spec depends on
  it except how timelines line up in a trace viewer.
- **Frame:** the interval between two `Collector::end_frame` calls (the first frame starts at
  `install`). The crate does not know what a caller's interval is; "frame" in this spec always
  means this interval. Two ways of calling are intended:
  - Once per main-loop iteration, after rendering (ARCHITECTURE.md §4). A frame then contains
    zero or more simulation steps, which the caller reports as a count in `FrameInfo`.
  - Once after each simulation step, with `FrameInfo { sim_steps: 1, last_tick: Some(tick) }`,
    and once after rendering, with `FrameInfo { sim_steps: 0, last_tick: None }`. Each call
    returns its own frame record, so there is one record per simulation step and one per
    rendered frame (ADR-0016 Decision: records "for each simulation step and rendered frame").
    `index` then counts records, not rendered frames, and the exporters treat every record
    alike (one `frame` event each in the Chrome trace; `frame_ns` in the summary is over all
    records).

  Which of the two `liminal_app` uses is decided by its spec (open question 11).
- **Frame record:** what `end_frame` returns: every record drained in that call, ordered and
  aggregated (TELE-R062), plus frame timing.
- **Span depth:** the number of spans open on the same thread when the span was entered (0 for
  an outermost span). Spans nest by scope: the guard returned by `span!` ends the span when it is
  dropped.
- **Dropped record:** a record that could not be appended because its thread's buffer was full.
  It is lost, and counted (TELE-R052).
- **Compiled out:** with the `telemetry` feature off, the four macros expand to code that type-checks
  its arguments but never evaluates them, never reads a clock, never touches thread-local
  storage, never allocates, and returns a zero-sized `SpanGuard` without drop glue. That is what
  "near-zero cost when disabled" means in this spec: zero work at run time per call site; the
  only cost is type-checking at compile time. The `Collector` and the exporters still exist and
  still work (frame records then contain only frame timing), so code that drives telemetry needs
  no `cfg`.

## 5. Public API

Signatures are normative; bodies and private items are not. Derives listed are part of the API.

### 5.1 Crate root: macros, `Level`, `SpanGuard`

```rust
/// Starts a span. `name: &'static str`. Returns a `SpanGuard`; the span ends when it is dropped.
/// let _span = span!("ecs.schedule.update");
/// let _span = span!(std::any::type_name::<S>());   // any &'static str expression
macro_rules! span { ($name:expr) => { /* … */ }; }

/// Adds `delta: u64` to the counter `name: &'static str` for the current frame.
/// counter!("ecs.entities_spawned", 3);
macro_rules! counter { ($name:expr, $delta:expr) => { /* … */ }; }

/// Sets the gauge `name: &'static str` to `value: f64` (last value in a frame wins).
/// gauge!("memory.frame_arena_bytes", used as f64);
macro_rules! gauge { ($name:expr, $value:expr) => { /* … */ }; }

/// Records a log event. `level: Level`, `message: &'static str`, then zero or more
/// `key = value` fields (key: identifier; value: `impl Into<liminal_core::error::ContextValue>`).
/// Trailing comma allowed.
/// log!(Level::Warn, "chunk load took longer than one frame", chunk = index, millis = ms);
macro_rules! log { ($level:expr, $message:expr $(, $key:ident = $value:expr)* $(,)?) => { /* … */ }; }

/// Log level, most severe first. `Ord` follows declaration order: Error < Warn < … < Trace.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Level { Error, Warn, Info, Debug, Trace }

impl Level {
    /// "error", "warn", "info", "debug", "trace".
    pub fn as_str(self) -> &'static str;
}

/// Ends its span when dropped. Not Send, not Sync (a span starts and ends on one thread).
/// With the `telemetry` feature off: zero-sized, no Drop impl.
#[must_use = "the span ends when the guard is dropped; bind it with `let _span = …`"]
pub struct SpanGuard { /* … */ }

/// Records per thread buffer. Fixed; see TELE-R051 and open question 3.
pub const THREAD_BUFFER_CAPACITY: usize = 16_384;
```

Messages are `&'static str`, not format strings: data goes into fields, so log events stay
machine-readable and the recording path does not format text. Name convention (SHOULD, not
checked): lowercase, dot-separated, crate area first (`ecs.schedule.update`,
`jobs.parallel_for`).

`let _ = span!(…)` drops the guard at once and records a span of about zero length; the docs of
`span!` and `SpanGuard` say so.

### 5.2 `liminal_telemetry::collect`

```rust
/// The one collector of a process. Turns recording on while it exists. Send, not Clone.
#[derive(Debug)]
pub struct Collector { /* … */ }

impl Collector {
    /// Errors: LIM-TELE-0001 if another `Collector` exists in this process.
    /// Discards records still in thread buffers. Max log level starts at `Level::Info`.
    pub fn install() -> Result<Collector, TelemetryError>;
    /// Log events with `level > max_level` are not recorded (fields not evaluated).
    pub fn set_max_level(&mut self, max_level: Level);
    pub fn max_level(&self) -> Level;
    /// Drains every thread buffer and closes the current frame.
    pub fn end_frame(&mut self, info: FrameInfo) -> FrameRecord;
}
impl Drop for Collector { /* recording becomes inactive; undrained records are discarded at the next install */ }

/// What the caller knows about the frame that is being closed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameInfo {
    /// Simulation steps (ticks) run in this frame. Default 0.
    pub sim_steps: u32,
    /// The last tick run in this frame, if any. Default None.
    pub last_tick: Option<liminal_core::time::Tick>,
}

/// One frame's telemetry. Fields are public so tests and tools can build records by hand.
#[derive(Clone, Debug, PartialEq)]
pub struct FrameRecord {
    /// 0 for the first frame after `install`, then +1 per `end_frame`.
    pub index: u64,
    /// Nanoseconds since the telemetry epoch (§4).
    pub start_ns: u64,
    pub end_ns: u64,
    pub sim_steps: u32,
    pub last_tick: Option<liminal_core::time::Tick>,
    /// Every thread buffer drained by this `end_frame`, ascending `index`.
    pub threads: Vec<ThreadInfo>,
    /// Ordered by (thread, start_ns, depth), stable.
    pub spans: Vec<SpanRecord>,
    /// One per name, ascending name.
    pub counters: Vec<CounterTotal>,
    /// One per name, ascending name.
    pub gauges: Vec<GaugeValue>,
    /// Ordered by (time_ns, thread), stable.
    pub logs: Vec<LogRecord>,
    /// Records lost to full buffers since the previous `end_frame` (or `install`).
    pub dropped_records: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThreadInfo { pub index: u32, pub name: Option<String> }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpanRecord {
    pub name: &'static str,
    pub thread: u32,
    pub depth: u16,
    pub start_ns: u64,
    pub duration_ns: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CounterTotal { pub name: &'static str, pub total: u64 }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GaugeValue { pub name: &'static str, pub value: f64 }

#[derive(Clone, Debug, PartialEq)]
pub struct LogRecord {
    pub level: Level,
    /// `module_path!()` of the call site.
    pub target: &'static str,
    pub message: &'static str,
    pub thread: u32,
    pub time_ns: u64,
    /// In the order written at the call site; key as written (`stringify!`).
    pub fields: Vec<(&'static str, liminal_core::error::ContextValue)>,
}
```

### 5.3 `liminal_telemetry::export`

```rust
/// Chrome trace event format (JSON object form), one line, no trailing newline (TELE-R080 … R084).
pub fn chrome_trace(frames: &[FrameRecord]) -> String;

/// Compact summary for agents, one line, no trailing newline (TELE-R090 … R095).
/// `top_spans`: maximum number of entries in `top_spans` (0 gives an empty list).
pub fn summary(frames: &[FrameRecord], top_spans: usize) -> String;
```

### 5.4 `liminal_telemetry::error`

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TelemetryError {
    CollectorAlreadyInstalled,   // LIM-TELE-0001
}
impl liminal_core::error::EngineError for TelemetryError { /* §8 */ }
impl std::error::Error for TelemetryError {}
impl core::fmt::Display for TelemetryError { /* liminal_core::error::write_display */ }
```

### 5.5 Hidden support items and feature selection

`#[doc(hidden)] pub mod __private` holds what the macro expansions call (the active check, the
clock, the thread-buffer push) and the disabled expansions `__private::off::{span, counter,
gauge, log}` with their guard type `__private::off::SpanGuard`. These are not API: only this
crate's macros and tests use them (review checks), and they may change without a spec change.
The disabled expansions are compiled in every build, so the tests of TELE-R002 run in the
`--all-features` builds of `cargo xtask ci`. With the feature off, the root macros and
`SpanGuard` are exactly these items (re-exports); with it on, they are the recording versions.

## 6. Requirements

### Feature and macro expansion (R001–R009)

| ID | Level | Requirement |
|---|---|---|
| TELE-R001 | MUST | The crate has a Cargo feature `telemetry` that is not in `default`. The feature changes only which expansion the four macros and `SpanGuard` use (§5.5). `Level`, `THREAD_BUFFER_CAPACITY`, `Collector`, `FrameInfo`, `FrameRecord` and its element types, `export` and `TelemetryError` exist with the same signatures and behave as specified with the feature on or off. |
| TELE-R002 | MUST | The disabled expansions `__private::off::{span, counter, gauge, log}` evaluate none of their argument expressions (an argument with a side effect, such as incrementing a `Cell`, leaves it unchanged after 1,000 calls of each macro), perform no heap allocation (0 allocations counted by a counting global allocator in the test binary over those calls), and record nothing: with a `Collector` installed before the calls, which run on a newly spawned thread, the next `end_frame` returns a frame record with empty `spans`, `counters`, `gauges` and `logs`, `dropped_records == 0`, and no thread in `threads` that was not registered before the calls. `__private::off::SpanGuard` has `size_of == 0` and `needs_drop == false`. |
| TELE-R003 | MUST | The disabled expansions type-check their arguments with the same types as the enabled ones (`&'static str` names and messages, `u64` delta, `f64` value, `Level`, `Into<ContextValue>` field values), and every variable named in an argument counts as used: a crate that compiles without warnings with the feature on also compiles without warnings with it off, as far as the macros are concerned. A `compile_fail` doc test shows that `counter!("x", 1.5_f64)` is rejected through the disabled expansion. |
| TELE-R004 | MUST | With the feature off, `liminal_telemetry::SpanGuard` is `__private::off::SpanGuard` and the root macros expand exactly as the `__private::off` macros. Checked by a test compiled only under `cfg(not(feature = "telemetry"))`; `cargo test -p liminal_telemetry --no-default-features` and `cargo test -p liminal_telemetry --features telemetry` both pass without warnings. `cargo xtask ci` runs the first in its `test-features-off` step (XTASK-R014) and covers the second with the `--all-features` `test` step. |
| TELE-R005 | MUST | The macros name every item they use through `$crate` paths, so a caller needs only a dependency on `liminal_telemetry` and no `use` statement for the expansion to compile (field values are converted with a `$crate`-reachable path to `ContextValue`). Checked by an integration test module with no `use` items. |
| TELE-R006 | MUST | With the feature on and recording active, every argument expression of a macro call is evaluated at most once per call (side-effect counters in a test show exactly one evaluation per recorded call). |

### Active state (R010–R019)

| ID | Level | Requirement |
|---|---|---|
| TELE-R010 | MUST | Recording is active exactly while a `Collector` exists (TELE-R060). While inactive, `span!` returns a guard that records nothing when dropped, and `counter!`, `gauge!` and `log!` record nothing; none of the four evaluates any argument expression. |
| TELE-R011 | MUST | A span records only if recording was active both when `span!` was called and when its guard is dropped. A span entered while inactive records nothing even if a collector is installed before the guard is dropped; a span whose collector is dropped before the guard records nothing. A span also records only into the collector it was entered under: if that collector is dropped and another one is installed before the guard is dropped, the span records nothing (the guard compares collector identities, §4), so no frame record holds a span with a `start_ns` before its collector's `install`. Test: enter a span under collector A, drop A, install B, drop the guard; B's next frame record has no span of that name. A guard entered while active restores the thread's open-span count when it is dropped (TELE-R021) whether or not it records; a span entered under B inside such a guard therefore has depth 1 although its parent is never recorded. |
| TELE-R012 | MUST | No recording call panics. A record that cannot be stored is either counted as dropped (buffer full, TELE-R052) or, if the calling thread's thread-local storage is already destroyed (a call from another thread-local destructor during thread exit), discarded without being counted. |

### Spans (R020–R029)

| ID | Level | Requirement |
|---|---|---|
| TELE-R020 | MUST | `span!(name)`, with `name: &'static str`, returns a `SpanGuard`. If the span records (TELE-R011), dropping the guard appends one `SpanRecord` to the calling thread's buffer with `name`, the thread index, `start_ns` read during `span!`, `duration_ns` = (clock read during the drop) − `start_ns`, saturating at 0, and `depth` (TELE-R021). |
| TELE-R021 | MUST | Each thread keeps a count of open recording spans. `span!` sets the new span's `depth` to that count and increments it; dropping a guard sets the count to the guard's `depth`. `depth` saturates at `u16::MAX`. With guards dropped in reverse order of creation, the outermost span has depth 0 and a span opened inside k open spans has depth k (tested with 3 levels and with two sibling spans inside one parent). A guard leaked with `mem::forget` records nothing and leaves the count raised until an enclosing guard is dropped. |
| TELE-R022 | MUST | `SpanGuard` is neither `Send` nor `Sync`, in both feature states (`compile_fail` doc tests that move a guard into `std::thread::spawn` and share `&SpanGuard` across threads). |
| TELE-R023 | MUST | `SpanGuard` is `#[must_use]`: a `compile_fail` doc test under `#![deny(unused_must_use)]` shows that the statement `span!("x");` is rejected. |

### Counters and gauges (R030–R039)

| ID | Level | Requirement |
|---|---|---|
| TELE-R030 | MUST | `counter!(name, delta)`, with `name: &'static str` and `delta: u64`, appends a counter record. In a frame record, `counters` holds exactly one `CounterTotal` per distinct name recorded in that frame (from any thread), whose `total` is the sum of its deltas, saturating at `u64::MAX`; a name recorded only with delta 0 is present with total 0. Entries are sorted by `name`, ascending byte order. |
| TELE-R031 | MUST | `gauge!(name, value)`, with `name: &'static str` and `value: f64`, appends a gauge record with the current timestamp. In a frame record, `gauges` holds exactly one `GaugeValue` per distinct name recorded in that frame, whose `value` is the value of the record that is greatest by (timestamp, thread index, order of appending on that thread); NaN and infinities are stored unchanged (compared by bits in tests). Entries are sorted by `name`, ascending byte order. Counter and gauge names are separate: the same name may appear in both lists. |

### Logs (R040–R049)

| ID | Level | Requirement |
|---|---|---|
| TELE-R040 | MUST | `log!(level, message, key = value, …)` accepts `level: Level`, `message: &'static str` and zero or more fields whose key is an identifier and whose value is any expression of a type with `Into<ContextValue>` (`liminal_core::error`), with an optional trailing comma. |
| TELE-R041 | MUST | While active, `log!` evaluates `level` once. If `level <= max_level` (TELE-R042) it evaluates `message` and the field values once each, left to right, and appends a `LogRecord` with `level`, `target` = `module_path!()` of the call site, `message`, the thread index, `time_ns` = the current timestamp, and `fields` in written order with each key as written. If `level > max_level`, it evaluates neither `message` nor any field value and records nothing. |
| TELE-R042 | MUST | `Collector::install` sets the maximum level to `Level::Info`. `set_max_level(l)` makes every `log!` call that starts after it returns use `l`; `max_level()` returns the current value. |
| TELE-R043 | MUST | `Level`'s order is `Error < Warn < Info < Debug < Trace` and `as_str` returns `"error"`, `"warn"`, `"info"`, `"debug"`, `"trace"` respectively. |

### Thread buffers (R050–R059)

| ID | Level | Requirement |
|---|---|---|
| TELE-R050 | MUST | The first record a thread appends while active registers a buffer for it, with the next thread index (1 for the first thread in the process; indices are never reused, also across collectors) and the thread's name at that moment (`std::thread::current().name()`, `None` if unnamed). Every frame record lists the registered threads in `threads`, ascending index. |
| TELE-R051 | MUST | A thread buffer holds up to `THREAD_BUFFER_CAPACITY` = 16,384 records. `end_frame` removes every record from every buffer and places it in the returned frame record (TELE-R062). Records of one thread keep the order in which they were appended wherever this spec orders by "order of appending". |
| TELE-R052 | MUST | When a thread's buffer holds 16,384 records, a further record from that thread is discarded and counted; records already in the buffer are kept. `dropped_records` of a frame record is the number of records discarded this way on all threads since the previous `end_frame` (or since `install`). Test: 16,385 spans on one thread without `end_frame` give 16,384 `SpanRecord`s (the first 16,384) and `dropped_records == 1`; the next frame has `dropped_records == 0`. A log record discarded this way has its payload (§4) freed before the `log!` call returns. Test: with a full buffer, 100 further `log!` calls with two fields each leave the number of live heap allocations of the calling thread unchanged (counting global allocator: allocations minus deallocations). |
| TELE-R053 | MUST | Records appended by a thread that has exited are returned by the next `end_frame`, which lists the thread in `threads`; frame records after that one do not list it. |
| TELE-R054 | MUST | Recording from several threads concurrently with `end_frame` loses and duplicates nothing: every record appended while active appears in exactly one frame record of that collector or is counted in `dropped_records`, or the collector is dropped before draining it. Tested (a) by loom models of the ring buffer (push against drain, full buffer, producer exit) with a small capacity, and (b) by a stress test: 4 threads × 100,000 `counter!("n", 1)` calls while the main thread calls `end_frame` in a loop; the sum of all `n` totals plus all `dropped_records` after the threads are joined and a final `end_frame` equals 400,000. |
| TELE-R055 | MUST | After a thread's buffer is registered, `span!` (including the guard's drop), `counter!` and `gauge!` on that thread perform no heap allocation (0 allocations counted by a counting global allocator over 10,000 calls of each, with recording active and the buffer drained between batches). |
| TELE-R056 | MUST | One buffered record occupies at most 48 bytes (`size_of` of the private record type ≤ 48, checked by a unit test), so one thread buffer occupies at most 16,384 × 48 = 786,432 bytes. The buffer is allocated once, at registration. A log record's payload is boxed as a whole (§4) and is not part of the 48 bytes or of the buffer's size; `log!` is the only recording call that allocates (TELE-R055). |
| TELE-R057 | MUST | The ring buffer has a reference implementation, a `Mutex<VecDeque<_>>` with the same capacity and drop rule, and a differential property test: for generated single-threaded sequences of push and drain operations (including full-buffer cases), both return the same records in the same order and the same dropped counts (AGENTS.md rule 4). |

### Collector and frame records (R060–R079)

| ID | Level | Requirement |
|---|---|---|
| TELE-R060 | MUST | `Collector::install` returns `Ok` when no other `Collector` exists in the process, and `Err(TelemetryError::CollectorAlreadyInstalled)` (LIM-TELE-0001) otherwise, leaving the existing collector unchanged. After a collector is dropped, `install` succeeds again. Recording is active from `install` returning `Ok` until the collector is dropped. |
| TELE-R061 | MUST | `install` discards every record that is in a thread buffer when it runs (freeing the payload of each discarded log record, §4), resets the dropped counts, and removes the buffers of threads that have exited. Test (one thread): records appended under a first collector that is dropped without `end_frame` do not appear in the first frame record of a second collector, whose `dropped_records` is 0. |
| TELE-R062 | MUST | `end_frame` returns every record appended since the previous `end_frame` (or `install`) and not dropped, as follows: `spans` sorted by (thread, `start_ns`, `depth`), stable with respect to order of appending; `counters` and `gauges` as in TELE-R030 and TELE-R031; `logs` sorted by (`time_ns`, thread), stable with respect to order of appending; `threads` as in TELE-R050. |
| TELE-R063 | MUST | The first `end_frame` after `install` returns `index` 0, and each further call returns the previous `index` + 1. `start_ns` of frame 0 is the timestamp read in `install`; `start_ns` of frame k > 0 equals `end_ns` of frame k − 1; `end_ns` is read once per call, after all buffers are drained, so `start_ns <= end_ns`. |
| TELE-R064 | MUST | `end_frame(info)` copies `info.sim_steps` and `info.last_tick` into the frame record unchanged. |
| TELE-R065 | MUST | Dropping the `Collector` makes recording inactive before the drop returns. Records not yet drained are not returned by any later call (they are discarded by the next `install`, TELE-R061). Until then they stay in their buffers, log payloads included: at most 16,384 records per registered thread, reachable from the process-wide registry (assumption: Miri's leak check does not report memory reachable from a `static`). |

### Chrome-trace export (R080–R089)

`T(ns)` below is a nanosecond value written as microseconds with exactly three decimals: the
decimal digits of `ns / 1000` (integer division, no leading zeros, `0` for zero), `.`, and
`ns % 1000` as exactly three digits. Example: `T(1_234_567)` = `1234.567`, `T(5)` = `0.005`.

| ID | Level | Requirement |
|---|---|---|
| TELE-R080 | MUST | `chrome_trace(frames)` returns one JSON object (RFC 8259) without whitespace outside strings and without a trailing newline: `{"displayTimeUnit":"ns","traceEvents":[` the events of TELE-R081, separated by `,`, then `]}`. |
| TELE-R081 | MUST | The events are, in this order: (1) `{"name":"process_name","ph":"M","pid":1,"tid":0,"args":{"name":"liminal"}}`; (2) `{"name":"thread_name","ph":"M","pid":1,"tid":0,"args":{"name":"frames"}}`; (3) for each thread index in the union of all frames' `threads`, ascending: `{"name":"thread_name","ph":"M","pid":1,"tid":I,"args":{"name":N}}` with `N` the name from the first frame (in slice order) that lists index `I`, or `thread-I` if that name is `None`; (4) for each frame in slice order: the frame event `{"name":"frame","cat":"frame","ph":"X","ts":T(start_ns),"dur":T(end_ns − start_ns, saturating at 0),"pid":1,"tid":0,"args":{"index":INDEX,"sim_steps":STEPS}}`, then one event per span in `spans` order `{"name":NAME,"cat":"span","ph":"X","ts":T(start_ns),"dur":T(duration_ns),"pid":1,"tid":THREAD}`, then one per log in `logs` order `{"name":MESSAGE,"cat":"log","ph":"i","s":"t","ts":T(time_ns),"pid":1,"tid":THREAD,"args":{"level":LEVEL,"target":TARGET,"fields":{K:V,…}}}` (fields in order, `{}` when none, repeated keys written as they are), then one per counter `{"name":NAME,"cat":"counter","ph":"C","ts":T(end_ns),"pid":1,"tid":0,"args":{"value":TOTAL}}`, then one per gauge in the same form with `"cat":"gauge"` and the gauge value. |
| TELE-R082 | MUST | Strings are escaped exactly as CORE-R009 of the [`liminal_core` spec](liminal_core.md) specifies for `render_json`. The integers this crate itself writes (frame and thread indices, counts, counter totals, `sim_steps`, nanosecond values; all unsigned) are decimal without sign or leading zeros. Log field values are written as CORE-R010 specifies for `ContextValue`, so a negative `Int` has a leading `-`; gauge values are written as CORE-R010 specifies for `Float` (finite: Rust's `Display` for `f64`; NaN and infinities: `null`). Timestamps and durations use `T`. |
| TELE-R083 | MUST | `chrome_trace` is a pure function of its input: equal slices give identical bytes in every call, process and supported platform. An empty slice gives the two metadata events of items (1) and (2) only. |
| TELE-R084 | SHOULD | A trace written by `chrome_trace` from a real run (several threads, nested spans, logs, counters) loads in the Perfetto UI (ui.perfetto.dev) and shows one track per thread with nested spans and the frame track. The same check covers two points this spec assumes but has not checked in a viewer (open question 12): a counter and a gauge with the same name, and a gauge event with `"value":null`. Checked by hand once per format change, result recorded in the journal (M1-T029). |

### Summary export (R090–R099)

| ID | Level | Requirement |
|---|---|---|
| TELE-R090 | MUST | `summary(frames, top_spans)` returns one JSON object without whitespace outside strings and without a trailing newline, with exactly these members in this order: `version` (`1`), `frame_count`, `first_frame` and `last_frame` (the `index` of the first and last frame in slice order, `null` for an empty slice), `frame_ns` (TELE-R091; `null` for an empty slice), `sim_steps` (the sum of the frames' `sim_steps`, computed as `u64`, saturating at `u64::MAX`; the sum of fewer than 2^32 `u32` values cannot reach it), `top_spans` (TELE-R092), `counters` and `gauges` (TELE-R093), `logs` (TELE-R094), `dropped_records` (sum of `dropped_records`, saturating at `u64::MAX`). Escaping and number formats as in TELE-R082. |
| TELE-R091 | MUST | `frame_ns` is `{"min":…,"p50":…,"p95":…,"max":…,"mean":…}` over the frame durations `d = end_ns − start_ns` (saturating at 0) of all frames. With the durations sorted ascending as `s[0..n]`, `pK = s[ceil(K × n / 100) − 1]` (nearest rank; in integers `s[(K × n + 99) / 100 − 1]`), `min = s[0]`, `max = s[n − 1]`, `mean` = floor(sum / n) with the sum computed without overflow (`u128`). |
| TELE-R092 | MUST | `top_spans` aggregates all spans of all frames by `name`: `count`, `total_ns` (sum of `duration_ns`, saturating), `mean_ns` = floor(`total_ns` / `count`), `max_ns`. Entries are sorted by `total_ns` descending, then `name` ascending byte order; the first `top_spans` entries are written, each as `{"name":…,"count":…,"total_ns":…,"mean_ns":…,"max_ns":…}`. |
| TELE-R093 | MUST | `counters` has one entry `{"name":…,"total":…}` per counter name in any frame, total = sum over frames, saturating; `gauges` has one entry `{"name":…,"last":…,"min":…,"max":…}` per gauge name, `last` = its value in the last frame (slice order) that has it, written as TELE-R082 writes a gauge value (`null` if it is NaN or infinite); `min` and `max` are taken over the finite values of all frames only, so NaN and infinities never take part, and both are `null` exactly when the gauge has no finite value. Both lists are sorted by name, ascending byte order. |
| TELE-R094 | MUST | `logs` is `{"error":…,"warn":…,"info":…,"debug":…,"trace":…}`: the number of log records of each level over all frames. |
| TELE-R095 | MUST | `summary` is a pure function of its input: equal arguments give identical bytes in every call, process and supported platform. |

### Errors (R100–R109)

| ID | Level | Requirement |
|---|---|---|
| TELE-R100 | MUST | `TelemetryError` implements `liminal_core::error::EngineError`; for every variant, `validate_error(&e, "TELE")` returns `Ok(())`, and code, message, help and context are exactly those of §8. |

### Assignment to implementation tasks

Every MUST requirement is assigned to exactly one task; the IDs are in the tasks'
`requirements` frontmatter.

| Task | Requirements |
|---|---|
| M1-T028 (recording, buffers, collector) | TELE-R001 to TELE-R006, TELE-R010 to TELE-R012, TELE-R020 to TELE-R023, TELE-R030, TELE-R031, TELE-R040 to TELE-R043, TELE-R050 to TELE-R057, TELE-R060 to TELE-R065, TELE-R100 |
| M1-T029 (export, benchmark) | TELE-R080 to TELE-R083, TELE-R090 to TELE-R095 |

TELE-R084 (SHOULD) is done by M1-T029 (its acceptance criteria require the Perfetto check).
M1-T028 implements `Collector::end_frame` completely, with the record contents (TELE-R062), frame
index and timing (TELE-R063) and the `FrameInfo` fields (TELE-R064), each with its tests, because
its own tests read records only through `end_frame`. No field of a frame record is provisional in
any task (changed in `spec_version` 2; AGENTS.md rule 1).

## 7. Invariants

- INV-1: Each thread buffer has exactly one producer (its thread) and at most one consumer at a
  time (the code holding the registry mutex: `end_frame` or `install`). Checked by loom models.
- INV-2: For each ring, `0 <= tail − head <= THREAD_BUFFER_CAPACITY` (positions as wrapping
  counters). `debug_assert!` on both sides.
- INV-3: At most one `Collector` exists in a process (TELE-R060).
- INV-4: Within one collector, frame indices increase by exactly 1 and frames tile time:
  `start_ns(k + 1) == end_ns(k)`, `start_ns(k) <= end_ns(k)`.
- INV-5: The recording path takes no lock and performs no read-modify-write atomic operation,
  except the relaxed `fetch_add` on a full buffer's dropped counter (§4). Checked in review of
  every change to `buffer` and `__private`.
- INV-6: Every `SpanGuard` is dropped on the thread that created it (`!Send`, TELE-R022), so a
  span record always goes to the buffer of the thread that read its start time.

## 8. Error codes

`liminal_telemetry` has one recoverable error: installing a second collector. Recording calls
never return errors (TELE-R012), and the exporters accept every input. The message and help
below are normative; backticks are part of the text.

| Code | Variant | Meaning (message) | Help | Context |
|---|---|---|---|---|
| LIM-TELE-0001 | `CollectorAlreadyInstalled` | a telemetry collector is already installed in this process. | drop the existing `Collector` before calling `Collector::install` again, or pass the existing collector to the code that needs it. | none |

Codes 0002 onward are unassigned; new codes take the next free number.

## 9. Performance budgets

None yet: every number below is set from a measurement by M1-T029 in `liminal_benches`
(ADR-0023), baseline `ci/perf/<machine-id>/liminal_telemetry.json` on the reference machine
(PERFORMANCE.md), and written into this table with a `spec_version` bump. Cases are
single-threaded unless stated. Benchmark case IDs: `liminal_telemetry/<group>/<case>`.

| Scenario | Budget | Machine | Source of number |
|---|---|---|---|
| Span overhead, active: `span!` + guard drop on one thread, buffer drained between samples (ns per span) | none yet | Reference | M1-T029, case `liminal_telemetry/span/active` |
| Span overhead, inactive (feature on, no collector) (ns per span) | none yet | Reference | M1-T029, case `liminal_telemetry/span/inactive` |
| Span overhead, compiled out: a loop calling `__private::off::span!` against the same loop without it (difference in instructions per iteration; expected 0 by construction, TELE-R002) | none yet | CI Linux x86_64 (instruction counts) | M1-T029 adds the case; counted once the instruction gate of M1-T033 exists |
| `counter!` active (ns per call) | none yet | Reference | M1-T029, case `liminal_telemetry/counter/active` |
| `log!` filtered out by level (ns per call) | none yet | Reference | M1-T029, case `liminal_telemetry/log/filtered` |
| `log!` active with two fields (`u64`, `&str`) (ns per call) | none yet | Reference | M1-T029, case `liminal_telemetry/log/two_fields` |

The span-overhead budget required by ADR-0016 is the first row. Until it is measured, this spec
makes no claim about recording cost beyond the operation counts of §4 and TELE-R055.

No task measures the cost of `end_frame` (for example draining 10,000 spans from each of 4
threads) or of the exporters (for example `chrome_trace` of 600 frames × 100 spans), so they have
no row (two "Not scheduled" rows were removed in `spec_version` 2). The task that prepares the
`stable` promotion of `liminal_telemetry` (not written yet) adds their scenarios here first.

The table has no case that compares the ring buffer with its `Mutex<VecDeque<_>>` reference; the
reason is in §10 "Ring buffer and its reference".

## 10. Test plan

**Ring buffer and its reference.** The one optimized path is the lock-free ring buffer. Its
reference is a `Mutex<VecDeque<_>>` buffer with the same capacity and drop rule (TELE-R057). The
reference is private test code (`#[cfg(test)]`), not a `liminal_telemetry::reference` item: the
ring is private too, so no benchmark or test outside the crate can name either, and
PERFORMANCE.md "Reference implementations reachable from benchmarks" item 3 keeps a reference
that only the crate's own tests use private. §5 therefore lists no `reference` module.

ADR-0018 says: "Unsafe for performance requires a benchmark showing the safe version is
insufficient, referenced in the commit or spec." This spec lists no such benchmark, for this
reason: the ring's `unsafe` is not there to make a safe buffer that meets the contract faster.
ADR-0016 decides "lock-free per-thread buffers", and §4 and INV-5 fix the recording path (no
lock, no read-modify-write atomic). The safe buffers in `std` do not meet that:

| Safe candidate | Why it does not meet the contract |
|---|---|
| `Mutex<VecDeque<_>>` (the reference) | Takes a lock on every record. |
| `std::sync::mpsc::sync_channel` with `try_send` | No mutex on the send path, but a compare-and-swap loop on the tail position (read in Rust 1.99.1, `library/std/src/sync/mpmc/array.rs`; an implementation detail, not a documented guarantee), which INV-5 excludes. |

Assumption (not proven here): a single-producer single-consumer ring without `unsafe` would need
a safe cell that two threads can access without a lock and that holds a whole record, and `std`
has none (its atomics hold one machine word). The comparison with `sync_channel` is by reading
its source, not by measurement, and the "no read-modify-write atomic" rule of INV-5 is this
spec's choice, not ADR-0016's wording. The absolute recording cost is measured (§9,
`span/active`). If a later change adds `unsafe` only to make recording faster than a safe
version that meets the contract, ADR-0018's clause applies to that change and it needs the
benchmark.

| Area | Test kinds | Oracle |
|---|---|---|
| Feature and expansion (R001–R006) | unit, integration, `compile_fail` doc tests, a `cfg(not(feature = "telemetry"))` test | Side-effect counters (`Cell`); counting global allocator in the test binary; empty frame records |
| Active state (R010–R012) | integration | Hand-written expected records |
| Spans (R020–R023) | integration, `compile_fail` doc tests | Expected names, depths and threads; durations checked only as `start_ns + duration_ns <= end_ns` of the frame and `>=` the inner span's (no wall-clock values asserted, TESTING.md hygiene) |
| Counters, gauges, logs (R030–R043) | unit, integration, property (counter totals equal the saturating sum of generated deltas; gauge last value) | Reference computation over the generated sequence in the test |
| Thread buffers (R050–R057) | unit, property (differential), loom, Miri, stress integration | `Mutex<VecDeque>` reference buffer; loom exhaustive interleavings with capacity 2 to 4; count conservation (R054) |
| Collector and frames (R060–R065) | integration | Hand-written expectations; frame tiling checked over 100 frames |
| Chrome trace (R080–R084) | snapshot, unit, manual check | Approved snapshot `crates/liminal_telemetry/tests/snapshots/chrome_trace.json` built from hand-made frame records (2 frames, 2 threads one unnamed, nested spans, a log with every `ContextValue` variant, a negative `Int` among them, and characters that need escaping, counters, a NaN gauge, a gauge with the values `+∞` and 1.5 in the two frames, a counter and a gauge with the same name); unit tests of `T(ns)` with hand-computed values (0, 5, 999, 1000, 1_234_567, `u64::MAX`); Perfetto load (R084) |
| Summary (R090–R095) | snapshot, unit | Approved snapshot `crates/liminal_telemetry/tests/snapshots/summary.json` from the same frames; percentile unit tests with hand-computed values for n = 1, 2, 20, 21 |
| Errors (R100) | unit | `liminal_core::error::validate_error`; §8 text |

Snapshot approval: the first snapshot of each format is checked once with an independent JSON
parser available on the machine (for example `python3 -m json.tool`; nothing is installed for it,
ADR-0022) and the result is recorded in the journal; if none is available, the journal says it
was not checked.

Tests that install a `Collector` MUST serialize on one lock per test binary, because a test
binary is one process and a process has at most one collector (TELE-R060):

| Tests | Process | Lock |
|---|---|---|
| Integration tests in `crates/liminal_telemetry/tests/*.rs` | One per file | A `static` mutex in `tests/common/mod.rs`, included by each file with `mod common;` |
| Unit tests in `src/` | One for the whole crate (the lib test binary) | One `#[cfg(test)]` `static` mutex in the crate, used by every unit test that installs a collector |
| Doc tests | One per doc test | None needed |
| Test binaries of other crates that install a `Collector` | One per binary | Their own `static` mutex per binary. This crate exports no lock; whether `liminal_testkit` provides one is for its spec (M1-T047) |

A test that panics while it holds the lock poisons it; the lock helper takes the guard out of
the poison error, so one failing test does not fail the others. Such tests assert only on
records of threads they created or on the calling thread's own records, because the test
harness may reuse threads.

**Miri** runs the whole test suite (`cargo +nightly miri test -p liminal_telemetry`, set up by
M1-T018). Expected exceptions, each to be listed with its reason by M1-T028: the stress test of
TELE-R054 (b) and full-buffer tests with 16,384 records may be too slow under Miri and run with
reduced counts under `cfg(miri)` instead of being excluded where possible. **loom** tests sit
behind `cfg(loom)` and model the ring buffer and the producer-exit hand-off.

Every requirement's verifying test carries `/// Verifies: TELE-Rnnn`. Primary entry points with
`# Examples`: `span!`, `counter!`, `gauge!`, `log!`, `Collector` (install, record, `end_frame`),
`export::chrome_trace`, `export::summary`.

## 11. Determinism

`liminal_telemetry` is `deterministic = false`: it reads `std::time::Instant`, and thread indices
depend on which thread records first. It still must not make deterministic crates
non-deterministic:

- Recording is write-only for the caller. `counter!`, `gauge!` and `log!` evaluate to `()`;
  `SpanGuard` exposes no data. Simulation code cannot read telemetry back except through the
  `Collector`, which only `liminal_app` and tools hold.
- Recording never changes the evaluation of the caller's arguments beyond evaluating them once
  (TELE-R006) or not at all (TELE-R002, TELE-R010, TELE-R041). Argument expressions in
  deterministic crates SHOULD therefore be free of side effects; review checks it.
- The order and aggregation rules of frame records (TELE-R062) and both exporters (TELE-R083,
  TELE-R095) are deterministic functions of the recorded data; only the data (timestamps, thread
  indices, how records split across frames) varies between runs.

Determinism tests (ADR-0024 `tests/determinism.rs`) do not apply: no output of this crate is
claimed to be reproducible across runs.

## 12. Open questions

1. **Requirement and error prefix.** M1-T027's task text names the prefix `TEL`; the reserved
   prefix in `docs/specs/README.md` is `TELE`, and CORE-R012 requires the error area to equal the
   spec's `req_prefix`. Resolved here: `TELE` (also open question 4 of the
   [`liminal_core` spec](liminal_core.md)).
2. **Feature-off build in CI.** Resolved by M1-T056 (`spec_version` 2): the `test-features-off`
   step of `cargo xtask ci` (XTASK-R014 in [`docs/specs/xtask.md`](xtask.md)) runs `cargo test
   --package <name> --no-default-features --locked` for every workspace package with features,
   so the test of TELE-R004 runs in CI once the crate exists. Still not checked by any command:
   each feature built alone (CODE_STYLE.md "Every feature must build alone"); with one feature
   in this crate, `--all-features` is that build.
3. **Buffer capacity.** 16,384 records per thread is fixed (assumption: at 60 frames per second
   it allows about 983,000 records per second per thread, more than any M1 workload is expected
   to produce). Make it configurable only if `dropped_records > 0` shows up in `headless_boids`
   or a demo. Resolve at the `stable` promotion.
4. **Span fields.** Spans carry only a name. Key-value arguments on spans (for example the entity
   count of a system run) are not in `spec_version` 1. Resolve with the `liminal_ecs` and
   `liminal_app` specs (M1-T037, M1-T044) if they need them; otherwise at M2 with the probe.
5. **Probe JSON.** `telemetry.frames` and `log.tail` (ADR-0019, M2) need a per-frame JSON form and
   a log tail across frames. Whether they reuse `export::summary`, add an exporter here, or are
   written by `liminal_probe` is decided by the `liminal_probe` spec (M2).
6. **GPU timings.** ARCHITECTURE.md §5.3 lists GPU timings in the frame record. GPU timestamps
   come from a different clock and arrive frames later. Whether they are gauges, spans on a
   separate track or a new record type is decided by the `liminal_gpu` / `liminal_render` specs
   (M3/M4), with a `spec_version` bump here if a new record type is needed.
7. **Counting allocator** (`docs/specs/liminal_memory.md` open question Q3, on its branch). This
   spec does not provide a public counting `#[global_allocator]`; it uses one in its own tests
   only. The question stays open for the `liminal_testkit` spec (M1-T047).
8. **`log` crate bridge.** External crates (`winit`) log through the `log` crate. Bridging needs
   an ADR (ADR-0016) and lives in `liminal_platform` (M3 kickoff).
9. **Who prints logs.** With no terminal output here, a developer running a game sees log events
   only if some binary prints them. Decide in the `liminal_app` spec (M1-T044) or the
   `liminal_cli` spec (M2).
10. **Cross-thread clock comparability.** Assumed in §4, unverified. Only trace display depends on
    it. Check by inspection of a multithreaded trace in Perfetto (M1-T029, TELE-R084).
11. **One record per simulation step.** ADR-0016 asks for records "for each simulation step and
    rendered frame". This crate supports both ways of calling `end_frame` described in §4
    "Frame". Whether `liminal_app` calls it once per main-loop iteration (simulation steps as a
    count) or after every step and after rendering is decided by the `liminal_app` spec
    (M1-T044). If per-step records need a field that tells a step record from a render record
    beyond `sim_steps`, that spec asks for it here with a `spec_version` bump.
12. **Counter tracks in the Chrome trace.** A counter and a gauge with the same name are both
    written as `"ph":"C"` events with the same `pid` and `name` and different `cat` (TELE-R081).
    Assumption (not checked in a viewer): the trace event format keys counter tracks by process,
    name and `id`, not by `cat`, so the two series would share one track. Also unchecked: how
    Perfetto shows a counter event with `"value":null` (a NaN or infinite gauge). M1-T056 could
    not check either: no code writes a trace yet, and the agent that did M1-T056 had no trace
    viewer. M1-T029 checks both with TELE-R084. If the series share a track, M1-T029 adds an
    `"id"` member (`"counter"` or `"gauge"`) or a name prefix to these events in TELE-R081; if
    a `null` value breaks the track, it omits the event for a non-finite gauge. Either change
    bumps `spec_version`, and the snapshot is approved with it.

## 13. Maturity

Current: `planned`. Next target: `usable` (via `experimental`). Evidence is prepared by M1-T028
and M1-T029 and checked by the session that reviews M1-T029 (MATURITY.md "Promotion procedure").

- [ ] Spec follows `docs/specs/_TEMPLATE.md` and passed a spec review (REVIEW.md §2) (evidence:
      review of M1-T027).
- [ ] Compiles, all lints pass, registered in `ci/coverage.toml` (evidence: M1-T028).
- [ ] All MUST requirements traced: `cargo xtask trace` with `maturity: usable` applied on a
      scratch copy reports every MUST tested (evidence: M1-T029).
- [ ] Line coverage ≥ 85 % (layer 0 pure logic, TESTING.md) (evidence: M1-T029).
- [ ] Every public item documented; the primary entry points of §10 have runnable `# Examples`.
- [ ] Integration tests in `crates/liminal_telemetry/tests/` use only the public API and the
      macros (the `__private::off` tests of TELE-R002 excepted).
- [ ] `unsafe = "allow"`: Miri passes on the test suite, exceptions listed with reasons; every
      `unsafe` block has a `SAFETY:` comment a reviewer has checked; loom tests of the ring buffer
      pass (ROADMAP M1 exit criterion).
- [ ] Feature-off build: `cargo test -p liminal_telemetry --no-default-features` passes
      (TELE-R004); run by the `test-features-off` step of `cargo xtask ci` (XTASK-R014).
- [ ] Span overhead measured and written into §9 (ADR-0016; evidence: M1-T029).
- [ ] At least one example or dependent crate uses it. Open item: `liminal_jobs` (M1-T036) and
      `liminal_app` (M1-T046) are the first users; M1-T029 either adds a runnable example under
      `crates/liminal_telemetry/examples/` or the promotion waits for one of them.

## 14. History

| spec_version | Date | Change | By task |
|---|---|---|---|
| 1 | 2026-11-10 | Initial spec | M1-T027 |
| 2 | 2026-11-10 | Follow-ups of the M1-T027 review. TELE-R063 and TELE-R064 move to M1-T028 (no provisional frame fields). §4: log payload boxed as a whole; collector identity; two ways of calling `end_frame` (per-step records). TELE-R011: a span records only into the collector it was entered under. TELE-R052, R056, R061, R065: discarded log records free their payload. TELE-R082: unsigned-integer rule limited to this crate's integers. TELE-R090: `sim_steps` sum is `u64`, saturating. TELE-R093: gauge `min`/`max` over finite values only. TELE-R004 and open question 2: feature-off test runs in `cargo xtask ci` (XTASK-R014). TELE-R084 and open question 12: counter-track checks. §9: two unscheduled rows removed. §10: ADR-0018 benchmark clause, private reference, collector lock per test binary. Open question 11 (per-step records in `liminal_app`). Links to the `liminal_core` spec | M1-T056 |
