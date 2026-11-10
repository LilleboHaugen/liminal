//! Determinism tests of `liminal_core` (ADR-0024, spec §11, CORE-R130 to CORE-R133).
//!
//! Each workload is a function that runs a seeded sequence of operations, feeds every result
//! into a `StateHasher` and returns the final hash. Each test runs its workload twice in this
//! process, compares the two hashes with each other and with a hash literal recorded here. The
//! spec claims cross-platform bit-identity for all four workloads, so CI checks the same
//! literals on macOS (arm64), Windows (`x86_64`) and Linux (`x86_64`).
//!
//! The literals were recorded from the first run of these tests (macOS arm64, 2026-11-10). The
//! `rng_streams` literal is also the output of the independent Python implementation
//! (`python3 crates/liminal_core/tests/reference/kat.py rng-streams`). They are oracles
//! (AGENTS.md rule 5): changing a workload's parameters, the hasher or an encoding changes
//! them, and needs a journal entry saying why.

use core::num::{NonZeroU32, NonZeroU64};

use liminal_core::collections::DetHashMap;
use liminal_core::error::{CoreError, EngineError};
use liminal_core::handle::{Handle, HandleMap};
use liminal_core::hash::{StateHash, StateHasher};
use liminal_core::rng::{Rng, StreamKey};
use liminal_core::time::{TickDuration, TickRate};

fn nz32(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).expect("test setup: the bound is not 0")
}

fn nz64(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).expect("test setup: the bound is not 0")
}

/// Runs `workload` twice and checks both hashes against each other and against `recorded`.
fn assert_deterministic(name: &str, workload: fn() -> u64, recorded: u64) {
    let first = workload();
    let second = workload();
    assert_eq!(first, second, "{name}: two runs in one process differ");
    assert_eq!(
        first, recorded,
        "{name}: hash {first:#018X} differs from the recorded {recorded:#018X}"
    );
}

/// 3 world seeds × 3 stream keys, 1,000 rounds of every `Rng` output method; every result is
/// hashed (floats by bits), then the generator itself.
fn rng_streams() -> u64 {
    const BOUNDS_32: [u32; 5] = [1, 6, 1000, 0x8000_0001, u32::MAX];
    const BOUNDS_64: [u64; 5] = [1, 6, 1000, 0x8000_0000_0000_0001, u64::MAX];
    let keys = [
        StreamKey::new(0),
        StreamKey::new(u64::MAX),
        StreamKey::from_name("determinism.rng_streams"),
    ];
    let mut hasher = StateHasher::new();
    for world_seed in [0, 42, u64::MAX] {
        for key in keys {
            let mut rng = Rng::new(world_seed, key);
            for round in 0..1000 {
                hasher.write_u64(rng.next_u64());
                hasher.write_u32(rng.next_u32());
                hasher.write_u32(rng.below_u32(nz32(BOUNDS_32[round % 5])));
                hasher.write_u64(rng.below_u64(nz64(BOUNDS_64[round % 5])));
                hasher.write_f32(rng.next_f32());
                hasher.write_f64(rng.next_f64());
            }
            rng.state_hash(&mut hasher);
        }
    }
    hasher.finish()
}

/// 10,000 seeded insert/remove operations on a `HandleMap<u64>` with 48 slots: 50 % inserts,
/// 40 % removes of a live handle, 10 % removes of any handle issued so far (usually stale).
/// Every returned handle, value and error code is hashed, then the final map.
fn handle_churn() -> u64 {
    let mut rng = Rng::new(7, StreamKey::from_name("determinism.handle_churn"));
    let mut map: HandleMap<u64> = HandleMap::with_slot_limit(48);
    let mut issued: Vec<Handle<u64>> = Vec::new();
    let mut live: Vec<Handle<u64>> = Vec::new();
    let (mut exhausted, mut stale_removes) = (0_u32, 0_u32);
    let mut hasher = StateHasher::new();
    let pick = |rng: &mut Rng, len: usize| {
        let bound = nz64(u64::try_from(len).expect("test setup: usize fits in u64"));
        usize::try_from(rng.below_u64(bound)).expect("test setup: the value is below a usize")
    };

    for _ in 0..10_000 {
        match rng.below_u32(nz32(10)) {
            0..=4 => {
                let value = rng.next_u64();
                match map.insert(value) {
                    Ok(handle) => {
                        hasher.write_u8(0);
                        handle.state_hash(&mut hasher);
                        issued.push(handle);
                        live.push(handle);
                    }
                    Err(error) => {
                        assert_eq!(error, CoreError::HandleSlotsExhausted { slot_limit: 48 });
                        hasher.write_u8(1);
                        hasher.write_str(&error.code().to_string());
                        exhausted += 1;
                    }
                }
            }
            5..=8 if !live.is_empty() => {
                let handle = live.swap_remove(pick(&mut rng, live.len()));
                hasher.write_u8(2);
                map.remove(handle).state_hash(&mut hasher);
            }
            _ if !issued.is_empty() => {
                let handle = issued[pick(&mut rng, issued.len())];
                let removed = map.remove(handle);
                if removed.is_some() {
                    live.retain(|candidate| *candidate != handle);
                } else {
                    stale_removes += 1;
                }
                hasher.write_u8(3);
                removed.state_hash(&mut hasher);
            }
            _ => hasher.write_u8(4),
        }
    }

    // The workload covers what the spec names: exhaustion and stale removes.
    assert!(exhausted > 100, "{exhausted} exhausted inserts");
    assert!(stale_removes > 100, "{stale_removes} stale removes");
    assert_eq!(map.iter().count(), live.len());

    hasher.write_u32(map.len());
    for (handle, value) in map.iter() {
        handle.state_hash(&mut hasher);
        hasher.write_u64(*value);
    }
    hasher.finish()
}

/// 10,000 seeded operations on a `DetHashMap<u64, u64>` over 300 keys: 50 % inserts, 30 %
/// removes, 20 % lookups. Every result is hashed, and the whole map every 100 operations.
fn det_map_churn() -> u64 {
    let mut rng = Rng::new(11, StreamKey::from_name("determinism.det_map_churn"));
    let mut map: DetHashMap<u64, u64> = DetHashMap::new();
    let mut hasher = StateHasher::new();
    for operation in 1..=10_000 {
        let key = rng.below_u64(nz64(300));
        match rng.below_u32(nz32(10)) {
            0..=4 => map.insert(key, rng.next_u64()).state_hash(&mut hasher),
            5..=7 => map.remove(&key).state_hash(&mut hasher),
            _ => map.get(&key).state_hash(&mut hasher),
        }
        if operation % 100 == 0 {
            map.state_hash(&mut hasher);
        }
    }
    assert!(map.len() > 100, "{} entries", map.len());
    hasher.finish()
}

/// `dt_secs_f32` and `dt_secs_f64` for every tick rate 1..=1000, and `ticks_to_secs_f64` and
/// `ticks_from_millis` for fixed counts; floats are hashed by bits.
fn time_conversions() -> u64 {
    const COUNTS: [u64; 10] = [
        0,
        1,
        59,
        60,
        61,
        1000,
        1 << 53,
        (1 << 53) + 1,
        u64::MAX - 1,
        u64::MAX,
    ];
    let mut hasher = StateHasher::new();
    for ticks_per_second in 1..=1000 {
        let rate = TickRate::new(ticks_per_second).expect("test setup: rates 1..=1000 are valid");
        rate.state_hash(&mut hasher);
        hasher.write_f32(rate.dt_secs_f32());
        hasher.write_f64(rate.dt_secs_f64());
        for count in COUNTS {
            hasher.write_f64(rate.ticks_to_secs_f64(TickDuration::new(count)));
            rate.ticks_from_millis(count).state_hash(&mut hasher);
        }
    }
    hasher.finish()
}

/// Verifies: CORE-R130
#[test]
fn rng_streams_is_deterministic() {
    assert_deterministic("rng_streams", rng_streams, 0x51B4_4F7B_1B0B_7FA7);
}

/// Verifies: CORE-R131
#[test]
fn handle_churn_is_deterministic() {
    assert_deterministic("handle_churn", handle_churn, 0x9755_282C_D3F5_78A9);
}

/// Verifies: CORE-R132
#[test]
fn det_map_churn_is_deterministic() {
    assert_deterministic("det_map_churn", det_map_churn, 0xABAD_7D9C_A877_1F95);
}

/// Verifies: CORE-R133
#[test]
fn time_conversions_is_deterministic() {
    assert_deterministic("time_conversions", time_conversions, 0xFA1B_6990_98B4_2ABA);
}
