//! Integration tests of `liminal_core::rng` through the public API (spec §10, CORE-R090 to
//! CORE-R098).
//!
//! Oracles:
//! - xoshiro256\*\* and `SplitMix64`: the published reference outputs in the `reference` tests of
//!   `rand_xoshiro` (github.com/rust-random/rngs, commit 84683fca, files
//!   `rand_xoshiro/src/xoshiro256starstar.rs` and `splitmix64.rs`), which state that the values
//!   "were produced with the reference implementation" (`xoshiro256starstar.c`, `splitmix64.c`
//!   of prng.di.unimi.it).
//! - Seeding, bounded integers and floats: literals computed by the independent Python
//!   implementation `tests/reference/kat.py` (commands named at each table), which checks itself
//!   against the published vectors above and against the xxHash sanity vectors.
//! - Test-local reference `splitmix64` (below), written from CORE-R091 and checked here against
//!   the 50 published outputs.

use core::num::{NonZeroU32, NonZeroU64};
use std::collections::BTreeSet;

use liminal_core::error::{CoreError, EngineError};
use liminal_core::hash::{StateHasher, state_hash_of};
use liminal_core::rng::{Rng, StreamKey};
use proptest::prelude::*;

/// Published outputs of xoshiro256\*\* for the state `[1, 2, 3, 4]` (`rand_xoshiro`, test
/// `reference`).
const XOSHIRO_1_2_3_4: [u64; 10] = [
    11_520,
    0,
    1_509_978_240,
    1_215_971_899_390_074_240,
    1_216_172_134_540_287_360,
    607_988_272_756_665_600,
    16_172_922_978_634_559_625,
    8_476_171_486_693_032_832,
    10_595_114_339_597_558_777,
    2_904_607_092_377_533_576,
];

/// Published outputs of `SplitMix64` for the seed 1477776061723855037 (`rand_xoshiro`, test
/// `reference`).
const SPLITMIX_SEED: u64 = 1_477_776_061_723_855_037;
const SPLITMIX_OUTPUTS: [u64; 50] = [
    1_985_237_415_132_408_290,
    2_979_275_885_539_914_483,
    13_511_426_838_097_143_398,
    8_488_337_342_461_049_707,
    15_141_737_807_933_549_159,
    17_093_170_987_380_407_015,
    16_389_528_042_912_955_399,
    13_177_319_091_862_933_652,
    10_841_969_400_225_389_492,
    17_094_824_097_954_834_098,
    3_336_622_647_361_835_228,
    9_678_412_372_263_018_368,
    11_111_587_619_974_030_187,
    7_882_215_801_036_322_410,
    5_709_234_165_213_761_869,
    7_799_681_907_651_786_826,
    4_616_320_717_312_661_886,
    4_251_077_652_075_509_767,
    7_836_757_050_122_171_900,
    5_054_003_328_188_417_616,
    12_919_285_918_354_108_358,
    16_477_564_761_813_870_717,
    5_124_667_218_451_240_549,
    18_099_554_314_556_827_626,
    7_603_784_838_804_469_118,
    6_358_551_455_431_362_471,
    3_037_176_434_532_249_502,
    3_217_550_417_701_719_149,
    9_958_699_920_490_216_947,
    5_965_803_675_992_506_258,
    12_000_828_378_049_868_312,
    12_720_568_162_811_471_118,
    245_696_019_213_873_792,
    8_351_371_993_958_923_852,
    14_378_754_021_282_935_786,
    5_655_432_093_647_472_106,
    5_508_031_680_350_692_005,
    8_515_198_786_865_082_103,
    6_287_793_597_487_164_412,
    14_963_046_237_722_101_617,
    3_630_795_823_534_910_476,
    8_422_285_279_403_485_710,
    10_554_287_778_700_714_153,
    10_871_906_555_720_704_584,
    8_659_066_966_120_258_468,
    9_420_238_805_069_527_062,
    10_338_115_333_623_340_156,
    13_514_802_760_105_037_173,
    14_635_952_304_031_724_449,
    15_419_692_541_594_102_413,
];

/// Bounds of the `below_u32` and `below_u64` known answers (`BOUNDS32`, `BOUNDS64` of
/// `kat.py`): 1, small values, powers of two, and bounds just above 2^31 / 2^63 and at the
/// maximum, where about half of the draws are rejected.
const BOUNDS_32: [u32; 7] = [1, 2, 6, 1000, 0x8000_0000, 0x8000_0001, 0xFFFF_FFFF];
const BOUNDS_64: [u64; 6] = [
    1,
    6,
    1000,
    0x8000_0000_0000_0000,
    0x8000_0000_0000_0001,
    0xFFFF_FFFF_FFFF_FFFF,
];

/// The outputs of one round of calls: `next_u64`, `next_u32`, `below_u32` for each of
/// `BOUNDS_32`, `below_u64` for each of `BOUNDS_64`, `next_f32`, `next_f64` (floats as bits).
struct Round {
    next_u64: u64,
    next_u32: u32,
    below_u32: [u32; 7],
    below_u64: [u64; 6],
    next_f32_bits: u32,
    next_f64_bits: u64,
}

/// `python3 crates/liminal_core/tests/reference/kat.py methods 42 7 3`
const METHODS_SEED_42_KEY_7: [Round; 3] = [
    Round {
        next_u64: 0xE4EF_D8DE_30CE_38F0,
        next_u32: 0x4BF5_D555,
        below_u32: [
            0x0000_0000,
            0x0000_0000,
            0x0000_0002,
            0x0000_02F7,
            0x3EE2_2CFE,
            0x6C9E_2CBA,
            0x851F_F0A8,
        ],
        below_u64: [
            0x0000_0000_0000_0000,
            0x0000_0000_0000_0003,
            0x0000_0000_0000_03D1,
            0x619C_E9CE_CD46_B80D,
            0x6549_E9AD_C562_54BF,
            0xE361_A9BC_33E7_1774,
        ],
        next_f32_bits: 0x3E45_42A4,
        next_f64_bits: 0x3FDF_402D_691D_F0E4,
    },
    Round {
        next_u64: 0x1A00_E79D_78FE_0F13,
        next_u32: 0xAB15_DA02,
        below_u32: [
            0x0000_0000,
            0x0000_0001,
            0x0000_0005,
            0x0000_0081,
            0x4A56_EFD1,
            0x100A_C63C,
            0xDB37_2E27,
        ],
        below_u64: [
            0x0000_0000_0000_0000,
            0x0000_0000_0000_0005,
            0x0000_0000_0000_02BE,
            0x473F_2A9D_A7A2_2C0B,
            0x5518_EB4C_57A1_1B25,
            0xC77A_AF73_960B_4239,
        ],
        next_f32_bits: 0x3D76_7E30,
        next_f64_bits: 0x3F71_9BA5_5127_BF00,
    },
    Round {
        next_u64: 0x310E_B750_8745_78F3,
        next_u32: 0xFBDA_BF9E,
        below_u32: [
            0x0000_0000,
            0x0000_0001,
            0x0000_0003,
            0x0000_01F7,
            0x0089_8CF1,
            0x4734_5EAA,
            0xA68A_AEB3,
        ],
        below_u64: [
            0x0000_0000_0000_0000,
            0x0000_0000_0000_0004,
            0x0000_0000_0000_0077,
            0x3685_F28C_12AD_D8C1,
            0x3F86_E1E9_5D67_74E0,
            0x5D16_ABF6_485A_F187,
        ],
        next_f32_bits: 0x3F79_7825,
        next_f64_bits: 0x3FD5_C154_8DEC_531C,
    },
];

/// `python3 crates/liminal_core/tests/reference/kat.py methods 0xFFFFFFFFFFFFFFFF 0 2`
const METHODS_SEED_MAX_KEY_0: [Round; 2] = [
    Round {
        next_u64: 0x414B_C310_F898_9A35,
        next_u32: 0x0D1C_E1AC,
        below_u32: [
            0x0000_0000,
            0x0000_0001,
            0x0000_0000,
            0x0000_0197,
            0x535C_06D3,
            0x6C45_467F,
            0x9892_F0C4,
        ],
        below_u64: [
            0x0000_0000_0000_0000,
            0x0000_0000_0000_0003,
            0x0000_0000_0000_01AB,
            0x4055_FA2A_602C_5B07,
            0x3C4A_A641_A8D0_9B00,
            0x5E8B_6F68_CEC2_9F5A,
        ],
        next_f32_bits: 0x3D00_F690,
        next_f64_bits: 0x3FE8_CF7E_295A_4227,
    },
    Round {
        next_u64: 0x2E5F_8FB6_E66F_331E,
        next_u32: 0x7BBA_D1CC,
        below_u32: [
            0x0000_0000,
            0x0000_0001,
            0x0000_0003,
            0x0000_005C,
            0x75E4_DC55,
            0x482D_C44A,
            0x9B14_1C20,
        ],
        below_u64: [
            0x0000_0000_0000_0000,
            0x0000_0000_0000_0002,
            0x0000_0000_0000_01C5,
            0x1F17_A532_5B2D_DDF6,
            0x63D1_4227_B655_9B16,
            0x02AF_520D_1502_45C7,
        ],
        next_f32_bits: 0x3F42_256F,
        next_f64_bits: 0x3FEA_E2F3_A504_405B,
    },
];

/// Test-local reference of `SplitMix64`, written from CORE-R091 with 128-bit arithmetic and
/// explicit reduction modulo 2^64 instead of wrapping operations.
fn reference_splitmix64(x: &mut u64) -> u64 {
    const MODULUS: u128 = 1 << 64;
    let reduce =
        |value: u128| u64::try_from(value % MODULUS).expect("test setup: reduced modulo 2^64");
    *x = reduce(u128::from(*x) + 0x9E37_79B9_7F4A_7C15);
    let mut z = *x;
    z = reduce(u128::from(z ^ (z >> 30)) * 0xBF58_476D_1CE4_E5B9);
    z = reduce(u128::from(z ^ (z >> 27)) * 0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The state CORE-R091 specifies for a seed and key, built from its parts: XXH64 of the 16
/// bytes through `StateHasher::write_bytes`, then four reference `SplitMix64` outputs.
fn reference_state(world_seed: u64, key: u64) -> [u64; 4] {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&world_seed.to_le_bytes());
    bytes.extend_from_slice(&key.to_le_bytes());
    assert_eq!(bytes.len(), 16);
    let mut hasher = StateHasher::new();
    hasher.write_bytes(&bytes);
    let mut x = hasher.finish();
    core::array::from_fn(|_| reference_splitmix64(&mut x))
}

fn nz32(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).expect("test setup: the bound is not 0")
}

fn nz64(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).expect("test setup: the bound is not 0")
}

/// Asserts that `rng` produces `rounds` with the call sequence of `kat.py methods`.
fn assert_rounds(mut rng: Rng, rounds: &[Round]) {
    for (index, round) in rounds.iter().enumerate() {
        assert_eq!(rng.next_u64(), round.next_u64, "round {index}: next_u64");
        assert_eq!(rng.next_u32(), round.next_u32, "round {index}: next_u32");
        for (bound, expected) in BOUNDS_32.into_iter().zip(round.below_u32) {
            assert_eq!(
                rng.below_u32(nz32(bound)),
                expected,
                "round {index}: below_u32({bound})"
            );
        }
        for (bound, expected) in BOUNDS_64.into_iter().zip(round.below_u64) {
            assert_eq!(
                rng.below_u64(nz64(bound)),
                expected,
                "round {index}: below_u64({bound})"
            );
        }
        assert_eq!(
            rng.next_f32().to_bits(),
            round.next_f32_bits,
            "round {index}: next_f32"
        );
        assert_eq!(
            rng.next_f64().to_bits(),
            round.next_f64_bits,
            "round {index}: next_f64"
        );
    }
}

/// Verifies: CORE-R090
#[test]
fn next_u64_matches_the_published_xoshiro256_starstar_outputs() {
    let mut rng = Rng::from_state([1, 2, 3, 4]).unwrap();
    for (index, expected) in XOSHIRO_1_2_3_4.into_iter().enumerate() {
        assert_eq!(rng.next_u64(), expected, "output {index}");
    }
}

/// Verifies: CORE-R090
#[test]
fn one_step_transforms_the_state_as_specified() {
    // State after one step from [1, 2, 3, 4], computed by hand from CORE-R090:
    // t = 2 << 17 = 0x40000; s2 = 3 ^ 1 = 2; s3 = 4 ^ 2 = 6; s1 = 2 ^ 2 = 0; s0 = 1 ^ 6 = 7;
    // s2 = 2 ^ 0x40000 = 0x40002; s3 = rotl(6, 45) = 6 << 45.
    let mut rng = Rng::from_state([1, 2, 3, 4]).unwrap();
    assert_eq!(rng.next_u64(), 11_520); // rotl(2 * 5, 7) * 9 = 1280 * 9
    assert_eq!(rng.state(), [7, 0, 0x4_0002, 6 << 45]);
}

/// Verifies: CORE-R091
#[test]
fn the_test_local_splitmix64_matches_the_published_outputs() {
    let mut x = SPLITMIX_SEED;
    for (index, expected) in SPLITMIX_OUTPUTS.into_iter().enumerate() {
        assert_eq!(reference_splitmix64(&mut x), expected, "output {index}");
    }
}

/// Verifies: CORE-R091
#[test]
fn new_seeds_the_state_with_splitmix64_from_xxh64_of_seed_and_key() {
    // `kat.py stream <world_seed> <key> 4`: the four state words, then the first outputs.
    let recorded: [(u64, u64, [u64; 4], [u64; 4]); 3] = [
        (
            0,
            0,
            [
                0x3A35_B77B_228C_4FE1,
                0x3B15_9AAA_946D_94C3,
                0x2419_D78E_9811_793C,
                0xFFCF_1851_9339_63C0,
            ],
            [
                0x6617_FE0B_A193_242B,
                0x9881_5DA0_263F_A645,
                0x6D72_BBE0_7F60_E9A1,
                0x8FFD_FD5F_027C_B80B,
            ],
        ),
        (
            42,
            7,
            [
                0x07AF_5892_01C1_D9FC,
                0x602C_4EED_31B2_8652,
                0xACE7_0032_8FFB_4E4B,
                0x0F0B_BDFF_41F7_055C,
            ],
            [
                0xE4EF_D8DE_30CE_38F0,
                0x4BF5_D555_7592_A4DC,
                0x9EC7_BABA_E864_141D,
                0x073C_C121_DFD7_FC7B,
            ],
        ),
        (
            u64::MAX,
            u64::MAX,
            [
                0x42D2_318D_6EE2_8EC9,
                0x44DB_76FF_4F9E_C8D6,
                0x0117_8B08_ACC0_4771,
                0xA7EF_796D_8E27_65B1,
            ],
            [
                0x49F5_707F_74A6_D08C,
                0x34F8_C575_0620_2B99,
                0x3678_F677_CF32_0CBD,
                0xDC92_9A68_535C_F397,
            ],
        ),
    ];
    for (world_seed, key, state, outputs) in recorded {
        let mut rng = Rng::new(world_seed, StreamKey::new(key));
        assert_eq!(rng.state(), state, "seed {world_seed}, key {key}");
        assert_eq!(rng.state(), reference_state(world_seed, key));
        for expected in outputs {
            assert_eq!(rng.next_u64(), expected, "seed {world_seed}, key {key}");
        }
    }

    // Seed and key are not interchangeable: the 16 bytes are seed first, then key.
    assert_ne!(
        Rng::new(1, StreamKey::new(2)),
        Rng::new(2, StreamKey::new(1))
    );
}

proptest! {
    /// Verifies: CORE-R091
    #[test]
    fn new_matches_the_reference_seeding(world_seed in any::<u64>(), key in any::<u64>()) {
        let rng = Rng::new(world_seed, StreamKey::new(key));
        prop_assert_eq!(rng.state(), reference_state(world_seed, key));
        prop_assert_ne!(rng.state(), [0; 4]);
    }

    /// Verifies: CORE-R092
    #[test]
    fn a_stream_key_holds_its_value(key in any::<u64>()) {
        prop_assert_eq!(StreamKey::new(key).value(), key);
    }

    /// Verifies: CORE-R092
    #[test]
    fn from_name_is_xxh64_of_the_utf8_bytes(name in ".{0,40}") {
        let mut hasher = StateHasher::new();
        hasher.write_bytes(name.as_bytes());
        prop_assert_eq!(StreamKey::from_name(&name), StreamKey::new(hasher.finish()));
    }
}

/// Verifies: CORE-R092
#[test]
fn from_name_matches_recorded_xxh64_values() {
    // `kat.py xxh64 boids.spawn "" a`. The empty input is also `XSUM_XXH64_testdata[0]` of the
    // xxHash sanity vectors.
    assert_eq!(
        StreamKey::from_name("boids.spawn").value(),
        0xE495_B0EA_11D5_69F3
    );
    assert_eq!(StreamKey::from_name("").value(), 0xEF46_DB37_51D8_E999);
    assert_eq!(StreamKey::from_name("a").value(), 0xD24E_C4F1_A98C_6E5B);
    // The bytes of the name only: no length prefix, unlike the state hash of a `str`.
    assert_ne!(StreamKey::from_name("a").value(), state_hash_of("a"));
    assert_ne!(StreamKey::from_name("a.b"), StreamKey::from_name("A.B"));
    assert_eq!(StreamKey::new(u64::MAX).value(), u64::MAX);
}

/// Verifies: CORE-R093
#[test]
fn the_first_outputs_of_1024_streams_are_pairwise_distinct() {
    let mut first_outputs = BTreeSet::new();
    for world_seed in 0..32 {
        for key in 0..32 {
            first_outputs.insert(Rng::new(world_seed, StreamKey::new(key)).next_u64());
        }
    }
    assert_eq!(first_outputs.len(), 1024);
}

/// Verifies: CORE-R094
#[test]
fn next_u32_is_the_upper_half_of_one_next_u64() {
    let mut rng = Rng::from_state([1, 2, 3, 4]).unwrap();
    for expected in XOSHIRO_1_2_3_4 {
        let mut twin = rng.clone();
        assert_eq!(u64::from(rng.next_u32()), expected >> 32);
        twin.next_u64();
        assert_eq!(rng, twin, "next_u32 takes exactly one step");
    }
    let mut rng = Rng::from_state([1, 2, 3, 4]).unwrap();
    // The published outputs divided by 2^32 (integer division, computed with Python).
    let upper: Vec<u32> = (0..10).map(|_| rng.next_u32()).collect();
    assert_eq!(
        upper,
        [
            0,
            0,
            0,
            283_115_520,
            283_162_140,
            141_558_300,
            3_765_552_066,
            1_973_512_462,
            2_466_867_291,
            676_281_538
        ]
    );
}

/// Verifies: CORE-R095, CORE-R098
#[test]
fn bounded_integers_and_floats_match_the_independent_implementation() {
    assert_rounds(Rng::new(42, StreamKey::new(7)), &METHODS_SEED_42_KEY_7);
    assert_rounds(
        Rng::new(u64::MAX, StreamKey::new(0)),
        &METHODS_SEED_MAX_KEY_0,
    );
}

/// Verifies: CORE-R095
#[test]
fn bounds_above_half_the_range_reject_and_draw_again() {
    // For the bound 2^31 + 1 the threshold is 2^31 − 1: almost half of the draws are rejected.
    let bound = nz32(0x8000_0001);
    let mut rng = Rng::new(3, StreamKey::new(3));
    let mut rejections = 0;
    for _ in 0..1000 {
        let mut twin = rng.clone();
        let value = rng.below_u32(bound);
        assert!(value < bound.get());
        twin.next_u64();
        if twin != rng {
            rejections += 1;
        }
    }
    assert!(
        (350..650).contains(&rejections),
        "{rejections} of 1000 draws rejected"
    );

    let bound = nz64(0x8000_0000_0000_0001);
    let mut rejections = 0;
    for _ in 0..1000 {
        let mut twin = rng.clone();
        let value = rng.below_u64(bound);
        assert!(value < bound.get());
        twin.next_u64();
        if twin != rng {
            rejections += 1;
        }
    }
    assert!(
        (350..650).contains(&rejections),
        "{rejections} of 1000 draws rejected"
    );
}

/// Verifies: CORE-R095
#[test]
fn a_power_of_two_bound_takes_the_top_bits_of_one_draw() {
    let mut rng = Rng::new(9, StreamKey::from_name("power-of-two"));
    for exponent in 0..32 {
        for _ in 0..50 {
            let mut twin = rng.clone();
            let draw = twin.next_u32();
            // `draw >> (32 − exponent)`; for exponent 0 that is 0.
            let expected = u64::from(draw) >> (32 - exponent);
            assert_eq!(u64::from(rng.below_u32(nz32(1 << exponent))), expected);
            assert_eq!(rng, twin, "no rejection for 2^{exponent}");
        }
    }
    for exponent in 0..64 {
        for _ in 0..50 {
            let mut twin = rng.clone();
            let draw = twin.next_u64();
            let expected = u128::from(draw) >> (64 - exponent);
            assert_eq!(u128::from(rng.below_u64(nz64(1 << exponent))), expected);
            assert_eq!(rng, twin, "no rejection for 2^{exponent}");
        }
    }
}

/// Verifies: CORE-R095
#[test]
fn every_value_below_a_small_bound_occurs_about_equally_often() {
    // 60,000 draws below 6: expected 10,000 each, standard deviation about 91. The seed is
    // fixed, so the test is deterministic; the interval is 5 standard deviations wide.
    let mut rng = Rng::new(2026, StreamKey::from_name("uniformity"));
    let mut counts_32 = [0_u32; 6];
    let mut counts_64 = [0_u32; 6];
    for _ in 0..60_000 {
        counts_32[usize::try_from(rng.below_u32(nz32(6))).unwrap()] += 1;
        counts_64[usize::try_from(rng.below_u64(nz64(6))).unwrap()] += 1;
    }
    for count in counts_32.into_iter().chain(counts_64) {
        assert!(
            (9_545..=10_455).contains(&count),
            "{counts_32:?} {counts_64:?}"
        );
    }
}

proptest! {
    /// Verifies: CORE-R095
    #[test]
    fn bounded_integers_are_below_their_bound(
        world_seed in any::<u64>(),
        bound_32 in 1_u32..,
        bound_64 in 1_u64..,
    ) {
        let mut rng = Rng::new(world_seed, StreamKey::new(0));
        for _ in 0..20 {
            prop_assert!(rng.below_u32(nz32(bound_32)) < bound_32);
            prop_assert!(rng.below_u64(nz64(bound_64)) < bound_64);
        }
        prop_assert_eq!(rng.below_u32(nz32(1)), 0);
        prop_assert_eq!(rng.below_u64(nz64(1)), 0);
    }

    /// Verifies: CORE-R096
    #[test]
    fn floats_are_in_the_unit_interval_on_their_grid(world_seed in any::<u64>()) {
        let mut rng = Rng::new(world_seed, StreamKey::new(1));
        for _ in 0..50 {
            let mut twin = rng.clone();
            let single = rng.next_f32();
            prop_assert!((0.0..1.0).contains(&single));
            // A multiple of 2^-24: scaling by 2^24 gives the integer `next_u64() >> 40`.
            let scaled = f64::from(single) * 16_777_216.0;
            prop_assert_eq!(scaled.to_bits(), f64::from(u32::try_from(twin.next_u64() >> 40).unwrap()).to_bits());
            prop_assert_eq!(&rng, &twin);

            let double = rng.next_f64();
            prop_assert!((0.0..1.0).contains(&double));
            // A multiple of 2^-53: scaling by 2^53 is exact and gives `next_u64() >> 11`.
            let scaled = double * 9_007_199_254_740_992.0;
            prop_assert_eq!(scaled.fract().to_bits(), 0.0_f64.to_bits());
            prop_assert_eq!(format!("{scaled:.0}"), (twin.next_u64() >> 11).to_string());
            prop_assert_eq!(&rng, &twin);
        }
    }
}

/// Verifies: CORE-R096
#[test]
fn the_smallest_and_largest_outputs_map_to_0_and_just_below_1() {
    // The second output from [1, 2, 3, 4] is 0 (published): both floats are exactly 0.
    let mut zero = Rng::from_state([1, 2, 3, 4]).unwrap();
    zero.next_u64();
    assert_eq!(zero.clone().next_f32().to_bits(), 0.0_f32.to_bits());
    assert_eq!(zero.next_f64().to_bits(), 0.0_f64.to_bits());

    // `kat.py max-output-state`: with this s1 the next output is 2^64 − 1, the largest.
    let max = Rng::from_state([1, 0x4FC7_1C71_C71C_71C7, 3, 4]).unwrap();
    assert_eq!(max.clone().next_u64(), u64::MAX);
    // 1 − 2^-24 and 1 − 2^-53: the largest values below 1 (all significand bits set).
    assert_eq!(max.clone().next_f32().to_bits(), 0x3F7F_FFFF);
    assert_eq!(max.clone().next_f64().to_bits(), 0x3FEF_FFFF_FFFF_FFFF);
    assert!(max.clone().next_f32() < 1.0 && max.clone().next_f64() < 1.0);
}

/// Verifies: CORE-R097
#[test]
fn a_restored_state_continues_the_sequence() {
    let mut original = Rng::new(5, StreamKey::from_name("save"));
    for _ in 0..17 {
        original.next_u64();
    }
    let saved = original.state();
    let mut restored = Rng::from_state(saved).unwrap();
    assert_eq!(restored, original);
    assert_eq!(restored.state(), saved);
    for _ in 0..100 {
        assert_eq!(restored.next_u64(), original.next_u64());
    }
    assert_eq!(
        restored.below_u32(nz32(1000)),
        original.below_u32(nz32(1000))
    );
    assert_eq!(restored.next_f64().to_bits(), original.next_f64().to_bits());
    assert_ne!(original.state(), saved);
}

/// Verifies: CORE-R097
#[test]
fn an_all_zero_state_is_rejected() {
    let error = Rng::from_state([0; 4]).unwrap_err();
    assert_eq!(error, CoreError::ZeroRngState);
    assert_eq!(error.code().to_string(), "LIM-CORE-0030");
    // One non-zero word is enough.
    for position in 0..4 {
        let mut state = [0; 4];
        state[position] = 1;
        assert_eq!(Rng::from_state(state).unwrap().state(), state);
    }
}

/// Verifies: CORE-R097
#[test]
fn a_clone_continues_the_same_sequence_independently() {
    let mut original = Rng::new(11, StreamKey::new(11));
    original.next_u64();
    let mut clone = original.clone();
    let before = original.state();

    let from_clone: Vec<u64> = (0..20).map(|_| clone.next_u64()).collect();
    // Advancing the clone did not advance the original.
    assert_eq!(original.state(), before);
    let from_original: Vec<u64> = (0..20).map(|_| original.next_u64()).collect();
    assert_eq!(from_clone, from_original);
    assert_eq!(clone, original);
}

/// Verifies: CORE-R098
#[test]
fn recorded_outputs_of_named_streams_hold_on_this_platform() {
    // `kat.py stream 42 0xE495B0EA11D569F3 3`, the key being XXH64 of "boids.spawn".
    let mut rng = Rng::new(42, StreamKey::from_name("boids.spawn"));
    assert_eq!(rng.next_u64(), 0xFC6C_1C52_B1ED_2B1A);
    assert_eq!(rng.next_u64(), 0xC0F4_0B3B_2FCF_78B1);
    assert_eq!(rng.next_u64(), 0xCF36_8FD8_9121_CBA5);
}
