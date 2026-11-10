---
crate: liminal_ecs
req_prefix: ECS
layer: 1
maturity: planned
spec_version: 1
last_reviewed: 2026-11-10
adrs: [ADR-0008, ADR-0009, ADR-0010, ADR-0016, ADR-0017, ADR-0018, ADR-0023, ADR-0024]
---

# Spec: `liminal_ecs`

## 1. Purpose

`liminal_ecs` is the data model of every Liminal game and engine subsystem (ADR-0008): a `World`
of entities with components stored in archetype tables, resources, buffered events, queries with
filters, deferred commands, change detection, and schedules that run systems (Rust functions whose
parameters declare their data access) on one thread or in parallel on `liminal_jobs`. It also
computes `World::state_hash()`, the hash of all reflected simulation state that determinism and
replay tests compare (ADR-0009 item 6). It is a separate crate at layer 1 so that every runtime
crate and subsystem communicates through one data model (ARCHITECTURE.md §1), and so that the
`unsafe` code needed for shared access to storage stays in one place (ADR-0018).

## 2. Scope

**In scope:**
- `entity`: `Entity`, a generational ID with the allocation rules of
  `liminal_core::handle::HandleAllocator`.
- `component`: the `Component` and `Bundle` traits, registration through `liminal_reflect`, the
  change-detecting reference `Mut`, `ComponentTicks`.
- `world`: `World`: archetype tables, spawn and despawn, component insert and remove (archetype
  moves), direct access, resources, events, change ticks, `World::state_hash()`.
- `query`: `Query` with data (`&T`, `&mut T`, `Option`, `Entity`, tuples) and filters (`With`,
  `Without`, `Added`, `Changed`, tuples), with a defined iteration order.
- `command`: `CommandBuffer`, deferred structural changes applied in a defined order.
- `system`: function systems and their parameters (`Query`, `Res`, `ResMut`, `EventReader`,
  `EventWriter`, `Commands`), and the access each parameter declares.
- `schedule`: `ScheduleBuilder` and `Schedule`: named stages, explicit ordering constraints,
  conflict detection at build time, the single-threaded executor (`Schedule::run`) and the
  parallel executor (`Schedule::run_parallel`).
- `reference`: the naive reference world (a map from entity to component set) that differential
  tests compare the archetype world against (AGENTS.md rule 4).
- The telemetry the crate records, and the crate's error enum `EcsError` (codes `LIM-ECS-NNNN`,
  §8).

**Out of scope / non-goals:**
- Owning schedules, deciding which schedules exist, the fixed-step loop, calling
  `World::update_events`, owning the thread pool, modules: `liminal_app` (M1-T044).
- Hashing one reflected value, type descriptors, the derive macro: `liminal_reflect`,
  `liminal_reflect_derive`. This crate decides which values are hashed and in which order.
- Running closures on threads: `liminal_jobs`. This crate decides what may run at the same time.
- `assert_deterministic`, replay, recorded inputs: `liminal_testkit` (ADR-0024 item 4).
- Serialization of worlds, scenes and prefabs: `liminal_serial` (M2), `liminal_scene` (M4).
- Entity hierarchies and transforms: `liminal_scene` (M4).
- Access to components by `StableTypeId` at runtime (inspect and edit by the probe):
  `liminal_probe` (M2), with a `spec_version` bump here (§12 item 14).
- Not in `spec_version` 1, each with an open question in §12: systems with exclusive `&mut World`
  access, run conditions and system sets (item 5); an `Entity` returned by `Commands::spawn`
  (item 4); parallel iteration inside one system (item 7); sparse-set storage (item 10);
  components without reflection (item 3); change detection for resources and detection of
  removed components (item 13); resources that are not `Send + Sync` (item 15).
- A derive macro for `Component`, `Resource` or `Event`. Each is a marker trait implemented in
  one line.
- Comparison with `bevy_ecs` and `hecs`: the competitive benchmark task (M1-T052).
- `no_std`. 32-bit targets (ADR-0005).

## 3. Dependencies

- Engine crates (`architecture.toml`: layer 1, `same_layer_deps = ["liminal_jobs"]`; every layer 0
  crate is allowed):
  - `liminal_core`: `handle::{HandleAllocator, RawHandle}` (entity allocation), `hash::{StateHasher,
    StateHash, FixedState}`, `error::{EngineError, ErrorCode, ErrorContext, write_display}`
    ([liminal_core.md](liminal_core.md)).
  - `liminal_reflect`: `value::{Reflect, Typed}`, `info::{StableTypeId, TypeInfo}`,
    `registry::TypeRegistry`, `hash::write_reflect` ([liminal_reflect.md](liminal_reflect.md)).
  - `liminal_jobs`: `pool::ThreadPool`, `graph::JobGraph` (the parallel executor,
    [liminal_jobs.md](liminal_jobs.md)).
  - `liminal_telemetry`: the `span!` and `counter!` macros
    ([liminal_telemetry.md](liminal_telemetry.md)). `liminal_ecs` depends on it without its
    `telemetry` feature; its dev-dependency enables the feature so that the tests of ECS-R210 and
    ECS-R211 can read records.
  - Not used: `liminal_math` (no math type appears in this crate's API or tests) and
    `liminal_memory` (the crate owns its storage, [liminal_memory.md](liminal_memory.md) §2).
- External crates: none at runtime (`external = []`). Dev-dependencies `proptest` and `loom`
  (`dev_external`, ADR-0010, ADR-0018). `loom` is used only if an implementation task adds
  shared state of its own (§10).
- `unsafe = "allow"` (ADR-0018). `unsafe` is allowed in these modules only:

  | Module | What needs `unsafe` |
  |---|---|
  | private `cell` | The interior-mutability wrapper around a world's storage and its `Send`/`Sync` impls, so that several systems can hold access to one `World` |
  | `query` | Handing out `&T` and `&mut T` into columns from shared world access; several mutable borrows of different columns at once |
  | `system` | Building system parameters from shared world access; calling type-erased systems |
  | `schedule` | Sharing the world and the systems with jobs of `liminal_jobs` in `run_parallel` |

  The modules `entity`, `component`, `world`, `command`, `reference`, `error` and the private
  storage module (archetypes and tables) declare `#![forbid(unsafe_code)]` at their top. Columns
  are typed (`Vec<T>` behind a private object-safe trait), not byte blobs (§12 item 11). A task
  that needs `unsafe` in another module changes this list with a `spec_version` bump. Every
  `unsafe` block has a `// SAFETY:` comment; the test suite runs under Miri (§10).
- ADR-0018 "unsafe for performance requires a benchmark": shared access by systems could be
  written in safe Rust with one lock per column and per resource. The benchmark pair that
  justifies not doing so is §9 rows 2 and 3 (archetype world against `reference::World`, which
  contains no `unsafe`), recorded by M1-T043. If the archetype world is not measurably faster
  there, that is reported as a finding against this spec.
- `deterministic = true`: the determinism lint configuration of CODE_STYLE.md applies (the
  crate's own `clippy.toml`, written by M1-T038; XTASK-R039), and the crate meets §11.
- Cargo features: none.
- Structure: public modules `entity`, `component`, `world`, `query`, `command`, `system`,
  `schedule`, `reference`, `error`. Every public item is public at exactly one path; the crate
  root declares the modules and re-exports nothing (CODE_STYLE.md "One obvious way").

## 4. Concepts

- **Entity:** an ID naming one row of data in one world: a slot index (`u32`) and a generation
  (`u32`, at least 1), issued by a `liminal_core::handle::HandleAllocator` (CORE-R033 to
  CORE-R037: a new world issues `0v1`, `1v1`, …; the most recently freed slot is reused first,
  with its generation increased by 1; a slot whose generation would pass `u32::MAX` is retired).
  An entity is *live* from the `spawn` that returned it until its `despawn`. This resolves
  [liminal_core.md](liminal_core.md) §12 item 5: the ECS uses the core allocator unchanged.
- **Component:** a value of a type implementing `Component`, attached to one entity. An entity
  has at most one component of each type. Every component type is reflected
  (`liminal_reflect::value::Typed`) and must be registered in the world before use (ADR-0008).
- **Bundle:** the argument of `spawn` and `insert`: `()`, one component, or a tuple of 1 to 8
  components of distinct types.
- **Archetype, table:** an archetype is a set of component types. Each archetype that has been
  used has one table: one column per component type (structure of arrays), one row per entity
  with exactly that set. Archetype 0 is the empty set and exists from `World::new`. Every other
  archetype is created, with the next archetype number, the first time an entity is to be
  stored in it. Archetypes are never removed. Changing an entity's component set moves its row
  to the table of the new archetype (an *archetype move*).

  ```
  archetype 1 {Position, Velocity}      archetype 2 {Position}
  row  entity  Position  Velocity       row  entity  Position
  0    0v1     p0        v0             0    2v1     p2
  1    1v1     p1        v1
  remove::<Velocity>(0v1): row 0 leaves table 1 (its last row, 1v1, moves into row 0)
                           and is appended to table 2 as row 1.
  ```
- **Row order:** a row entering a table is appended after its last row. When a row leaves a
  table (despawn or archetype move), the table's last row is moved into the vacated position
  (if the leaving row was the last row, nothing moves). Row order is therefore a function of the
  sequence of operations on the world, and of nothing else.
- **Query:** a view of the world chosen by two type parameters: *data* `D` (what each item
  contains) and *filter* `F` (which entities are yielded). §5.4 lists the forms. A query has:
  - a *required* set: the component types of `&T`, `&mut T`, `With<T>`, `Added<T>`, `Changed<T>`;
  - an *excluded* set: the types of `Without<T>`;
  - a *read* set: the types of `&T`, `Option<&T>`, `Added<T>`, `Changed<T>`;
  - a *write* set: the types of `&mut T`, `Option<&mut T>`.

  A query *matches* an archetype that contains every required type and no excluded type.
  Two queries are *disjoint* if a type is required by one and excluded by the other (no archetype
  matches both).
- **Resource:** at most one value per type implementing `Resource`, stored in the world and not
  attached to an entity. A resource whose type is also `Typed` can be *registered*; only
  registered resources take part in the state hash.
- **Event:** a value of a type implementing `Event`, sent to the world's buffer for that type
  and read by any number of readers. Each event type has two buffers, *previous* and *current*.
  `send` appends to *current*. `World::update_events` drops *previous* and makes *current* the
  new *previous*, so an event stays readable until the second `update_events` after it was sent.
  Each event of one type has a sequence number: the count of events of that type sent before it.
- **Command, command buffer, sync point:** a command is a recorded structural change (spawn,
  despawn, insert, remove). A `CommandBuffer` holds commands in recording order; nothing changes
  until the buffer is applied. In a schedule, each system has its own buffer, and all buffers of
  a stage are applied at the stage's *sync point*, after every system of the stage has returned
  (ADR-0008, ADR-0009 item 4).
- **Change tick:** each world has a change tick, a `u64` starting at 1 that never decreases.
  Every component instance stores two ticks: *added* (when it was attached to its entity) and
  *changed* (when it was last attached, replaced or mutably dereferenced). A tick is 8 bytes, so
  change detection costs 16 bytes per component instance (§12 item 12); a `u64` does not wrap
  in practice (2^64 ticks at 10^6 per second is 5.8 × 10^5 years), so there is no wrap handling.
- **System:** a function or closure whose parameters all implement `SystemParam`, added to a
  schedule under a unique name. Its *access* is the list of its parameters' accesses:

  | Parameter | Access |
  |---|---|
  | `Query<D, F>` | The query's read, write, required and excluded sets |
  | `Res<R>` | Reads resource `R` |
  | `ResMut<R>` | Writes resource `R` |
  | `EventReader<E>` | Reads the buffers of `E` (its position in them is private to the parameter) |
  | `EventWriter<E>` | Writes the buffers of `E` |
  | `Commands` | None (each system records into its own buffer) |

- **Conflict:** two accesses conflict if they are
  - two queries that are not disjoint, where a type in the write set of one is in the read or
    write set of the other; or
  - `ResMut<R>` and `Res<R>` or `ResMut<R>` of the same `R`; or
  - `EventWriter<E>` and `EventReader<E>` or `EventWriter<E>` of the same `E`.

  Two systems conflict if any access of one conflicts with any access of the other. Two
  parameters of one system must not conflict (ECS-R151). Inside one query's data, a type must not
  appear twice if one of the appearances is mutable (ECS-R065).
- **Schedule, stage, order:** a schedule has a name, an ordered list of named stages, and
  systems, each in one stage. Stages run in the order they were added. An *order* `(first, then)`
  between two systems of one stage makes `then` start only after `first` has returned. Orders
  form a directed acyclic graph per stage. Two conflicting systems of one stage need an order
  path between them (directly or through other systems); without one, building the schedule is
  an error, never a guess (ADR-0008). Systems of different stages never run at the same time.
- **Execution order:** the order in which `Schedule::run` calls systems: stage by stage; inside
  a stage, repeatedly the system with the lowest `SystemId` index that has not run and whose
  `first` systems have all run; after the stage's last system, the stage's sync point.
- **Run tick:** during a schedule run that starts with world change tick `T`, the k-th position
  of the execution order (k from 0, counting every system and every sync point as one position)
  has run tick `T + 1 + k`. A system's mutable dereferences stamp its run tick; a sync point
  stamps its run tick on everything its commands attach or replace. After the run, the world's
  change tick is `T + 1 + n`, where `n` is the number of positions. A system's *baseline* is its
  run tick in its previous run of this schedule, or 0 if it has not run. `Added<T>` and
  `Changed<T>` compare a component's ticks with the baseline.

  ```
  stage "update":  a (index 0)   b (index 1)   order (b, a)
  stage "late":    c (index 2)
  execution order: b, a, sync(update), c, sync(late)
  run ticks, T=7:  8  9  10            11 12          world change tick after the run: 13
  ```

  Because run ticks follow the execution order, a write is seen by each `Changed` filter exactly
  once: a system ordered after the writer sees it in the same run, a system ordered before it
  sees it in the next run.

## 5. Public API

Signatures are normative; bodies and private items are not. Derives listed are part of the API.
A trait marked *sealed* has a private supertrait: it cannot be implemented outside this crate,
and its methods are not part of the public API. The signatures of §5.1 to §5.7 were type-checked
with stub bodies and a sample of user code (rustc 1.99.1, M1-T037 handoff); no implementation
exists yet.

### 5.1 `liminal_ecs::entity`

```rust
/// Generational entity ID. 8 bytes, Copy. Stores `generation × 2^32 + index`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Entity { /* bits: u64 */ }

impl Entity {
    pub fn index(self) -> u32;
    pub fn generation(self) -> core::num::NonZeroU32;
    /// `generation × 2^32 + index` (the `to_bits` of the underlying `RawHandle`).
    pub fn to_bits(self) -> u64;
    /// `None` if the upper 32 bits (the generation) are zero. Does not check liveness.
    pub fn from_bits(bits: u64) -> Option<Entity>;
}
impl PartialOrd for Entity { /* by index, then generation */ }
impl Ord for Entity { /* by index, then generation */ }
impl core::fmt::Display for Entity { /* `{index}v{generation}`, e.g. `3v2` */ }
impl liminal_core::hash::StateHash for Entity { /* `to_bits()` as u64 */ }
/// Type path `liminal_ecs::entity::Entity`, shape `Primitive(PrimitiveKind::U64)` (ECS-R004).
impl liminal_reflect::value::Reflect for Entity { /* … */ }
impl liminal_reflect::value::Typed for Entity { /* … */ }
```

`Entity` is reflected so that components and events with `Entity` fields can derive `Reflect`.
Its reflected form is the one `u64` it stores. Reflection cannot write it field by field: a
primitive is written through `downcast_mut::<T>()`, which succeeds only for `T = Entity`.

### 5.2 `liminal_ecs::component`

```rust
/// Marker for component types. Implemented by hand: `impl Component for Position {}`.
pub trait Component: liminal_reflect::value::Typed + Send + Sync {}

/// Largest number of components in one tuple bundle.
pub const MAX_BUNDLE_COMPONENTS: usize = 8;

/// Sealed. Implemented for `()`, for every `T: Component`, and for tuples `(T0,)` … of 1 to
/// MAX_BUNDLE_COMPONENTS components. Tuples do not nest.
pub trait Bundle: Send + 'static { /* sealed */ }

/// Mutable access to one component that records the change: `DerefMut` sets the component's
/// changed tick (ECS-R121). `Deref` does not.
pub struct Mut<'w, T: Component> { /* … */ }
impl<T: Component> core::ops::Deref for Mut<'_, T> { type Target = T; /* … */ }
impl<T: Component> core::ops::DerefMut for Mut<'_, T> { /* … */ }

/// The two change ticks of one component instance (§4 "Change tick").
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ComponentTicks {
    pub added: u64,
    pub changed: u64,
}
```

### 5.3 `liminal_ecs::world`

```rust
/// Marker for resource types. Implemented by hand: `impl Resource for Gravity {}`.
pub trait Resource: Send + Sync + 'static {}

/// Marker for event types. Implemented by hand: `impl Event for Collision {}`.
pub trait Event: liminal_reflect::value::Typed + Send + Sync {}

/// Entities, components, resources and events. Send + Sync; not Clone.
pub struct World { /* … */ }
impl Default for World { /* new() */ }

impl World {
    /// An empty world with entity limit `u32::MAX` and change tick 1.
    pub fn new() -> World;
    /// At most `entity_limit` entity slots (indices 0..entity_limit) are ever used.
    pub fn with_entity_limit(entity_limit: u32) -> World;

    // --- registration ---
    /// Idempotent. Errors: LIM-ECS-0006.
    pub fn register_component<T: Component>(&mut self) -> Result<(), EcsError>;
    /// Makes resource type `R` part of the state hash. Idempotent; independent of whether a
    /// value is present. Errors: LIM-ECS-0006.
    pub fn register_resource<R: Resource + liminal_reflect::value::Typed>(&mut self) -> Result<(), EcsError>;
    /// Creates the buffers of `E`. Idempotent. Errors: LIM-ECS-0006.
    pub fn register_event<E: Event>(&mut self) -> Result<(), EcsError>;
    /// Descriptors of every registered component, resource and event type.
    pub fn type_registry(&self) -> &liminal_reflect::registry::TypeRegistry;

    // --- entities and components ---
    /// Errors: LIM-ECS-0003, 0004, 0002.
    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> Result<Entity, EcsError>;
    /// Errors: LIM-ECS-0001.
    pub fn despawn(&mut self, entity: Entity) -> Result<(), EcsError>;
    pub fn contains(&self, entity: Entity) -> bool;
    pub fn entity_count(&self) -> u32;
    /// Live entities in ascending index order.
    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_;
    /// Adds the bundle's components; replaces the value of types the entity already has.
    /// Errors: LIM-ECS-0003, 0004, 0001.
    pub fn insert<B: Bundle>(&mut self, entity: Entity, bundle: B) -> Result<(), EcsError>;
    /// `Ok(None)` if the entity has no `T`. Errors: LIM-ECS-0003, 0001.
    pub fn remove<T: Component>(&mut self, entity: Entity) -> Result<Option<T>, EcsError>;
    /// Errors: LIM-ECS-0003, 0001, 0005.
    pub fn get<T: Component>(&self, entity: Entity) -> Result<&T, EcsError>;
    /// Errors: LIM-ECS-0003, 0001, 0005.
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Result<Mut<'_, T>, EcsError>;
    /// Errors: LIM-ECS-0003, 0001.
    pub fn has<T: Component>(&self, entity: Entity) -> Result<bool, EcsError>;
    /// A query outside a schedule; baseline 0, stamps the world's change tick.
    /// Errors: LIM-ECS-0003, 0020.
    pub fn query<D: QueryData, F: QueryFilter>(&mut self) -> Result<Query<'_, D, F>, EcsError>;

    // --- resources ---
    /// Returns the value it replaced, if any.
    pub fn insert_resource<R: Resource>(&mut self, value: R) -> Option<R>;
    pub fn remove_resource<R: Resource>(&mut self) -> Option<R>;
    /// Errors: LIM-ECS-0010.
    pub fn resource<R: Resource>(&self) -> Result<&R, EcsError>;
    /// Errors: LIM-ECS-0010.
    pub fn resource_mut<R: Resource>(&mut self) -> Result<&mut R, EcsError>;

    // --- events ---
    /// Errors: LIM-ECS-0011.
    pub fn send_event<E: Event>(&mut self, event: E) -> Result<(), EcsError>;
    /// Every buffered event of `E`, oldest first. Errors: LIM-ECS-0011.
    pub fn buffered_events<E: Event>(&self) -> Result<impl Iterator<Item = &E> + '_, EcsError>;
    /// For every event type: drops the previous buffer; the current buffer becomes the previous.
    pub fn update_events(&mut self);

    // --- commands ---
    /// Applies the commands in recording order and empties the buffer. Errors: LIM-ECS-0040
    /// (after every other command was applied).
    pub fn apply_commands(&mut self, buffer: &mut CommandBuffer) -> Result<(), EcsError>;

    // --- change ticks ---
    pub fn change_tick(&self) -> u64;
    /// Adds 1 and returns the new value.
    pub fn advance_change_tick(&mut self) -> u64;
    /// Errors: LIM-ECS-0003, 0001, 0005.
    pub fn component_ticks<T: Component>(&self, entity: Entity) -> Result<ComponentTicks, EcsError>;

    // --- state hash ---
    /// Hash of all reflected simulation state in canonical order (ECS-R190).
    pub fn state_hash(&self) -> u64;
}
```

`QueryData`, `QueryFilter` and `Query` are items of `liminal_ecs::query`, `CommandBuffer` of
`liminal_ecs::command`, `EcsError` of `liminal_ecs::error`.

### 5.4 `liminal_ecs::query`

```rust
/// Largest tuple of query data and of query filters.
pub const MAX_QUERY_TUPLE: usize = 8;

/// Sealed. What a query yields per entity. Implemented for:
///   Entity          item Entity
///   &T              item &T
///   &mut T          item Mut<T>
///   Option<&T>      item Option<&T>        (None where the entity has no T)
///   Option<&mut T>  item Option<Mut<T>>
///   tuples of 1 to MAX_QUERY_TUPLE of these; item: the tuple of their items
/// for T: Component.
pub trait QueryData { type Item<'a>; /* sealed */ }

/// Sealed marker: query data without `&mut`. Implemented for Entity, &T, Option<&T> and tuples
/// of them.
pub trait ReadOnlyQueryData: QueryData {}

/// Sealed. Which entities a query yields. Implemented for `()` (all), `With<T>`, `Without<T>`,
/// `Added<T>`, `Changed<T>`, and tuples of 1 to MAX_QUERY_TUPLE filters (all must hold).
pub trait QueryFilter { /* sealed */ }

/// The entity has a `T`. Does not access the value.
pub struct With<T: Component> { /* PhantomData */ }
/// The entity has no `T`.
pub struct Without<T: Component> { /* PhantomData */ }
/// The entity has a `T` whose added tick is greater than the query's baseline.
pub struct Added<T: Component> { /* PhantomData */ }
/// The entity has a `T` whose changed tick is greater than the query's baseline.
pub struct Changed<T: Component> { /* PhantomData */ }

/// A view of the entities matched by `D` and `F`. Created by `World::query`, or passed to a
/// system as a parameter.
pub struct Query<'w, D: QueryData, F: QueryFilter = ()> { /* … */ }

impl<'w, D: QueryData, F: QueryFilter> Query<'w, D, F> {
    /// Items in iteration order (ECS-R062).
    pub fn iter(&self) -> QueryIter<'_, D, F> where D: ReadOnlyQueryData;
    pub fn iter_mut(&mut self) -> QueryIter<'_, D, F>;
    /// Errors: LIM-ECS-0001, 0021.
    pub fn get(&self, entity: Entity) -> Result<D::Item<'_>, EcsError> where D: ReadOnlyQueryData;
    /// Errors: LIM-ECS-0001, 0021.
    pub fn get_mut(&mut self, entity: Entity) -> Result<D::Item<'_>, EcsError>;
    /// Number of items `iter_mut` would yield.
    pub fn count(&self) -> usize;
}

pub struct QueryIter<'q, D: QueryData, F: QueryFilter> { /* … */ }
impl<'q, D: QueryData, F: QueryFilter> Iterator for QueryIter<'q, D, F> {
    type Item = D::Item<'q>;
    /* … */
}
```

### 5.5 `liminal_ecs::command`

```rust
/// Recorded structural changes, in recording order. Send.
pub struct CommandBuffer { /* … */ }
impl Default for CommandBuffer { /* new() */ }

impl CommandBuffer {
    pub fn new() -> CommandBuffer;
    /// Records "spawn an entity with this bundle". No entity ID is returned (§12 item 4).
    pub fn spawn<B: Bundle>(&mut self, bundle: B);
    pub fn despawn(&mut self, entity: Entity);
    pub fn insert<B: Bundle>(&mut self, entity: Entity, bundle: B);
    pub fn remove<T: Component>(&mut self, entity: Entity);
    /// Number of recorded commands not yet applied.
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

### 5.6 `liminal_ecs::system`

```rust
/// Largest number of parameters of a system function.
pub const MAX_SYSTEM_PARAMS: usize = 8;

/// Sealed. Implemented for `Query<'_, D, F>` (D and F `'static`), `Res<'_, R>`, `ResMut<'_, R>`,
/// `EventReader<'_, E>`, `EventWriter<'_, E>` and `Commands<'_>`.
pub trait SystemParam { /* sealed */ }

/// Sealed. Implemented for every `F: FnMut(P1, …, Pn) + Send + 'static` with 0 to
/// MAX_SYSTEM_PARAMS parameters that all implement `SystemParam`. `Marker` is inferred; callers
/// never name it.
pub trait IntoSystem<Marker>: Send + 'static { /* sealed */ }

/// Shared access to resource `R`.
pub struct Res<'w, R: Resource> { /* … */ }
impl<R: Resource> core::ops::Deref for Res<'_, R> { type Target = R; /* … */ }

/// Exclusive access to resource `R`.
pub struct ResMut<'w, R: Resource> { /* … */ }
impl<R: Resource> core::ops::Deref for ResMut<'_, R> { type Target = R; /* … */ }
impl<R: Resource> core::ops::DerefMut for ResMut<'_, R> { /* … */ }

/// Reads events of `E`. Each `EventReader` parameter remembers which events it has yielded.
pub struct EventReader<'w, E: Event> { /* … */ }
impl<E: Event> EventReader<'_, E> {
    /// The buffered events this parameter has not yielded yet, oldest first; marks them yielded.
    pub fn read(&mut self) -> impl Iterator<Item = &E> + '_;
}

/// Sends events of `E`.
pub struct EventWriter<'w, E: Event> { /* … */ }
impl<E: Event> EventWriter<'_, E> {
    pub fn send(&mut self, event: E);
}

/// The system's own command buffer, applied at the stage's sync point. Record with the methods
/// of `CommandBuffer`: `commands.spawn(…)`, `commands.despawn(entity)`.
pub struct Commands<'w> { /* … */ }
impl core::ops::Deref for Commands<'_> { type Target = CommandBuffer; /* … */ }
impl core::ops::DerefMut for Commands<'_> { /* … */ }
```

### 5.7 `liminal_ecs::schedule`

```rust
/// Identifies a system of the `ScheduleBuilder` whose `add_system` returned it.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SystemId { /* index */ }
impl SystemId {
    /// 0 for the first system added to its builder, 1 for the next, and so on (over all stages).
    pub fn index(self) -> usize;
}

/// Collects stages, systems and orders. Validates what it can at each call.
pub struct ScheduleBuilder { /* … */ }

impl ScheduleBuilder {
    /// `name` is used in errors and as the telemetry span name of a run (ECS-R210).
    pub fn new(name: &'static str) -> ScheduleBuilder;
    /// Appends a stage. Errors: LIM-ECS-0030.
    pub fn add_stage(&mut self, stage: &'static str) -> Result<(), EcsError>;
    /// Adds `system` to `stage` under `name` (unique in the schedule; used in errors and as its
    /// telemetry span name). Errors: LIM-ECS-0031, 0032, 0020, 0036.
    pub fn add_system<Marker, S: IntoSystem<Marker>>(
        &mut self,
        stage: &str,
        name: &'static str,
        system: S,
    ) -> Result<SystemId, EcsError>;
    /// Makes `then` start only after `first` has returned. Both must be in one stage.
    /// Errors: LIM-ECS-0033, 0034, 0035.
    pub fn add_order(&mut self, first: SystemId, then: SystemId) -> Result<(), EcsError>;
    /// Checks registrations and conflicts and binds the schedule to `world`.
    /// Errors: LIM-ECS-0003, 0011, 0037.
    pub fn build(self, world: &World) -> Result<Schedule, EcsError>;
}

/// A validated schedule, bound to the world it was built for. Send.
pub struct Schedule { /* … */ }

impl Schedule {
    pub fn name(&self) -> &'static str;
    pub fn system_count(&self) -> usize;
    /// Runs every system once, on the calling thread, in the execution order (§4).
    /// Errors: LIM-ECS-0038, 0039 (nothing ran); LIM-ECS-0041 (the run completed).
    pub fn run(&mut self, world: &mut World) -> Result<(), EcsError>;
    /// Like `run`, with systems that have no order path between them running at the same time
    /// on `pool`. Same errors, same resulting world (ECS-R180).
    pub fn run_parallel(
        &mut self,
        world: &mut World,
        pool: &liminal_jobs::pool::ThreadPool,
    ) -> Result<(), EcsError>;
}
```

A `SystemId` carries only an index. Using a `SystemId` of one builder with another builder is not
detected if the index exists there; the docs of `SystemId` say so.

Example of intended use (normative for the behaviour it shows, not for formatting; `Position`,
`Velocity` and `Frozen` are components, `Gravity` is a resource, all registered or inserted):

```rust
fn integrate(mut bodies: Query<(&mut Position, &Velocity), Without<Frozen>>, gravity: Res<Gravity>) {
    for (mut position, velocity) in bodies.iter_mut() {
        position.y += velocity.y + gravity.per_tick;
    }
}

fn remove_fallen(bodies: Query<(Entity, &Position)>, mut commands: Commands) {
    for (entity, position) in bodies.iter() {
        if position.y < 0.0 {
            commands.despawn(entity);
        }
    }
}

let mut builder = ScheduleBuilder::new("fixed_update");
builder.add_stage("update")?;
let integrate_id = builder.add_system("update", "integrate", integrate)?;
let remove_id = builder.add_system("update", "remove_fallen", remove_fallen)?;
// Both systems access `Position`, one of them mutably: without this line `build` returns
// LIM-ECS-0037.
builder.add_order(integrate_id, remove_id)?;
let mut schedule = builder.build(&world)?;
schedule.run(&mut world)?;
```

### 5.8 `liminal_ecs::reference`

The correctness oracle of the archetype world (AGENTS.md rule 4; PERFORMANCE.md "Reference
implementations reachable from benchmarks", items 1 to 4): a world that stores
`BTreeMap<Entity, BTreeMap<StableTypeId, component>>`, with no archetypes, no tables and no
`unsafe`. It is not the fast path, and nothing else in the crate uses it. Its items have the
names and signatures of their counterparts in `liminal_ecs::world` and take the same `Entity`,
`Bundle`, `Component`, `Mut`, `ComponentTicks` and `EcsError` types.

```rust
/// Naive reference world. Same behaviour as `liminal_ecs::world::World` for the methods below.
pub struct World { /* … */ }
impl Default for World { /* new() */ }

impl World {
    pub fn new() -> World;
    pub fn with_entity_limit(entity_limit: u32) -> World;
    pub fn register_component<T: Component>(&mut self) -> Result<(), EcsError>;
    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> Result<Entity, EcsError>;
    pub fn despawn(&mut self, entity: Entity) -> Result<(), EcsError>;
    pub fn contains(&self, entity: Entity) -> bool;
    pub fn entity_count(&self) -> u32;
    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_;
    pub fn insert<B: Bundle>(&mut self, entity: Entity, bundle: B) -> Result<(), EcsError>;
    pub fn remove<T: Component>(&mut self, entity: Entity) -> Result<Option<T>, EcsError>;
    pub fn get<T: Component>(&self, entity: Entity) -> Result<&T, EcsError>;
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Result<Mut<'_, T>, EcsError>;
    pub fn has<T: Component>(&self, entity: Entity) -> Result<bool, EcsError>;
    pub fn change_tick(&self) -> u64;
    pub fn advance_change_tick(&mut self) -> u64;
    pub fn component_ticks<T: Component>(&self, entity: Entity) -> Result<ComponentTicks, EcsError>;
    pub fn state_hash(&self) -> u64;
}
```

The reference world has no queries, resources, events, commands or schedules. Query results are
checked against it by computing the expected items in the test from `entities`, `has` and `get`
(ECS-R073). The parallel executor's reference is the single-threaded executor (§10).

| Optimized path | Reference | Differential test |
|---|---|---|
| Archetype tables, archetype moves, the cache of moves between archetypes (`world::World`) | `reference::World` | `tests/world_differential.rs` (ECS-R201, ECS-R044) |
| Query matching and iteration over tables (`query::Query`) | Items computed from `reference::World` in the test | `tests/query_differential.rs` (ECS-R073) |
| `World::state_hash` (walks tables) | `reference::World::state_hash` (walks the map) | `tests/state_hash.rs` (ECS-R194) |
| `Schedule::run_parallel` | `Schedule::run` | `tests/executor_equivalence.rs` (ECS-R180) |

### 5.9 `liminal_ecs::error`

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EcsError {
    NoSuchEntity { entity: Entity },                                               // LIM-ECS-0001
    EntityLimitReached { entity_limit: u32 },                                      // LIM-ECS-0002
    ComponentNotRegistered { component: String },                                  // LIM-ECS-0003
    DuplicateComponent { component: String },                                      // LIM-ECS-0004
    MissingComponent { entity: Entity, component: String },                        // LIM-ECS-0005
    TypeRegistrationFailed { type_path: String, cause_code: String, cause: String }, // LIM-ECS-0006
    MissingResource { resource: String },                                          // LIM-ECS-0010
    EventNotRegistered { event: String },                                          // LIM-ECS-0011
    QueryAccessConflict { component: String },                                     // LIM-ECS-0020
    QueryNoMatch { entity: Entity },                                               // LIM-ECS-0021
    DuplicateStage { schedule: &'static str, stage: &'static str },                // LIM-ECS-0030
    UnknownStage { schedule: &'static str, stage: String },                        // LIM-ECS-0031
    DuplicateSystemName { schedule: &'static str, system: &'static str },          // LIM-ECS-0032
    UnknownSystem { schedule: &'static str, system_index: usize, system_count: usize }, // LIM-ECS-0033
    OrderAcrossStages { schedule: &'static str, first: &'static str, first_stage: &'static str,
                        then: &'static str, then_stage: &'static str },            // LIM-ECS-0034
    OrderCycle { schedule: &'static str, first: &'static str, then: &'static str }, // LIM-ECS-0035
    SystemParamConflict { schedule: &'static str, system: &'static str,
                          access: &'static str, type_name: String },               // LIM-ECS-0036
    SystemAccessConflict { schedule: &'static str, stage: &'static str, first: &'static str,
                           second: &'static str, access: &'static str, type_name: String }, // LIM-ECS-0037
    WorldMismatch { schedule: &'static str },                                      // LIM-ECS-0038
    SystemResourceMissing { schedule: &'static str, system: &'static str, resource: String }, // LIM-ECS-0039
    CommandsRejected { rejected: u32, first_cause_code: String, first_cause: String }, // LIM-ECS-0040
    ScheduleCommandsRejected { schedule: &'static str, system: &'static str, rejected: u32,
                               first_cause_code: String, first_cause: String },    // LIM-ECS-0041
}
impl liminal_core::error::EngineError for EcsError { /* §8 */ }
impl std::error::Error for EcsError {}
impl core::fmt::Display for EcsError { /* liminal_core::error::write_display */ }
```

Names in error fields: a component or event type is named by its type path
(`Typed::type_path()`); a resource type by `core::any::type_name::<R>()`, because a resource
need not be reflected. `type_name` output is not guaranteed to be the same across compiler
versions; it is for reading, and no test compares it with a literal (tests compare with
`type_name` evaluated in the test). `access` is `"component"`, `"resource"` or `"event"`.

## 6. Requirements

"For thread counts 1, 2 and 4" in a test statement means: the test runs once with a
`liminal_jobs` pool of each of these `threads` values, on any machine. "In this order" in an
error statement means: the checks are made in the order written and the first failure is
returned. "The world is unchanged" means: `state_hash`, `entities`, the iteration order of every
query, every `component_ticks` value, `change_tick` and the buffered events are the same as
before the call. Tests use component types defined in the tests; their field types are
primitives, arrays and `Entity`.

### Entities and the world (R001–R019)

| ID | Level | Requirement |
|---|---|---|
| ECS-R001 | MUST | `World::new()` and `World::default()` return a world with `entity_count() == 0`, no registered type, no resource, `change_tick() == 1` and entity limit `u32::MAX`. `World::with_entity_limit(n)` is the same with entity limit `n`. `World` is `Send + Sync` (static assertion). |
| ECS-R002 | MUST | Entities are issued by the rules of `liminal_core::handle::HandleAllocator` (CORE-R033 to CORE-R035, CORE-R037): for every sequence of successful `spawn` and `despawn` calls, the entities returned by `spawn` equal (by `to_bits`) the handles returned by `allocate` on a `HandleAllocator::with_slot_limit` of the same limit for the same sequence of `allocate` and `free`. The first `spawn` of a new world returns `0v1`. Property test over generated sequences of up to 1,000 operations with limits 4 and `u32::MAX`. |
| ECS-R003 | MUST | `Entity`: `to_bits()` returns `generation × 2^32 + index`; `from_bits(b)` returns `None` if `b < 2^32`, else the entity with `index = b mod 2^32` and `generation = b / 2^32`; `from_bits(e.to_bits()) == Some(e)`; `index()` and `generation()` return the parts; `Display` writes `{index}v{generation}` in decimal; `Ord` orders by index, then by generation; `size_of::<Entity>()` is 8; `Entity` is `Copy`. `StateHash` for `Entity` writes `to_bits()` as `u64` (CORE-R053). |
| ECS-R004 | MUST | `Entity` implements `Reflect` and `Typed`: `type_path()` is `"liminal_ecs::entity::Entity"`; `type_info().shape` is `Shape::Primitive(PrimitiveKind::U64)`; `reflect_ref` returns `ReflectRef::Primitive(PrimitiveRef::U64(r))` with `*r == to_bits()`; `reflect_mut` returns `ReflectMut::Primitive(PrimitiveKind::U64)`. `liminal_reflect::value::validate_reflect(&e)` returns `Ok(())`, `TypeRegistry::register::<Entity>()` returns `Ok`, and `liminal_reflect::hash::reflect_state_hash(&e) == liminal_core::hash::state_hash_of(&e.to_bits())`. `downcast_mut::<u64>()` on an `Entity` returns `Err` (LIM-REFL-0011). |
| ECS-R005 | MUST | `spawn(bundle)` with every component type of the bundle registered and distinct returns `Ok(entity)`: `entity` is live (`contains` true), `entity_count()` is 1 higher, the entity has exactly the bundle's component types (`has::<T>()` true for each, false for every other registered type), and `get::<T>()` returns a value equal to the one passed. `spawn(())` gives a live entity with no components. `MAX_BUNDLE_COMPONENTS == 8`; `Bundle` is implemented for `()`, a single component and tuples of 1 to 8 components (a test spawns each arity). |
| ECS-R006 | MUST | `spawn(bundle)` checks, in this order: a component type of the bundle is not registered gives `Err(EcsError::ComponentNotRegistered)` (LIM-ECS-0003) naming the first such type in bundle order; a component type occurs twice in the bundle gives `Err(EcsError::DuplicateComponent)` (LIM-ECS-0004) naming the first type that repeats an earlier one; no entity slot is available gives `Err(EcsError::EntityLimitReached)` (LIM-ECS-0002) with the world's entity limit. On error the world is unchanged and every component of the bundle is dropped exactly once. |
| ECS-R007 | MUST | `despawn(entity)` of a live entity returns `Ok(())`: `contains(entity)` is false, `entity_count()` is 1 lower, and each of its components is dropped exactly once (counted by a drop counter in a test component). `despawn` of an entity that is not live (despawned, never issued by this world, or a stale generation) returns `Err(EcsError::NoSuchEntity)` (LIM-ECS-0001) and the world is unchanged. |
| ECS-R008 | MUST | `contains(e)` is true exactly for entities returned by `spawn` (or spawned by an applied command) and not yet despawned. `entity_count()` is their number. `entities()` yields each of them once, in ascending index order. |
| ECS-R009 | MUST | Dropping a `World` drops every component, every resource and every buffered event exactly once (drop counters; Miri reports no leak). |

### Components and registration (R020–R039)

| ID | Level | Requirement |
|---|---|---|
| ECS-R020 | MUST | `register_component::<T>()` registers `T` in the world's `TypeRegistry` (`TypeRegistry::register::<T>()`) and returns `Ok(())`; afterwards `type_registry().get(T::stable_type_id())` is `Some`. Registering the same type again returns `Ok(())` and the world is unchanged. If `TypeRegistry::register` returns an error `r`, the call returns `Err(EcsError::TypeRegistrationFailed)` (LIM-ECS-0006) with `type_path = T::type_path()`, `cause_code = r.code().to_string()` and `cause = r.message()`, and the world is unchanged (tested with two component types that have the same type path, LIM-REFL-0001). |
| ECS-R021 | MUST | A world has one `TypeRegistry` for components, registered resources and events. One Rust type may be registered in several of these roles. `type_registry()` contains exactly the types registered through `register_component`, `register_resource` and `register_event`. |
| ECS-R022 | MUST | `get::<T>(entity)` checks, in this order: `T` is not registered gives `Err(ComponentNotRegistered)` (LIM-ECS-0003); `entity` is not live gives `Err(NoSuchEntity)` (LIM-ECS-0001); the entity has no `T` gives `Err(EcsError::MissingComponent)` (LIM-ECS-0005) with the entity and `T::type_path()`. Otherwise it returns `Ok` with a reference to the stored value. `has::<T>(entity)` makes the first two checks and returns `Ok(true)` or `Ok(false)`. |
| ECS-R023 | MUST | `get_mut::<T>(entity)` makes the checks of ECS-R022 and returns `Ok(Mut<T>)` referring to the stored value: a value written through `DerefMut` is the value `get::<T>` returns afterwards, and no other component of any entity changes. |

### Insert and remove: archetype moves (R040–R059)

| ID | Level | Requirement |
|---|---|---|
| ECS-R040 | MUST | `insert(entity, bundle)` checks, in this order: an unregistered component type in the bundle gives `Err(ComponentNotRegistered)` (first in bundle order); a repeated type gives `Err(DuplicateComponent)`; `entity` is not live gives `Err(NoSuchEntity)`. On error the world is unchanged and every component of the bundle is dropped exactly once. Otherwise it returns `Ok(())`, and the entity's component types are the union of its previous types and the bundle's; for a type it already had, the stored value is replaced by the bundle's and the old value is dropped exactly once; its other components keep their values. `insert(entity, ())` changes nothing. |
| ECS-R041 | MUST | `remove::<T>(entity)` checks, in this order: `T` is not registered gives `Err(ComponentNotRegistered)`; `entity` is not live gives `Err(NoSuchEntity)`. If the entity has a `T`, it returns `Ok(Some(value))` with the stored value, the entity no longer has `T`, and its other components keep their values. If it has no `T`, it returns `Ok(None)` and the world is unchanged. |
| ECS-R042 | MUST | An archetype move (ECS-R040 with at least one new type, ECS-R041 with `Some`) keeps the value of every component the entity has before and after, and leaves every other entity with the components and values it had (also the entity whose row takes the vacated position, §4 "Row order"). `insert` whose types the entity all has already, and `remove` returning `Ok(None)`, move no row. |
| ECS-R043 | MUST | Archetypes and rows follow §4 "Archetype, table" and "Row order": archetype 0 is the empty set; a new archetype gets the next number the first time an entity is to be stored in it; archetypes are never removed; an entering row is appended; when a row leaves, the last row of its table takes its position. Observed through ECS-R062. Test: a hand-computed sequence (spawn 5 entities over 2 archetypes, remove a component from the first, despawn the third, insert into the last) gives the hand-computed entity order for `Query<Entity>` and for two filtered queries. |
| ECS-R044 | MUST | Differential property test (AGENTS.md rule 4), `tests/world_differential.rs`, extended from ECS-R201: for generated sequences of up to 512 operations (`spawn` of bundles of 0 to 4 of 6 component types, `despawn`, `insert`, `remove`, `get`, `get_mut` with a write, `has`, including calls on stale entities and with unregistered types), `world::World` and `reference::World` return equal results for every call (same `Ok` value or same `EcsError`), and after every operation `entities()` and, for every live entity and component type, `has` and `get` are equal in both. The generator MUST produce sequences in which entities move through at least 8 archetypes. |
| ECS-R045 | SHOULD | The archetype reached by adding or removing one set of component types from an archetype is found without searching all archetypes from the second time on (a cache of moves per archetype, ADR-0008 "archetype graph"). Checked by review and by the `add_remove` benchmark of §9. |

### Queries and filters (R060–R089)

| ID | Level | Requirement |
|---|---|---|
| ECS-R060 | MUST | A query yields an item for an entity exactly if the entity's archetype is matched (§4 "Query": it has every required type and no excluded type) and every `Added<T>` and `Changed<T>` filter of `F` holds for it. Tuples of filters hold if every element holds; `()` always holds. A query whose filter requires and excludes the same type yields nothing. |
| ECS-R061 | MUST | The item for an entity is built from `D`: `Entity` gives the entity; `&T` a reference to its `T`; `&mut T` a `Mut<T>` to its `T`; `Option<&T>` and `Option<&mut T>` give `Some` of these if the entity has a `T` and `None` otherwise (they do not restrict which entities match); a tuple gives the tuple of its elements' items. `MAX_QUERY_TUPLE == 8`; tuples of 1 to 8 are implemented for data and for filters (a test uses arity 8 of each). |
| ECS-R062 | MUST | Iteration order: `iter` and `iter_mut` yield the items of matched archetypes in ascending archetype number, and inside one archetype in ascending row (ECS-R043). The order depends only on the sequence of operations applied to the world since `World::new`: two worlds that received the same sequence of calls yield the same entity sequence for every query. |
| ECS-R063 | MUST | `iter` and `iter_mut` yield each entity that ECS-R060 selects exactly once. `count()` returns the number of items `iter_mut` would yield, without changing any tick. A value written through an item of `iter_mut` is the value `World::get` returns afterwards. |
| ECS-R064 | MUST | `Query::get(entity)` and `get_mut(entity)` return the item of ECS-R061 for `entity` if ECS-R060 selects it. Otherwise, in this order: `entity` is not live gives `Err(NoSuchEntity)` (LIM-ECS-0001); else `Err(EcsError::QueryNoMatch)` (LIM-ECS-0021) with the entity. |
| ECS-R065 | MUST | `World::query::<D, F>()` checks, in this order: a component type named in `D` or `F` is not registered gives `Err(ComponentNotRegistered)` naming the first such type (the elements of `D` left to right, then of `F`); a component type appears more than once in `D` and at least one appearance is `&mut T` or `Option<&mut T>` gives `Err(EcsError::QueryAccessConflict)` (LIM-ECS-0020) naming the first such type. `(&T, &T)` and a type that appears in `D` and in `F` are accepted. Tested with `(&mut A, &A)`, `(&mut A, &mut A)` and `(&mut A, Option<&A>)` (the rejected cases of the aliasing rule). |
| ECS-R066 | MUST | `iter` and `get` exist only for read-only data: calling `iter` on a `Query<&mut A>` does not compile (`compile_fail` doc test). `iter` takes `&self`, so two iterations of one read-only query can be nested; `iter_mut` and `get_mut` take `&mut self`, so two items of one query that could alias cannot exist at the same time (`compile_fail` doc test: holding an item of `iter_mut` across a `get_mut` call). |
| ECS-R067 | MUST | A query returned by `World::query` uses baseline 0 for `Added` and `Changed`: `Added<T>` and `Changed<T>` hold for every entity that has a `T` and for no other. (Stamping by its `Mut` items: ECS-R121.) |
| ECS-R068 | MUST | `Query::iter`, `iter_mut`, `get`, `get_mut`, `count` and `QueryIter::next` perform no heap allocation: 0 allocations counted by a counting global allocator over these calls on a world of 1,000 entities in 8 archetypes, for a query created before counting starts, in a test binary that runs nothing else (`tests/no_alloc.rs`). `World::query` itself may allocate. |
| ECS-R073 | MUST | Differential property test, `tests/query_differential.rs`: after each generated operation sequence of ECS-R044, for each of at least 8 query types (covering `Entity`, `&T`, `&mut T`, both `Option` forms, tuples, `With`, `Without` and tuples of filters), the multiset of items yielded by `iter_mut` on `world::World` (compared as entity plus component values) equals the multiset computed in the test from `reference::World::entities`, `has` and `get`, and `count()` equals its size. |

### Resources (R090–R099)

| ID | Level | Requirement |
|---|---|---|
| ECS-R090 | MUST | `insert_resource(value)` stores `value` as the world's resource of type `R` and returns the value it replaced (`None` if there was none; the replaced value is returned, not dropped). `resource::<R>()` and `resource_mut::<R>()` return a reference to the stored value, or `Err(EcsError::MissingResource)` (LIM-ECS-0010) with `core::any::type_name::<R>()` if there is none. `remove_resource::<R>()` removes and returns it, or returns `None`. A resource type needs no registration to be inserted and read. |
| ECS-R091 | MUST | `register_resource::<R>()` registers `R` in the world's `TypeRegistry` and marks `R` as part of the state hash (ECS-R190), whether or not a value is present, and returns `Ok(())`. It is idempotent. Errors as in ECS-R020 (LIM-ECS-0006). It neither inserts nor removes a value. |

### Events (R100–R109)

| ID | Level | Requirement |
|---|---|---|
| ECS-R100 | MUST | `register_event::<E>()` registers `E` in the world's `TypeRegistry`, creates the two empty buffers of `E` and returns `Ok(())`. It is idempotent: registering again keeps buffered events. Errors as in ECS-R020 (LIM-ECS-0006). `send_event::<E>` and `buffered_events::<E>` for an `E` that is not registered return `Err(EcsError::EventNotRegistered)` (LIM-ECS-0011) with `E::type_path()`. |
| ECS-R101 | MUST | `send_event(event)` appends `event` to the current buffer of `E` and returns `Ok(())`. `buffered_events::<E>()` yields the events of the previous buffer and then of the current buffer, each in the order they were sent. |
| ECS-R102 | MUST | `update_events()` does, for every registered event type: drops every event of the previous buffer exactly once, then makes the current buffer the previous one and the current buffer empty. An event is therefore yielded by `buffered_events` after 0 and after 1 calls of `update_events` since it was sent, and not after 2. Test: send a, update, send b, update, send c: `buffered_events` yields b, c. |

### Commands (R110–R119)

| ID | Level | Requirement |
|---|---|---|
| ECS-R110 | MUST | `CommandBuffer::new()` and `default()` are empty. `spawn`, `despawn`, `insert` and `remove` each record one command without changing any world; `len()` is the number of recorded commands and `is_empty()` is `len() == 0`. `CommandBuffer` is `Send` (static assertion). |
| ECS-R111 | MUST | `World::apply_commands(buffer)` applies the recorded commands in recording order, each with the effect of the `World` method of the same name with the same arguments, with two differences: a `despawn`, `insert` or `remove` whose entity is not live when the command is applied is skipped (it changes nothing and is not an error); a `remove` of a component the entity does not have changes nothing. Components attached or replaced by the commands get the tick `World::change_tick()` (ECS-R121). Afterwards the buffer is empty (`len() == 0`). |
| ECS-R112 | MUST | A `spawn` or `insert` command for which the `World` method would return `ComponentNotRegistered`, `DuplicateComponent` or `EntityLimitReached`, and a `remove::<T>` command with `T` not registered, is *rejected*: it changes nothing. Rejection is decided before the liveness check, as in ECS-R040 and ECS-R041 (an `insert` of an unregistered type on an entity that is not live is rejected, not skipped). Every other command of the buffer is still applied. If at least one command was rejected, `apply_commands` returns `Err(EcsError::CommandsRejected)` (LIM-ECS-0040) with `rejected` the number of rejected commands and `first_cause_code` and `first_cause` the code (text form) and message of the error of the first rejected command in recording order; otherwise `Ok(())`. Test: a buffer with a valid spawn, a spawn of an unregistered type and a valid insert applies the first and third and returns the error with `rejected: 1` and `first_cause_code: "LIM-ECS-0003"`. |
| ECS-R113 | MUST | Applying several buffers one after the other has the same effect as applying one buffer that holds their commands in that order: for generated command lists split at generated points into up to 4 buffers, the world after applying the buffers in order equals (same `state_hash`, `entities`, `Query<Entity>` order and `component_ticks`) the world after applying the unsplit list. Each applied command sees the effect of all commands applied before it (a `despawn` recorded after an `insert` for the same entity removes the entity; an `insert` recorded after a `despawn` of its entity is skipped). |
| ECS-R114 | MUST | Every component value held by a command is dropped exactly once: when the command is applied and the value is later dropped by the world, when the command is skipped or rejected, and when a `CommandBuffer` is dropped with commands in it (drop counters; Miri reports no leak). |
| ECS-R115 | SHOULD | A `CommandBuffer` keeps its memory after `apply_commands`: recording and applying the same commands again performs no heap allocation in the buffer. Not required in `spec_version` 1 because it needs type-erased storage (§12 item 11); M1-T040 implements it or reports why not in its handoff. |

### Change detection (R120–R129)

| ID | Level | Requirement |
|---|---|---|
| ECS-R120 | MUST | `change_tick()` of a new world is 1. `advance_change_tick()` adds 1 and returns the new value. No method other than `advance_change_tick`, `Schedule::run` and `Schedule::run_parallel` changes it, and it never decreases. |
| ECS-R121 | MUST | Stamping, with `t = World::change_tick()` for calls on a `World` (the run tick, ECS-R124, inside a schedule): a component attached to an entity by `spawn`, or by `insert` of a type the entity did not have, gets `added = changed = t`; a component replaced by `insert` gets `changed = t` and keeps `added`; `DerefMut` on a `Mut<T>` (from `World::get_mut` or from an item of a query) sets `changed = t`; `Deref` on a `Mut<T>`, `get`, `has`, `iter` and creating a `Mut<T>` without dereferencing it mutably change no tick. `component_ticks::<T>(entity)` returns the two ticks, with the checks and errors of ECS-R022. Test: spawn at tick 1, advance to 2, `get_mut` with a write, advance to 3, `insert` replacing, `insert` of a new type: ticks are (1, 2), then (1, 3) for the first type, and (3, 3) for the new one. |
| ECS-R122 | MUST | Commands applied by `World::apply_commands` stamp `World::change_tick()` by the rules of ECS-R121. An archetype move (ECS-R042) keeps both change ticks of every component the entity has before and after, and changes no tick of any other entity. `reference::World` implements `change_tick`, `advance_change_tick` and `component_ticks` with the same results as `world::World` for every operation sequence (the differential test of ECS-R044 also compares `component_ticks` of every live entity and component after every operation, with `advance_change_tick` among the generated operations). |
| ECS-R124 | MUST | Run ticks (§4 "Run tick"): in a run of a schedule that starts with `World::change_tick() == T`, the k-th position of the execution order (ECS-R170; every system and every sync point is one position, k from 0) has run tick `T + 1 + k`. A system's `Mut` dereferences stamp its run tick. Commands applied at a sync point stamp the sync point's run tick. After a run that executed, `change_tick()` is `T + 1 + n` with `n` the number of systems plus the number of stages. A run that returns LIM-ECS-0038 or LIM-ECS-0039 leaves `change_tick()` unchanged. Test: the schedule of the §4 example with `T = 7` gives the ticks shown there (read with `component_ticks` after the run). |
| ECS-R125 | MUST | In a system, `Added<T>` holds for an entity whose `T` has `added > baseline`, and `Changed<T>` for `changed > baseline`, where the baseline is the system's run tick in its previous run of this schedule, or 0 if it has not run. Consequences, each with a test: in its first run a system's `Added<T>` and `Changed<T>` hold for every entity with a `T`; a write by system `w` is seen by the `Changed<T>` query of a system ordered after `w` in the same run and not again in the next run; it is seen by a system ordered before `w` in the next run and not again in the one after; a system does not see its own writes; a component attached by a command is seen by `Added<T>` in every system of later stages in the same run and in every system of the same and earlier stages in the next run, once each; a write made directly on the `World` between two runs is seen once by every system. |

### Systems and their parameters (R130–R149)

| ID | Level | Requirement |
|---|---|---|
| ECS-R130 | MUST | `IntoSystem` is implemented for functions and closures `FnMut(P1, …, Pn) + Send + 'static` with 0 to 8 parameters that implement `SystemParam` (`MAX_SYSTEM_PARAMS == 8`; a test adds systems of arity 0, 1 and 8, a named function and a closure that captures and mutates a counter). A function with a parameter that is not a `SystemParam` (for example `&mut World`) is rejected at compile time (`compile_fail` doc test). |
| ECS-R131 | MUST | In a system, `Res<R>` dereferences to the world's resource of type `R`, and `ResMut<R>` dereferences mutably to it: a value written through `ResMut<R>` is the value `World::resource::<R>()` returns after the run and the value later systems of the run read. |
| ECS-R132 | MUST | `EventWriter::<E>::send(event)` appends `event` to the current buffer of `E`, as `World::send_event` does. Events sent by several systems are in the buffer in the order of the sends within a system and, between systems, in the systems' execution order (two writers of one event type always have an order path, ECS-R153). |
| ECS-R133 | MUST | `EventReader::<E>::read()` yields, oldest first, every buffered event of `E` that this parameter has not yielded before, and marks them yielded. Each `EventReader` parameter of each system keeps its own position, across runs. The first `read` yields every buffered event. An event dropped by `update_events` before a parameter read it is never yielded to it. Test: a writer system ordered before a reader system: the reader gets each event exactly once over 3 runs with `update_events` between runs; a reader ordered before the writer gets each event exactly once, one run later. |
| ECS-R134 | MUST | `Commands` dereferences to a `CommandBuffer` that belongs to the system. Commands recorded in a run are applied at the sync point of the system's stage in that run (ECS-R172) and are then gone: a system that records nothing in a later run applies nothing. |
| ECS-R135 | MUST | A `Query<D, F>` parameter behaves as the query of ECS-R060 to ECS-R064 on the world at the time the system runs, with the system's baseline (ECS-R125) and the system's run tick for stamping (ECS-R124). Entities spawned and archetypes created after the schedule was built are matched. |

### Schedule construction and conflicts (R150–R169)

| ID | Level | Requirement |
|---|---|---|
| ECS-R150 | MUST | `ScheduleBuilder::new(name)` has no stage and no system. `add_stage(stage)` appends a stage and returns `Ok(())`; stages run in the order added. A stage name already in the builder gives `Err(EcsError::DuplicateStage)` (LIM-ECS-0030) and leaves the builder unchanged. A stage may have no systems. |
| ECS-R151 | MUST | `add_system(stage, name, system)` checks, in this order: `stage` is not a stage of the builder gives `Err(EcsError::UnknownStage)` (LIM-ECS-0031); `name` is already the name of a system of the builder (in any stage) gives `Err(EcsError::DuplicateSystemName)` (LIM-ECS-0032); a `Query` parameter violates the rule of ECS-R065 on repeated types gives `Err(QueryAccessConflict)` (LIM-ECS-0020, first such parameter); two parameters of the system conflict (§4 "Conflict") gives `Err(EcsError::SystemParamConflict)` (LIM-ECS-0036) with the system name, the access kind and the type name of the first conflicting pair (pairs ordered by first parameter, then second). On error the builder is unchanged. Otherwise it returns `Ok(SystemId)`; the k-th successful call on a builder (k from 0) returns `index() == k`. Two `EventReader<E>` of one `E`, two `Res<R>` and two disjoint queries in one system are accepted. |
| ECS-R152 | MUST | `add_order(first, then)` checks, in this order: an ID whose index is not below the number of systems gives `Err(EcsError::UnknownSystem)` (LIM-ECS-0033) with the index of `first` if that one is out of range, otherwise of `then`; the two systems are in different stages gives `Err(EcsError::OrderAcrossStages)` (LIM-ECS-0034) with both system and stage names; `first == then`, or `first` already has to run after `then` directly or through other orders, gives `Err(EcsError::OrderCycle)` (LIM-ECS-0035). On error the builder is unchanged. Otherwise the order is recorded and it returns `Ok(())`; adding an order that exists changes nothing. |
| ECS-R153 | MUST | `build(world)` checks, in this order: for systems in ascending index and their parameters left to right, a component type named in a `Query` parameter that is not registered in `world` gives `Err(ComponentNotRegistered)` (LIM-ECS-0003), and an event type of an `EventReader` or `EventWriter` that is not registered gives `Err(EventNotRegistered)` (LIM-ECS-0011); then, for pairs of systems `(i, j)` with `i < j` of one stage, in ascending `i`, then `j`: the two systems conflict (§4 "Conflict") and no order path connects them in either direction gives `Err(EcsError::SystemAccessConflict)` (LIM-ECS-0037) with the stage, the names of systems `i` and `j`, the access kind and the type name of the first conflicting pair of accesses. Otherwise it returns `Ok(Schedule)` with `name()` the builder's name and `system_count()` its number of systems. Resources are not checked by `build` (ECS-R171). |
| ECS-R154 | MUST | The conflict rule is exactly §4 "Conflict". Tests, each with two systems in one stage and no order, expecting LIM-ECS-0037: `Query<&mut A>` and `Query<&A>`; `Query<&mut A>` and `Query<&mut A>`; `Query<&mut A>` and `Query<Entity, Changed<A>>`; `Query<Option<&mut A>>` and `Query<&A>`; `ResMut<R>` and `Res<R>`; `ResMut<R>` twice; `EventWriter<E>` and `EventReader<E>`; `EventWriter<E>` twice. Expecting `Ok`: `Query<&A>` twice; `Query<&mut A>` and `Query<&mut B>`; `Query<&mut A, With<B>>` and `Query<&mut A, Without<B>>` (disjoint); `Query<(&mut A, &B)>` and `Query<&mut A, Without<B>>` (disjoint); `Query<&mut A>` and `Query<Entity, With<A>>`; `Res<R>` twice; `EventReader<E>` twice; `Commands` twice; each conflicting pair above with an order between the two, with an order path through a third system, and with the two systems in different stages. |
| ECS-R155 | MUST | A `Schedule` is bound to the world passed to `build`: `run` and `run_parallel` with any other `World` value return `Err(EcsError::WorldMismatch)` (LIM-ECS-0038) without running a system. The check uses identity only (not content): a second world built by the same calls is another world. |

### Single-threaded executor (R170–R179)

| ID | Level | Requirement |
|---|---|---|
| ECS-R170 | MUST | `Schedule::run(world)` calls every system exactly once, on the calling thread, in the execution order of §4: stages in the order added; inside a stage, repeatedly the system with the lowest `SystemId` index that has not run and whose `first` systems (ECS-R152) have all run; each call returns before the next starts. State captured by a system closure persists between runs. Test: systems 0 to 3 in one stage with orders (3, 0) and (2, 1) run in the order 2, 1, 3, 0 (recorded in a resource). |
| ECS-R171 | MUST | Before calling any system, `run` checks that the resource of every `Res<R>` and `ResMut<R>` parameter exists in `world`, for systems in ascending index and parameters left to right; the first missing one gives `Err(EcsError::SystemResourceMissing)` (LIM-ECS-0039) with the schedule name, the system name and `core::any::type_name::<R>()`. Then no system ran and the world is unchanged. |
| ECS-R172 | MUST | Sync point: after the last system of a stage has returned, and before the first system of the next stage starts, `run` applies the command buffers of the stage's systems in ascending `SystemId` index, each by the rules of ECS-R111 and ECS-R112, on the calling thread. Commands recorded in a stage are therefore not visible to systems of the same stage in that run and are visible to every system of later stages. A stage without systems has a sync point that applies nothing. Test: three systems of one stage with orders that make them execute as 2, 0, 1 each record one spawn of a distinct component type; the three entities are spawned in the order of system indices 0, 1, 2 (entity indices and `Query<Entity>` order), and a system of the next stage sees all three. |
| ECS-R173 | MUST | If commands were rejected at the sync points of a run (ECS-R112), `run` still executes every stage and applies every other command, and then returns `Err(EcsError::ScheduleCommandsRejected)` (LIM-ECS-0041) with the schedule name, the name of the system that recorded the first rejected command in application order, `rejected` the number of rejected commands of the whole run, and `first_cause_code` and `first_cause` of that first command. Otherwise `run` returns `Ok(())`. |
| ECS-R174 | MUST | A panic in a system propagates out of `run` with its payload unchanged. Systems and sync points after it in the execution order do not run. Afterwards the `World` and the `Schedule` can be dropped without undefined behaviour or leak, and every command recorded in the unfinished stage is dropped exactly once (Miri test). The content of the world after a panic is not specified, and running the schedule again is not specified (§12 item 18). |
| ECS-R175 | MUST | After 10 warm-up runs, `Schedule::run` performs no heap allocation for a schedule whose systems make no structural change: 0 allocations counted by a counting global allocator over 100 further runs of a schedule with 2 stages and 4 systems (queries iterated with `iter` and `iter_mut` over 1,000 entities in 8 archetypes, `Res`, `ResMut`, an `EventReader` with no events sent, a `Commands` parameter that records nothing), with no telemetry `Collector` installed, in a test binary that runs nothing else (`tests/no_alloc.rs`). |

### Parallel executor (R180–R189)

| ID | Level | Requirement |
|---|---|---|
| ECS-R180 | MUST | Equivalence (ADR-0009 Verification; AGENTS.md rule 4): for a schedule accepted by `build` whose systems follow the rules of §11, `run_parallel(world, pool)` returns the same result as `run(world)` (`Ok`, or the same `EcsError`) and leaves the world equal to the one `run` leaves: same `state_hash()`, `entities()`, entity sequence of `Query<Entity>` and of every query type of the test, `component_ticks` of every component, `change_tick()`, buffered events of every type, and resources. Property test, `tests/executor_equivalence.rs`, for thread counts 1, 2 and 4: for generated worlds (up to 500 entities over 6 component types) and generated schedules (1 to 3 stages, up to 12 systems drawn from a fixed set of system functions that read, write, send events, read events and record commands, with generated orders completed so that `build` succeeds), 5 consecutive runs on two worlds built by the same calls, one with `run` and one with `run_parallel`, are compared after every run. |
| ECS-R181 | MUST | In `run_parallel`, a system starts only after every system with an order path to it has returned, every system of the stage has returned before the stage's sync point starts, and the sync point has finished before a system of the next stage starts. Two systems that conflict therefore never run at the same time (they have an order path, ECS-R153). Test, for thread counts 2 and 4: every system takes a start number and an end number from one shared `AtomicUsize`; for every order `(first, then)` the end of `first` is less than the start of `then`, and every start of a later stage is greater than every end of an earlier stage. With a pool of `threads() == 1`, `run_parallel` calls the systems in exactly the execution order of ECS-R170. |
| ECS-R182 | MUST | `run_parallel` makes the checks of ECS-R155 and ECS-R171 before running any system, assigns run ticks by ECS-R124 from the execution order of ECS-R170 (not from the order in which systems happen to run), applies sync points on the calling thread by ECS-R172, and reports rejected commands by ECS-R173. |
| ECS-R183 | MUST | A panic in a system during `run_parallel` propagates out of `run_parallel` after every system of that stage that had started has returned or panicked. Systems with an order path from the panicked system, the stage's sync point and later stages do not run. Unlike in `run` (ECS-R174), systems of the stage without an order path from the panicked system still run. If several systems panic, the payload of the one with the lowest `SystemId` index is resumed. Afterwards the pool is usable (JOBS-R100), and the `World` and the `Schedule` can be dropped as in ECS-R174. |
| ECS-R184 | SHOULD | After 10 warm-up runs, `run_parallel` performs no heap allocation on any thread for the schedule of ECS-R175. This depends on JOBS-R087 (a SHOULD of `liminal_jobs`); see §12 item 8. |

### State hash (R190–R199)

| ID | Level | Requirement |
|---|---|---|
| ECS-R190 | MUST | `World::state_hash()` returns `finish()` of a new `liminal_core::hash::StateHasher` after writing exactly these bytes, all integers as `u64` little-endian (`write_u64`): (1) the number of live entities; (2) for each live entity in ascending index: `entity.to_bits()`, the number of its components, and for each of its components in ascending `StableTypeId` value: the `StableTypeId` value, then the bytes of `liminal_reflect::hash::write_reflect` for the component (REFL-R100); (3) the number of resource types that are registered (ECS-R091) and have a value, and for each of them in ascending `StableTypeId` value: the `StableTypeId` value, then `write_reflect` of the value; (4) the number of registered event types, and for each in ascending `StableTypeId` value: the `StableTypeId` value, the length of the previous buffer, `write_reflect` of each of its events in order, the length of the current buffer, `write_reflect` of each of its events in order. Known-answer test: for a world with 2 entities, 2 component types, 1 registered resource and 1 event type with one event in each buffer, the hash equals `finish()` of a `StateHasher` fed the byte sequence written out by hand in the test. A new world hashes the 24 zero bytes of three zero counts. |
| ECS-R191 | MUST | The hash depends on content, not on history: two worlds with the same live entities (equal `to_bits`), equal component values per entity, equal values of registered resources and equal event buffers have equal hashes, whatever sequence of operations built them (tested with: components attached by one `spawn`, by `spawn` then `insert` in two orders, and by commands; entities spawned and despawned in between; types registered in different orders). Two worlds that differ in the set of live entities, in the component types of an entity, in any reflected field of any component, registered resource or buffered event, or in which buffer an event is in, have different hashes (property test over generated pairs; the test accepts the 2^-64 collision probability). |
| ECS-R192 | MUST | The hash does not include, and is not changed by: resources that are not registered; change ticks (`advance_change_tick`, a `Mut` write of an equal value); freed entity slots and the order of future entity IDs; archetype numbers and row order; unapplied command buffers; state of schedules and systems (run ticks, event reader positions, state captured by closures). Test for each of the first four: an operation that changes only that state leaves `state_hash()` unchanged. §12 item 9 states what follows for replay. |
| ECS-R193 | MUST | `state_hash()` takes `&self`, changes nothing in the world (no tick, no archetype, no buffer) and returns the same value when called again without an intervening change. |
| ECS-R194 | MUST | `reference::World::state_hash()` writes the bytes of ECS-R190 from its map (with zero resources and zero event types). Differential property test, `tests/state_hash.rs`: after every operation of the generated sequences of ECS-R044, on a `world::World` without resources and events, `world::World::state_hash() == reference::World::state_hash()`. |

### Reference world (R200–R209)

| ID | Level | Requirement |
|---|---|---|
| ECS-R200 | MUST | `liminal_ecs::reference::World` has the items of §5.8 with the signatures shown, stores components in ordered maps keyed by `Entity` and `StableTypeId` (no archetypes, no tables, no cache), and its module declares `#![forbid(unsafe_code)]`. Its `new`, `with_entity_limit`, `register_component`, `spawn`, `despawn`, `contains`, `entity_count`, `entities`, `get`, `get_mut` and `has` meet ECS-R001 (entity part), ECS-R002, ECS-R005 to ECS-R008, ECS-R020, ECS-R022 and ECS-R023. |
| ECS-R201 | MUST | Differential property test (AGENTS.md rule 4), `tests/world_differential.rs`: for generated sequences of up to 512 operations (`register_component`, `spawn` of bundles of 0 to 4 of 6 component types, `despawn`, `get`, `get_mut` with a write, `has`, `contains`, including stale entities, unregistered types, duplicate types in a bundle and a world with entity limit 8 that is driven to LIM-ECS-0002), `world::World` and `reference::World` return equal results for every call, and after every operation `entity_count()`, `entities()` and, for every live entity and component type, `has` and `get` are equal in both. |
| ECS-R202 | MUST | `reference::World::insert` and `remove` meet ECS-R040 and ECS-R041 (the differential test of ECS-R044 compares them with `world::World`). |

### Telemetry (R210–R219)

Spans and counters are recorded with `liminal_telemetry::span!` and `counter!` and exist only as
far as [liminal_telemetry.md](liminal_telemetry.md) says (feature `telemetry` on, a `Collector`
installed). The tests install a `Collector`, serialized on one lock, in one test binary
(`tests/telemetry.rs`).

| ID | Level | Requirement |
|---|---|---|
| ECS-R210 | MUST | Each call of `Schedule::run` and `Schedule::run_parallel` that passes the checks of ECS-R155 and ECS-R171 records exactly one span named with the schedule's name on the calling thread, covering the whole run, and exactly one span named with the system's name for each system call, on the thread that calls the system, covering the call. Test: a schedule `"fixed_update"` with 4 systems gives 1 span of that name and 1 span per system name with `run`, and the same with `run_parallel` on a pool of 4 threads (other spans, for example those of `liminal_jobs`, may be present). |
| ECS-R211 | MUST | `World::apply_commands` and each sync point that skipped at least one command (ECS-R111) add the number of skipped commands to the counter `ecs.commands_skipped` with one `counter!` call; with no skipped command they record no counter. Test: a buffer with 3 commands for despawned entities gives a `CounterTotal` of 3 in the frame record. |

### Determinism (R220–R229)

| ID | Level | Requirement |
|---|---|---|
| ECS-R220 | MUST | Workload `world_churn` in `crates/liminal_ecs/tests/determinism.rs` (§11) gives the same sequence of state hashes in two runs in one process, and its final hash equals the hash literal recorded in the test on every supported platform. |
| ECS-R221 | MUST | Workload `schedule_ticks` (§11) gives the same sequence of state hashes with `Schedule::run`, with `Schedule::run_parallel` on pools of 1, 2 and 4 threads and on a pool with `PoolConfig::threads: None`, and in two runs of each in one process; its final hash equals the hash literal recorded in the test on every supported platform. |

### Errors (R230–R239)

| ID | Level | Requirement |
|---|---|---|
| ECS-R230 | MUST | `EcsError` implements `liminal_core::error::EngineError`. Each variant has the code, message, help and context fields of §8, and one value of every variant passes `liminal_core::error::validate_error(&e, "ECS")` (snapshot of the text and JSON form of one value per variant, `tests/snapshots/ecs_errors.txt`, with a fixed string in place of `type_name` output). |

### Assignment to implementation tasks

Every MUST requirement is assigned to exactly one task; the IDs are in the tasks' `requirements`
frontmatter.

| Task | Requirements |
|---|---|
| M1-T038 (entities, registration, storage, spawn, despawn, direct access, reference world) | ECS-R001 to ECS-R009, ECS-R020 to ECS-R023, ECS-R200, ECS-R201, ECS-R230 |
| M1-T039 (archetype moves, queries, filters) | ECS-R040 to ECS-R044, ECS-R060 to ECS-R068, ECS-R073, ECS-R202 |
| M1-T040 (resources, events, commands, change ticks) | ECS-R090, ECS-R091, ECS-R100 to ECS-R102, ECS-R110 to ECS-R114, ECS-R120 to ECS-R122 |
| M1-T041 (systems, schedules, single-threaded executor) | ECS-R124, ECS-R125, ECS-R130 to ECS-R135, ECS-R150 to ECS-R155, ECS-R170 to ECS-R175, ECS-R210, ECS-R211 |
| M1-T042 (parallel executor) | ECS-R180 to ECS-R183 |
| M1-T043 (state hash, determinism, benchmarks, `usable`) | ECS-R190 to ECS-R194, ECS-R220, ECS-R221 |

Notes on the split:
- M1-T038 implements all of `EcsError` (ECS-R230), including the variants whose producers come
  in later tasks, and `Entity` with its `Reflect` and `StateHash` impls (ECS-R003, ECS-R004).
- `Mut` exists from M1-T038 (ECS-R023 needs it). Tick storage and stamping (ECS-R121) come with
  M1-T040, which adds the tick columns to the tables and to the reference world; until then
  `Mut` records nothing. M1-T038 and M1-T039 therefore do not test `component_ticks`, and the
  tick comparisons (ECS-R122: ticks kept by archetype moves, ticks in the differential tests)
  are added by M1-T040.
- `Added` and `Changed` exist as filters from M1-T039 with baseline 0 (ECS-R067: they hold for
  every entity that has the component). Their behaviour in systems is ECS-R125 (M1-T041).
- ECS-R211 covers `World::apply_commands` (code of M1-T040) and sync points (M1-T041). M1-T041
  adds the `liminal_telemetry` dependency and the counter in both places.
- ECS-R210 covers `run_parallel`; M1-T042 extends the test of M1-T041 to it. ECS-R182 lists what
  M1-T042 must keep from the single-threaded executor.
- SHOULD requirements: ECS-R045 is implemented by M1-T039; ECS-R115 is decided by M1-T040 and
  ECS-R184 by M1-T042 (implemented, or reported with the reason in the handoff).

## 7. Invariants

- INV-1: Every live entity is stored in exactly one row of exactly one table, and the world's
  entity location (archetype, row) for it points at that row; a table's entity column and the
  locations agree. (`debug_assert!` after every structural change; differential tests ECS-R044.)
- INV-2: In every table, all columns and both tick columns of every component have the same
  length, equal to the number of rows. (`debug_assert!` after every structural change.)
- INV-3: An archetype's component set is never changed after creation, and no two archetypes
  have the same set. (`debug_assert!` when an archetype is created.)
- INV-4: For every component instance, `1 <= added <= changed <= World::change_tick()` outside a
  schedule run. (Property test over the sequences of ECS-R122.)
- INV-5: `World::change_tick()` never decreases (ECS-R120).
- INV-6: While a schedule run is in progress, no table gains or loses a row and no archetype is
  created except at sync points, which run on the calling thread while no system runs
  (ECS-R172, ECS-R181). Every `SAFETY:` comment on shared world access in `cell`, `query`,
  `system` and `schedule` refers to this invariant and to INV-7.
- INV-7: Two systems that run at the same time do not conflict (§4): no column, resource or
  event buffer is written by one while the other reads or writes it. Follows from ECS-R153 and
  ECS-R181. Checked by Miri (data-race detection) on the tests of ECS-R180 and ECS-R181 with 2
  threads.
- INV-8: Two items of one query that are alive at the same time never refer mutably to the same
  component instance (ECS-R065, ECS-R066; `iter_mut` yields each entity once, ECS-R063).
- INV-9: The orders of each stage form a directed acyclic graph (ECS-R152), so the execution
  order exists and `run` cannot fail to schedule a system.
- INV-10: Each registered event type has exactly two buffers, and the sequence numbers of
  buffered events are consecutive and increasing from the previous buffer to the current one.

## 8. Error codes

`liminal_ecs` has recoverable errors wherever the caller supplies an entity, a type or a schedule
description. Systems themselves return no errors (they return `()`); what goes wrong in their
deferred commands is reported by the run (ECS-R173). Messages and help below are normative;
`{x}` is the value of context field `x`, and the backticks shown are part of the text.

| Code | Variant | Meaning (message) | Help | Context |
|---|---|---|---|---|
| LIM-ECS-0001 | `NoSuchEntity` | entity `{entity}` is not live. | the entity was despawned or was not spawned in this world; check `World::contains` first, or use commands, which skip entities that are gone. | `entity` |
| LIM-ECS-0002 | `EntityLimitReached` | all `{entity_limit}` entity slots of the world are in use or retired. | despawn entities that are no longer needed, or create the world with a higher limit (`World::with_entity_limit`; `World::new` allows 4294967295). | `entity_limit` |
| LIM-ECS-0003 | `ComponentNotRegistered` | component type `{component}` is not registered. | call `World::register_component` for this type before it is spawned, inserted, removed or queried. | `component` |
| LIM-ECS-0004 | `DuplicateComponent` | component type `{component}` occurs more than once in one bundle. | pass each component type at most once per `spawn` or `insert`. | `component` |
| LIM-ECS-0005 | `MissingComponent` | entity `{entity}` has no component `{component}`. | check `World::has` first, or use a query with `Option<&T>`. | `entity`, `component` |
| LIM-ECS-0006 | `TypeRegistrationFailed` | type `{type_path}` cannot be registered (`{cause_code}`): {cause} | look up `{cause_code}` in `docs/specs/liminal_reflect.md` §8; two types with the same type path need `#[reflect(type_path = "…")]` on one of them. | `type_path`, `cause_code`, `cause` |
| LIM-ECS-0010 | `MissingResource` | resource `{resource}` is not in the world. | insert it with `World::insert_resource` before it is read. | `resource` |
| LIM-ECS-0011 | `EventNotRegistered` | event type `{event}` is not registered. | call `World::register_event` for this type before events are sent or read, and before a schedule that uses it is built. | `event` |
| LIM-ECS-0020 | `QueryAccessConflict` | a query names component `{component}` more than once and at least once mutably. | name a component once in a query's data when it is accessed with `&mut`. | `component` |
| LIM-ECS-0021 | `QueryNoMatch` | entity `{entity}` is live but is not selected by the query. | the entity lacks a required component, has an excluded one, or is filtered out by `Added` or `Changed`; `World::has` shows which components it has. | `entity` |
| LIM-ECS-0030 | `DuplicateStage` | schedule `{schedule}` already has a stage `{stage}`. | give each stage of a schedule a distinct name. | `schedule`, `stage` |
| LIM-ECS-0031 | `UnknownStage` | schedule `{schedule}` has no stage `{stage}`. | add the stage with `ScheduleBuilder::add_stage` before adding systems to it. | `schedule`, `stage` |
| LIM-ECS-0032 | `DuplicateSystemName` | schedule `{schedule}` already has a system `{system}`. | give each system of a schedule a distinct name. | `schedule`, `system` |
| LIM-ECS-0033 | `UnknownSystem` | system `{system_index}` does not exist in schedule `{schedule}`, which has `{system_count}` systems. | pass only `SystemId` values returned by `add_system` of the same `ScheduleBuilder`. | `schedule`, `system_index`, `system_count` |
| LIM-ECS-0034 | `OrderAcrossStages` | systems `{first}` (stage `{first_stage}`) and `{then}` (stage `{then_stage}`) of schedule `{schedule}` are in different stages. | stages already run in the order they were added; order two systems only if they are in one stage, or move one of them. | `schedule`, `first`, `first_stage`, `then`, `then_stage` |
| LIM-ECS-0035 | `OrderCycle` | making system `{then}` run after system `{first}` in schedule `{schedule}` would create a cycle. | `{first}` already has to run after `{then}`, directly or through other systems (or they are the same system); remove one of these orders. | `schedule`, `first`, `then` |
| LIM-ECS-0036 | `SystemParamConflict` | two parameters of system `{system}` in schedule `{schedule}` conflict on {access} `{type_name}`. | merge the two parameters into one, or make the two queries disjoint with `With`/`Without` filters. | `schedule`, `system`, `access`, `type_name` |
| LIM-ECS-0037 | `SystemAccessConflict` | systems `{first}` and `{second}` in stage `{stage}` of schedule `{schedule}` both access {access} `{type_name}`, at least one of them for writing, and no order is declared between them. | declare which runs first with `ScheduleBuilder::add_order`, move one of them to another stage, or make their queries disjoint with `With`/`Without` filters. | `schedule`, `stage`, `first`, `second`, `access`, `type_name` |
| LIM-ECS-0038 | `WorldMismatch` | schedule `{schedule}` was built for another world. | run a schedule only with the `World` that was passed to `ScheduleBuilder::build`. | `schedule` |
| LIM-ECS-0039 | `SystemResourceMissing` | system `{system}` of schedule `{schedule}` needs resource `{resource}`, which is not in the world. | insert the resource with `World::insert_resource` before the schedule runs. | `schedule`, `system`, `resource` |
| LIM-ECS-0040 | `CommandsRejected` | `{rejected}` commands were rejected; the first (`{first_cause_code}`): {first_cause} | every other command was applied; fix the cause named by `{first_cause_code}`. | `rejected`, `first_cause_code`, `first_cause` |
| LIM-ECS-0041 | `ScheduleCommandsRejected` | `{rejected}` commands of schedule `{schedule}` were rejected; the first, recorded by system `{system}` (`{first_cause_code}`): {first_cause} | the run completed and every other command was applied; fix the cause named by `{first_cause_code}` in system `{system}`. | `schedule`, `system`, `rejected`, `first_cause_code`, `first_cause` |

Each message ends with `.`; for 0006, 0040 and 0041 the final `.` is that of the embedded cause
message. `Entity` context values are `Text` in the `Display` form (`3v2`); `u32` and `usize`
values are `UInt`; all others are `Text`. Code groups: 0001–0009 entities and components,
0010–0019 resources and events, 0020–0029 queries, 0030–0039 schedules, 0040–0049 commands. Codes
0007 to 0009, 0012 to 0019, 0022 to 0029 and 0042 onward are unassigned; new codes take the next
free number in their group.

## 9. Performance budgets

None yet: every number below is set from a measurement in `liminal_benches` (ADR-0023), baseline
`ci/perf/<machine-id>/liminal_ecs.json` on the reference machine (PERFORMANCE.md: Apple M5 Pro),
and written into this table with a `spec_version` bump. Benchmark case IDs:
`liminal_ecs/<group>/<case>`. Workloads are modelled on ecs_bench_suite (PERFORMANCE.md
"Competitive benchmarks"); component types are plain structs of `f32` arrays (`Transform` 16
`f32`, `Position`, `Rotation`, `Velocity` 3 `f32` each). Multithreaded cases are excluded from
the CI instruction-count gate (ADR-0023 Decision 4).

| Scenario | Budget | Machine | Source of number |
|---|---|---|---|
| `simple_insert`: spawn 10,000 entities with `(Transform, Position, Rotation, Velocity)` into a new world (µs per run) | none yet | Reference | M1-T043, case `liminal_ecs/world/simple_insert`; single-threaded, so also in the instruction gate once M1-T033 exists |
| `simple_iter`: `Query<(&Velocity, &mut Position)>` over 10,000 such entities, adding velocity to position (µs per run) | none yet | Reference | M1-T043, case `liminal_ecs/query/simple_iter` |
| `simple_iter` on `reference::World` (the same update through `entities`, `get` and `get_mut`). Not a budget: the comparison AGENTS.md rule 4 and ADR-0018 require for row 2 | none yet | Reference | M1-T043, case `liminal_ecs/reference/simple_iter` |
| `frag_iter`: 26 component types, 20 entities each with `(Data, one of the 26)`, `Query<&mut Data>` (ns per run) | none yet | Reference | M1-T043, case `liminal_ecs/query/frag_iter` |
| `add_remove`: 10,000 entities with `A`; insert `B` on each, then remove `B` from each (µs per run) | none yet | Reference | M1-T043, case `liminal_ecs/world/add_remove` |
| `schedule`: 10,000 entities each of `(A, B)`, `(A, B, C)`, `(A, B, C, D)`, `(A, B, C, E)`; 3 systems that swap values between two components; `run`, and `run_parallel` with threads 1, 2, 4, all (µs per run) | none yet | Reference | M1-T043, cases `liminal_ecs/schedule/run`, `liminal_ecs/schedule/run_parallel_t<n>` |
| `state_hash`: 100,000 entities with `(Position, Velocity, Rotation)` (ms per call) | none yet | Reference | M1-T043, case `liminal_ecs/world/state_hash_100k` |
| The first, second, fourth, fifth and sixth workloads against `bevy_ecs` and `hecs` | none (comparison, parity not required in M1) | Reference | M1-T052, `ci/perf/competitive/liminal_ecs-<date>.json`, summarized here |

Until these are measured, this spec makes no claim about speed beyond the allocation counts of
ECS-R068 and ECS-R175.

## 10. Test plan

Optimized paths and their references (AGENTS.md rule 4): the table of §5.8.

| Area | Test kinds | Oracle |
|---|---|---|
| Entities (R001–R009) | unit, property, differential, integration, doc test | `liminal_core::handle::HandleAllocator` (R002); `liminal_reflect::value::validate_reflect` and `state_hash_of` (R004); drop counters; `reference::World` |
| Registration and direct access (R020–R023) | unit, integration | Hand-written expectations; `TypeRegistry` contents |
| Archetype moves (R040–R044) | unit, property, differential (`tests/world_differential.rs`) | `reference::World`; hand-computed row order (R043) |
| Queries (R060–R073) | unit, property, differential (`tests/query_differential.rs`), `compile_fail` doc tests, counting allocator (`tests/no_alloc.rs`) | Items computed from `reference::World`; hand-computed iteration order |
| Resources, events (R090–R102) | unit, integration | Hand-written expectations; drop counters |
| Commands (R110–R114) | unit, property (split buffers, R113), integration | The same operations made directly on a second `World`; drop counters |
| Change ticks (R120–R125) | unit, property, differential, integration | Hand-computed ticks (the §4 example); `reference::World` ticks |
| Systems and schedule construction (R130–R155) | unit (one test per accepted and per rejected case of R154), integration, `compile_fail` doc test, doc tests | Hand-written expectations; error codes |
| Single-threaded executor (R170–R175) | integration, counting allocator, Miri (panic) | Order recorded in a resource; hand-computed entity order |
| Parallel executor (R180–R183) | property (`tests/executor_equivalence.rs`), integration, Miri | `Schedule::run` on a second world built by the same calls; start and end numbers against every order |
| State hash (R190–R194) | unit known-answer, property, differential (`tests/state_hash.rs`) | Byte sequence written by hand and hashed with `StateHasher`; `reference::World::state_hash` |
| Telemetry (R210, R211) | integration with a `Collector` (`tests/telemetry.rs`) | Expected span names and counter totals in the `FrameRecord` |
| Determinism (R220, R221) | determinism (`tests/determinism.rs`, ADR-0024) | Hash sequences equal across runs and executors; recorded hash literals |
| Errors (R230) | unit, snapshot | `liminal_core::error::validate_error`; approved snapshot |

Rules for these tests:
- No sleeps and no timing assertions (TESTING.md). A test never requires that two systems
  actually overlapped.
- Tests that install a telemetry `Collector` are in one test binary and take one lock. The
  allocation tests are in their own binary (`tests/no_alloc.rs`), one test function per
  requirement taking one lock, because the allocation counter is process-wide.
- **Miri** runs the whole test suite (`cargo +nightly miri test -p liminal_ecs`, set up by
  M1-T018). Property and stress tests use reduced counts under `cfg(miri)`, chosen by the
  implementing task and listed in its handoff, instead of being excluded. The tests of ECS-R180
  and ECS-R181 run under Miri with 2 threads so that its data-race detector checks INV-7. Any
  test excluded from Miri is listed here with its reason by the task that excludes it.
- **loom:** `spec_version` 1 specifies no lock-free structure and no shared mutable state of
  this crate's own: the parallel executor hands systems to `liminal_jobs` (one job-graph node
  per system, one dependency per order), whose deque and wake protocol have loom tests
  (JOBS-R010, JOBS-R023). If M1-T042 adds atomics or other shared state of its own (for example
  for collecting panics or rejected commands), it adds loom models for them behind `cfg(loom)`
  and lists them here with a `spec_version` bump (ADR-0018, §12 item 16).

Every requirement's verifying test carries `/// Verifies: ECS-Rnnn`. Primary entry points with
`# Examples`: `World` (register, spawn, get, query), `Entity`, `Component`, `Query`,
`CommandBuffer`, `ScheduleBuilder` (the example of §5.7), `Schedule::run`,
`Schedule::run_parallel`, `World::state_hash`, `reference::World`.

## 11. Determinism

`liminal_ecs` is `deterministic = true` (ADR-0009):

- It reads no clock, no environment and no OS randomness. It uses no floating-point arithmetic of
  its own; component values are hashed by their bits (REFL-R100).
- No collection with a randomly seeded hasher is used. Lookups by Rust `TypeId` use
  `liminal_core::hash::FixedState` or ordered maps, and no result depends on the iteration order
  of such a map: everything that is iterated has a defined order (ADR-0009 item 3):

  | Iteration | Order | Requirement |
  |---|---|---|
  | Entity IDs issued | `HandleAllocator` rules | ECS-R002 |
  | `World::entities` | Ascending index | ECS-R008 |
  | Query items | Archetype number, then row; both functions of the operation sequence | ECS-R043, ECS-R062 |
  | Buffered events | Send order | ECS-R101, ECS-R132 |
  | Commands | Recording order per buffer; buffers in ascending `SystemId` index at sync points | ECS-R111, ECS-R172 |
  | Systems in `run` | Execution order | ECS-R170 |
  | Run ticks | Execution order, also in `run_parallel` | ECS-R124, ECS-R182 |
  | State hash | Entity index, then `StableTypeId` | ECS-R190 |

- Deterministic parallelism (ADR-0009 item 4): conflicting access without an order is a build
  error (ECS-R153); commands are applied at sync points on one thread in a fixed order
  (ECS-R172); `run_parallel` gives the world `run` gives (ECS-R180).
- The identity check of ECS-R155 needs a value that distinguishes worlds in one process. It is
  compared for equality only and is never hashed, ordered, printed or put into an error.
- `core::any::TypeId` and `core::any::type_name` are used inside one process only (lookups and
  error text) and never enter a hash; the state hash uses `StableTypeId` (REFL-R005).

Rules for system authors. `run_parallel` equals `run` (ECS-R180) only for systems that follow
them; they are checked in review and by each crate's own executor equivalence test:
1. A system reads and writes simulation state only through its parameters. State captured by a
   system closure is private to that system and is not part of the state hash.
2. A system does not observe which thread runs it, or when (thread ID, thread-local data,
   clocks), and shares no state with other systems outside the world (no `Arc<Mutex<…>>`, no
   atomics, no channels).
3. Randomness comes from `liminal_core::rng::Rng` values stored in components or in registered
   resources, so that the generator state is hashed and its use is ordered by the conflict rule.
4. Transcendental functions come from `liminal_math::det` (ADR-0009 item 5).

**Workloads of `crates/liminal_ecs/tests/determinism.rs`** (ADR-0024). Each workload runs at
least twice in the same process and compares the sequences of state hashes with each other, and
its final hash with a hash literal recorded in the test. Inputs come from
`liminal_core::rng::Rng` with fixed world seeds and stream keys. Component fields are integers,
`f32`, arrays of them and `Entity`; arithmetic on `f32` is addition and multiplication only.
Exact parameters are fixed in the test; changing them changes the literal and is an oracle
change (AGENTS.md rule 5).

| Workload | What it does | Cross-platform bit-identity claimed |
|---|---|---|
| `world_churn` (ECS-R220) | 10,000 seeded operations on one `World` with 6 component types, 1 registered resource and 1 event type: `spawn`, `despawn`, `insert`, `remove`, `get_mut` writes, `send_event`, `update_events`, command buffers applied with `apply_commands` (stale entities and rejected commands included). After every 100 operations it hashes `state_hash()`, the entity sequence of 3 queries and every error code returned | Yes (integer arithmetic, float bits, orders defined by this spec) |
| `schedule_ticks` (ECS-R221) | A world of 10,000 entities over 5 component types and a schedule of 3 stages and 12 systems: integration of positions from velocities (`f32` add and multiply), a seeded `Rng` resource consumed by one system, events sent and read across stages and across runs, commands that spawn and despawn about 1 % of the entities per run, and systems using `Added`, `Changed`, `With`, `Without` and `Option`. 200 runs with `update_events` between runs; `state_hash()` is recorded after every run. Executed with `run` and with `run_parallel` on pools of 1, 2, 4 and the default number of threads | Yes (IEEE 754 addition and multiplication are exactly specified and no supported target uses extended precision, [liminal_core.md](liminal_core.md) §11; the executors are equivalent by ECS-R180) |

Cross-platform bit-identity has **not been verified yet**: it is claimed for both workloads and
verified when M1-T043's tests pass on the three CI operating systems.

## 12. Open questions

Answers to questions that other specs addressed to this one:

| Question | Answer |
|---|---|
| [liminal_core.md](liminal_core.md) §12 item 5 (entities and handles) | Resolved: `Entity` follows `HandleAllocator` unchanged (LIFO reuse, retirement), ECS-R002. No change to the core spec |
| [liminal_jobs.md](liminal_jobs.md) §12 item 2 (per-worker data) | Not needed: command buffers are per system, not per worker (ECS-R134) |
| [liminal_jobs.md](liminal_jobs.md) §12 item 8 (automatic chunk size) | Not needed in `spec_version` 1: the ECS has no parallel loop of its own (item 7 below) |
| [liminal_jobs.md](liminal_jobs.md) §12 item 10 (allocation in `run_graph`) | Stays SHOULD for now: ECS-R184 is SHOULD (item 8 below) |
| [liminal_jobs.md](liminal_jobs.md) §12 item 11 (graph node results, span fields) and [liminal_telemetry.md](liminal_telemetry.md) §12 item 4 (span fields) | Not needed: systems return `()`, and spans carry the schedule or system name only (ECS-R210) |
| [liminal_reflect.md](liminal_reflect.md) §12 items 1 and 2, [liminal_math.md](liminal_math.md) §12 item 7 (`Reflect` for math and core types) | Item 1 below |

Open:

1. **`Reflect` for `liminal_math` and `liminal_core` types.** The ECS itself needs neither (its
   API and tests use primitives, arrays and `Entity`). Components of games do: `headless_boids`
   (M1-T050) stores vectors. Decision of this spec, as [liminal_reflect.md](liminal_reflect.md)
   §12 item 1 asks: option (a), `liminal_reflect` gets `liminal_math` in its `same_layer_deps`
   and implements `Reflect` for the math types, so that `liminal_math` stays free of reflection.
   For `liminal_core` types (item 2 there), the form this spec uses for `Entity` (a primitive
   shape over the stored bits, written only through `downcast_mut` of the whole type, ECS-R004)
   is one candidate. The change is outside this crate: task M1-T062 makes it (the
   `architecture.toml` edge, a `spec_version` bump of the reflect spec, the impls), and
   M1-T050 depends on it.
2. **`Entity` as a reflected primitive.** ECS-R004 gives `Entity` the shape
   `Primitive(U64)` under its own type path. `liminal_reflect` `spec_version` 1 accepts this
   (REFL-R061, REFL-R080), but no consumer exists yet. Resolve in the `liminal_serial` spec
   (M2, OQ-002): whether a serializer may treat a non-`u64` type with a primitive shape as its
   primitive, or whether reflect needs an "opaque" shape (M1-T062 may answer it first).
3. **Components without reflection.** Every component is `Typed` and hashed (ADR-0008 requires it
   for simulation state). Components that are not simulation state or cannot be reflected (GPU
   handles, interpolation caches) have no place yet. Resolve at the M3 kickoff (first
   non-simulation components: `liminal_platform`, `liminal_gpu`).
4. **No `Entity` from deferred spawns.** `CommandBuffer::spawn` returns nothing, because an ID
   reserved from several threads would depend on scheduling. A system can therefore not spawn an
   entity and refer to it in the same run. Candidates: IDs reserved per system from a block
   handed out in system index order; a command that runs a closure with `&mut World`. Resolve in
   the `liminal_scene` spec (M4 kickoff, hierarchies), or earlier if M1-T050 needs it.
5. **Exclusive systems, run conditions, system sets.** Not in `spec_version` 1. Work that needs
   `&mut World` is done between runs by the owner of the schedule. Resolve in the `liminal_app`
   spec (M1-T044).
6. **Who calls `update_events`, and when.** With one call per tick, an event is readable in the
   tick it was sent and in the next (§4). A schedule that does not run every tick (a frame
   schedule next to a fixed-step schedule) can miss events. Resolve in the `liminal_app` spec
   (M1-T044).
7. **Parallel iteration inside a system.** Systems run in parallel with each other, but one
   system iterates on one thread. A boids update over 100,000 entities is one system. Resolve:
   M1-T050 measures `headless_boids`; if it needs a parallel loop, a `spec_version` bump here
   adds one with chunk boundaries that depend on the data only
   ([liminal_jobs.md](liminal_jobs.md) §11 rule 3).
8. **Allocation in `run_parallel`.** ECS-R184 is SHOULD because it depends on JOBS-R087
   (SHOULD) and on how the executor hands borrowed systems to a reusable `JobGraph`. Resolve:
   M1-T042 reports the allocation count per run; if it is not 0, a task makes both requirements
   MUST or records why not (CODE_STYLE.md "No allocation in per-frame hot paths").
9. **What the state hash leaves out.** Freed entity slots, change ticks and event reader
   positions affect future behaviour and are not hashed (ECS-R192), so two worlds with equal
   hashes can diverge later. This does not weaken determinism tests (they compare runs that
   start from the same state), but restoring a world from saved state needs these too. Resolve
   in the `liminal_testkit` spec (M1-T047, replay) and the `liminal_serial` spec (M2).
10. **Sparse-set storage.** ADR-0008 allows it for components that are added and removed often
    "if benchmarks justify it". Not in `spec_version` 1. Resolve after M1-T052 from the
    `add_remove` comparison.
11. **Typed columns and boxed commands.** Columns are `Vec<T>` behind a private trait and the
    storage module forbids `unsafe` (§3); ECS-R115 is SHOULD because allocation-free commands
    need type-erased byte storage. Both cost dynamic dispatch or allocation. Resolve at the
    `stable` promotion from the M1-T043 and M1-T052 numbers; a change to byte storage adds
    `unsafe` to the storage module with a `spec_version` bump and its own benchmark (ADR-0018).
12. **Size of change ticks.** 16 bytes per component instance (§4). A `u32` tick halves it but
    needs wrap handling. Resolve at the `stable` promotion from memory measurements of
    `headless_boids` (M1-T050).
13. **Change detection for resources; removed components.** Not in `spec_version` 1. Resolve
    when a consumer needs them (expected: `liminal_scene`, M4 kickoff).
14. **Access by `StableTypeId`.** The probe (ADR-0019) must list an entity's components and read
    and write them without knowing their Rust types. `World` has no such methods yet. Resolve in
    the `liminal_probe` spec (M2) with a `spec_version` bump here.
15. **Resources that are not `Send + Sync`** (window and GPU handles on some platforms). Resolve
    at the M3 kickoff.
16. **loom and the M1 exit criterion.** ROADMAP.md §M1 and TESTING.md name `liminal_ecs` among
    the crates with lock-free code and loom tests. This spec has none (§10). Resolve: M1-T042
    either confirms that the executor has no shared state of its own, or adds loom tests; the
    M1 exit review (M1-T053) reads the criterion accordingly.
17. **Events must be reflected.** `Event: Typed`, so that buffered events are hashed. Events that
    carry values without reflection have no place yet. Resolve together with item 3.
18. **After a panic or a rejected command.** A system panic leaves the world's content
    unspecified (ECS-R174); rejected commands are reported after the run completed (ECS-R173),
    which is an error result for a run that did happen. Whether the app stops, logs or continues
    is decided in the `liminal_app` spec (M1-T044), which may ask for a different report form.
19. **`type_name` in errors.** Resource names in errors come from `core::any::type_name`, whose
    output can change between compiler versions (§5.9). Resolve with item 3 or 14 (a name for
    types without reflection).
20. **loom, Miri and nightly in CI.** As [liminal_jobs.md](liminal_jobs.md) §12 item 9: decided
    by M1-T018; §10 is updated if it chooses otherwise.

## 13. Maturity

Current: `planned`. Next target: `usable` (via `experimental`). Evidence is prepared by M1-T038
to M1-T043 and checked by the session that reviews M1-T043 (MATURITY.md "Promotion procedure").

- [ ] Spec follows `docs/specs/_TEMPLATE.md` and passed a spec review (REVIEW.md §2) (evidence:
      review of M1-T037).
- [ ] Compiles, all lints pass, determinism lint configuration accepted by `check-arch`,
      registered in `ci/coverage.toml` (evidence: M1-T038).
- [ ] All MUST requirements traced: `cargo xtask trace` with `maturity: usable` applied on a
      scratch copy reports every ECS MUST tested (evidence: M1-T043).
- [ ] Line coverage ≥ 85 % (layer 1 pure logic, TESTING.md) (evidence: M1-T043).
- [ ] Every public item documented; the primary entry points of §10 have runnable `# Examples`.
- [ ] Integration tests in `crates/liminal_ecs/tests/` use only the public API.
- [ ] `unsafe = "allow"`: Miri passes on the test suite, exceptions listed in §10 with reasons;
      every `unsafe` block has a `SAFETY:` comment a reviewer has checked; loom tests pass for
      any shared state M1-T042 added (§12 item 16).
- [ ] The benchmark cases of §9 are recorded with `cargo xtask bench`, including the pair of
      rows 2 and 3 (ADR-0018) (evidence: M1-T043).
- [ ] `tests/determinism.rs` with the two workloads of §11 and recorded literals, passing on the
      three CI operating systems, single- and multithreaded (ADR-0024 item 2).
- [ ] At least one example or dependent crate uses it. Open item: `liminal_app` (M1-T045) is the
      first dependent crate; M1-T043 either adds a runnable example under
      `crates/liminal_ecs/examples/` or the promotion waits for `liminal_app`.

## 14. History

| spec_version | Date | Change | By task |
|---|---|---|---|
| 1 | 2026-11-10 | Initial spec | M1-T037 |
