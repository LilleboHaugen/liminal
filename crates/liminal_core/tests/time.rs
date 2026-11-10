//! Integration tests of `liminal_core::time` through the public API (spec §10, CORE-R110 to
//! CORE-R114). Oracle: hand-computed values; float bit patterns and the `u64::MAX` cases were
//! computed with Python (`python3 crates/liminal_core/tests/reference/kat.py time`).

use liminal_core::error::{CoreError, EngineError};
use liminal_core::time::{Tick, TickDuration, TickRate};
use proptest::prelude::*;

const TWO_POW_53: u64 = 1 << 53;

fn rate(ticks_per_second: u32) -> TickRate {
    TickRate::new(ticks_per_second).expect("test setup: the rate is in 1..=1000")
}

/// Verifies: CORE-R110
#[test]
fn tick_rates_from_1_to_1000_are_valid() {
    for ticks_per_second in 1..=1000 {
        let rate = TickRate::new(ticks_per_second).expect("test setup: the rate is in 1..=1000");
        assert_eq!(rate.ticks_per_second(), ticks_per_second);
    }
    for invalid in [0, 1001, 1002, 60_000, u32::MAX] {
        let error = TickRate::new(invalid).unwrap_err();
        assert_eq!(
            error,
            CoreError::InvalidTickRate {
                ticks_per_second: invalid
            }
        );
        assert_eq!(error.code().to_string(), "LIM-CORE-0020");
    }
}

/// Verifies: CORE-R110
#[test]
fn the_default_tick_rate_is_60() {
    assert_eq!(TickRate::DEFAULT.ticks_per_second(), 60);
    assert_eq!(TickRate::default().ticks_per_second(), 60);
    assert_eq!(TickRate::default(), TickRate::DEFAULT);
    assert_eq!(TickRate::DEFAULT, rate(60));
    // Rates order by ticks per second.
    assert!(rate(30) < rate(60) && rate(60) < rate(1000));
}

/// Verifies: CORE-R111
#[test]
fn dt_is_one_correctly_rounded_division() {
    // (rate, bits of 1/rate as f32, bits of 1/rate as f64): 1, 1/3, 1/60, 1/1000.
    for (ticks_per_second, single, double) in [
        (1, 0x3F80_0000_u32, 0x3FF0_0000_0000_0000_u64),
        (3, 0x3EAA_AAAB, 0x3FD5_5555_5555_5555),
        (60, 0x3C88_8889, 0x3F91_1111_1111_1111),
        (1000, 0x3A83_126F, 0x3F50_624D_D2F1_A9FC),
    ] {
        let rate = rate(ticks_per_second);
        assert_eq!(
            rate.dt_secs_f32().to_bits(),
            single,
            "rate {ticks_per_second}"
        );
        assert_eq!(
            rate.dt_secs_f64().to_bits(),
            double,
            "rate {ticks_per_second}"
        );
    }
    // Powers of two are exact.
    assert_eq!(rate(2).dt_secs_f32().to_bits(), 0.5_f32.to_bits());
    assert_eq!(
        rate(512).dt_secs_f64().to_bits(),
        0.001_953_125_f64.to_bits()
    );
}

/// Verifies: CORE-R111
#[test]
fn dt_times_the_rate_is_within_one_rounding_of_1_for_every_rate() {
    for ticks_per_second in 1..=1000_u16 {
        let rate = rate(u32::from(ticks_per_second));
        // A correctly rounded quotient q of 1 / n satisfies |q × n − 1| <= n × ulp(q) / 2.
        let single = f64::from(rate.dt_secs_f32()) * f64::from(ticks_per_second);
        assert!(
            (single - 1.0).abs() <= f64::from(f32::EPSILON) / 2.0,
            "f32, rate {ticks_per_second}"
        );
        let double = rate.dt_secs_f64() * f64::from(ticks_per_second);
        assert!(
            (double - 1.0).abs() <= f64::EPSILON,
            "f64, rate {ticks_per_second}"
        );
        // The f32 value is the f64 value rounded once (1/n in f64, then to f32, agrees with the
        // direct f32 division for every rate up to 1000; checked here against the same value
        // parsed from decimal).
        let parsed: f32 = format!("{:e}", rate.dt_secs_f64()).parse().unwrap();
        assert_eq!(
            rate.dt_secs_f32().to_bits(),
            parsed.to_bits(),
            "rate {ticks_per_second}"
        );
    }
}

/// Verifies: CORE-R112
#[test]
fn ticks_to_seconds_converts_the_count_and_divides_once() {
    let secs = |ticks_per_second: u32, count: u64| {
        rate(ticks_per_second)
            .ticks_to_secs_f64(TickDuration::new(count))
            .to_bits()
    };
    assert_eq!(secs(60, 0), 0.0_f64.to_bits());
    assert_eq!(secs(60, 120), 2.0_f64.to_bits());
    assert_eq!(secs(60, 90), 1.5_f64.to_bits());
    // A division, not a multiplication by `dt`: 49 × (1/49) would be 1 − 2^-53.
    assert_eq!(secs(49, 49), 1.0_f64.to_bits());
    assert_eq!(secs(1000, 1), 0x3F50_624D_D2F1_A9FC); // 0.001
    assert_eq!(secs(3, 1), 0x3FD5_5555_5555_5555); // 1/3
    assert_eq!(secs(1, TWO_POW_53), 0x4340_0000_0000_0000); // 2^53, exact
    // Counts above 2^53 round to the nearest f64, ties to even: 2^53 + 1 -> 2^53,
    // 2^53 + 3 -> 2^53 + 4.
    assert_eq!(secs(1, TWO_POW_53 + 1), 0x4340_0000_0000_0000);
    assert_eq!(secs(1, TWO_POW_53 + 3), 0x4340_0000_0000_0002);
    // u64::MAX rounds to 2^64, then one division.
    assert_eq!(secs(1000, u64::MAX), 0x4350_624D_D2F1_A9FC);
    assert_eq!(secs(7, u64::MAX), 0x43C2_4924_9249_2492);
}

/// Verifies: CORE-R113
#[test]
fn milliseconds_round_to_the_nearest_tick_with_ties_up() {
    let ticks = |ticks_per_second: u32, millis: u64| {
        rate(ticks_per_second).ticks_from_millis(millis).count()
    };
    // 60 ticks per second: one tick is 16.67 ms.
    assert_eq!(ticks(60, 0), 0);
    assert_eq!(ticks(60, 8), 0); // 0.48 ticks
    assert_eq!(ticks(60, 9), 1); // 0.54 ticks
    assert_eq!(ticks(60, 16), 1); // 0.96
    assert_eq!(ticks(60, 17), 1); // 1.02
    assert_eq!(ticks(60, 24), 1); // 1.44
    assert_eq!(ticks(60, 25), 2); // 1.5: tie, up
    assert_eq!(ticks(60, 1000), 60);
    assert_eq!(ticks(60, 3_600_000), 216_000);
    // 50 ticks per second: 20 ms per tick; 10 ms and 250 ms are ties.
    assert_eq!(ticks(50, 9), 0);
    assert_eq!(ticks(50, 10), 1);
    assert_eq!(ticks(50, 250), 13);
    assert_eq!(ticks(50, 249), 12);
    // 1 tick per second: ties at 500 ms.
    assert_eq!(ticks(1, 499), 0);
    assert_eq!(ticks(1, 500), 1);
    assert_eq!(ticks(1, 1499), 1);
    assert_eq!(ticks(1, 1500), 2);
    // 1000 ticks per second: one tick per millisecond, for every input.
    assert_eq!(ticks(1000, 0), 0);
    assert_eq!(ticks(1000, 12_345), 12_345);
    assert_eq!(ticks(1000, u64::MAX), u64::MAX);
    // No overflow at the largest input: (u64::MAX × n + 500) / 1000 with integer division.
    assert_eq!(ticks(1, u64::MAX), 18_446_744_073_709_552);
    assert_eq!(ticks(7, u64::MAX), 129_127_208_515_966_861);
}

proptest! {
    /// Verifies: CORE-R113
    #[test]
    fn ticks_from_millis_is_monotone_and_never_panics(
        ticks_per_second in 1_u32..=1000,
        millis in any::<u64>(),
        step in 0_u64..5000,
    ) {
        let rate = rate(ticks_per_second);
        let ticks = rate.ticks_from_millis(millis).count();
        let later = rate.ticks_from_millis(millis.saturating_add(step)).count();
        prop_assert!(ticks <= later);
        // At most one tick per millisecond.
        prop_assert!(ticks <= millis);
        // Within half a tick of the exact value: |ticks × 1000 − millis × n| <= 500.
        let exact = u128::from(millis) * u128::from(ticks_per_second);
        prop_assert!((u128::from(ticks) * 1000).abs_diff(exact) <= 500);
    }

    /// Verifies: CORE-R113
    #[test]
    fn whole_seconds_convert_exactly(ticks_per_second in 1_u32..=1000, seconds in 0_u64..(u64::MAX / 1000)) {
        let ticks = rate(ticks_per_second).ticks_from_millis(seconds * 1000).count();
        prop_assert_eq!(u128::from(ticks), u128::from(seconds) * u128::from(ticks_per_second));
    }

    /// Verifies: CORE-R114
    #[test]
    fn tick_arithmetic_matches_checked_integer_arithmetic(a in any::<u64>(), b in any::<u64>()) {
        prop_assert_eq!(
            Tick::new(a).checked_add(TickDuration::new(b)).map(Tick::index),
            a.checked_add(b)
        );
        prop_assert_eq!(
            Tick::new(a).since(Tick::new(b)).map(TickDuration::count),
            a.checked_sub(b)
        );
        prop_assert_eq!(
            TickDuration::new(a).checked_add(TickDuration::new(b)).map(TickDuration::count),
            a.checked_add(b)
        );
        prop_assert_eq!(Tick::new(a).cmp(&Tick::new(b)), a.cmp(&b));
        prop_assert_eq!(TickDuration::new(a).cmp(&TickDuration::new(b)), a.cmp(&b));
        prop_assert_eq!(Tick::new(a) == Tick::new(b), a == b);
        prop_assert_eq!(TickDuration::new(a) == TickDuration::new(b), a == b);
        prop_assert_eq!(Tick::new(a).index(), a);
        prop_assert_eq!(TickDuration::new(a).count(), a);
    }
}

/// Verifies: CORE-R114
#[test]
fn ticks_count_steps_from_zero() {
    assert_eq!(Tick::ZERO.index(), 0);
    assert_eq!(Tick::ZERO, Tick::new(0));
    assert_eq!(Tick::default(), Tick::ZERO);
    assert_eq!(TickDuration::ZERO.count(), 0);
    assert_eq!(TickDuration::ZERO, TickDuration::new(0));
    assert_eq!(TickDuration::default(), TickDuration::ZERO);

    assert_eq!(Tick::ZERO.next(), Tick::new(1));
    assert_eq!(Tick::new(41).next().index(), 42);
    assert_eq!(Tick::new(u64::MAX - 1).next(), Tick::new(u64::MAX));

    assert_eq!(
        Tick::new(40).checked_add(TickDuration::new(2)),
        Some(Tick::new(42))
    );
    assert_eq!(
        Tick::new(u64::MAX).checked_add(TickDuration::ZERO),
        Some(Tick::new(u64::MAX))
    );
    assert_eq!(Tick::new(u64::MAX).checked_add(TickDuration::new(1)), None);
    assert_eq!(Tick::new(1).checked_add(TickDuration::new(u64::MAX)), None);

    assert_eq!(
        Tick::new(42).since(Tick::new(40)),
        Some(TickDuration::new(2))
    );
    assert_eq!(Tick::new(42).since(Tick::new(42)), Some(TickDuration::ZERO));
    assert_eq!(Tick::new(40).since(Tick::new(42)), None);
    assert_eq!(
        Tick::new(u64::MAX).since(Tick::ZERO),
        Some(TickDuration::new(u64::MAX))
    );

    assert_eq!(
        TickDuration::new(40).checked_add(TickDuration::new(2)),
        Some(TickDuration::new(42))
    );
    assert_eq!(
        TickDuration::new(u64::MAX).checked_add(TickDuration::new(1)),
        None
    );

    assert!(Tick::new(3) < Tick::new(4) && Tick::new(4) > Tick::new(3));
    assert!(TickDuration::new(3) < TickDuration::new(4));
    assert_eq!(Tick::new(7).max(Tick::new(9)), Tick::new(9));
}

/// Verifies: CORE-R114
#[test]
#[should_panic(expected = "invariant: fewer than 2^64 simulation steps")]
fn next_panics_at_the_last_tick() {
    let _ = Tick::new(u64::MAX).next();
}
