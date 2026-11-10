//! Integration tests of `liminal_core::hash` through the public API (spec §10, CORE-R050 to
//! CORE-R057, CORE-R060, CORE-R061).
//!
//! Oracles:
//! - XXH64: the published sanity-test vectors of the xxHash repository
//!   (`common/xxh64_vectors.rs`).
//! - Encodings (CORE-R053, CORE-R054, CORE-R061): the expected byte sequence written by hand in
//!   the test and hashed with `write_bytes`.
//! - Recorded literals (CORE-R056, CORE-R061): XXH64 (seed 0) of those hand-written byte
//!   sequences, computed by the independent Python implementation
//!   `tests/reference/kat.py` (`kat.py xxh64hex <name>=<hex of the bytes>`), which checks itself
//!   against the same published vectors.

#[path = "common/xxh64_vectors.rs"]
mod xxh64_vectors;

use core::hash::{BuildHasher, Hasher};
use std::collections::HashMap;

use liminal_core::collections::{DetHashMap, DetHashSet};
use liminal_core::handle::{Handle, RawHandle};
use liminal_core::hash::{DetHasher, FixedState, StateHash, StateHasher, state_hash_of};
use liminal_core::rng::{Rng, StreamKey};
use liminal_core::time::{Tick, TickDuration, TickRate};
use proptest::prelude::*;
use xxh64_vectors::{XXH64_SEED_0, sanity_buffer};

/// XXH64 (seed 0) of the empty input: `XSUM_XXH64_testdata[0]` of the xxHash sanity vectors.
const XXH64_OF_EMPTY: u64 = 0xEF46_DB37_51D8_E999;

/// A type that implements no trait: `Handle<Opaque>` still has a state hash.
struct Opaque;

/// The hash of a byte sequence written in one `write_bytes` call.
fn hash_of_bytes(bytes: &[u8]) -> u64 {
    let mut hasher = StateHasher::new();
    hasher.write_bytes(bytes);
    hasher.finish()
}

/// The hash after one typed write.
fn hash_of_write(write: impl FnOnce(&mut StateHasher)) -> u64 {
    let mut hasher = StateHasher::new();
    write(&mut hasher);
    hasher.finish()
}

/// `T::default()`, for unit structs too (clippy rejects `FixedState::default()` written out).
fn default_of<T: Default>() -> T {
    T::default()
}

/// Concatenates byte slices.
fn bytes(parts: &[&[u8]]) -> Vec<u8> {
    parts.concat()
}

/// Verifies: CORE-R050
#[test]
fn finish_is_xxh64_seed_0_of_the_bytes_written() {
    let longest = XXH64_SEED_0
        .iter()
        .map(|(length, _)| *length)
        .max()
        .unwrap();
    let buffer = sanity_buffer(longest);
    assert_eq!(XXH64_SEED_0.len(), 121);
    for (length, expected) in XXH64_SEED_0 {
        assert_eq!(
            hash_of_bytes(&buffer[..*length]),
            *expected,
            "length {length}"
        );
    }
}

/// Verifies: CORE-R050
#[test]
fn a_new_hasher_returns_xxh64_of_the_empty_input() {
    assert_eq!(StateHasher::new().finish(), XXH64_OF_EMPTY);
    assert_eq!(hash_of_bytes(&[]), XXH64_OF_EMPTY);
}

/// Verifies: CORE-R051
#[test]
fn every_two_way_split_and_single_byte_writes_give_the_published_hash() {
    for (length, expected) in XXH64_SEED_0.iter().filter(|(length, _)| *length <= 129) {
        let buffer = sanity_buffer(*length);
        for split in 0..=*length {
            let mut hasher = StateHasher::new();
            hasher.write_bytes(&buffer[..split]);
            hasher.write_bytes(&[]);
            hasher.write_bytes(&buffer[split..]);
            assert_eq!(hasher.finish(), *expected, "length {length}, split {split}");
        }
        let mut hasher = StateHasher::new();
        for byte in &buffer {
            hasher.write_bytes(core::slice::from_ref(byte));
        }
        assert_eq!(hasher.finish(), *expected, "length {length}, byte by byte");
    }
}

/// Verifies: CORE-R052
#[test]
fn finish_does_not_change_the_hasher() {
    let buffer = sanity_buffer(100);
    let expected = |length: usize| {
        XXH64_SEED_0
            .iter()
            .find(|(candidate, _)| *candidate == length)
            .unwrap()
            .1
    };
    let mut hasher = StateHasher::new();
    assert_eq!(hasher.finish(), expected(0));
    let mut written = 0;
    for end in [1, 7, 31, 32, 33, 64, 99, 100] {
        hasher.write_bytes(&buffer[written..end]);
        written = end;
        assert_eq!(hasher.finish(), expected(end), "after {end} bytes");
        assert_eq!(hasher.finish(), expected(end), "second finish, {end} bytes");
    }
}

proptest! {
    /// Verifies: CORE-R051, CORE-R052
    #[test]
    fn any_split_into_writes_gives_the_hash_of_one_write(
        data in proptest::collection::vec(any::<u8>(), 0..400),
        cuts in proptest::collection::vec(any::<prop::sample::Index>(), 0..12),
        finish_between in any::<bool>(),
    ) {
        let mut positions: Vec<usize> = cuts.iter().map(|cut| cut.index(data.len() + 1)).collect();
        positions.sort_unstable();
        let mut hasher = StateHasher::new();
        let mut start = 0;
        for position in positions {
            hasher.write_bytes(&data[start..position]);
            if finish_between {
                prop_assert_eq!(hasher.finish(), hash_of_bytes(&data[..position]));
            }
            start = position;
        }
        hasher.write_bytes(&data[start..]);
        prop_assert_eq!(hasher.finish(), hash_of_bytes(&data));
        prop_assert_eq!(hasher.finish(), hasher.clone().finish());
    }
}

/// One typed write per row: a name, the hash after the write, and the bytes it must append.
#[expect(
    clippy::too_many_lines,
    reason = "one table row per typed write method of CORE-R053"
)]
fn typed_write_cases(nan_32: f32, nan_64: f64) -> Vec<(&'static str, u64, Vec<u8>)> {
    vec![
        ("u8", hash_of_write(|h| h.write_u8(0xAB)), vec![0xAB]),
        (
            "u16",
            hash_of_write(|h| h.write_u16(0x1234)),
            vec![0x34, 0x12],
        ),
        (
            "u32",
            hash_of_write(|h| h.write_u32(0x1234_5678)),
            vec![0x78, 0x56, 0x34, 0x12],
        ),
        (
            "u64",
            hash_of_write(|h| h.write_u64(0x0123_4567_89AB_CDEF)),
            vec![0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45, 0x23, 0x01],
        ),
        (
            "u128",
            hash_of_write(|h| h.write_u128(0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210)),
            vec![
                0x10, 0x32, 0x54, 0x76, 0x98, 0xBA, 0xDC, 0xFE, 0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45,
                0x23, 0x01,
            ],
        ),
        ("i8", hash_of_write(|h| h.write_i8(-2)), vec![0xFE]),
        ("i16", hash_of_write(|h| h.write_i16(-2)), vec![0xFE, 0xFF]),
        (
            "i32",
            hash_of_write(|h| h.write_i32(-2)),
            vec![0xFE, 0xFF, 0xFF, 0xFF],
        ),
        (
            "i64",
            hash_of_write(|h| h.write_i64(-2)),
            bytes(&[&[0xFE], &[0xFF; 7]]),
        ),
        (
            "i128",
            hash_of_write(|h| h.write_i128(-2)),
            bytes(&[&[0xFE], &[0xFF; 15]]),
        ),
        (
            "i32 positive",
            hash_of_write(|h| h.write_i32(0x0102_0304)),
            vec![0x04, 0x03, 0x02, 0x01],
        ),
        (
            "usize as u64",
            hash_of_write(|h| h.write_usize(0x0102_0304)),
            vec![0x04, 0x03, 0x02, 0x01, 0, 0, 0, 0],
        ),
        (
            "isize as i64",
            hash_of_write(|h| h.write_isize(-2)),
            bytes(&[&[0xFE], &[0xFF; 7]]),
        ),
        (
            "bool false",
            hash_of_write(|h| h.write_bool(false)),
            vec![0x00],
        ),
        (
            "bool true",
            hash_of_write(|h| h.write_bool(true)),
            vec![0x01],
        ),
        (
            "char U+00E9",
            hash_of_write(|h| h.write_char('\u{E9}')),
            vec![0xE9, 0x00, 0x00, 0x00],
        ),
        (
            "char U+1F600",
            hash_of_write(|h| h.write_char('\u{1F600}')),
            vec![0x00, 0xF6, 0x01, 0x00],
        ),
        (
            "f32 1.5",
            hash_of_write(|h| h.write_f32(1.5)),
            vec![0x00, 0x00, 0xC0, 0x3F],
        ),
        (
            "f32 -0.0",
            hash_of_write(|h| h.write_f32(-0.0)),
            vec![0x00, 0x00, 0x00, 0x80],
        ),
        (
            "f32 NaN payload 1",
            hash_of_write(|h| h.write_f32(nan_32)),
            vec![0x01, 0x00, 0xC0, 0x7F],
        ),
        (
            "f64 1.5",
            hash_of_write(|h| h.write_f64(1.5)),
            vec![0, 0, 0, 0, 0, 0, 0xF8, 0x3F],
        ),
        (
            "f64 -0.0",
            hash_of_write(|h| h.write_f64(-0.0)),
            vec![0, 0, 0, 0, 0, 0, 0, 0x80],
        ),
        (
            "f64 NaN payload 1",
            hash_of_write(|h| h.write_f64(nan_64)),
            vec![0x01, 0, 0, 0, 0, 0, 0xF8, 0x7F],
        ),
        (
            "str h\u{E9}llo",
            hash_of_write(|h| h.write_str("h\u{E9}llo")),
            bytes(&[&[6, 0, 0, 0, 0, 0, 0, 0], b"h", &[0xC3, 0xA9], b"llo"]),
        ),
        ("str empty", hash_of_write(|h| h.write_str("")), vec![0; 8]),
    ]
}

/// Verifies: CORE-R053
#[test]
fn typed_writes_append_the_specified_bytes() {
    let nan_32 = f32::from_bits(0x7FC0_0001);
    let nan_64 = f64::from_bits(0x7FF8_0000_0000_0001);
    for (name, hash, expected) in &typed_write_cases(nan_32, nan_64) {
        assert_eq!(*hash, hash_of_bytes(expected), "{name}");
    }

    // No canonicalization: zero and negative zero, and two NaN payloads, hash differently.
    assert_ne!(
        hash_of_write(|h| h.write_f32(0.0)),
        hash_of_write(|h| h.write_f32(-0.0))
    );
    assert_ne!(
        hash_of_write(|h| h.write_f64(f64::from_bits(0x7FF8_0000_0000_0002))),
        hash_of_write(|h| h.write_f64(nan_64))
    );

    // Consecutive typed writes append to one sequence, without separators.
    let mut hasher = StateHasher::new();
    hasher.write_u16(0x1234);
    hasher.write_bool(true);
    hasher.write_str("ab");
    hasher.write_i8(-1);
    assert_eq!(
        hasher.finish(),
        hash_of_bytes(&[0x34, 0x12, 0x01, 2, 0, 0, 0, 0, 0, 0, 0, b'a', b'b', 0xFF])
    );
}

/// One `StateHash` encoding case: a name, the value's state hash, the expected bytes written by
/// hand, and the recorded XXH64 (seed 0) of those bytes. The recorded values were computed from the hex
/// form of each byte sequence by `python3 crates/liminal_core/tests/reference/kat.py xxh64hex
/// <name>=<hex>`, not by `StateHasher`.
struct Encoding {
    name: &'static str,
    hash: u64,
    bytes: Vec<u8>,
    recorded: u64,
}

fn encoding<T: StateHash + ?Sized>(
    name: &'static str,
    value: &T,
    bytes: Vec<u8>,
    recorded: u64,
) -> Encoding {
    Encoding {
        name,
        hash: state_hash_of(value),
        bytes,
        recorded,
    }
}

/// One value per `StateHash` impl of CORE-R054, with its hand-written encoding.
#[expect(
    clippy::too_many_lines,
    reason = "one table row per StateHash impl of CORE-R054; splitting it would hide the list"
)]
fn encodings() -> Vec<Encoding> {
    const LEN_3: [u8; 8] = [3, 0, 0, 0, 0, 0, 0, 0];
    const LEN_2: [u8; 8] = [2, 0, 0, 0, 0, 0, 0, 0];
    const U16_1_2_3: [u8; 6] = [1, 0, 2, 0, 3, 0];

    let raw = RawHandle::from_bits(0x0000_0002_0000_0003).expect("test setup: generation 2");
    let handle_bytes = vec![3, 0, 0, 0, 2, 0, 0, 0];

    let mut map: DetHashMap<u32, u8> = DetHashMap::new();
    map.insert(5, 1);
    map.insert(2, 9);
    let mut set: DetHashSet<u16> = DetHashSet::new();
    set.insert(7);
    set.insert(3);

    let rng = Rng::from_state([1, 2, 3, 0x0A0B_0C0D_0E0F_1011]).expect("test setup: not zero");

    vec![
        encoding("u8", &0xAB_u8, vec![0xAB], 0xC846_70DA_8B5B_5EA8),
        encoding("u16", &0x1234_u16, vec![0x34, 0x12], 0x55F8_6579_1144_52E7),
        encoding(
            "u32",
            &0x1234_5678_u32,
            vec![0x78, 0x56, 0x34, 0x12],
            0xEB51_8304_FEC0_2E06,
        ),
        encoding(
            "u64",
            &0x0123_4567_89AB_CDEF_u64,
            vec![0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45, 0x23, 0x01],
            0xEA3C_5208_1E98_43EC,
        ),
        encoding(
            "u128",
            &0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210_u128,
            vec![
                0x10, 0x32, 0x54, 0x76, 0x98, 0xBA, 0xDC, 0xFE, 0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45,
                0x23, 0x01,
            ],
            0x2DB1_FC27_338B_8DC3,
        ),
        encoding(
            "usize",
            &0x0102_0304_usize,
            vec![0x04, 0x03, 0x02, 0x01, 0, 0, 0, 0],
            0x3D40_D031_6172_2A70,
        ),
        encoding("i8", &-2_i8, vec![0xFE], 0x17E1_AAF6_7878_28AB),
        encoding("i16", &-2_i16, vec![0xFE, 0xFF], 0x9E26_0B96_F9DE_AED6),
        encoding(
            "i32",
            &-2_i32,
            vec![0xFE, 0xFF, 0xFF, 0xFF],
            0x01FA_F0C9_6F54_AC67,
        ),
        encoding(
            "i64",
            &-2_i64,
            bytes(&[&[0xFE], &[0xFF; 7]]),
            0xBAB3_B83C_F164_D97F,
        ),
        encoding(
            "i128",
            &-2_i128,
            bytes(&[&[0xFE], &[0xFF; 15]]),
            0xC2E4_AA14_D79F_0BD1,
        ),
        encoding(
            "isize",
            &-2_isize,
            bytes(&[&[0xFE], &[0xFF; 7]]),
            0xBAB3_B83C_F164_D97F,
        ),
        encoding("bool", &true, vec![0x01], 0x8A41_2781_1B21_E730),
        encoding(
            "char",
            &'\u{1F600}',
            vec![0x00, 0xF6, 0x01, 0x00],
            0xF067_FA2C_10E5_E132,
        ),
        encoding(
            "f32",
            &1.5_f32,
            vec![0x00, 0x00, 0xC0, 0x3F],
            0x4F2D_8259_5C48_3A0D,
        ),
        encoding(
            "f64",
            &-0.0_f64,
            vec![0, 0, 0, 0, 0, 0, 0, 0x80],
            0x3F42_5EAC_F015_44E0,
        ),
        encoding(
            "str",
            "h\u{E9}llo",
            bytes(&[&[6, 0, 0, 0, 0, 0, 0, 0], b"h", &[0xC3, 0xA9], b"llo"]),
            0xE98E_32AC_9C70_AD14,
        ),
        encoding(
            "String",
            &String::from("ab"),
            bytes(&[&LEN_2, b"ab"]),
            0x4A86_26E4_92E6_6169,
        ),
        encoding(
            "[T]",
            &[1_u16, 2, 3][..],
            bytes(&[&LEN_3, &U16_1_2_3]),
            0xE8EE_E958_7047_A266,
        ),
        encoding(
            "Vec<T>",
            &vec![1_u16, 2, 3],
            bytes(&[&LEN_3, &U16_1_2_3]),
            0xE8EE_E958_7047_A266,
        ),
        encoding(
            "[T; N]",
            &[1_u16, 2, 3],
            bytes(&[&LEN_3, &U16_1_2_3]),
            0xE8EE_E958_7047_A266,
        ),
        encoding(
            "empty Vec",
            &Vec::<u64>::new(),
            vec![0; 8],
            0x34C9_6ACD_CADB_1BBB,
        ),
        encoding(
            "Option None",
            &None::<u32>,
            vec![0x00],
            0xE934_A84A_DB05_2768,
        ),
        encoding(
            "Option Some",
            &Some(7_u32),
            vec![0x01, 7, 0, 0, 0],
            0x9C10_B54F_1B64_C2CC,
        ),
        encoding("&T", &&0x1234_u16, vec![0x34, 0x12], 0x55F8_6579_1144_52E7),
        encoding(
            "&&str",
            &&"ab",
            bytes(&[&LEN_2, b"ab"]),
            0x4A86_26E4_92E6_6169,
        ),
        encoding("tuple of 1", &(0xAB_u8,), vec![0xAB], 0xC846_70DA_8B5B_5EA8),
        encoding(
            "tuple of 2",
            &(0xAB_u8, 0x1234_u16),
            vec![0xAB, 0x34, 0x12],
            0xB326_84CB_5240_91D5,
        ),
        encoding(
            "tuple of 3",
            &(0xAB_u8, 0x1234_u16, true),
            vec![0xAB, 0x34, 0x12, 0x01],
            0x35C8_A143_B664_ED05,
        ),
        encoding(
            "tuple of 4",
            &(0xAB_u8, 0x1234_u16, true, "ab"),
            bytes(&[&[0xAB, 0x34, 0x12, 0x01], &LEN_2, b"ab"]),
            0x3479_7203_8FE9_CA8B,
        ),
        encoding(
            "nested",
            &vec![Some((1_u8, [2_u8])), None, Some((3, [4]))],
            bytes(&[
                &LEN_3,
                &[0x01, 1],
                &[1, 0, 0, 0, 0, 0, 0, 0, 2],
                &[0x00],
                &[0x01, 3],
                &[1, 0, 0, 0, 0, 0, 0, 0, 4],
            ]),
            0x4016_6F12_C326_4B72,
        ),
        encoding(
            "RawHandle",
            &raw,
            handle_bytes.clone(),
            0x3586_FED8_6251_53CE,
        ),
        encoding(
            "Handle<T>",
            &Handle::<Opaque>::from_raw(raw),
            handle_bytes,
            0x3586_FED8_6251_53CE,
        ),
        encoding(
            "DetHashMap",
            &map,
            bytes(&[&LEN_2, &[5, 0, 0, 0, 1], &[2, 0, 0, 0, 9]]),
            0xFDFD_F770_B6AE_59A6,
        ),
        encoding(
            "DetHashSet",
            &set,
            bytes(&[&LEN_2, &[7, 0], &[3, 0]]),
            0x968D_CAD8_4E32_60AB,
        ),
        encoding(
            "Rng",
            &rng,
            bytes(&[
                &[1, 0, 0, 0, 0, 0, 0, 0],
                &[2, 0, 0, 0, 0, 0, 0, 0],
                &[3, 0, 0, 0, 0, 0, 0, 0],
                &[0x11, 0x10, 0x0F, 0x0E, 0x0D, 0x0C, 0x0B, 0x0A],
            ]),
            0x89C4_16D8_D0B8_76D9,
        ),
        encoding(
            "StreamKey",
            &StreamKey::new(0x0A0B),
            vec![0x0B, 0x0A, 0, 0, 0, 0, 0, 0],
            0x035C_CCEF_5B99_5644,
        ),
        encoding(
            "Tick",
            &Tick::new(0x0102),
            vec![0x02, 0x01, 0, 0, 0, 0, 0, 0],
            0xC156_F93A_7EE0_5879,
        ),
        encoding(
            "TickDuration",
            &TickDuration::new(0x0102),
            vec![0x02, 0x01, 0, 0, 0, 0, 0, 0],
            0xC156_F93A_7EE0_5879,
        ),
        encoding(
            "TickRate",
            &TickRate::new(60).expect("test setup: 60 is a valid rate"),
            vec![60, 0, 0, 0],
            0xF4E8_6D3F_EAD8_74F0,
        ),
    ]
}

/// Verifies: CORE-R054
#[test]
fn state_hash_impls_write_the_specified_encoding() {
    for case in encodings() {
        assert_eq!(case.hash, hash_of_bytes(&case.bytes), "{}", case.name);
    }
}

/// Verifies: CORE-R056
#[test]
fn recorded_hashes_hold_on_this_platform() {
    for case in encodings() {
        assert_eq!(case.hash, case.recorded, "{}", case.name);
    }
    // CORE-R050 vectors, as literals.
    assert_eq!(hash_of_bytes(&sanity_buffer(1)), 0xE934_A84A_DB05_2768);
    assert_eq!(hash_of_bytes(&sanity_buffer(32)), XXH64_SEED_0[32].1);
    assert_eq!(XXH64_SEED_0[32].0, 32);
}

/// Verifies: CORE-R055
#[test]
fn state_hash_of_is_finish_of_a_new_hasher_after_state_hash() {
    fn by_hand<T: StateHash + ?Sized>(value: &T) -> u64 {
        let mut hasher = StateHasher::new();
        value.state_hash(&mut hasher);
        hasher.finish()
    }
    assert_eq!(state_hash_of(&7_u32), by_hand(&7_u32));
    assert_eq!(state_hash_of("text"), by_hand("text"));
    assert_eq!(
        state_hash_of(&[1.5_f32, -0.0][..]),
        by_hand(&[1.5_f32, -0.0][..])
    );
    assert_eq!(
        state_hash_of(&(Some(Tick::new(3)), vec![true, false])),
        by_hand(&(Some(Tick::new(3)), vec![true, false]))
    );
    // It starts from a new hasher: nothing written before the value.
    assert_eq!(state_hash_of(&0xAB_u8), hash_of_bytes(&[0xAB]));
    assert_ne!(state_hash_of(&7_u32), state_hash_of(&8_u32));
}

/// Verifies: CORE-R057
#[test]
fn the_default_hasher_is_in_the_state_of_new() {
    let mut default = StateHasher::default();
    let mut new = StateHasher::new();
    assert_eq!(default.finish(), XXH64_OF_EMPTY);
    assert_eq!(format!("{default:?}"), format!("{new:?}"));

    let buffer = sanity_buffer(100);
    for hasher in [&mut default, &mut new] {
        hasher.write_u32(7);
        hasher.write_bytes(&buffer);
        hasher.write_str("same writes");
    }
    assert_eq!(default.finish(), new.finish());
    assert_ne!(default.finish(), XXH64_OF_EMPTY);

    // Writing the sanity buffer to a default hasher gives the published hash.
    let mut published = StateHasher::default();
    published.write_bytes(&buffer);
    assert_eq!(published.finish(), XXH64_SEED_0[100].1);
    assert_eq!(XXH64_SEED_0[100].0, 100);
}

/// The fixed sequence of `Hasher` calls of the CORE-R061 tests, one call per integer method.
fn write_every_integer(hasher: &mut DetHasher) {
    hasher.write_u8(0xAB);
    hasher.write_u16(0x1234);
    hasher.write_u32(0x1234_5678);
    hasher.write_u64(0x0123_4567_89AB_CDEF);
    hasher.write_u128(0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210);
    hasher.write_usize(0x0102_0304);
    hasher.write_i8(-2);
    hasher.write_i16(-2);
    hasher.write_i32(-2);
    hasher.write_i64(-2);
    hasher.write_i128(-2);
    hasher.write_isize(-2);
    hasher.write(&[1, 2, 3]);
}

/// The bytes [`write_every_integer`] must append, written by hand.
fn every_integer_bytes() -> Vec<u8> {
    bytes(&[
        &[0xAB],
        &[0x34, 0x12],
        &[0x78, 0x56, 0x34, 0x12],
        &[0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45, 0x23, 0x01],
        &[
            0x10, 0x32, 0x54, 0x76, 0x98, 0xBA, 0xDC, 0xFE, 0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45,
            0x23, 0x01,
        ],
        &[0x04, 0x03, 0x02, 0x01, 0, 0, 0, 0],
        &[0xFE],
        &[0xFE, 0xFF],
        &[0xFE, 0xFF, 0xFF, 0xFF],
        &[0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        &[0xFE],
        &[0xFF; 15],
        &[0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        &[1, 2, 3],
    ])
}

/// Verifies: CORE-R060
#[test]
fn every_built_hasher_starts_in_the_same_state() {
    let first = FixedState.build_hasher();
    let second = default_of::<FixedState>().build_hasher();
    // The first recorded sequence is the empty one: XXH64 (seed 0) of no bytes.
    assert_eq!(first.finish(), XXH64_OF_EMPTY);
    assert_eq!(second.finish(), XXH64_OF_EMPTY);
    assert_eq!(format!("{first:?}"), format!("{second:?}"));

    // The same calls on two hashers give the same result, equal to the recorded literal
    // (computed by `kat.py xxh64hex` from the hex form of `every_integer_bytes()`).
    let (mut first, mut second) = (first, second);
    write_every_integer(&mut first);
    write_every_integer(&mut second);
    assert_eq!(first.finish(), second.finish());
    assert_eq!(first.finish(), 0xF2A8_DD91_D909_FA70);
}

/// Verifies: CORE-R060
#[test]
fn two_fixed_states_hash_values_alike_within_one_process() {
    let (a, b) = (FixedState, FixedState);
    assert_eq!(a.hash_one(7_u32), b.hash_one(7_u32));
    assert_eq!(a.hash_one("text"), b.hash_one("text"));
    assert_eq!(
        a.hash_one((1_u8, [2_u64, 3])),
        b.hash_one((1_u8, [2_u64, 3]))
    );
    assert_eq!(
        a.hash_one(vec![1_i16, -1]),
        default_of::<FixedState>().hash_one(vec![1_i16, -1])
    );
    assert_ne!(a.hash_one(7_u32), a.hash_one(8_u32));

    // A `std` map with the fixed state finds its keys (its iteration order stays undefined).
    let mut map: HashMap<String, u32, FixedState> = HashMap::with_hasher(FixedState);
    for (index, name) in ["a", "b", "c", "d"].into_iter().enumerate() {
        map.insert(name.to_string(), u32::try_from(index).unwrap());
    }
    assert_eq!(map.get("c"), Some(&2));
    assert_eq!(map.get("e"), None);
    assert_eq!(map.len(), 4);
}

/// Verifies: CORE-R061
#[test]
fn det_hasher_appends_little_endian_bytes_of_fixed_width() {
    fn after(write: impl FnOnce(&mut DetHasher)) -> u64 {
        let mut hasher = FixedState.build_hasher();
        write(&mut hasher);
        hasher.finish()
    }
    let cases: Vec<(&str, u64, Vec<u8>)> = vec![
        ("write", after(|h| h.write(&[9, 8, 7])), vec![9, 8, 7]),
        ("write empty", after(|h| h.write(&[])), vec![]),
        ("u8", after(|h| h.write_u8(0xAB)), vec![0xAB]),
        ("u16", after(|h| h.write_u16(0x1234)), vec![0x34, 0x12]),
        (
            "u32",
            after(|h| h.write_u32(0x1234_5678)),
            vec![0x78, 0x56, 0x34, 0x12],
        ),
        (
            "u64",
            after(|h| h.write_u64(0x0123_4567_89AB_CDEF)),
            vec![0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45, 0x23, 0x01],
        ),
        (
            "u128",
            after(|h| h.write_u128(0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210)),
            vec![
                0x10, 0x32, 0x54, 0x76, 0x98, 0xBA, 0xDC, 0xFE, 0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45,
                0x23, 0x01,
            ],
        ),
        (
            "usize: 8 bytes",
            after(|h| h.write_usize(0x0102_0304)),
            vec![0x04, 0x03, 0x02, 0x01, 0, 0, 0, 0],
        ),
        ("i8", after(|h| h.write_i8(-2)), vec![0xFE]),
        ("i16", after(|h| h.write_i16(-2)), vec![0xFE, 0xFF]),
        (
            "i32",
            after(|h| h.write_i32(-2)),
            vec![0xFE, 0xFF, 0xFF, 0xFF],
        ),
        (
            "i64",
            after(|h| h.write_i64(-2)),
            bytes(&[&[0xFE], &[0xFF; 7]]),
        ),
        (
            "i128",
            after(|h| h.write_i128(-2)),
            bytes(&[&[0xFE], &[0xFF; 15]]),
        ),
        (
            "isize: 8 bytes",
            after(|h| h.write_isize(-2)),
            bytes(&[&[0xFE], &[0xFF; 7]]),
        ),
    ];
    for (name, hash, expected) in &cases {
        assert_eq!(*hash, hash_of_bytes(expected), "{name}");
    }

    // The whole sequence: finish is XXH64 (seed 0) of the appended bytes, and equals the
    // literal recorded from `kat.py xxh64hex` for the hand-written bytes.
    let mut hasher = FixedState.build_hasher();
    write_every_integer(&mut hasher);
    assert_eq!(hasher.finish(), hash_of_bytes(&every_integer_bytes()));
    assert_eq!(hasher.finish(), 0xF2A8_DD91_D909_FA70);

    // `finish` does not end the sequence, and a clone continues independently.
    let mut clone = hasher.clone();
    clone.write_u8(1);
    hasher.write_u8(1);
    assert_eq!(clone.finish(), hasher.finish());
    assert_eq!(
        hasher.finish(),
        hash_of_bytes(&bytes(&[&every_integer_bytes(), &[1]]))
    );

    // A second recorded sequence: `write_u32(7)`, `write_usize(2)` is 07 00 00 00, then 8 bytes.
    let mut short = FixedState.build_hasher();
    short.write_u32(7);
    short.write_usize(2);
    assert_eq!(short.finish(), 0xABC2_D292_8B35_AC84);
}
