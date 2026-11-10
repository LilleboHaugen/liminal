//! Determinism tests of `liminal_reflect` (ADR-0024, spec §11, REFL-R130 to REFL-R132).
//!
//! Each workload runs a seeded sequence of operations on derived test types, feeds every result
//! into a `StateHasher` and returns the final hash. Each test runs its workload twice in this
//! process and compares the two hashes with each other and with a hash literal recorded here.
//! The spec claims cross-platform bit-identity for all three workloads, so CI checks the same
//! literals on macOS (arm64), Windows (`x86_64`) and Linux (`x86_64`).
//!
//! The literals were recorded from the first run of these tests (macOS arm64, 2026-11-10,
//! M1-T032). They are oracles (AGENTS.md rule 5): changing a workload's parameters, a test type,
//! the hasher or the encoding changes them, and needs a journal entry saying why. The test types
//! are derived, so their default paths start with `determinism::`.

use core::num::NonZeroU32;

use liminal_core::error::EngineError;
use liminal_core::hash::StateHasher;
use liminal_core::rng::{Rng, StreamKey};
use liminal_reflect::hash::{reflect_state_hash, write_reflect};
use liminal_reflect::value::{Reflect, Typed};

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

fn below(rng: &mut Rng, bound: u32) -> u32 {
    rng.below_u32(NonZeroU32::new(bound).expect("test setup: the bound is not 0"))
}

/// A struct with every `PrimitiveKind`.
#[derive(Clone, Reflect)]
struct Primitives {
    b: bool,
    u8: u8,
    u16: u16,
    u32: u32,
    u64: u64,
    u128: u128,
    usize: usize,
    i8: i8,
    i16: i16,
    i32: i32,
    i64: i64,
    i128: i128,
    isize: isize,
    f32: f32,
    f64: f64,
    c: char,
    s: String,
}

/// An enum with unit, tuple and struct variants.
#[derive(Clone, Reflect)]
enum Behaviour {
    Rest,
    Wander(f32, u16),
    Flee { from: u32, panic: bool },
}

/// A list element.
#[derive(Clone, Reflect)]
struct Point(i32, i32);

/// The value of workloads 1 and 2.
#[derive(Clone, Reflect)]
struct Record {
    primitives: Primitives,
    behaviour: Behaviour,
    maybe: Option<i64>,
    points: Vec<Point>,
    triple: [u16; 3],
}

/// A generic derived type.
#[derive(Reflect)]
struct Both<T, U> {
    left: T,
    right: U,
}

/// A derived type with a `type_path` override.
#[derive(Reflect)]
#[reflect(type_path = "determinism_override::Renamed")]
struct Renamed {
    value: u8,
}

const STRINGS: [&str; 4] = ["", "boid", "grüße", "日本語 ✓"];

fn primitives(rng: &mut Rng, index: u32) -> Primitives {
    // Every 7th value has a NaN with a payload and -0.0 (no canonicalization, REFL-R100).
    let special = index.is_multiple_of(7);
    Primitives {
        b: rng.next_u32() & 1 == 1,
        u8: rng.next_u32().to_le_bytes()[0],
        u16: u16::from_le_bytes([rng.next_u32().to_le_bytes()[0], 3]),
        u32: rng.next_u32(),
        u64: rng.next_u64(),
        u128: (u128::from(rng.next_u64()) << 64) | u128::from(rng.next_u64()),
        usize: usize::try_from(rng.next_u32()).expect("test setup: 64-bit targets"),
        i8: rng.next_u32().to_le_bytes()[1].cast_signed(),
        i16: i16::from_le_bytes([rng.next_u32().to_le_bytes()[0], 0x80]),
        i32: rng.next_u32().cast_signed(),
        i64: rng.next_u64().cast_signed(),
        i128: ((u128::from(rng.next_u64()) << 64) | u128::from(rng.next_u64())).cast_signed(),
        isize: isize::try_from(rng.next_u32().cast_signed()).expect("test setup: 64-bit targets"),
        f32: if special {
            f32::from_bits(0x7FC0_1234)
        } else {
            rng.next_f32()
        },
        f64: if special { -0.0 } else { rng.next_f64() },
        c: char::from_u32(0x20 + below(rng, 0x2000)).unwrap_or('?'),
        s: STRINGS[below(rng, 4) as usize].to_string(),
    }
}

fn behaviour(rng: &mut Rng) -> Behaviour {
    match below(rng, 3) {
        0 => Behaviour::Rest,
        1 => Behaviour::Wander(rng.next_f32(), u16::from_le_bytes([7, 1])),
        _ => Behaviour::Flee {
            from: rng.next_u32(),
            panic: rng.next_u32() & 1 == 0,
        },
    }
}

fn record(rng: &mut Rng, index: u32) -> Record {
    let primitives = primitives(rng, index);
    let behaviour = behaviour(rng);
    let maybe = (rng.next_u32() & 1 == 1).then(|| rng.next_u64().cast_signed());
    let points = (0..below(rng, 4))
        .map(|_| Point(rng.next_u32().cast_signed(), rng.next_u32().cast_signed()))
        .collect();
    let triple = [1, 2, 3]
        .map(|n: u16| n.wrapping_mul(u16::from_le_bytes([rng.next_u32().to_le_bytes()[0], 0])));
    Record {
        primitives,
        behaviour,
        maybe,
        points,
        triple,
    }
}

/// REFL-R130: 1,000 generated values; each value's `reflect_state_hash` is hashed.
fn reflect_hash_values() -> u64 {
    let mut rng = Rng::new(42, StreamKey::from_name("determinism.reflect_hash_values"));
    let mut hasher = StateHasher::new();
    for index in 0..1000 {
        let value = record(&mut rng, index);
        hasher.write_u64(reflect_state_hash(&value));
    }
    hasher.finish()
}

/// Field names of `Record` and of its nested values, by depth: `(outer, inner)`; inner names
/// are requested from the outer field's value. `scratch` and `0`/`1` on structs are errors.
const WRITES: [(&str, &str); 12] = [
    ("primitives", "u32"),
    ("primitives", "f32"),
    ("primitives", "s"),
    ("primitives", "c"),
    ("primitives", "scratch"),
    ("behaviour", "0"),
    ("behaviour", "1"),
    ("behaviour", "from"),
    ("behaviour", "panic"),
    ("maybe", "0"),
    ("points", "0"),
    ("triple", "0"),
];

/// Writes one generated value by name; returns the error code text, if any.
fn write_field(value: &mut dyn Reflect, outer: &str, inner: &str, rng: &mut Rng) -> Option<String> {
    let target = match value
        .field_mut(outer)
        .and_then(|field| field.field_mut(inner))
    {
        Ok(target) => target,
        Err(error) => return Some(error.code().to_string()),
    };
    let result = match inner {
        "u32" | "from" => target.downcast_mut::<u32>().map(|x| *x = rng.next_u32()),
        "f32" | "0" => target.downcast_mut::<f32>().map(|x| *x = rng.next_f32()),
        "s" => target
            .downcast_mut::<String>()
            .map(|x| *x = STRINGS[below(rng, 4) as usize].to_string()),
        "c" => target.downcast_mut::<char>().map(|x| *x = 'λ'),
        "panic" => target.downcast_mut::<bool>().map(|x| *x = !*x),
        _ => target.downcast_mut::<u16>().map(|x| *x = 9),
    };
    result.err().map(|error| error.code().to_string())
}

/// REFL-R131: 10,000 seeded writes by field name into 64 values; every error code is hashed;
/// the whole collection is hashed with `write_reflect` every 100 writes.
fn reflect_field_writes() -> u64 {
    let mut rng = Rng::new(7, StreamKey::from_name("determinism.reflect_field_writes"));
    let mut values: Vec<Record> = (0..64).map(|index| record(&mut rng, index)).collect();
    let mut hasher = StateHasher::new();
    for write in 1..=10_000_u32 {
        let index = below(&mut rng, 64) as usize;
        let (outer, inner) = WRITES[below(&mut rng, 12) as usize];
        let slot = values
            .as_mut_slice()
            .get_mut(index)
            .expect("test setup: index below 64");
        match write_field(slot, outer, inner, &mut rng) {
            Some(code) => hasher.write_str(&code),
            None => hasher.write_u8(0),
        }
        if write.is_multiple_of(100) {
            write_reflect(&values, &mut hasher);
        }
    }
    hasher.finish()
}

/// REFL-R132: type paths and stable type IDs in a fixed list order.
fn stable_type_ids() -> u64 {
    let ids = [
        (bool::type_path(), bool::stable_type_id()),
        (u8::type_path(), u8::stable_type_id()),
        (u16::type_path(), u16::stable_type_id()),
        (u32::type_path(), u32::stable_type_id()),
        (u64::type_path(), u64::stable_type_id()),
        (u128::type_path(), u128::stable_type_id()),
        (usize::type_path(), usize::stable_type_id()),
        (i8::type_path(), i8::stable_type_id()),
        (i16::type_path(), i16::stable_type_id()),
        (i32::type_path(), i32::stable_type_id()),
        (i64::type_path(), i64::stable_type_id()),
        (i128::type_path(), i128::stable_type_id()),
        (isize::type_path(), isize::stable_type_id()),
        (f32::type_path(), f32::stable_type_id()),
        (f64::type_path(), f64::stable_type_id()),
        (char::type_path(), char::stable_type_id()),
        (String::type_path(), String::stable_type_id()),
        (<Vec<u8>>::type_path(), <Vec<u8>>::stable_type_id()),
        (
            <Option<String>>::type_path(),
            <Option<String>>::stable_type_id(),
        ),
        (<[f32; 4]>::type_path(), <[f32; 4]>::stable_type_id()),
        (Record::type_path(), Record::stable_type_id()),
        (
            <Both<u8, Point>>::type_path(),
            <Both<u8, Point>>::stable_type_id(),
        ),
        (Renamed::type_path(), Renamed::stable_type_id()),
    ];
    let mut hasher = StateHasher::new();
    for (path, id) in ids {
        hasher.write_str(&path);
        hasher.write_u64(id.value());
    }
    hasher.finish()
}

/// Verifies: REFL-R130
#[test]
fn reflect_hash_values_is_deterministic() {
    assert_deterministic(
        "reflect_hash_values",
        reflect_hash_values,
        0x38D6_86D1_3D43_BAEA,
    );
}

/// Verifies: REFL-R131
#[test]
fn reflect_field_writes_is_deterministic() {
    assert_deterministic(
        "reflect_field_writes",
        reflect_field_writes,
        0x67DD_8493_ACDC_4DE1,
    );
}

/// Verifies: REFL-R132
#[test]
fn stable_type_ids_is_deterministic() {
    assert_eq!(
        <Both<u8, Point>>::type_path(),
        "determinism::Both<u8, determinism::Point>"
    );
    assert_eq!(Renamed::type_path(), "determinism_override::Renamed");
    assert_deterministic("stable_type_ids", stable_type_ids, 0x41D7_3B88_CD7A_142A);
}
