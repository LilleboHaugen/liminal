//! Tick-based simulation time (spec §5.6, ADR-0009 item 1).
//!
//! The simulation advances in ticks of fixed length `dt = 1 / rate` seconds. Time inside the
//! simulation is counted in ticks, which are integers, so it never accumulates rounding error:
//!
//! - [`TickRate`]: ticks per second (1 to 1000, default 60) and conversions to seconds.
//! - [`Tick`]: the index of a simulation step. `Tick::ZERO` is the first step.
//! - [`TickDuration`]: a number of ticks.
//!
//! This module reads no clock. Turning variable frame time into fixed steps (accumulator,
//! interpolation) is `liminal_app`'s job.

use core::num::NonZeroU32;

use crate::error::CoreError;
use crate::hash::{StateHash, StateHasher};

/// Highest supported tick rate, in ticks per second.
const MAX_TICKS_PER_SECOND: u32 = 1000;

/// The fixed simulation rate in ticks per second, 1 to 1000 (INV-5).
///
/// The default is 60 ticks per second (ADR-0009 item 1): `dt` = 1/60 s.
///
/// # Examples
///
/// ```
/// use liminal_core::error::CoreError;
/// use liminal_core::time::{TickDuration, TickRate};
///
/// let rate = TickRate::new(50)?;
/// assert_eq!(rate.ticks_per_second(), 50);
/// assert_eq!(rate.dt_secs_f32(), 0.02);
///
/// // A 250 ms cooldown is 12.5 ticks: rounded to the nearest tick, ties up.
/// let cooldown = rate.ticks_from_millis(250);
/// assert_eq!(cooldown, TickDuration::new(13));
/// assert_eq!(rate.ticks_to_secs_f64(cooldown), 0.26);
///
/// assert_eq!(TickRate::default(), TickRate::DEFAULT);
/// assert_eq!(TickRate::DEFAULT.ticks_per_second(), 60);
/// assert!(TickRate::new(0).is_err());
/// # Ok::<(), CoreError>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct TickRate {
    ticks_per_second: NonZeroU32,
}

impl TickRate {
    /// 60 ticks per second, the default fixed timestep (ADR-0009 item 1).
    pub const DEFAULT: TickRate = TickRate {
        ticks_per_second: NonZeroU32::new(60).expect("invariant: 60 is not zero"),
    };

    /// The rate of `ticks_per_second` ticks per second.
    ///
    /// # Errors
    ///
    /// [`CoreError::InvalidTickRate`] (LIM-CORE-0020) with the given value if `ticks_per_second`
    /// is 0 or above 1000.
    pub fn new(ticks_per_second: u32) -> Result<TickRate, CoreError> {
        NonZeroU32::new(ticks_per_second)
            .filter(|rate| rate.get() <= MAX_TICKS_PER_SECOND)
            .map(|ticks_per_second| TickRate { ticks_per_second })
            .ok_or(CoreError::InvalidTickRate { ticks_per_second })
    }

    /// Ticks per second, 1 to 1000.
    #[must_use]
    pub fn ticks_per_second(self) -> u32 {
        self.ticks_per_second.get()
    }

    /// Length of one tick in seconds: `1.0_f32 / ticks_per_second as f32`, one correctly rounded
    /// division.
    #[must_use]
    pub fn dt_secs_f32(self) -> f32 {
        #[expect(
            clippy::cast_precision_loss,
            reason = "the rate is at most 1000 (INV-5), so the conversion is exact"
        )]
        let rate = self.ticks_per_second.get() as f32;
        1.0 / rate
    }

    /// Length of one tick in seconds: `1.0_f64 / ticks_per_second as f64`, one correctly rounded
    /// division.
    #[must_use]
    pub fn dt_secs_f64(self) -> f64 {
        1.0 / f64::from(self.ticks_per_second.get())
    }

    /// `duration` in seconds: `duration.count() as f64 / ticks_per_second as f64`. Counts above
    /// 2^53 are first rounded to the nearest `f64` (ties to even), then divided once.
    #[must_use]
    pub fn ticks_to_secs_f64(self, duration: TickDuration) -> f64 {
        #[expect(
            clippy::cast_precision_loss,
            reason = "rounding counts above 2^53 to the nearest f64 is the specified behaviour"
        )]
        let count = duration.count() as f64;
        count / f64::from(self.ticks_per_second.get())
    }

    /// `millis` milliseconds as a number of ticks, rounded to the nearest tick with ties up:
    /// `(millis × ticks_per_second + 500) / 1000` in 128-bit integer arithmetic. Never overflows.
    ///
    /// # Panics
    ///
    /// Only if the type invariant is broken (a bug in this crate): with a rate of at most 1000
    /// the result is at most `millis`.
    #[must_use]
    pub fn ticks_from_millis(self, millis: u64) -> TickDuration {
        let ticks = (u128::from(millis) * u128::from(self.ticks_per_second.get()) + 500) / 1000;
        TickDuration(
            u64::try_from(ticks).expect("invariant: at most 1000 ticks per second (INV-5)"),
        )
    }
}

/// [`TickRate::DEFAULT`]: 60 ticks per second.
impl Default for TickRate {
    fn default() -> TickRate {
        TickRate::DEFAULT
    }
}

/// Ticks per second as `u32`.
impl StateHash for TickRate {
    fn state_hash(&self, hasher: &mut StateHasher) {
        hasher.write_u32(self.ticks_per_second.get());
    }
}

/// Index of a simulation step. `Tick::ZERO` (also the `Default`) is the first step. Ordered by
/// index.
///
/// # Examples
///
/// ```
/// use liminal_core::time::{Tick, TickDuration};
///
/// let start = Tick::ZERO;
/// let now = start.next().next();
/// assert_eq!(now, Tick::new(2));
/// assert_eq!(now.since(start), Some(TickDuration::new(2)));
/// assert_eq!(start.since(now), None); // `start` is earlier
/// assert_eq!(now.checked_add(TickDuration::new(8)), Some(Tick::new(10)));
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct Tick(u64);

impl Tick {
    /// The first simulation step, index 0.
    pub const ZERO: Tick = Tick(0);

    /// The step with this index.
    #[must_use]
    pub const fn new(index: u64) -> Tick {
        Tick(index)
    }

    /// The step's index.
    #[must_use]
    pub fn index(self) -> u64 {
        self.0
    }

    /// The step after this one.
    ///
    /// # Panics
    ///
    /// If the index is `u64::MAX`. Counting from 0 this is unreachable in practice: 2^64 ticks
    /// at 1000 ticks per second take 5.8 × 10^8 years. Use [`Tick::checked_add`] for indices
    /// that do not come from counting.
    #[must_use]
    pub fn next(self) -> Tick {
        Tick(
            self.0
                .checked_add(1)
                .expect("invariant: fewer than 2^64 simulation steps are ever counted"),
        )
    }

    /// The step `duration` after this one, or `None` if the index would exceed `u64::MAX`.
    #[must_use]
    pub fn checked_add(self, duration: TickDuration) -> Option<Tick> {
        self.0.checked_add(duration.0).map(Tick)
    }

    /// The number of ticks from `earlier` to this step: `Some(self − earlier)` if
    /// `self >= earlier`, else `None`.
    #[must_use]
    pub fn since(self, earlier: Tick) -> Option<TickDuration> {
        self.0.checked_sub(earlier.0).map(TickDuration)
    }
}

/// The index as `u64`.
impl StateHash for Tick {
    fn state_hash(&self, hasher: &mut StateHasher) {
        hasher.write_u64(self.0);
    }
}

/// A number of ticks. `TickDuration::ZERO` is also the `Default`. Ordered by count.
///
/// # Examples
///
/// ```
/// use liminal_core::time::TickDuration;
///
/// let total = TickDuration::new(40).checked_add(TickDuration::new(2));
/// assert_eq!(total.map(TickDuration::count), Some(42));
/// assert_eq!(TickDuration::new(u64::MAX).checked_add(TickDuration::new(1)), None);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct TickDuration(u64);

impl TickDuration {
    /// No ticks.
    pub const ZERO: TickDuration = TickDuration(0);

    /// The duration of `count` ticks.
    #[must_use]
    pub const fn new(count: u64) -> TickDuration {
        TickDuration(count)
    }

    /// The number of ticks.
    #[must_use]
    pub fn count(self) -> u64 {
        self.0
    }

    /// The sum of two durations, or `None` if it would exceed `u64::MAX` ticks.
    #[must_use]
    pub fn checked_add(self, other: TickDuration) -> Option<TickDuration> {
        self.0.checked_add(other.0).map(TickDuration)
    }
}

/// The count as `u64`.
impl StateHash for TickDuration {
    fn state_hash(&self, hasher: &mut StateHasher) {
        hasher.write_u64(self.0);
    }
}
