//! A small seeded simulation built only on `liminal_core`: handles for the things in the world,
//! one random stream per consumer, an ordered map for derived state, ticks for time, and a
//! state hash to prove that two runs with the same seed are identical.
//!
//! Run with `cargo run -p liminal_core --example seeded_world`. It prints one line per run and
//! exits with an error if two runs with the same seed differ or two different seeds agree.

#![allow(
    clippy::print_stdout,
    reason = "an example reports its result on standard output"
)]

use core::num::NonZeroU32;

use liminal_core::collections::DetHashMap;
use liminal_core::error::CoreError;
use liminal_core::handle::{Handle, HandleMap};
use liminal_core::hash::{StateHash, StateHasher};
use liminal_core::rng::{Rng, StreamKey};
use liminal_core::time::{Tick, TickRate};

/// Side length of the square grid the walkers move on, in cells.
const GRID: u32 = 16;

/// A walker on the grid.
struct Walker {
    x: u32,
    y: u32,
    /// The tick at which the walker is removed.
    expires: Tick,
}

impl StateHash for Walker {
    fn state_hash(&self, hasher: &mut StateHasher) {
        self.x.state_hash(hasher);
        self.y.state_hash(hasher);
        self.expires.state_hash(hasher);
    }
}

/// The whole simulation state.
struct World {
    tick: Tick,
    walkers: HandleMap<Walker>,
    /// Number of walkers per occupied cell `(x, y)`, in the order cells were first occupied.
    occupancy: DetHashMap<(u32, u32), u32>,
    spawn_rng: Rng,
    move_rng: Rng,
}

impl World {
    fn new(world_seed: u64) -> World {
        World {
            tick: Tick::ZERO,
            walkers: HandleMap::with_slot_limit(64),
            occupancy: DetHashMap::new(),
            // One stream per consumer: changing how walkers move does not change where they
            // spawn.
            spawn_rng: Rng::new(world_seed, StreamKey::from_name("seeded_world.spawn")),
            move_rng: Rng::new(world_seed, StreamKey::from_name("seeded_world.move")),
        }
    }

    /// Advances the simulation by one fixed step.
    fn step(&mut self, rate: TickRate) -> Result<(), CoreError> {
        let grid = NonZeroU32::new(GRID).expect("invariant: the grid is not empty");
        let four = NonZeroU32::new(4).expect("invariant: 4 is not zero");

        // Spawn one walker every tenth of a second, while slots are free. A walker lives for
        // 1.5 to 2.5 seconds.
        let spawn_interval = rate.ticks_from_millis(100).count().max(1);
        if self.tick.index().is_multiple_of(spawn_interval) {
            let lifetime_millis = 1500 + u64::from(self.spawn_rng.below_u32(four)) * 250;
            let walker = Walker {
                x: self.spawn_rng.below_u32(grid),
                y: self.spawn_rng.below_u32(grid),
                expires: self
                    .tick
                    .checked_add(rate.ticks_from_millis(lifetime_millis))
                    .unwrap_or(Tick::new(u64::MAX)),
            };
            match self.walkers.insert(walker) {
                // A full map is an expected state here: skip this spawn.
                Ok(_) | Err(CoreError::HandleSlotsExhausted { .. }) => {}
                Err(other) => return Err(other),
            }
        }

        // Move every walker one cell in a random direction, wrapping at the edges. `iter_mut`
        // visits walkers in ascending slot order, so the draws are assigned deterministically.
        for (_, walker) in self.walkers.iter_mut() {
            match self.move_rng.below_u32(four) {
                0 => walker.x = (walker.x + 1) % GRID,
                1 => walker.x = (walker.x + GRID - 1) % GRID,
                2 => walker.y = (walker.y + 1) % GRID,
                _ => walker.y = (walker.y + GRID - 1) % GRID,
            }
        }

        // Remove expired walkers. Their handles become stale; the slots are reused.
        let expired: Vec<Handle<Walker>> = self
            .walkers
            .iter()
            .filter(|(_, walker)| walker.expires <= self.tick)
            .map(|(handle, _)| handle)
            .collect();
        for handle in expired {
            self.walkers.remove(handle);
        }

        // Derived state: how many walkers stand on each occupied cell.
        self.occupancy.clear();
        for (_, walker) in self.walkers.iter() {
            let cell = (walker.x, walker.y);
            let count = self.occupancy.get(&cell).copied().unwrap_or(0);
            self.occupancy.insert(cell, count + 1);
        }

        self.tick = self.tick.next();
        Ok(())
    }
}

impl StateHash for World {
    fn state_hash(&self, hasher: &mut StateHasher) {
        self.tick.state_hash(hasher);
        self.walkers.len().state_hash(hasher);
        for (handle, walker) in self.walkers.iter() {
            handle.state_hash(hasher);
            walker.state_hash(hasher);
        }
        self.occupancy.state_hash(hasher);
        self.spawn_rng.state_hash(hasher);
        self.move_rng.state_hash(hasher);
    }
}

/// Runs the simulation for `seconds` simulated seconds and returns the hash of every step's
/// state and the number of walkers at the end.
fn run(world_seed: u64, seconds: u32) -> Result<(u64, u32), CoreError> {
    let rate = TickRate::DEFAULT;
    let mut world = World::new(world_seed);
    let mut hasher = StateHasher::new();
    for _ in 0..seconds * rate.ticks_per_second() {
        world.step(rate)?;
        world.state_hash(&mut hasher);
    }
    Ok((hasher.finish(), world.walkers.len()))
}

fn main() -> Result<(), CoreError> {
    let (first, walkers) = run(42, 5)?;
    let (second, _) = run(42, 5)?;
    let (other, _) = run(43, 5)?;
    println!("seed 42: state hash {first:#018x}, {walkers} walkers after 300 ticks");
    println!("seed 42: state hash {second:#018x} (second run)");
    println!("seed 43: state hash {other:#018x}");
    assert_eq!(first, second, "two runs with the same seed differ");
    assert_ne!(first, other, "two different seeds give the same states");
    Ok(())
}
