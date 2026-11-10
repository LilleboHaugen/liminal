---
crate: liminal_jobs
req_prefix: JOBS
layer: 1
maturity: planned
spec_version: 1
last_reviewed: 2026-11-10
adrs: [ADR-0009, ADR-0010, ADR-0016, ADR-0017, ADR-0018, ADR-0023]
---

# Spec: `liminal_jobs`

## 1. Purpose

`liminal_jobs` runs closures on several threads: a thread pool with one work-stealing deque per
worker, scoped tasks and `join` that may borrow from the caller's stack, parallel loops over
index ranges and mutable slices with caller-chosen chunk sizes, and job graphs whose nodes run
after their dependencies. It is a separate crate at layer 1 so that `liminal_ecs` (parallel
system execution, ADR-0008), `liminal_app` and later subsystems share one pool and one set of
rules for what parallel code may expose to the simulation (ADR-0009 item 4), and so that the
`unsafe` and lock-free code needed for it stays in one place (ADR-0018).

## 2. Scope

**In scope:**
- `ThreadPool`: thread count, start, shutdown, and the sequential mode (`threads == 1`).
- The fixed-capacity work-stealing deque (`deque`) and the locked reference implementation it is
  checked against (`reference`).
- Scoped tasks (`ThreadPool::scope`, `Scope::spawn`) and `ThreadPool::join`.
- Parallel loops with chunking: `parallel_for`, `parallel_chunks_mut`, `parallel_map_fold`.
- Job graphs with dependencies: `JobGraph`, `ThreadPool::run_graph`.
- Panic behaviour of every entry point.
- Which results are independent of scheduling and thread count (§11).
- The telemetry spans the crate records.
- The crate's error enum `JobsError` (codes `LIM-JOBS-NNNN`, §8).

**Out of scope / non-goals:**
- Deciding which systems may run in parallel, access-conflict checks and command-buffer order:
  `liminal_ecs` (ADR-0008, M1-T037). This crate runs what it is given.
- Owning the engine's pool and choosing its thread count for a game: `liminal_app` (M1-T044).
  This crate has no global pool and no global state.
- Detached jobs that outlive the call that created them (`'static` spawn, futures, task
  handles), needed for asset loading: not in `spec_version` 1 (open question 1).
- Asynchronous I/O, timers, an async executor. Nothing here reads a clock.
- Cancellation. Every job that was created runs, also after another job panicked (§4 "Panic
  rule"; open question 7).
- Thread priorities, core affinity, performance- and efficiency-core selection (open question 4).
- Per-worker data (a public worker index, per-worker scratch memory) (open question 2).
- Parallel iterator adaptors (`map`, `filter`, `zip`, …). The three loop functions of §5.3 are
  the whole loop API.
- Comparison with `rayon`: the competitive benchmark task (M1-T051).

## 3. Dependencies

- Engine crates (`architecture.toml`: layer 1, `same_layer_deps = []`; every layer 0 crate is
  allowed): `liminal_core` (`liminal_core::error::{ErrorCode, EngineError, ErrorContext,
  ContextValue, write_display}`, the error model, [liminal_core.md](liminal_core.md)) and
  `liminal_telemetry` (the `span!` macro, [liminal_telemetry.md](liminal_telemetry.md)).
  `liminal_jobs` depends on `liminal_telemetry` without its `telemetry` feature; its
  dev-dependency on `liminal_telemetry` enables the feature so that the tests of JOBS-R120 to
  JOBS-R122 can read spans.
- External crates: none at runtime (`external = []`). Dev-dependencies `proptest` and `loom`
  (`dev_external`, ADR-0010, ADR-0018). `std` only otherwise (`std::thread`, `std::sync`).
- `unsafe = "allow"` (ADR-0018). `unsafe` is allowed in three modules only: `deque` (reading a
  slot back as `&'a T`), the private module `job` (type and lifetime erasure of closures, jobs
  that live on a waiting thread's stack) and `pool` (executing erased jobs). The modules
  `parallel`, `graph`, `reference` and `error` declare `#![forbid(unsafe_code)]` at their top
  (the loop functions and `run_graph` are built on `scope` and `join`). Every
  `unsafe` block has a `// SAFETY:` comment; the test suite runs under Miri and the lock-free
  code under loom (§10).
- ADR-0018 "unsafe for performance requires a benchmark": the lock-free deque is the one place
  where `unsafe` and lock-free code are chosen for speed over a safe alternative (the locked
  `reference` deque). M1-T036 records the benchmark pair of §9 rows 1 and 2; if the lock-free
  deque is not measurably faster there or in the pool-level cases, that is reported as a finding
  against this spec. The `unsafe` in `job` and `pool` is not for speed: borrowing from the
  caller's stack in a job that another thread runs cannot be expressed in safe Rust
  (`std::thread::scope` uses `unsafe` for the same reason).
- `deterministic = false`: which thread runs a job, and in which order independent jobs run, is
  not reproducible. §11 states what callers in deterministic crates may rely on. The crate reads
  no clock and no OS randomness.
- Cargo features: none.
- Structure: public modules `pool`, `deque`, `graph`, `reference`, `error`; private modules
  `job` and `parallel` (the loop functions are inherent methods of `pool::ThreadPool` implemented
  in `parallel`; `run_graph` is implemented in `graph`). Every public item is public at exactly
  one path; the crate root declares the modules and re-exports nothing.

## 4. Concepts

- **Job:** one closure handed to the pool by `Scope::spawn`, `join`, a loop function (one job
  per chunk, §"Chunk") or `run_graph` (one job per node). A job runs exactly once, on one thread.
- **Pool call:** a call of `scope`, `join`, `parallel_for`, `parallel_chunks_mut`,
  `parallel_map_fold` or `run_graph`. Every pool call blocks until all jobs it created
  (directly, or through a `Scope` it passed on) have finished. No job outlives the pool call
  that created it; this is what makes borrowing from the caller's stack sound.
- **Worker:** a thread spawned and owned by a `ThreadPool`. A pool with `threads() == n` has
  `n − 1` workers, named `liminal-jobs-1` … `liminal-jobs-(n−1)`.
- **Caller, participant:** the caller is the thread that makes a pool call. It may be a worker of
  the same pool (a job making a nested pool call) or any other thread (an *external* caller, for
  example the main thread). The caller takes part in the work: while it waits for its pool call,
  it runs queued jobs of that pool. `threads()` therefore counts the participants of a pool call
  made by one external caller: `n − 1` workers plus the caller.

  ```
  external caller ──push──> [injector: Mutex<VecDeque>] <──take── workers, waiting callers
  worker 1: push/pop (LIFO) ──> [deque 1] <──steal (FIFO)── other workers, waiting callers
  worker 2: push/pop (LIFO) ──> [deque 2] <──steal (FIFO)── …
  ```
- **Deque:** each worker owns one `deque` of capacity `PoolConfig::queue_capacity`. The owner
  pushes and pops at one end (last in, first out); other threads steal from the other end (first
  in, first out). The deque stores `&'a T` in one atomic pointer per slot, so no slot is ever
  read non-atomically while another thread may write it; the pool stores references to its
  private job headers. The deque does not grow: when a worker's deque is full, the worker runs
  the new job itself, at once, before the creating call returns (JOBS-R043).
- **Injector:** one queue per pool for jobs created by external callers, a `Mutex<VecDeque<_>>`
  allocated with capacity `queue_capacity` at pool creation. It may grow (heap allocation) if
  more jobs than that are queued by external callers at one time. The injector is not on the
  path between workers.
- **Helping:** a thread that waits inside a pool call does not only block: it takes jobs from
  its own deque, the injector and other workers' deques and runs them until its pool call is
  complete. It blocks in the OS (parks) only when it finds no job. Consequences callers must
  know: (1) a job may make any pool call on the same pool, to any depth, without deadlock
  (JOBS-R045); (2) a waiting thread may run an unrelated job of the same pool on top of its
  stack, so a lock held across a pool call may be requested again by that unrelated job on the
  same thread. Rule for callers: never hold a lock across a pool call if any job of that pool
  can take the same lock. (3) Stack depth grows with nesting (open question 5).
- **Sequential mode:** a pool with `threads() == 1` has no worker and no job queue in use. Every
  job runs on the caller, inside the call that creates it, in the *sequential order*:
  `Scope::spawn(body)` runs `body` before it returns; `join(a, b)` runs `a`, then `b`; the loop
  functions run chunk 0, 1, 2, …; `run_graph` repeatedly runs the lowest-index node whose
  dependencies have all finished. Sequential mode is the reference executor: equivalence tests
  (ADR-0009 Verification) compare a pool with more threads against it.
- **Chunk:** the loop functions cut an index range `start..end` with a chunk size `c ≥ 1` into
  chunks. With `len = end − start` (0 if `start >= end`), there are `ceil(len / c)` chunks, and
  chunk `k` (from 0) is `start + k·c .. min(start + (k+1)·c, end)`. Chunk boundaries depend on
  `start`, `end` and `c` only, never on the thread count or on scheduling. There is no automatic
  chunk size (open question 8).
- **Job graph:** a set of nodes (each a name and an `FnMut()` closure) and dependencies "node X
  depends on node Y". The dependencies always form a directed acyclic graph, because an edge that
  would close a cycle is rejected when it is added (JOBS-R083). A run executes every node once,
  each after all nodes it depends on. A graph can be run any number of times.
- **Panic rule:** a panic in a job is caught on the thread that ran it. Workers never exit
  because of a job panic. No job is cancelled, with one exception: graph nodes that depend,
  directly or transitively, on a panicked node are not run. When all jobs of the pool call have
  finished, the pool call resumes one of the caught panics on the caller
  (`std::panic::resume_unwind`, payload unchanged); which one is fixed per entry point
  (JOBS-R101 to JOBS-R104).

## 5. Public API

Signatures are normative; bodies and private items are not. Derives listed are part of the API.

### 5.1 `liminal_jobs::pool`

```rust
/// Largest accepted `PoolConfig::threads`.
pub const MAX_THREADS: usize = 256;
/// Default `PoolConfig::queue_capacity` (open question 3).
pub const DEFAULT_QUEUE_CAPACITY: usize = 1024;

/// How a pool is created. Adding a field is a spec change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoolConfig {
    /// Participants of a pool call: the caller plus `threads − 1` workers. 1 ..= MAX_THREADS.
    /// `None` (default): `std::thread::available_parallelism()`, clamped to 1 ..= MAX_THREADS,
    /// or 1 if the OS cannot report it.
    pub threads: Option<usize>,
    /// Capacity of each worker's deque and initial capacity of the injector, in jobs.
    /// A power of two in deque::MIN_CAPACITY ..= deque::MAX_CAPACITY. Default 1024.
    pub queue_capacity: usize,
}
impl Default for PoolConfig { /* threads: None, queue_capacity: DEFAULT_QUEUE_CAPACITY */ }

/// A set of worker threads. Send + Sync; not Clone. Dropping it stops and joins the workers.
pub struct ThreadPool { /* … */ }
impl core::fmt::Debug for ThreadPool { /* threads and queue_capacity only */ }

impl ThreadPool {
    /// Errors: LIM-JOBS-0001 (threads), LIM-JOBS-0002 (queue_capacity), LIM-JOBS-0003 (the OS
    /// refused to start a worker thread).
    pub fn new(config: PoolConfig) -> Result<ThreadPool, JobsError>;

    /// Participants of a pool call (the resolved `PoolConfig::threads`). At least 1.
    pub fn threads(&self) -> usize;

    /// Runs `f` on the calling thread and returns its value after every job spawned on the
    /// scope has finished.
    pub fn scope<'scope, F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Scope<'scope>) -> R;

    /// Runs `a` and `b`, possibly in parallel, and returns both results. `a` runs on the
    /// calling thread.
    pub fn join<A, B, RA, RB>(&self, a: A, b: B) -> (RA, RB)
    where
        A: FnOnce() -> RA + Send,
        B: FnOnce() -> RB + Send,
        RA: Send,
        RB: Send;
}

/// Handle for spawning jobs that may borrow data living at least as long as `'scope`.
/// Send + Sync. Created only by `ThreadPool::scope`.
pub struct Scope<'scope> { /* … */ }

impl<'scope> Scope<'scope> {
    /// Creates a job that runs `body` once, on any participant, possibly before `spawn`
    /// returns. One heap allocation per call (none in sequential mode).
    pub fn spawn<B>(&self, body: B)
    where
        B: FnOnce(&Scope<'scope>) + Send + 'scope;
}
```

### 5.2 `liminal_jobs::deque` and `liminal_jobs::reference`

`deque` is the lock-free deque the pool uses. `reference` is its correctness oracle: the same
three items with the same names and signatures, implemented as `Mutex<VecDeque<&'a T>>` without
`unsafe` (PERFORMANCE.md "Reference implementations reachable from benchmarks", items 1 to 4).
`reference` is not the fast path and the pool never uses it. Differential test:
`crates/liminal_jobs/tests/deque_differential.rs` (JOBS-R028). Benchmark pair: §9 rows 1 and 2.

```rust
// liminal_jobs::deque
pub const MIN_CAPACITY: usize = 2;
pub const MAX_CAPACITY: usize = 1 << 20;

/// Creates an empty deque for `capacity` items and returns its two ends.
/// Errors: LIM-JOBS-0002 unless `capacity` is a power of two in MIN_CAPACITY ..= MAX_CAPACITY.
pub fn bounded<'a, T: Sync>(capacity: usize) -> Result<(Owner<'a, T>, Stealer<'a, T>), JobsError>;

/// The end used by the one owning thread. Send; not Sync; not Clone.
pub struct Owner<'a, T: Sync> { /* … */ }
impl<'a, T: Sync> Owner<'a, T> {
    /// Adds `item` at the owner end. `Err(item)` if the deque holds `capacity` items.
    pub fn push(&self, item: &'a T) -> Result<(), &'a T>;
    /// Removes the item pushed most recently among those still in the deque.
    pub fn pop(&self) -> Option<&'a T>;
    pub fn capacity(&self) -> usize;
}

/// The end used by other threads. Clone + Send + Sync.
pub struct Stealer<'a, T: Sync> { /* … */ }
impl<'a, T: Sync> Clone for Stealer<'a, T> { /* same deque */ }
impl<'a, T: Sync> Stealer<'a, T> {
    /// Removes the item pushed least recently among those still in the deque.
    pub fn steal(&self) -> Steal<&'a T>;
}

/// Result of `Stealer::steal`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Steal<T> {
    /// The deque was empty.
    Empty,
    /// One item was removed.
    Taken(T),
    /// Another `pop` or `steal` on the same deque overlapped this call and nothing was removed.
    /// Call again.
    Retry,
}

// liminal_jobs::reference: the oracle. Same names, same signatures, same documented behaviour.
pub fn bounded<'a, T: Sync>(capacity: usize) -> Result<(Owner<'a, T>, Stealer<'a, T>), JobsError>;
pub struct Owner<'a, T: Sync> { /* push, pop, capacity as above */ }
pub struct Stealer<'a, T: Sync> { /* Clone; steal as above, returns liminal_jobs::deque::Steal, never Retry */ }
```

### 5.3 Loop functions (inherent methods of `pool::ThreadPool`)

```rust
impl ThreadPool {
    /// Calls `body(chunk)` once for every chunk (§4) of `range` with `chunk_size`.
    pub fn parallel_for<F>(&self, range: core::ops::Range<usize>, chunk_size: core::num::NonZeroUsize, body: F)
    where
        F: Fn(core::ops::Range<usize>) + Sync;

    /// Calls `body(offset, part)` once for every chunk of `0..slice.len()`: `part` is the
    /// sub-slice of that chunk and `offset` the index of its first element in `slice`.
    pub fn parallel_chunks_mut<T, F>(&self, slice: &mut [T], chunk_size: core::num::NonZeroUsize, body: F)
    where
        T: Send,
        F: Fn(usize, &mut [T]) + Sync;

    /// Calls `map(chunk)` once for every chunk, then folds the results on the calling thread
    /// in ascending chunk order: `fold(… fold(fold(init, t0), t1) …, t(n−1))`.
    /// One heap allocation (the per-chunk results).
    pub fn parallel_map_fold<T, A, M, F>(
        &self,
        range: core::ops::Range<usize>,
        chunk_size: core::num::NonZeroUsize,
        map: M,
        init: A,
        fold: F,
    ) -> A
    where
        T: Send,
        M: Fn(core::ops::Range<usize>) -> T + Sync,
        F: FnMut(A, T) -> A;
}
```

### 5.4 `liminal_jobs::graph`

```rust
/// Identifies a node of the `JobGraph` whose `add_node` returned it.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct NodeId { /* index */ }
impl NodeId {
    /// 0 for the first node added to its graph, 1 for the next, and so on.
    pub fn index(self) -> usize;
}

/// Nodes and dependencies; always acyclic. Send. Reusable: can be run many times.
pub struct JobGraph<'a> { /* … */ }
impl<'a> Default for JobGraph<'a> { /* new() */ }
impl<'a> core::fmt::Debug for JobGraph<'a> { /* node names and dependencies, not closures */ }

impl<'a> JobGraph<'a> {
    /// An empty graph.
    pub fn new() -> JobGraph<'a>;
    /// Adds a node. `name` is the node's telemetry span name (JOBS-R121); names need not be
    /// unique.
    pub fn add_node<F>(&mut self, name: &'static str, job: F) -> NodeId
    where
        F: FnMut() + Send + 'a;
    /// Makes `node` run only after `depends_on` has finished.
    /// Errors: LIM-JOBS-0010 (unknown node), LIM-JOBS-0011 (would create a cycle).
    pub fn add_dependency(&mut self, node: NodeId, depends_on: NodeId) -> Result<(), JobsError>;
    pub fn node_count(&self) -> usize;
}

impl ThreadPool {
    /// Runs every node of `graph` once, each after the nodes it depends on.
    pub fn run_graph(&self, graph: &mut JobGraph<'_>);
}
```

A `NodeId` carries only an index. Using a `NodeId` of one graph with another graph is not
detected if the index exists there; the docs of `NodeId` say so.

### 5.5 `liminal_jobs::error`

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum JobsError {
    InvalidThreadCount { threads: usize },                                    // LIM-JOBS-0001
    InvalidQueueCapacity { capacity: usize },                                 // LIM-JOBS-0002
    ThreadSpawnFailed { worker: usize, os_error: String },                    // LIM-JOBS-0003
    UnknownNode { node: usize, node_count: usize },                           // LIM-JOBS-0010
    DependencyCycle { node: usize, node_name: &'static str, depends_on: usize, depends_on_name: &'static str }, // LIM-JOBS-0011
}
impl liminal_core::error::EngineError for JobsError { /* §8 */ }
impl std::error::Error for JobsError {}
impl core::fmt::Display for JobsError { /* liminal_core::error::write_display */ }
```

## 6. Requirements

"For thread counts 1, 2 and 4" in a test statement means: the test runs once with a pool of
each of these `threads` values, on any machine (the OS runs more threads than cores if needed).
Tests never assert which thread ran a job, except where a requirement states it.

### Pool: creation, threads, shutdown (R001–R019)

| ID | Level | Requirement |
|---|---|---|
| JOBS-R001 | MUST | `PoolConfig::default()` is `PoolConfig { threads: None, queue_capacity: 1024 }`; `MAX_THREADS == 256` and `DEFAULT_QUEUE_CAPACITY == 1024`. With `threads: None`, `ThreadPool::new` uses `std::thread::available_parallelism()` clamped to `1 ..= 256`, or 1 if that call returns an error; `threads()` returns the value used. Test: `threads()` of a default pool equals the same expression evaluated in the test. |
| JOBS-R002 | MUST | With `threads: Some(n)`, `1 <= n <= 256`, and a valid `queue_capacity`, `ThreadPool::new` returns `Ok`, `threads()` returns `n`, and exactly `n − 1` worker threads have been started when `new` returns (counted in a unit test through the private function that starts a worker). |
| JOBS-R003 | MUST | `ThreadPool::new` validates in this order and starts no thread if a check fails: `threads` is `Some(0)` or `Some(n)` with `n > 256` gives `Err(JobsError::InvalidThreadCount { threads: n })` (LIM-JOBS-0001); `queue_capacity` that is not a power of two in `2 ..= 1_048_576` gives `Err(JobsError::InvalidQueueCapacity { capacity })` (LIM-JOBS-0002). Tested with `threads` 0, 257 and `usize::MAX`, and `queue_capacity` 0, 1, 3, 1000, 2_097_152. |
| JOBS-R004 | MUST | If the OS refuses to start worker `i` (`std::thread::Builder::spawn` returns `Err(e)`), `ThreadPool::new` stops and joins the workers it already started and returns `Err(JobsError::ThreadSpawnFailed { worker: i, os_error })` (LIM-JOBS-0003), where `os_error` is `e.to_string()`. Tested in a unit test through the private worker-start function made to fail at the third worker of five: the error names `worker: 3`, and the two started workers have exited when `new` returns. |
| JOBS-R005 | MUST | Worker `i` (1 to `threads() − 1`) is an OS thread named `liminal-jobs-<i>` (decimal, no padding). Test: with 4 threads, the thread names observed inside 10,000 jobs spawned on one scope are the caller's name or one of `liminal-jobs-1`, `liminal-jobs-2`, `liminal-jobs-3`. |
| JOBS-R006 | MUST | Dropping a `ThreadPool` makes every worker exit and joins it before `drop` returns. Test: jobs create a thread-local value whose destructor increments a shared counter; after the pool is dropped, the counter equals the number of worker threads that created the value. |
| JOBS-R007 | MUST | `ThreadPool` is `Send + Sync` (static assertion). Pool calls from several external callers at the same time on one pool each meet their requirements. Test: 4 `std::thread::scope` threads each run the recursive `join` sum of JOBS-R044 100 times on one pool of 4 threads and get 2,147,450,880 every time. |
| JOBS-R008 | MUST | Sequential mode (§4): a pool with `threads() == 1` starts no thread, and every closure of `scope` and `join` runs on the calling thread (`std::thread::current().id()` equal in every closure) in the sequential order: `Scope::spawn(body)` runs `body` to its end before `spawn` returns; `join(a, b)` runs `a` then `b`. Tested by recording the order of closure calls in a `Vec`, with nested spawns and nested joins. (Loop functions: JOBS-R065; `run_graph`: JOBS-R085.) |
| JOBS-R009 | MUST | Pools are independent and there is no global pool: two pools exist at once, and a job of pool A that makes a pool call on pool B completes with the specified result (pool B treats the thread as an external caller). Test: jobs spawned on a scope of pool A each run the recursive `join` sum of JOBS-R044 on pool B, for thread counts (A, B) = (1, 2), (2, 1) and (2, 2). |
| JOBS-R010 | MUST | No pool call hangs: it returns (or resumes a panic, JOBS-R100) once all its jobs have finished, including when a job becomes available while every other participant is parked. Tested by (a) a loom model of the park and wake protocol: one job is made available while the only worker is about to park, and in every interleaving the job is run and the waiting caller is woken; (b) a stress test: 10,000 `join` calls with empty closures and 1,000 `scope` calls with 8 spawns each, for thread counts 2 and 4, complete (reduced to 100 and 10 under `cfg(miri)`). |
| JOBS-R011 | SHOULD | A participant that finds no job after checking its own deque, the injector and every other worker's deque once parks (blocks in the OS) until a job is made available, its pool call completes or the pool shuts down. It does not spin. Checked in review of every change to `pool`; no test asserts CPU time (TESTING.md hygiene). |

### Deque and its reference (R020–R039)

Requirements R020 to R025 apply to `liminal_jobs::deque`. JOBS-R027 applies them to
`liminal_jobs::reference`.

| ID | Level | Requirement |
|---|---|---|
| JOBS-R020 | MUST | `bounded(capacity)` returns `Ok` with the two ends of one empty deque if `capacity` is a power of two in `2 ..= 1_048_576`, and `Err(JobsError::InvalidQueueCapacity { capacity })` (LIM-JOBS-0002) otherwise (tested with 0, 1, 3, 6, 2_097_152, `usize::MAX`). `Owner::capacity()` returns `capacity`. `MIN_CAPACITY == 2`, `MAX_CAPACITY == 1 << 20`. |
| JOBS-R021 | MUST | With no overlapping calls: `push(item)` returns `Ok(())` and stores `item` if the deque holds fewer than `capacity` items; otherwise it returns `Err(item)` (the same reference, `core::ptr::eq`) and the deque is unchanged. `pop()` removes and returns the most recently pushed item still in the deque, or `None` if it is empty. `steal()` removes and returns as `Steal::Taken` the least recently pushed item still in the deque, or returns `Steal::Empty` if it is empty. Returned references are the pushed ones (`core::ptr::eq`). The deque works for any number of operations (positions wrap around the ring: test with capacity 2 and 1,000 push/steal pairs). |
| JOBS-R022 | MUST | `steal()` returns `Steal::Retry` only if a `pop` or another `steal` on the same deque overlapped the call in time; then it removes nothing. With no overlapping calls it never returns `Retry` (asserted in every single-threaded deque test). |
| JOBS-R023 | MUST | With one owner thread and any number of stealing threads: every item for which `push` returned `Ok` is returned exactly once, by exactly one `pop` or one `Steal::Taken`, or is still in the deque; no call returns an item that was not pushed; the items one `Stealer` clone takes on one thread come in the order they were pushed; `push` returns `Err` only if the deque held `capacity` items at some moment during the call. Tested by (a) loom models with capacity 2 or 4: owner push, push, pop against one stealer; one item with owner pop against two stealers (exactly one of the three gets it); push, steal, push, steal across the wrap-around; (b) a stress test: 1 owner pushing 100,000 distinct items (popping when full) against 3 stealers, then the sum of all returned items checked (1,000 items under `cfg(miri)`). |
| JOBS-R024 | MUST | `bounded` performs the deque's only heap allocations. `push`, `pop`, `steal` and `Stealer::clone` allocate nothing on the heap (0 allocations counted by a counting global allocator in the test binary over 10,000 calls of each; `Stealer::clone` may increment a reference count). |
| JOBS-R025 | MUST | `deque::Owner` is `Send`, not `Sync` and not `Clone`; `deque::Stealer` is `Send`, `Sync` and `Clone`; the same holds for `reference::Owner` and `reference::Stealer`. Checked by static assertions and by `compile_fail` doc tests that share `&Owner` between two threads. |
| JOBS-R026 | MUST | Dropping an `Owner` while `Stealer` clones exist, or the reverse, is sound: the remaining end keeps working (`steal` returns the remaining items, then `Empty`; `push` and `pop` work), and the memory is freed when the last end is dropped (Miri reports no leak and no use after free in a test of both drop orders). |
| JOBS-R027 | MUST | `liminal_jobs::reference::{bounded, Owner, Stealer}` have the signatures of §5.2, meet JOBS-R020, JOBS-R021, JOBS-R023 (stress form) and JOBS-R026, never return `Steal::Retry`, and their module declares `#![forbid(unsafe_code)]`. |
| JOBS-R028 | MUST | Differential property test (AGENTS.md rule 4), `tests/deque_differential.rs`: for generated single-threaded sequences of up to 512 operations (`push` of a fresh item, `pop`, `steal`) and capacities 2, 4, 8 and 64, `deque` and `reference` return identical results for every operation (same `Ok`/`Err`, same item by `core::ptr::eq`, same `Steal` variant). The generator MUST produce sequences that fill the deque and that empty it. |

### Scoped tasks and `join` (R040–R059)

| ID | Level | Requirement |
|---|---|---|
| JOBS-R040 | MUST | `scope(f)` calls `f` exactly once, on the calling thread, with a `Scope`, and returns the value `f` returned. |
| JOBS-R041 | MUST | Every `body` passed to `Scope::spawn` is called exactly once with a `&Scope<'scope>` of the same scope, and may spawn further jobs on it. `scope` returns only after `f` and every job spawned on the scope, directly or from other jobs of the scope, has returned or panicked. Test, for thread counts 1, 2 and 4: 1,000 spawns that each spawn 10 more increment an `AtomicUsize` borrowed from the caller's stack; it reads 11,000 directly after `scope` returns. |
| JOBS-R042 | MUST | Jobs may borrow data that outlives the `scope` call, mutably if each borrow goes to one job (doc example: two jobs write the two halves of a local array obtained with `split_at_mut`). A job cannot borrow a local variable declared inside `f` (a `compile_fail` doc test). `Scope<'scope>` is `Send + Sync` (static assertion). |
| JOBS-R043 | MUST | `Scope::spawn` and `join` never fail, block for queue space or lose a job, whatever `queue_capacity` is: when the calling worker's deque is full, the job runs on the calling thread before the creating call returns. Tests: (a) a pool of 2 threads with `queue_capacity: 2`: a scope spawns 4 jobs that each spawn 2,500 jobs on the same scope; all 10,000 run exactly once (per-job `AtomicU8` flags). (b) Because (a) cannot force its jobs onto a worker, a unit test in `pool` fills a worker's deque through the private push function and checks that the next job created on that worker has run when the creating call returns. |
| JOBS-R044 | MUST | `join(a, b)` calls `a` exactly once on the calling thread and `b` exactly once on any participant, returns `(ra, rb)` with `ra` the value of `a` and `rb` the value of `b`, and returns only after both have returned or panicked. Test, for thread counts 1, 2 and 4: a recursive sum over `0..65_536` that splits with `join` down to single elements equals 2,147,450,880. |
| JOBS-R045 | MUST | A job may make any pool call on its own pool, to any depth. Test, for thread counts 1, 2 and 4: `scope` → spawned job → `parallel_for` (100 chunks) → `join` → `run_graph` (3 nodes in a chain) → `parallel_map_fold`; the call returns and every closure ran the specified number of times. |

### Loop functions (R060–R079)

| ID | Level | Requirement |
|---|---|---|
| JOBS-R060 | MUST | `parallel_for(range, chunk_size, body)` calls `body` exactly once with each chunk of `range` (§4 "Chunk": `ceil(len / c)` non-empty, disjoint, consecutive sub-ranges covering `range`, all of length `c` except possibly the last) and with nothing else, and returns after all calls returned or panicked. An empty or reversed range (`start >= end`) gives zero calls. Chunk arithmetic does not overflow for any `usize` values (tested with `usize::MAX − 5 .. usize::MAX` and chunk sizes 1, 4, `usize::MAX`). Property test, for thread counts 1, 2 and 4: for generated `start`, `len <= 10_000` and `c <= 2_000`, the multiset of ranges passed to `body` equals the chunk list computed in the test. |
| JOBS-R061 | MUST | `parallel_chunks_mut(slice, chunk_size, body)` calls `body(offset, part)` exactly once for each chunk `r` of `0..slice.len()`, with `offset == r.start` and `part` the sub-slice `slice[r]` (same address and length), and returns after all calls returned or panicked. An empty slice gives zero calls. Property test, for thread counts 1, 2 and 4: each `body` writes `offset + i` to `part[i]`; afterwards `slice[j] == j` for every `j`, and the number of calls is `ceil(len / c)`. |
| JOBS-R062 | MUST | `parallel_map_fold(range, chunk_size, map, init, fold)` calls `map` exactly once with each chunk of `range`, giving `t_k` for chunk `k`; then, after every `map` call has returned, calls `fold` exactly once per chunk on the calling thread, in ascending `k`, as `acc = fold(acc, t_k)` starting from `acc = init`, and returns the final `acc`. With zero chunks it calls neither closure and returns `init`. |
| JOBS-R063 | MUST | In `parallel_for`, `parallel_chunks_mut` and `parallel_map_fold`, the chunk boundaries are the same for every thread count and every run (they depend on the range or slice length and `chunk_size` only). Test: the sorted list of chunks seen by `body` or `map` is equal for thread counts 1, 2 and 4 for the same arguments. |
| JOBS-R064 | MUST | After 10 warm-up calls with the same arguments on the same pool, `join` (a recursive tree of depth 10 with 1,024 leaves), `parallel_for` (`0..100_000`, chunk size 100) and `parallel_chunks_mut` (100,000 `u64`, chunk size 100) perform no heap allocation on any thread: 0 allocations counted by a counting global allocator over 100 further calls of each, for thread counts 1 and 4, called from an external caller, in a test binary that runs nothing else and installs no telemetry `Collector`. (`Scope::spawn`, `parallel_map_fold` and `run_graph` may allocate, §5.) |
| JOBS-R065 | MUST | In sequential mode (`threads() == 1`), the loop functions call `body` or `map` on the calling thread in ascending chunk order (chunk 0 first), each call finishing before the next starts. Tested by recording the chunks in a `Vec` for each of the three functions. |

### Job graphs (R080–R099)

| ID | Level | Requirement |
|---|---|---|
| JOBS-R080 | MUST | `JobGraph::new()` and `JobGraph::default()` have 0 nodes. The k-th call of `add_node` on a graph (k from 0) returns a `NodeId` with `index() == k` and stores the closure without calling it; `node_count()` returns the number of `add_node` calls. |
| JOBS-R081 | MUST | `add_dependency(node, depends_on)` returns `Ok(())` and records the dependency if both IDs have an index below `node_count()` and the dependency does not create a cycle. Adding a dependency that already exists returns `Ok(())` and changes nothing. |
| JOBS-R082 | MUST | `add_dependency` with an ID whose index is `>= node_count()` returns `Err(JobsError::UnknownNode { node, node_count })` (LIM-JOBS-0010), where `node` is the index of `node` if that one is out of range, otherwise of `depends_on`; the graph is unchanged. |
| JOBS-R083 | MUST | `add_dependency(node, depends_on)` returns `Err(JobsError::DependencyCycle { node, node_name, depends_on, depends_on_name })` (LIM-JOBS-0011) with the two indices and names, and leaves the graph unchanged, if `node == depends_on` or if `depends_on` already depends on `node` directly or through other nodes. Tested with a self-dependency, a 2-cycle and a 4-cycle closed by the last edge; after the error, `run_graph` runs the graph as it was before the rejected call. |
| JOBS-R084 | MUST | `run_graph(graph)` calls the closure of every node exactly once; a node's closure is called only after the closures of all nodes it depends on have returned; `run_graph` returns after all closures returned or panicked. A graph with 0 nodes returns at once. Property test, for thread counts 1, 2 and 4: for generated graphs of up to 64 nodes (edges only from higher to lower index, so they are acyclic), each closure takes a start and an end number from one shared `AtomicUsize`; every node has exactly one start, and for every dependency the end of the dependency is less than the start of the dependent. |
| JOBS-R085 | MUST | In sequential mode (`threads() == 1`), `run_graph` runs the nodes on the calling thread in this order: repeatedly the lowest-index node that has not run and whose dependencies have all run. Test: nodes 0, 1, 2, 3 with 0 depending on 3, and 1 depending on 2, run in the order 2, 1, 3, 0. |
| JOBS-R086 | MUST | A `JobGraph` can be run any number of times, on the same or different pools; each run meets JOBS-R084, and state captured by the `FnMut` closures persists between runs. Nodes and dependencies may be added between runs. Test: a node that increments a captured counter reads 3 after three runs; a node added after the first run runs in the second. |
| JOBS-R087 | SHOULD | From the second run of an unchanged graph on the same pool, `run_graph` performs no heap allocation (counted as in JOBS-R064 for a graph of 64 nodes). Open question 10. |

### Panics (R100–R109)

| ID | Level | Requirement |
|---|---|---|
| JOBS-R100 | MUST | A panic in any closure passed to a pool call is caught on the thread that ran the closure. After the pool call has resumed the panic on its caller, the pool is fully usable: a following `parallel_map_fold` on the same pool returns the sequential result, and `threads()` is unchanged. The resumed payload is the caught payload unchanged (a `&'static str` or `String` payload is recovered with `downcast` and compared). Tested for each of the six pool calls, for thread counts 1, 2 and 4. |
| JOBS-R101 | MUST | `scope`: if `f` or any spawned job panics, every job spawned on the scope still runs to its end or its own panic (none is skipped), and `scope` then resumes a panic instead of returning: the payload of `f` if `f` panicked, otherwise the payload of one of the panicked jobs (the only one, if exactly one panicked; unspecified which, if several). Test: 100 spawned jobs of which job 37 panics; the other 99 ran; the payload is job 37's. |
| JOBS-R102 | MUST | `join(a, b)`: `a` and `b` are both called, whatever the other does. If `a` panicked, `join` resumes the payload of `a` (also if `b` panicked too); otherwise, if `b` panicked, the payload of `b`. |
| JOBS-R103 | MUST | Loop functions: every chunk's `body` or `map` is called, whatever the others do. If any call panicked, the function resumes the payload of the panicking call with the lowest chunk index, and `parallel_map_fold` does not call `fold`. A panic in `fold` propagates directly (it runs on the caller after all jobs have finished). Test, for thread counts 1, 2 and 4: chunks 3 and 7 of 10 panic with their index as payload; all 10 were called; the payload is 3. |
| JOBS-R104 | MUST | `run_graph`: if a node's closure panics, the nodes that depend on it directly or transitively are not called in this run; every other node is called as in JOBS-R084. `run_graph` then resumes the payload of the panicking node with the lowest index. A later `run_graph` on the same graph calls every node again. Test, for thread counts 1, 2 and 4: a chain 0 ← 1 ← 2 (1 depends on 0, 2 on 1) and an independent node 3; node 1 panics: 0 and 3 ran, 2 did not, the payload is node 1's; in a second run with the panic switched off all four run. |

### Results exposed to deterministic code (R110–R119)

| ID | Level | Requirement |
|---|---|---|
| JOBS-R110 | MUST | The value returned by `parallel_map_fold` equals, bit for bit, the value of the sequential computation `chunks.map(map).fold(init, fold)` over the chunk list of §4, for every thread count, provided `map` returns the same value for the same chunk on every call. This holds for folds that are neither associative nor commutative. Property test, for thread counts 1, 2 and 4: generated `Vec<f32>` (up to 10,000 finite values) summed per chunk and folded with `+`: the result's bits (`f32::to_bits`) equal those of the sequential computation written in the test; and a fold that builds a `Vec<usize>` of chunk start indices returns them in ascending order. |
| JOBS-R111 | MUST | For closures whose effects on shared data are confined to what the API hands them (their chunk of `parallel_chunks_mut`, their return value in `join` and `parallel_map_fold`, data reached only by one graph node or by nodes ordered by dependencies), the data after the pool call is identical for every thread count. Test (`tests/determinism.rs`, ADR-0024): a seeded workload (a `parallel_chunks_mut` over 100,000 `u64` that writes a function of the index, a `join` tree, a `parallel_map_fold`, and a 32-node graph whose nodes each write one slot from the slots of their dependencies) is hashed with `liminal_core::hash::StateHasher` for thread counts 1, 2, 4 and the default; all hashes are equal, and equal across two runs in one process. |

### Telemetry (R120–R129)

Spans are recorded with `liminal_telemetry::span!` and exist only as far as
[liminal_telemetry.md](liminal_telemetry.md) says (feature `telemetry` on, a `Collector`
installed). The tests install a `Collector`, serialized on one lock per test binary.

| ID | Level | Requirement |
|---|---|---|
| JOBS-R120 | MUST | Each call of `scope`, `parallel_for`, `parallel_chunks_mut`, `parallel_map_fold` and `run_graph` records exactly one span on the calling thread, named `jobs.scope`, `jobs.parallel_for`, `jobs.parallel_chunks_mut`, `jobs.parallel_map_fold` and `jobs.run_graph` respectively. The span starts before the first closure of the call starts and ends after the last one has finished (its interval contains the node spans of JOBS-R121 for `run_graph`). It is recorded also for an empty range, slice or graph, and when the call resumes a panic. |
| JOBS-R121 | MUST | `run_graph` records exactly one span per node closure called, on the thread that calls it, named with the node's `name`, covering the closure call. Nodes that are not called (JOBS-R104) record no span. Test: a 16-node graph with distinct names on a pool of 4 threads gives 16 node spans, one per name, in the frame record. |
| JOBS-R122 | MUST | The crate records nothing else: no span for `join`, for a spawned job or for a chunk, no counter, no gauge and no log event. Test: a frame in which only a `parallel_for` over 1,000 chunks whose bodies call `join` runs contains exactly one span and no other record. |

### Errors (R140–R149)

| ID | Level | Requirement |
|---|---|---|
| JOBS-R140 | MUST | `JobsError` implements `liminal_core::error::EngineError`. For the variants `InvalidThreadCount`, `InvalidQueueCapacity` and `ThreadSpawnFailed`, `validate_error(&e, "JOBS")` returns `Ok(())`, and code, message, help and context are exactly those of §8. |
| JOBS-R141 | MUST | For the variants `UnknownNode` and `DependencyCycle`, `validate_error(&e, "JOBS")` returns `Ok(())`, and code, message, help and context are exactly those of §8. |

### Assignment to implementation tasks

Every MUST requirement is assigned to exactly one task; the IDs are in the tasks'
`requirements` frontmatter.

| Task | Requirements |
|---|---|
| M1-T035 (pool, deques, scoped tasks, `join`) | JOBS-R001 to JOBS-R010, JOBS-R020 to JOBS-R028, JOBS-R040 to JOBS-R044, JOBS-R101, JOBS-R102, JOBS-R140 |
| M1-T036 (loop functions, job graphs, telemetry, `usable`) | JOBS-R045, JOBS-R060 to JOBS-R065, JOBS-R080 to JOBS-R086, JOBS-R100, JOBS-R103, JOBS-R104, JOBS-R110, JOBS-R111, JOBS-R120 to JOBS-R122, JOBS-R141 |

Notes on the split:
- M1-T036's task text lists "panic propagation". The panic rules of `scope` and `join`
  (JOBS-R101, JOBS-R102) are assigned to M1-T035 instead: `scope` and `join` are unsound unless
  they wait for their jobs when a closure panics, so M1-T035 cannot ship them without the panic
  path, and a half-defined panic path would be a placeholder (AGENTS.md rule 1). M1-T036 adds
  the rules of the entry points it writes and the test over all six (JOBS-R100).
- The requirements assigned to M1-T035 are testable with `scope` and `join` alone. Requirements
  whose tests need the loop functions or `run_graph` (JOBS-R045 nesting, JOBS-R100 over all six
  pool calls, JOBS-R065 and JOBS-R085 sequential order) are assigned to M1-T036.
- JOBS-R120 includes the `jobs.scope` span; M1-T036 adds it to M1-T035's `scope` (M1-T036's
  scope lists "telemetry spans"). M1-T035 adds the `liminal_telemetry` dependency.
- SHOULD requirements: JOBS-R011 is implemented by M1-T035, JOBS-R087 is decided by M1-T036
  (implemented, or reported with the reason in its handoff).

## 7. Invariants

- INV-1: No job outlives the pool call that created it: a pool call does not return and does not
  unwind while a job it created is queued or running (JOBS-R041, JOBS-R044, JOBS-R101,
  JOBS-R102). Every `SAFETY:` comment on lifetime erasure in `job` and `pool` refers to this
  invariant. Checked by Miri on the panic tests.
- INV-2: A job is executed at most once: a job reference is removed from a queue by exactly one
  `pop`, `steal` or injector take (JOBS-R023), and a job's "taken" state is asserted with
  `debug_assert!` when it starts.
- INV-3: For each deque, with `top` and `bottom` as wrapping position counters,
  `0 <= bottom − top <= capacity` whenever no `pop` is in progress on the owner (`debug_assert!`
  in `push` and after `pop`).
- INV-4: Only the owning thread calls `push` and `pop` on a deque (`Owner` is not `Sync` and not
  `Clone`, JOBS-R025).
- INV-5: `push`, `pop` and `steal` of `liminal_jobs::deque` take no lock, do not park and do not
  loop unboundedly: `steal` returns `Retry` instead of repeating a failed compare-exchange.
  Checked in review of every change to `deque`.
- INV-6: A deque slot is read and written only through an atomic pointer type, so a `steal` that
  loses a race reads a stale pointer value but never performs a data race, and never
  dereferences the value it read unless its compare-exchange on `top` succeeded. Checked by loom
  and Miri.
- INV-7: A `JobGraph` is acyclic at all times (JOBS-R083), so `run_graph` cannot fail and has no
  error result.
- INV-8: A worker thread exits only when its pool is dropped (JOBS-R006, JOBS-R100). A
  `ThreadPool` is never dropped on one of its own workers: a job can reach the pool only through
  a borrow that the blocked caller keeps alive.
- INV-9: While a graph run or a loop call is in progress, each node closure and each chunk is
  accessed by at most one thread (nodes are `FnMut`: `run_graph` takes `&mut JobGraph`, so two
  runs of one graph cannot overlap).

## 8. Error codes

`liminal_jobs` has recoverable errors only where configuration or graph structure is supplied:
`ThreadPool::new`, `deque::bounded`, `reference::bounded` and `JobGraph::add_dependency`. Pool
calls return no errors: their failure mode is a panic in a caller-supplied closure, which is
resumed on the caller (§4 "Panic rule"). Messages and help below are normative; `{x}` is the
value of context field `x`, and the backticks shown are part of the text.

| Code | Variant | Meaning (message) | Help | Context |
|---|---|---|---|---|
| LIM-JOBS-0001 | `InvalidThreadCount` | thread count `{threads}` is outside 1 to 256. | set `PoolConfig::threads` to `None` to use the machine's available parallelism, or to `Some(n)` with n from 1 to 256; 1 runs every job on the calling thread. | `threads` |
| LIM-JOBS-0002 | `InvalidQueueCapacity` | queue capacity `{capacity}` is not a power of two from 2 to 1048576. | use a power of two from 2 to 1048576; the default is 1024 (`DEFAULT_QUEUE_CAPACITY`). | `capacity` |
| LIM-JOBS-0003 | `ThreadSpawnFailed` | the operating system refused to start worker thread `{worker}`: {os_error}. | lower `PoolConfig::threads`, or raise the process limit on threads. | `worker`, `os_error` |
| LIM-JOBS-0010 | `UnknownNode` | node `{node}` does not exist in this job graph, which has `{node_count}` nodes. | pass only `NodeId` values returned by `add_node` of the same `JobGraph`. | `node`, `node_count` |
| LIM-JOBS-0011 | `DependencyCycle` | making node `{node}` (`{node_name}`) depend on node `{depends_on}` (`{depends_on_name}`) would create a dependency cycle. | node `{depends_on}` already depends on node `{node}`, directly or through other nodes (or they are the same node); remove one of these dependencies so that every node can run after the nodes it depends on. | `node`, `node_name`, `depends_on`, `depends_on_name` |

Each message ends with `.`. `usize` context values are `UInt`, names and `os_error` are `Text`.
Codes 0004 to 0009 (pool) and 0012 onward (graph) are unassigned; new codes take the next free
number in their group.

## 9. Performance budgets

None yet: every number below is set from a measurement in `liminal_benches` (ADR-0023),
baseline `ci/perf/<machine-id>/liminal_jobs.json` on the reference machine (PERFORMANCE.md:
Apple M5 Pro, 6 performance and 12 efficiency cores), and written into this table with a
`spec_version` bump. "All" is `PoolConfig::threads: None` on that machine. Benchmark case IDs:
`liminal_jobs/<group>/<case>`. Multithreaded cases are excluded from the CI instruction-count
gate (ADR-0023 Decision 4) and marked so in their case definition.

| Scenario | Budget | Machine | Source of number |
|---|---|---|---|
| `deque`: push 1,024 items then pop 1,024 on one thread, capacity 1,024 (ns per push/pop pair) | none yet | Reference | M1-T036, case `liminal_jobs/deque/push_pop`; single-threaded, so also in the instruction gate once M1-T033 exists |
| `reference` deque: the same workload (ns per pair). Not a budget: the comparison ADR-0018 requires for row 1 | none yet | Reference | M1-T036, case `liminal_jobs/reference_deque/push_pop` |
| `join` tree: recursive `join` to depth 16 (65,536 leaves, each adds one `u64`), threads 1, 2, 4, all (ns per leaf) | none yet | Reference | M1-T036, cases `liminal_jobs/join/tree_16_t<n>` |
| `parallel_for` overhead: `0..1_048_576`, chunk size 1,024, empty body, threads 1, 2, 4, all (ns per chunk) | none yet | Reference | M1-T036, cases `liminal_jobs/parallel_for/empty_t<n>` |
| `parallel_chunks_mut` scaling: 1,048,576 `u64`, chunk size 1,024, each element set to 32 rounds of a 64-bit mix function of its index, threads 1, 2, 4, all (ms per call; speed-up over threads 1) | none yet | Reference | M1-T036, cases `liminal_jobs/parallel_chunks_mut/mix_t<n>` |
| The `join` tree and `parallel_for` workloads against `rayon` | none (comparison, parity not required in M1) | Reference | M1-T051, `ci/perf/competitive/liminal_jobs-<date>.json`, summarized here |

Until these are measured, this spec makes no claim about speed or scaling beyond the allocation
counts of JOBS-R024 and JOBS-R064.

## 10. Test plan

Optimized paths and their references (AGENTS.md rule 4):

| Optimized path | Reference |
|---|---|
| Lock-free deque `liminal_jobs::deque` | `liminal_jobs::reference` (`Mutex<VecDeque>`), differential test JOBS-R028 |
| Parallel execution with `threads() > 1` (stealing, helping, any interleaving) | Sequential mode, `threads() == 1` (JOBS-R008, JOBS-R065, JOBS-R085), in which the order of every closure call is specified; every behavioural test runs for thread counts 1, 2 and 4, and JOBS-R110 and JOBS-R111 compare results across thread counts |
| `parallel_map_fold` | The sequential `map` and `fold` over the chunk list, written in the test (JOBS-R110) |

| Area | Test kinds | Oracle |
|---|---|---|
| Pool creation, shutdown, sequential mode (R001–R009, R065, R085) | unit (private worker-start function for R002, R004), integration, doc test | Hand-written expectations; thread names and thread IDs read inside closures; destructor counter |
| Liveness (R010, R011) | loom (park and wake protocol), stress integration, review | Exhaustive interleavings; completion of the stress test (a hang fails through the CI job timeout) |
| Deque (R020–R028) | unit, property (differential), loom, Miri, stress integration, `compile_fail` doc tests, counting allocator | `liminal_jobs::reference`; conservation of items (sum and exactly-once flags); loom models with capacity 2 or 4 |
| Scope and `join` (R040–R045) | integration, property, `compile_fail` doc test, doc tests | Exact counts in atomics; closed-form sums; sequential mode order |
| Loop functions (R060–R064) | property, integration, counting allocator (a test binary of its own, `tests/no_alloc.rs`) | Chunk list computed in the test from §4; sequential computation |
| Job graphs (R080–R087) | unit, property (generated acyclic graphs), integration | Start and end numbers checked against every dependency; sequential order of R085 by hand |
| Panics (R100–R104) | integration (`std::panic::catch_unwind` around the pool call) | Expected payloads; per-job "ran" flags |
| Determinism (R110, R111) | property, determinism (`tests/determinism.rs`, ADR-0024) | Sequential computation; equal `StateHasher` hashes across thread counts and runs |
| Telemetry (R120–R122) | integration with a `Collector` | Expected span names and counts in the `FrameRecord` |
| Errors (R140, R141) | unit | `liminal_core::error::validate_error`; §8 text |

Rules for these tests:
- No sleeps and no timing assertions (TESTING.md). A test that needs two jobs to overlap uses a
  `std::sync::Barrier` or an atomic flag, and only with a thread count that makes the overlap
  possible; a test never requires that work was stolen.
- Panic tests install no panic hook; the default hook's output on stderr is accepted.
- Tests that install a telemetry `Collector` are in one integration test binary
  (`tests/telemetry.rs`) and take one lock (liminal_telemetry.md §10). The allocation tests are
  in their own binary and take one lock, because the counter is process-wide.
- **Miri** runs the whole test suite (`cargo +nightly miri test -p liminal_jobs`, set up by
  M1-T018). Stress and property tests use reduced counts under `cfg(miri)` (stated in
  JOBS-R010 and JOBS-R023; others chosen by M1-T035 and M1-T036 and listed in their handoffs)
  instead of being excluded. Any test excluded from Miri is listed here with its reason by the
  task that excludes it.
- **loom** tests sit behind `cfg(loom)` and cover: the deque models of JOBS-R023 (a) and the
  park and wake model of JOBS-R010 (a). The modules under test take their atomics, `UnsafeCell`
  and thread primitives from `loom` under `cfg(loom)`.

Every requirement's verifying test carries `/// Verifies: JOBS-Rnnn`. Primary entry points with
`# Examples`: `ThreadPool` (create, `join`), `ThreadPool::scope`, `ThreadPool::parallel_for`,
`ThreadPool::parallel_chunks_mut`, `ThreadPool::parallel_map_fold`, `JobGraph` (build and
`run_graph`), `deque::bounded`.

## 11. Determinism

`liminal_jobs` is `deterministic = false` (`architecture.toml`): which thread runs a job, the
order of independent jobs, and whether a `steal` returns `Retry` differ between runs. ADR-0009
item 4 requires that results exposed to the simulation do not: "parallel reductions MUST be
order-independent or performed in a fixed order". This crate provides the fixed order, and the
rules below say what a deterministic crate may use.

Independent of thread count and scheduling (guaranteed by this spec):

| Fact | Requirement |
|---|---|
| Chunk boundaries of all loop functions | JOBS-R060, JOBS-R063 |
| The result of `parallel_map_fold`: per-chunk results are folded on the caller in ascending chunk order, so the fold needs to be neither associative nor commutative (floating-point sums are bit-identical to the sequential fold) | JOBS-R062, JOBS-R110 |
| Positions of `join` results: `(ra, rb)` | JOBS-R044 |
| Which closures run, and how often, including when closures panic; which panic is resumed by `join`, the loop functions and `run_graph` | JOBS-R101 to JOBS-R104 |
| The order between graph nodes connected by dependencies | JOBS-R084 |
| The complete order of closure calls in sequential mode | JOBS-R008, JOBS-R065, JOBS-R085 |

Not reproducible, and therefore not to be observed by simulation code: the thread a closure runs
on (thread ID, thread name, thread-local data); the relative order of chunks, of spawned jobs and
of graph nodes without a dependency path; the order of effects on shared state written by
several jobs (atomics, locks, channels); `Steal::Retry` and anything else about `deque` used
from several threads; which panic `scope` resumes when several jobs panic; `threads()` with
`PoolConfig::threads: None` (it depends on the machine).

Rules for callers in `deterministic = true` crates (checked in their reviews, and by their
single- versus multithreaded equivalence tests, ADR-0009 Verification):
1. A job writes only data that the API handed to it alone (its chunk, its return value, state
   captured by one graph node) or data ordered by graph dependencies.
2. Reductions go through `parallel_map_fold`, or are order-independent in the exact sense
   (integer addition with wrapping, bitwise operations, `min`/`max` on totally ordered values).
3. Results must not depend on `threads()`. Chunk sizes are constants or functions of the data
   size, never of `threads()`.
4. RNG use inside jobs takes a stream per chunk or per item from `liminal_core::rng` keyed by
   the chunk or item index, never one generator shared by jobs.

The crate has no public way to ask "which worker am I" (open question 2), reads no clock, and
uses no OS randomness; the order in which a thread looks for jobs to steal is unspecified and
affects only timing. Cross-platform bit-identity: the guarantees above do not depend on the
platform (they contain no floating-point or platform-dependent operation of this crate's own);
`tests/determinism.rs` runs on the three CI operating systems (ADR-0024), without a recorded
hash literal, because the workload is defined by the test and not by this spec.

## 12. Open questions

1. **Detached jobs.** No `'static` spawn, task handle or future exists in `spec_version` 1,
   which keeps INV-1 simple. Asset loading (`liminal_assets`, M4) needs background work that
   outlives a frame. Resolve in the `liminal_assets` spec (M4 kickoff), with a `spec_version`
   bump here.
2. **Per-worker data.** A stable worker index (for per-worker scratch buffers or command
   buffers) is not exposed, because results that depend on it are not deterministic. Resolve in
   the `liminal_ecs` spec (M1-T037): if the parallel executor needs it, add it here with a rule
   for its use.
3. **Queue capacity and `MAX_THREADS`.** `DEFAULT_QUEUE_CAPACITY` = 1,024 (8 KiB of pointers per
   worker) and `MAX_THREADS` = 256 are assumptions, not measurements. A full deque is handled
   (JOBS-R043), so the capacity affects speed only. Resolve: M1-T036 compares 256, 1,024 and
   4,096 in the `join` tree and `parallel_for` cases and records the result; change the default
   only with numbers.
4. **Performance and efficiency cores, affinity, priority.** The default thread count uses every
   logical core (18 on the reference machine), with no affinity or quality-of-service class.
   Whether the 12 efficiency cores help or hurt frame-critical work is unmeasured. Resolve:
   read from the "all" versus 4-thread cases of M1-T036; decide at the `stable` promotion.
5. **Stack depth under helping.** A waiting thread runs other jobs on top of its stack (§4), so
   deep nesting of pool calls can nest stacks further than the caller's own recursion. Workers
   use the default `std` stack size (2 MiB; unverified on Windows). No bound is specified.
   Resolve: if `headless_boids` (M1-T050) or an ECS test overflows a stack, add a stack-size
   field to `PoolConfig` or restrict what a waiting thread may run; otherwise at the `stable`
   promotion.
6. **Counters.** No counters (jobs run, steals, parks) are recorded, since one record per job
   would fill the telemetry buffers (16,384 records per thread and frame). Per-frame totals need
   someone to flush them once per frame. Resolve in the `liminal_app` spec (M1-T044).
7. **Cancellation after a panic.** All jobs run after a panic (except dependents in a graph),
   which makes the outcome deterministic but wastes work on a failing frame. Resolve at the
   `stable` promotion, from use in the ECS executor (M1-T042).
8. **Automatic chunk size.** Not provided: a chunk size derived from `threads()` would make
   chunk boundaries, and so `parallel_map_fold` results, depend on the machine. The
   `liminal_ecs` spec (M1-T037) chooses its chunk sizes from data sizes.
9. **loom and Miri in CI.** How loom tests are run (`--cfg loom`), the pinned nightly, and how a
   test excluded from Miri is marked are decided by M1-T018. This spec assumes
   `cfg(loom)` and `cfg(miri)` are available to the tests; if M1-T018 chooses otherwise, §10 is
   updated with a `spec_version` bump.
10. **Allocation in `run_graph`.** JOBS-R087 is SHOULD because no user needs it yet. If the ECS
    parallel executor (M1-T042) runs one graph per schedule per tick, it becomes MUST (CODE_STYLE.md
    "No allocation in per-frame hot paths"). Resolve in the `liminal_ecs` spec (M1-T037).
11. **Graph node results and spans with fields.** Nodes return `()` and pass data through captured
    state; node spans carry only the name (liminal_telemetry.md open question 4). Resolve with
    the `liminal_ecs` spec (M1-T037).
12. **`NodeId` of another graph.** Not detected when the index exists (§5.4). Embedding a graph
    identity in `NodeId` would detect it at the cost of a process-wide counter. Resolve at the
    `stable` promotion.

## 13. Maturity

Current: `planned`. Next target: `usable` (via `experimental`). Evidence is prepared by M1-T035
and M1-T036 and checked by the session that reviews M1-T036 (MATURITY.md "Promotion procedure").

- [ ] Spec follows `docs/specs/_TEMPLATE.md` and passed a spec review (REVIEW.md §2) (evidence:
      review of M1-T034).
- [ ] Compiles, all lints pass, registered in `ci/coverage.toml` (evidence: M1-T035).
- [ ] All MUST requirements traced: `cargo xtask trace` with `maturity: usable` applied on a
      scratch copy reports every MUST tested (evidence: M1-T036).
- [ ] Line coverage ≥ 85 % (layer 1 pure logic, TESTING.md) (evidence: M1-T036).
- [ ] Every public item documented; the primary entry points of §10 have runnable `# Examples`.
- [ ] Integration tests in `crates/liminal_jobs/tests/` use only the public API.
- [ ] `unsafe = "allow"`: Miri passes on the test suite, exceptions listed in §10 with reasons;
      every `unsafe` block has a `SAFETY:` comment a reviewer has checked; loom tests of the
      deque and of the park and wake protocol pass (ROADMAP M1 exit criterion).
- [ ] The benchmark pair of §9 rows 1 and 2 is recorded (ADR-0018), and the scaling cases at 1,
      2, 4 and all threads are recorded with `cargo xtask bench` (evidence: M1-T036).
- [ ] `tests/determinism.rs` (JOBS-R111) passes on the three CI operating systems. (The crate is
      `deterministic = false`; the MATURITY.md gate "determinism tests in place" applies to what
      §11 guarantees.)
- [ ] At least one example or dependent crate uses it. Open item: `liminal_ecs` (M1-T042) is the
      first dependent; M1-T036 either adds a runnable example under
      `crates/liminal_jobs/examples/` or the promotion waits for M1-T042.

## 14. History

| spec_version | Date | Change | By task |
|---|---|---|---|
| 1 | 2026-11-10 | Initial spec | M1-T034 |
