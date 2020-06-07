use crate::error;
#[cfg(std)]
use crate::Instant;
use core::{
    cmp::Ordering,
    convert::{TryFrom, TryInto},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
    time::Duration as StdDuration,
};
#[allow(unused_imports)]
use standback::prelude::*;

/// A span of time with nanosecond precision.
///
/// Each `Duration` is composed of a whole number of seconds and a fractional
/// part represented in nanoseconds.
///
/// `Duration` implements many traits, including [`Add`], [`Sub`], [`Mul`], and
/// [`Div`], among others.
///
/// This implementation allows for negative durations, unlike
/// [`core::time::Duration`].
#[cfg_attr(serde, derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Duration {
    /// Number of whole seconds.
    seconds: i64,
    /// Number of nanoseconds within the second. The sign always matches the
    /// `seconds` field.
    nanoseconds: i32, // always -10^9 < nanoseconds < 10^9
}

/// The number of seconds in one minute.
const SECONDS_PER_MINUTE: i64 = 60;

/// The number of seconds in one hour.
const SECONDS_PER_HOUR: i64 = 60 * SECONDS_PER_MINUTE;

/// The number of seconds in one day.
const SECONDS_PER_DAY: i64 = 24 * SECONDS_PER_HOUR;

/// The number of seconds in one week.
const SECONDS_PER_WEEK: i64 = 7 * SECONDS_PER_DAY;

impl Duration {
    /// Equivalent to `1.days()`.
    ///
    /// ```rust
    /// # use time::{Duration, prelude::*};
    /// assert_eq!(Duration::day, 1.days());
    /// ```
    #[allow(non_upper_case_globals)]
    pub const day: Self = Self::days(1);
    /// Equivalent to `1.hours()`.
    ///
    /// ```rust
    /// # use time::{Duration, prelude::*};
    /// assert_eq!(Duration::hour, 1.hours());
    /// ```
    #[allow(non_upper_case_globals)]
    pub const hour: Self = Self::hours(1);
    /// The maximum possible duration. Adding any positive duration to this will
    /// cause an overflow.
    ///
    /// The value returned by this method may change at any time.
    #[allow(non_upper_case_globals)]
    pub const max_value: Self = Self {
        seconds: i64::max_value(),
        nanoseconds: 999_999_999,
    };
    /// Equivalent to `1.microseconds()`.
    ///
    /// ```rust
    /// # use time::{Duration, prelude::*};
    /// assert_eq!(Duration::microsecond, 1.microseconds());
    /// ```
    #[allow(non_upper_case_globals)]
    pub const microsecond: Self = Self::microseconds(1);
    /// Equivalent to `1.milliseconds()`.
    ///
    /// ```rust
    /// # use time::{Duration, prelude::*};
    /// assert_eq!(Duration::millisecond, 1.milliseconds());
    /// ```
    #[allow(non_upper_case_globals)]
    pub const millisecond: Self = Self::milliseconds(1);
    /// The minimum possible duration. Adding any negative duration to this will
    /// cause an overflow.
    ///
    /// The value returned by this method may change at any time.
    #[allow(non_upper_case_globals)]
    pub const min_value: Self = Self {
        seconds: i64::min_value(),
        nanoseconds: -999_999_999,
    };
    /// Equivalent to `1.minutes()`.
    ///
    /// ```rust
    /// # use time::{Duration, prelude::*};
    /// assert_eq!(Duration::minute, 1.minutes());
    /// ```
    #[allow(non_upper_case_globals)]
    pub const minute: Self = Self::minutes(1);
    /// Equivalent to `1.nanoseconds()`.
    ///
    /// ```rust
    /// # use time::{Duration, prelude::*};
    /// assert_eq!(Duration::nanosecond, 1.nanoseconds());
    /// ```
    #[allow(non_upper_case_globals)]
    pub const nanosecond: Self = Self::nanoseconds(1);
    /// Equivalent to `1.seconds()`.
    ///
    /// ```rust
    /// # use time::{Duration, prelude::*};
    /// assert_eq!(Duration::second, 1.seconds());
    /// ```
    #[allow(non_upper_case_globals)]
    pub const second: Self = Self::seconds(1);
    /// Equivalent to `1.weeks()`.
    ///
    /// ```rust
    /// # use time::{Duration, prelude::*};
    /// assert_eq!(Duration::week, 1.weeks());
    /// ```
    #[allow(non_upper_case_globals)]
    pub const week: Self = Self::weeks(1);
    /// Equivalent to `0.seconds()`.
    ///
    /// ```rust
    /// # use time::{Duration, prelude::*};
    /// assert_eq!(Duration::zero, 0.seconds());
    /// ```
    #[allow(non_upper_case_globals)]
    pub const zero: Self = Self::seconds(0);

    /// Check if a duration is exactly zero.
    ///
    /// ```rust
    /// # use time::prelude::*;
    /// assert!(0.seconds().is_zero());
    /// assert!(!1.nanoseconds().is_zero());
    /// ```
    #[inline(always)]
    pub fn is_zero(self) -> bool {
        (self.seconds == 0) && (self.nanoseconds == 0)
    }

    /// Check if a duration is negative.
    ///
    /// ```rust
    /// # use time::prelude::*;
    /// assert!((-1).seconds().is_negative());
    /// assert!(!0.seconds().is_negative());
    /// assert!(!1.seconds().is_negative());
    /// ```
    #[inline(always)]
    pub fn is_negative(self) -> bool {
        (self.seconds < 0) || (self.nanoseconds < 0)
    }

    /// Check if a duration is positive.
    ///
    /// ```rust
    /// # use time::{prelude::*};
    /// assert!(1.seconds().is_positive());
    /// assert!(!0.seconds().is_positive());
    /// assert!(!(-1).seconds().is_positive());
    /// ```
    #[inline(always)]
    pub fn is_positive(self) -> bool {
        (self.seconds > 0) || (self.nanoseconds > 0)
    }

    /// Get the absolute value of the duration.
    ///
    /// ```rust
    /// # use time::prelude::*;
    /// assert_eq!(1.seconds().abs(), 1.seconds());
    /// assert_eq!(0.seconds().abs(), 0.seconds());
    /// assert_eq!((-1).seconds().abs(), 1.seconds());
    /// ```
    ///
    /// This function is `const fn` when using rustc >= 1.39.0.
    #[inline(always)]
    #[cfg(const_num_abs)]
    pub const fn abs(self) -> Self {
        Self {
            seconds: self.seconds.abs(),
            nanoseconds: self.nanoseconds.abs(),
        }
    }

    /// Get the absolute value of the duration.
    ///
    /// ```rust
    /// # use time::prelude::*;
    /// assert_eq!(1.seconds().abs(), 1.seconds());
    /// assert_eq!(0.seconds().abs(), 0.seconds());
    /// assert_eq!((-1).seconds().abs(), 1.seconds());
    /// ```
    ///
    /// This function is `const fn` when using rustc >= 1.39.0.
    #[inline(always)]
    #[cfg(not(const_num_abs))]
    pub fn abs(self) -> Self {
        Self {
            seconds: self.seconds.abs(),
            nanoseconds: self.nanoseconds.abs(),
        }
    }

    /// Convert the existing `Duration` to a `std::time::Duration` and its sign.
    // This doesn't actually require the standard library, but is currently only
    // used when it's enabled.
    #[inline(always)]
    #[cfg(std)]
    pub(crate) fn abs_std(self) -> StdDuration {
        StdDuration::new(self.seconds.abs() as u64, self.nanoseconds.abs() as u32)
    }

    /// Create a new `Duration` with the provided seconds and nanoseconds. If
    /// nanoseconds is at least 10<sup>9</sup>, it will wrap to the number of
    /// seconds.
    ///
    /// ```rust
    /// # use time::{Duration, prelude::*};
    /// assert_eq!(Duration::new(1, 0), 1.seconds());
    /// assert_eq!(Duration::new(-1, 0), (-1).seconds());
    /// assert_eq!(Duration::new(1, 2_000_000_000), 3.seconds());
    /// ```
    #[inline(always)]
    pub fn new(seconds: i64, nanoseconds: i32) -> Self {
        match seconds.checked_add(nanoseconds as i64 / 1_000_000_000) {
            Some(seconds) => Self {
                seconds,
                nanoseconds: nanoseconds % 1_000_000_000,
            },
            None if seconds > 0 => Self::max_value,
            None => Self::min_value,
        }
    }

    /// Create a new `Duration` with the given number of weeks. Equivalent to
    /// `Duration::seconds(weeks * 604_800)`.
    ///
    /// ```rust
    /// # use time::{Duration, prelude::*};
    /// assert_eq!(Duration::weeks(1), 604_800.seconds());
    /// ```
    #[inline(always)]
    pub const fn weeks(weeks: i64) -> Self {
        Self::seconds(weeks * SECONDS_PER_WEEK)
    }

    /// Get the number of whole weeks in the duration.
    ///
    /// ```rust
    /// # use time::prelude::*;
    /// assert_eq!(1.weeks().whole_weeks(), 1);
    /// assert_eq!((-1).weeks().whole_weeks(), -1);
    /// assert_eq!(6.days().whole_weeks(), 0);
    /// assert_eq!((-6).days().whole_weeks(), 0);
    /// ```
    #[inline(always)]
    pub const fn whole_weeks(self) -> i64 {
        self.whole_seconds() / SECONDS_PER_WEEK
    }

    /// Create a new `Duration` with the given number of days. Equivalent to
    /// `Duration::seconds(days * 86_400)`.
    ///
    /// ```rust
    /// # use time::{Duration, prelude::*};
    /// assert_eq!(Duration::days(1), 86_400.seconds());
    /// ```
    #[inline(always)]
    pub const fn days(days: i64) -> Self {
        Self::seconds(days * SECONDS_PER_DAY)
    }

    /// Get the number of whole days in the duration.
    ///
    /// ```rust
    /// # use time::prelude::*;
    /// assert_eq!(1.days().whole_days(), 1);
    /// assert_eq!((-1).days().whole_days(), -1);
    /// assert_eq!(23.hours().whole_days(), 0);
    /// assert_eq!((-23).hours().whole_days(), 0);
    /// ```
    #[inline(always)]
    pub const fn whole_days(self) -> i64 {
        self.whole_seconds() / SECONDS_PER_DAY
    }

    /// Create a new `Duration` with the given number of hours. Equivalent to
    /// `Duration::seconds(hours * 3_600)`.
    ///
    /// ```rust
    /// # use time::{Duration, prelude::*};
    /// assert_eq!(Duration::hours(1), 3_600.seconds());
    /// ```
    #[inline(always)]
    pub const fn hours(hours: i64) -> Self {
        Self::seconds(hours * SECONDS_PER_HOUR)
    }

    /// Get the number of whole hours in the duration.
    ///
    /// ```rust
    /// # use time::prelude::*;
    /// assert_eq!(1.hours().whole_hours(), 1);
    /// assert_eq!((-1).hours().whole_hours(), -1);
    /// assert_eq!(59.minutes().whole_hours(), 0);
    /// assert_eq!((-59).minutes().whole_hours(), 0);
    /// ```
    #[inline(always)]
    pub const fn whole_hours(self) -> i64 {
        self.whole_seconds() / SECONDS_PER_HOUR
    }

    /// Create a new `Duration` with the given number of minutes. Equivalent to
    /// `Duration::seconds(minutes * 60)`.
    ///
    /// ```rust
    /// # use time::{Duration, prelude::*};
    /// assert_eq!(Duration::minutes(1), 60.seconds());
    /// ```
    #[inline(always)]
    pub const fn minutes(minutes: i64) -> Self {
        Self::seconds(minutes * SECONDS_PER_MINUTE)
    }

    /// Get the number of whole minutes in the duration.
    ///
    /// ```rust
    /// # use time::prelude::*;
    /// assert_eq!(1.minutes().whole_minutes(), 1);
    /// assert_eq!((-1).minutes().whole_minutes(), -1);
    /// assert_eq!(59.seconds().whole_minutes(), 0);
    /// assert_eq!((-59).seconds().whole_minutes(), 0);
    /// ```
    #[inline(always)]
    pub const fn whole_minutes(self) -> i64 {
        self.whole_seconds() / SECONDS_PER_MINUTE
    }

    /// Create a new `Duration` with the given number of seconds.
    ///
    /// ```rust
    /// # use time::{Duration, prelude::*};
    /// assert_eq!(Duration::seconds(1), 1_000.milliseconds());
    /// ```
    #[inline(always)]
    pub const fn seconds(seconds: i64) -> Self {
        Self {
            seconds,
            nanoseconds: 0,
        }
    }

    /// Get the number of whole seconds in the duration.
    ///
    /// ```rust
    /// # use time::prelude::*;
    /// assert_eq!(1.seconds().whole_seconds(), 1);
    /// assert_eq!((-1).seconds().whole_seconds(), -1);
    /// assert_eq!(1.minutes().whole_seconds(), 60);
    /// assert_eq!((-1).minutes().whole_seconds(), -60);
    /// ```
    #[inline(always)]
    pub const fn whole_seconds(self) -> i64 {
        self.seconds
    }

    /// Creates a new `Duration` from the specified number of seconds
    /// represented as `f64`.
    ///
    /// ```rust
    /// # use time::{Duration, prelude::*};
    /// assert_eq!(Duration::seconds_f64(0.5), 0.5.seconds());
    /// assert_eq!(Duration::seconds_f64(-0.5), -0.5.seconds());
    /// ```
    #[inline(always)]
    pub fn seconds_f64(seconds: f64) -> Self {
        Self {
            seconds: seconds as i64,
            nanoseconds: ((seconds % 1.) * 1_000_000_000.) as i32,
        }
    }

    /// Get the number of fractional seconds in the duration.
    ///
    /// ```rust
    /// # use time::prelude::*;
    /// assert_eq!(1.5.seconds().as_seconds_f64(), 1.5);
    /// assert_eq!((-1.5).seconds().as_seconds_f64(), -1.5);
    /// ```
    #[inline(always)]
    pub fn as_seconds_f64(self) -> f64 {
        self.seconds as f64 + self.nanoseconds as f64 / 1_000_000_000.
    }

    /// Creates a new `Duration` from the specified number of seconds
    /// represented as `f32`.
    ///
    /// ```rust
    /// # use time::{Duration, prelude::*};
    /// assert_eq!(Duration::seconds_f32(0.5), 0.5.seconds());
    /// assert_eq!(Duration::seconds_f32(-0.5), (-0.5).seconds());
    /// ```
    #[inline(always)]
    pub fn seconds_f32(seconds: f32) -> Self {
        Self {
            seconds: seconds as i64,
            nanoseconds: ((seconds % 1.) * 1_000_000_000.) as i32,
        }
    }

    /// Get the number of fractional seconds in the duration.
    ///
    /// ```rust
    /// # use time::prelude::*;
    /// assert_eq!(1.5.seconds().as_seconds_f32(), 1.5);
    /// assert_eq!((-1.5).seconds().as_seconds_f32(), -1.5);
    /// ```
    #[inline(always)]
    pub fn as_seconds_f32(self) -> f32 {
        self.seconds as f32 + self.nanoseconds as f32 / 1_000_000_000.
    }

    /// Create a new `Duration` with the given number of milliseconds.
    ///
    /// ```rust
    /// # use time::{Duration, prelude::*};
    /// assert_eq!(Duration::milliseconds(1), 1_000.microseconds());
    /// assert_eq!(Duration::milliseconds(-1), (-1_000).microseconds());
    /// ```
    #[inline(always)]
    pub const fn milliseconds(milliseconds: i64) -> Self {
        Self {
            seconds: milliseconds / 1_000,
            nanoseconds: ((milliseconds % 1_000) * 1_000_000) as i32,
        }
    }

    /// Get the number of whole milliseconds in the duration.
    ///
    /// ```rust
    /// # use time::prelude::*;
    /// assert_eq!(1.seconds().whole_milliseconds(), 1_000);
    /// assert_eq!((-1).seconds().whole_milliseconds(), -1_000);
    /// assert_eq!(1.milliseconds().whole_milliseconds(), 1);
    /// assert_eq!((-1).milliseconds().whole_milliseconds(), -1);
    /// ```
    #[inline(always)]
    pub const fn whole_milliseconds(self) -> i128 {
        self.seconds as i128 * 1_000 + self.nanoseconds as i128 / 1_000_000
    }

    /// Get the number of milliseconds past the number of whole seconds.
    ///
    /// Always in the range `-1_000..1_000`.
    ///
    /// ```rust
    /// # use time::prelude::*;
    /// assert_eq!(1.4.seconds().subsec_milliseconds(), 400);
    /// assert_eq!((-1.4).seconds().subsec_milliseconds(), -400);
    /// ```
    // Allow the lint, as the value is guaranteed to be less than 1000.
    #[inline(always)]
    pub const fn subsec_milliseconds(self) -> i16 {
        (self.nanoseconds / 1_000_000) as i16
    }

    /// Create a new `Duration` with the given number of microseconds.
    ///
    /// ```rust
    /// # use time::{Duration, prelude::*};
    /// assert_eq!(Duration::microseconds(1), 1_000.nanoseconds());
    /// assert_eq!(Duration::microseconds(-1), (-1_000).nanoseconds());
    /// ```
    #[inline(always)]
    pub const fn microseconds(microseconds: i64) -> Self {
        Self {
            seconds: microseconds / 1_000_000,
            nanoseconds: ((microseconds % 1_000_000) * 1_000) as i32,
        }
    }

    /// Get the number of whole microseconds in the duration.
    ///
    /// ```rust
    /// # use time::prelude::*;
    /// assert_eq!(1.milliseconds().whole_microseconds(), 1_000);
    /// assert_eq!((-1).milliseconds().whole_microseconds(), -1_000);
    /// assert_eq!(1.microseconds().whole_microseconds(), 1);
    /// assert_eq!((-1).microseconds().whole_microseconds(), -1);
    /// ```
    #[inline(always)]
    pub const fn whole_microseconds(self) -> i128 {
        self.seconds as i128 * 1_000_000 + self.nanoseconds as i128 / 1_000
    }

    /// Get the number of microseconds past the number of whole seconds.
    ///
    /// Always in the range `-1_000_000..1_000_000`.
    ///
    /// ```rust
    /// # use time::prelude::*;
    /// assert_eq!(1.0004.seconds().subsec_microseconds(), 400);
    /// assert_eq!((-1.0004).seconds().subsec_microseconds(), -400);
    /// ```
    #[inline(always)]
    pub const fn subsec_microseconds(self) -> i32 {
        self.nanoseconds / 1_000
    }

    /// Create a new `Duration` with the given number of nanoseconds.
    ///
    /// ```rust
    /// # use time::{Duration, prelude::*};
    /// assert_eq!(Duration::nanoseconds(1), 1.microseconds() / 1_000);
    /// assert_eq!(Duration::nanoseconds(-1), (-1).microseconds() / 1_000);
    /// ```
    #[inline(always)]
    pub const fn nanoseconds(nanoseconds: i64) -> Self {
        Self {
            seconds: nanoseconds / 1_000_000_000,
            nanoseconds: (nanoseconds % 1_000_000_000) as i32,
        }
    }

    /// Create a new `Duration` with the given number of nanoseconds.
    // TODO Convert `nanoseconds()` to accept an i128 in a future major release
    // after const if/match lands on stable
    #[inline]
    pub(crate) fn nanoseconds_i128(nanoseconds: i128) -> Self {
        if nanoseconds > Duration::max_value.whole_nanoseconds() {
            Duration::max_value
        } else if nanoseconds < Duration::min_value.whole_nanoseconds() {
            Duration::min_value
        } else {
            Self {
                seconds: (nanoseconds / 1_000_000_000) as i64,
                nanoseconds: (nanoseconds % 1_000_000_000) as i32,
            }
        }
    }

    /// Get the number of nanoseconds in the duration.
    ///
    /// ```rust
    /// # use time::prelude::*;
    /// assert_eq!(1.microseconds().whole_nanoseconds(), 1_000);
    /// assert_eq!((-1).microseconds().whole_nanoseconds(), -1_000);
    /// assert_eq!(1.nanoseconds().whole_nanoseconds(), 1);
    /// assert_eq!((-1).nanoseconds().whole_nanoseconds(), -1);
    /// ```
    #[inline(always)]
    pub const fn whole_nanoseconds(self) -> i128 {
        self.seconds as i128 * 1_000_000_000 + self.nanoseconds as i128
    }

    /// Get the number of nanoseconds past the number of whole seconds.
    ///
    /// The returned value will always be in the range
    /// `-1_000_000_000..1_000_000_000`.
    ///
    /// ```rust
    /// # use time::prelude::*;
    /// assert_eq!(1.000_000_400.seconds().subsec_nanoseconds(), 400);
    /// assert_eq!((-1.000_000_400).seconds().subsec_nanoseconds(), -400);
    /// ```
    #[inline(always)]
    pub const fn subsec_nanoseconds(self) -> i32 {
        self.nanoseconds
    }

    /// Runs a closure, returning the duration of time it took to run. The
    /// return value of the closure is provided in the second part of the tuple.
    #[inline(always)]
    #[cfg(std)]
    #[cfg_attr(docs, doc(cfg(feature = "std")))]
    pub fn time_fn<T>(f: impl FnOnce() -> T) -> (Self, T) {
        let start = Instant::now();
        let return_value = f();
        let end = Instant::now();

        (end - start, return_value)
    }
}

impl TryFrom<StdDuration> for Duration {
    type Error = error::ConversionRange;

    #[inline(always)]
    fn try_from(original: StdDuration) -> Result<Self, error::ConversionRange> {
        Ok(Self::new(
            original
                .as_secs()
                .try_into()
                .map_err(|_| error::ConversionRange)?,
            original
                .subsec_nanos()
                .try_into()
                .map_err(|_| error::ConversionRange)?,
        ))
    }
}

impl TryFrom<Duration> for StdDuration {
    type Error = error::ConversionRange;

    #[inline(always)]
    fn try_from(duration: Duration) -> Result<Self, error::ConversionRange> {
        Ok(Self::new(
            duration
                .seconds
                .try_into()
                .map_err(|_| error::ConversionRange)?,
            duration
                .nanoseconds
                .try_into()
                .map_err(|_| error::ConversionRange)?,
        ))
    }
}

impl Add for Duration {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Duration::nanoseconds_i128(
            self.whole_nanoseconds()
                .saturating_add(rhs.whole_nanoseconds()),
        )
    }
}

impl Add<StdDuration> for Duration {
    type Output = Self;

    #[inline(always)]
    fn add(self, std_duration: StdDuration) -> Self::Output {
        Duration::nanoseconds_i128(
            self.whole_nanoseconds()
                .saturating_add(std_duration.as_nanos() as i128),
        )
    }
}

impl Add<Duration> for StdDuration {
    type Output = Duration;

    #[inline(always)]
    fn add(self, rhs: Duration) -> Self::Output {
        rhs + self
    }
}

impl AddAssign for Duration {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl AddAssign<StdDuration> for Duration {
    #[inline(always)]
    fn add_assign(&mut self, rhs: StdDuration) {
        *self = *self + rhs;
    }
}

impl Neg for Duration {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self::Output {
        -1 * self
    }
}

impl Sub for Duration {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Duration::nanoseconds_i128(
            self.whole_nanoseconds()
                .saturating_sub(rhs.whole_nanoseconds()),
        )
    }
}

impl Sub<StdDuration> for Duration {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: StdDuration) -> Self::Output {
        Duration::nanoseconds_i128(
            self.whole_nanoseconds()
                .saturating_sub(rhs.as_nanos() as i128),
        )
    }
}

impl Sub<Duration> for StdDuration {
    type Output = Duration;

    #[inline(always)]
    fn sub(self, rhs: Duration) -> Self::Output {
        Duration::nanoseconds_i128(
            (self.as_nanos() as i128).saturating_sub(rhs.whole_nanoseconds()),
        )
    }
}

impl SubAssign for Duration {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl SubAssign<StdDuration> for Duration {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: StdDuration) {
        *self = *self - rhs;
    }
}

impl SubAssign<Duration> for StdDuration {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Duration) {
        // Don't saturate here, as `std::time::Duration` doesn't allow for
        // negative values. The lowest is zero seconds, which is likely not
        // what's desired.
        *self = (*self - rhs).try_into().expect(
            "Cannot represent a resulting duration in core::time::Duration. Try `let x = x - \
             rhs;`, which will change the type.",
        );
    }
}

macro_rules! duration_mul_div_int {
    ($($type:ty),+) => {
        $(
            impl Mul<$type> for Duration {
                type Output = Self;

                #[inline(always)]
                fn mul(self, rhs: $type) -> Self::Output {
                    Self::nanoseconds_i128(self.whole_nanoseconds().saturating_mul(rhs as i128))
                }
            }

            impl MulAssign<$type> for Duration {
                #[inline(always)]
                fn mul_assign(&mut self, rhs: $type) {
                    *self = *self * rhs;
                }
            }

            impl Mul<Duration> for $type {
                type Output = Duration;

                #[inline(always)]
                fn mul(self, rhs: Duration) -> Self::Output {
                    rhs * self
                }
            }

            impl Div<$type> for Duration {
                type Output = Self;

                #[inline(always)]
                fn div(self, rhs: $type) -> Self::Output {
                    Self::nanoseconds_i128(self.whole_nanoseconds() / rhs as i128)
                }
            }

            impl DivAssign<$type> for Duration {
                #[inline(always)]
                fn div_assign(&mut self, rhs: $type) {
                    *self = *self / rhs;
                }
            }
        )+
    };
}
duration_mul_div_int![i8, i16, i32, u8, u16, u32];

impl Mul<f32> for Duration {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: f32) -> Self::Output {
        Self::seconds_f32(self.as_seconds_f32() * rhs)
    }
}

impl MulAssign<f32> for Duration {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl Mul<Duration> for f32 {
    type Output = Duration;

    #[inline(always)]
    fn mul(self, rhs: Duration) -> Self::Output {
        rhs * self
    }
}

impl Mul<f64> for Duration {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: f64) -> Self::Output {
        Self::seconds_f64(self.as_seconds_f64() * rhs)
    }
}

impl MulAssign<f64> for Duration {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs;
    }
}

impl Mul<Duration> for f64 {
    type Output = Duration;

    #[inline(always)]
    fn mul(self, rhs: Duration) -> Self::Output {
        rhs * self
    }
}

impl Div<f32> for Duration {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: f32) -> Self::Output {
        Self::seconds_f32(self.as_seconds_f32() / rhs)
    }
}

impl DivAssign<f32> for Duration {
    #[inline(always)]
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs;
    }
}

impl Div<f64> for Duration {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: f64) -> Self::Output {
        Self::seconds_f64(self.as_seconds_f64() / rhs)
    }
}

impl DivAssign<f64> for Duration {
    #[inline(always)]
    fn div_assign(&mut self, rhs: f64) {
        *self = *self / rhs;
    }
}

impl Div<Duration> for Duration {
    type Output = f64;

    #[inline(always)]
    fn div(self, rhs: Self) -> Self::Output {
        self.as_seconds_f64() / rhs.as_seconds_f64()
    }
}

impl Div<StdDuration> for Duration {
    type Output = f64;

    #[inline(always)]
    fn div(self, rhs: StdDuration) -> Self::Output {
        self.as_seconds_f64() / rhs.as_secs_f64()
    }
}

impl Div<Duration> for StdDuration {
    type Output = f64;

    #[inline(always)]
    fn div(self, rhs: Duration) -> Self::Output {
        self.as_secs_f64() / rhs.as_seconds_f64()
    }
}

impl PartialEq<StdDuration> for Duration {
    #[inline(always)]
    fn eq(&self, rhs: &StdDuration) -> bool {
        Ok(*self) == Self::try_from(*rhs)
    }
}

impl PartialEq<Duration> for StdDuration {
    #[inline(always)]
    fn eq(&self, rhs: &Duration) -> bool {
        rhs == self
    }
}

impl PartialOrd for Duration {
    #[inline(always)]
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

impl PartialOrd<StdDuration> for Duration {
    #[inline(always)]
    fn partial_cmp(&self, rhs: &StdDuration) -> Option<Ordering> {
        if rhs.as_secs() > i64::max_value() as u64 {
            return Some(Ordering::Greater);
        }

        Some(
            self.seconds
                .cmp(&(rhs.as_secs() as i64))
                .then_with(|| self.nanoseconds.cmp(&(rhs.subsec_nanos() as i32))),
        )
    }
}

impl PartialOrd<Duration> for StdDuration {
    #[inline(always)]
    fn partial_cmp(&self, rhs: &Duration) -> Option<Ordering> {
        rhs.partial_cmp(self).map(Ordering::reverse)
    }
}

impl Ord for Duration {
    #[inline]
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.seconds
            .cmp(&rhs.seconds)
            .then_with(|| self.nanoseconds.cmp(&rhs.nanoseconds))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{NumericalDuration, NumericalStdDuration};

    #[test]
    fn unit_values() {
        assert_eq!(Duration::zero, 0.seconds());
        assert_eq!(Duration::nanosecond, 1.nanoseconds());
        assert_eq!(Duration::microsecond, 1.microseconds());
        assert_eq!(Duration::millisecond, 1.milliseconds());
        assert_eq!(Duration::second, 1.seconds());
        assert_eq!(Duration::minute, 60.seconds());
        assert_eq!(Duration::hour, 3_600.seconds());
        assert_eq!(Duration::day, 86_400.seconds());
        assert_eq!(Duration::week, 604_800.seconds());
    }

    #[test]
    fn is_zero() {
        assert!(!(-1).nanoseconds().is_zero());
        assert!(0.seconds().is_zero());
        assert!(!1.nanoseconds().is_zero());
    }

    #[test]
    fn is_negative() {
        assert!((-1).seconds().is_negative());
        assert!(!0.seconds().is_negative());
        assert!(!1.seconds().is_negative());
    }

    #[test]
    fn is_positive() {
        assert!(!(-1).seconds().is_positive());
        assert!(!0.seconds().is_positive());
        assert!(1.seconds().is_positive());
    }

    #[test]
    fn abs() {
        assert_eq!(1.seconds().abs(), 1.seconds());
        assert_eq!(0.seconds().abs(), 0.seconds());
        assert_eq!((-1).seconds().abs(), 1.seconds());
    }

    #[test]
    fn new() {
        assert_eq!(Duration::new(1, 0), 1.seconds());
        assert_eq!(Duration::new(-1, 0), (-1).seconds());
        assert_eq!(Duration::new(1, 2_000_000_000), 3.seconds());

        assert!(Duration::new(0, 0).is_zero());
        assert!(Duration::new(0, 1_000_000_000).is_positive());
        assert!(Duration::new(-1, 1_000_000_000).is_zero());
        assert!(Duration::new(-2, 1_000_000_000).is_negative());
    }

    #[test]
    fn weeks() {
        assert_eq!(Duration::weeks(1), 604_800.seconds());
        assert_eq!(Duration::weeks(2), (2 * 604_800).seconds());
        assert_eq!(Duration::weeks(-1), (-604_800).seconds());
        assert_eq!(Duration::weeks(-2), (2 * -604_800).seconds());
    }

    #[test]
    fn whole_weeks() {
        assert_eq!(Duration::weeks(1).whole_weeks(), 1);
        assert_eq!(Duration::weeks(-1).whole_weeks(), -1);
        assert_eq!(Duration::days(6).whole_weeks(), 0);
        assert_eq!(Duration::days(-6).whole_weeks(), 0);
    }

    #[test]
    fn days() {
        assert_eq!(Duration::days(1), 86_400.seconds());
        assert_eq!(Duration::days(2), (2 * 86_400).seconds());
        assert_eq!(Duration::days(-1), (-86_400).seconds());
        assert_eq!(Duration::days(-2), (2 * -86_400).seconds());
    }

    #[test]
    fn whole_days() {
        assert_eq!(Duration::days(1).whole_days(), 1);
        assert_eq!(Duration::days(-1).whole_days(), -1);
        assert_eq!(Duration::hours(23).whole_days(), 0);
        assert_eq!(Duration::hours(-23).whole_days(), 0);
    }

    #[test]
    fn hours() {
        assert_eq!(Duration::hours(1), 3_600.seconds());
        assert_eq!(Duration::hours(2), (2 * 3_600).seconds());
        assert_eq!(Duration::hours(-1), (-3_600).seconds());
        assert_eq!(Duration::hours(-2), (2 * -3_600).seconds());
    }

    #[test]
    fn whole_hours() {
        assert_eq!(Duration::hours(1).whole_hours(), 1);
        assert_eq!(Duration::hours(-1).whole_hours(), -1);
        assert_eq!(Duration::minutes(59).whole_hours(), 0);
        assert_eq!(Duration::minutes(-59).whole_hours(), 0);
    }

    #[test]
    fn minutes() {
        assert_eq!(Duration::minutes(1), 60.seconds());
        assert_eq!(Duration::minutes(2), (2 * 60).seconds());
        assert_eq!(Duration::minutes(-1), (-60).seconds());
        assert_eq!(Duration::minutes(-2), (2 * -60).seconds());
    }

    #[test]
    fn whole_minutes() {
        assert_eq!(1.minutes().whole_minutes(), 1);
        assert_eq!((-1).minutes().whole_minutes(), -1);
        assert_eq!(59.seconds().whole_minutes(), 0);
        assert_eq!((-59).seconds().whole_minutes(), 0);
    }

    #[test]
    fn seconds() {
        assert_eq!(Duration::seconds(1), 1_000.milliseconds());
        assert_eq!(Duration::seconds(2), (2 * 1_000).milliseconds());
        assert_eq!(Duration::seconds(-1), (-1_000).milliseconds());
        assert_eq!(Duration::seconds(-2), (2 * -1_000).milliseconds());
    }

    #[test]
    fn whole_seconds() {
        assert_eq!(1.seconds().whole_seconds(), 1);
        assert_eq!((-1).seconds().whole_seconds(), -1);
        assert_eq!(1.minutes().whole_seconds(), 60);
        assert_eq!((-1).minutes().whole_seconds(), -60);
    }

    #[test]
    fn seconds_f64() {
        assert_eq!(Duration::seconds_f64(0.5), 0.5.seconds());
        assert_eq!(Duration::seconds_f64(-0.5), (-0.5).seconds());
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn as_seconds_f64() {
        assert_eq!(1.seconds().as_seconds_f64(), 1.0);
        assert_eq!((-1).seconds().as_seconds_f64(), -1.0);
        assert_eq!(1.minutes().as_seconds_f64(), 60.0);
        assert_eq!((-1).minutes().as_seconds_f64(), -60.0);
        assert_eq!(1.5.seconds().as_seconds_f64(), 1.5);
        assert_eq!((-1.5).seconds().as_seconds_f64(), -1.5);
    }

    #[test]
    fn seconds_f32() {
        assert_eq!(Duration::seconds_f32(0.5), 0.5.seconds());
        assert_eq!(Duration::seconds_f32(-0.5), (-0.5).seconds());
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn as_seconds_f32() {
        assert_eq!(1.seconds().as_seconds_f32(), 1.0);
        assert_eq!((-1).seconds().as_seconds_f32(), -1.0);
        assert_eq!(1.minutes().as_seconds_f32(), 60.0);
        assert_eq!((-1).minutes().as_seconds_f32(), -60.0);
        assert_eq!(1.5.seconds().as_seconds_f32(), 1.5);
        assert_eq!((-1.5).seconds().as_seconds_f32(), -1.5);
    }

    #[test]
    fn milliseconds() {
        assert_eq!(Duration::milliseconds(1), 1_000.microseconds());
        assert_eq!(Duration::milliseconds(-1), (-1000).microseconds());
    }

    #[test]
    fn whole_milliseconds() {
        assert_eq!(1.seconds().whole_milliseconds(), 1_000);
        assert_eq!((-1).seconds().whole_milliseconds(), -1_000);
        assert_eq!(1.milliseconds().whole_milliseconds(), 1);
        assert_eq!((-1).milliseconds().whole_milliseconds(), -1);
    }

    #[test]
    fn subsec_milliseconds() {
        assert_eq!(1.4.seconds().subsec_milliseconds(), 400);
        assert_eq!((-1.4).seconds().subsec_milliseconds(), -400);
    }

    #[test]
    fn microseconds() {
        assert_eq!(Duration::microseconds(1), 1_000.nanoseconds());
        assert_eq!(Duration::microseconds(-1), (-1_000).nanoseconds());
    }

    #[test]
    fn whole_microseconds() {
        assert_eq!(1.milliseconds().whole_microseconds(), 1_000);
        assert_eq!((-1).milliseconds().whole_microseconds(), -1_000);
        assert_eq!(1.microseconds().whole_microseconds(), 1);
        assert_eq!((-1).microseconds().whole_microseconds(), -1);
    }

    #[test]
    fn subsec_microseconds() {
        assert_eq!(1.0004.seconds().subsec_microseconds(), 400);
        assert_eq!((-1.0004).seconds().subsec_microseconds(), -400);
    }

    #[test]
    fn nanoseconds() {
        assert_eq!(Duration::nanoseconds(1), 1.microseconds() / 1_000);
        assert_eq!(Duration::nanoseconds(-1), (-1).microseconds() / 1_000);
    }

    #[test]
    fn whole_nanoseconds() {
        assert_eq!(1.microseconds().whole_nanoseconds(), 1_000);
        assert_eq!((-1).microseconds().whole_nanoseconds(), -1_000);
        assert_eq!(1.nanoseconds().whole_nanoseconds(), 1);
        assert_eq!((-1).nanoseconds().whole_nanoseconds(), -1);
    }

    #[test]
    fn subsec_nanoseconds() {
        assert_eq!(1.000_000_4.seconds().subsec_nanoseconds(), 400);
        assert_eq!((-1.000_000_4).seconds().subsec_nanoseconds(), -400);
    }

    #[test]
    #[cfg(std)]
    fn time_fn() {
        let (time, value) = Duration::time_fn(|| {
            std::thread::sleep(100.std_milliseconds());
            0
        });

        assert!(time >= 100.milliseconds());
        assert_eq!(value, 0);
    }

    #[test]
    fn try_from_std_duration() {
        assert_eq!(Duration::try_from(0.std_seconds()), Ok(0.seconds()));
        assert_eq!(Duration::try_from(1.std_seconds()), Ok(1.seconds()));
    }

    #[test]
    fn try_to_std_duration() {
        assert_eq!(StdDuration::try_from(0.seconds()), Ok(0.std_seconds()));
        assert_eq!(StdDuration::try_from(1.seconds()), Ok(1.std_seconds()));
        assert!(StdDuration::try_from((-1).seconds()).is_err());
    }

    #[test]
    fn add() {
        assert_eq!(1.seconds() + 1.seconds(), 2.seconds());
        assert_eq!(500.milliseconds() + 500.milliseconds(), 1.seconds());
        assert_eq!(1.seconds() + (-1).seconds(), 0.seconds());
    }

    #[test]
    fn add_std() {
        assert_eq!(1.seconds() + 1.std_seconds(), 2.seconds());
        assert_eq!(500.milliseconds() + 500.std_milliseconds(), 1.seconds());
        assert_eq!((-1).seconds() + 1.std_seconds(), 0.seconds());
    }

    #[test]
    fn std_add() {
        assert_eq!(1.std_seconds() + 1.seconds(), 2.seconds());
        assert_eq!(500.std_milliseconds() + 500.milliseconds(), 1.seconds());
        assert_eq!(1.std_seconds() + (-1).seconds(), 0.seconds());
    }

    #[test]
    fn add_assign() {
        let mut duration = 1.seconds();
        duration += 1.seconds();
        assert_eq!(duration, 2.seconds());

        let mut duration = 500.milliseconds();
        duration += 500.milliseconds();
        assert_eq!(duration, 1.seconds());

        let mut duration = 1.seconds();
        duration += (-1).seconds();
        assert_eq!(duration, 0.seconds());
    }

    #[test]
    fn add_assign_std() {
        let mut duration = 1.seconds();
        duration += 1.std_seconds();
        assert_eq!(duration, 2.seconds());

        let mut duration = 500.milliseconds();
        duration += 500.std_milliseconds();
        assert_eq!(duration, 1.seconds());

        let mut duration = (-1).seconds();
        duration += 1.std_seconds();
        assert_eq!(duration, 0.seconds());
    }

    #[test]
    fn neg() {
        assert_eq!(-(1.seconds()), (-1).seconds());
        assert_eq!(-(-1).seconds(), 1.seconds());
        assert_eq!(-(0.seconds()), 0.seconds());
    }

    #[test]
    fn sub() {
        assert_eq!(1.seconds() - 1.seconds(), 0.seconds());
        assert_eq!(1_500.milliseconds() - 500.milliseconds(), 1.seconds());
        assert_eq!(1.seconds() - (-1).seconds(), 2.seconds());
    }

    #[test]
    fn sub_std() {
        assert_eq!(1.seconds() - 1.std_seconds(), 0.seconds());
        assert_eq!(1_500.milliseconds() - 500.std_milliseconds(), 1.seconds());
        assert_eq!((-1).seconds() - 1.std_seconds(), (-2).seconds());
    }

    #[test]
    fn std_sub() {
        assert_eq!(1.std_seconds() - 1.seconds(), 0.seconds());
        assert_eq!(1_500.std_milliseconds() - 500.milliseconds(), 1.seconds());
        assert_eq!(1.std_seconds() - (-1).seconds(), 2.seconds());
    }

    #[test]
    fn sub_assign() {
        let mut duration = 1.seconds();
        duration -= 1.seconds();
        assert_eq!(duration, 0.seconds());

        let mut duration = 1_500.milliseconds();
        duration -= 500.milliseconds();
        assert_eq!(duration, 1.seconds());

        let mut duration = 1.seconds();
        duration -= (-1).seconds();
        assert_eq!(duration, 2.seconds());
    }

    #[test]
    fn sub_assign_std() {
        let mut duration = 1.seconds();
        duration -= 1.std_seconds();
        assert_eq!(duration, 0.seconds());

        let mut duration = 1_500.milliseconds();
        duration -= 500.std_milliseconds();
        assert_eq!(duration, 1.seconds());

        let mut duration = (-1).seconds();
        duration -= 1.std_seconds();
        assert_eq!(duration, (-2).seconds());
    }

    #[test]
    fn std_sub_assign() {
        let mut duration = 1.std_seconds();
        duration -= 1.seconds();
        assert_eq!(duration, 0.seconds());

        let mut duration = 1_500.std_milliseconds();
        duration -= 500.milliseconds();
        assert_eq!(duration, 1.seconds());
    }

    #[test]
    #[should_panic]
    fn std_sub_assign_panic() {
        let mut duration = 1.std_seconds();
        duration -= 2.seconds();
    }

    #[test]
    fn mul_int() {
        assert_eq!(1.seconds() * 2, 2.seconds());
        assert_eq!(1.seconds() * -2, (-2).seconds());
    }

    #[test]
    fn mul_int_assign() {
        let mut duration = 1.seconds();
        duration *= 2;
        assert_eq!(duration, 2.seconds());

        let mut duration = 1.seconds();
        duration *= -2;
        assert_eq!(duration, (-2).seconds());
    }

    #[test]
    fn int_mul() {
        assert_eq!(2 * 1.seconds(), 2.seconds());
        assert_eq!(-2 * 1.seconds(), (-2).seconds());
    }

    #[test]
    fn div_int() {
        assert_eq!(1.seconds() / 2, 500.milliseconds());
        assert_eq!(1.seconds() / -2, (-500).milliseconds());
    }

    #[test]
    fn div_int_assign() {
        let mut duration = 1.seconds();
        duration /= 2;
        assert_eq!(duration, 500.milliseconds());

        let mut duration = 1.seconds();
        duration /= -2;
        assert_eq!(duration, (-500).milliseconds());
    }

    #[test]
    fn mul_float() {
        assert_eq!(1.seconds() * 1.5_f32, 1_500.milliseconds());
        assert_eq!(1.seconds() * 2.5_f32, 2_500.milliseconds());
        assert_eq!(1.seconds() * -1.5_f32, (-1_500).milliseconds());
        assert_eq!(1.seconds() * 0_f32, 0.seconds());

        assert_eq!(1.seconds() * 1.5_f64, 1_500.milliseconds());
        assert_eq!(1.seconds() * 2.5_f64, 2_500.milliseconds());
        assert_eq!(1.seconds() * -1.5_f64, (-1_500).milliseconds());
        assert_eq!(1.seconds() * 0_f64, 0.seconds());
    }

    #[test]
    fn float_mul() {
        assert_eq!(1.5_f32 * 1.seconds(), 1_500.milliseconds());
        assert_eq!(2.5_f32 * 1.seconds(), 2_500.milliseconds());
        assert_eq!(-1.5_f32 * 1.seconds(), (-1_500).milliseconds());
        assert_eq!(0_f32 * 1.seconds(), 0.seconds());

        assert_eq!(1.5_f64 * 1.seconds(), 1_500.milliseconds());
        assert_eq!(2.5_f64 * 1.seconds(), 2_500.milliseconds());
        assert_eq!(-1.5_f64 * 1.seconds(), (-1_500).milliseconds());
        assert_eq!(0_f64 * 1.seconds(), 0.seconds());
    }

    #[test]
    fn mul_float_assign() {
        let mut duration = 1.seconds();
        duration *= 1.5_f32;
        assert_eq!(duration, 1_500.milliseconds());

        let mut duration = 1.seconds();
        duration *= 2.5_f32;
        assert_eq!(duration, 2_500.milliseconds());

        let mut duration = 1.seconds();
        duration *= -1.5_f32;
        assert_eq!(duration, (-1_500).milliseconds());

        let mut duration = 1.seconds();
        duration *= 0_f32;
        assert_eq!(duration, 0.seconds());

        let mut duration = 1.seconds();
        duration *= 1.5_f64;
        assert_eq!(duration, 1_500.milliseconds());

        let mut duration = 1.seconds();
        duration *= 2.5_f64;
        assert_eq!(duration, 2_500.milliseconds());

        let mut duration = 1.seconds();
        duration *= -1.5_f64;
        assert_eq!(duration, (-1_500).milliseconds());

        let mut duration = 1.seconds();
        duration *= 0_f64;
        assert_eq!(duration, 0.seconds());
    }

    #[test]
    fn div_float() {
        assert_eq!(1.seconds() / 1_f32, 1.seconds());
        assert_eq!(1.seconds() / 2_f32, 500.milliseconds());
        assert_eq!(1.seconds() / -1_f32, (-1).seconds());

        assert_eq!(1.seconds() / 1_f64, 1.seconds());
        assert_eq!(1.seconds() / 2_f64, 500.milliseconds());
        assert_eq!(1.seconds() / -1_f64, (-1).seconds());
    }

    #[test]
    fn div_float_assign() {
        let mut duration = 1.seconds();
        duration /= 1_f32;
        assert_eq!(duration, 1.seconds());

        let mut duration = 1.seconds();
        duration /= 2_f32;
        assert_eq!(duration, 500.milliseconds());

        let mut duration = 1.seconds();
        duration /= -1_f32;
        assert_eq!(duration, (-1).seconds());

        let mut duration = 1.seconds();
        duration /= 1_f64;
        assert_eq!(duration, 1.seconds());

        let mut duration = 1.seconds();
        duration /= 2_f64;
        assert_eq!(duration, 500.milliseconds());

        let mut duration = 1.seconds();
        duration /= -1_f64;
        assert_eq!(duration, (-1).seconds());
    }

    #[test]
    fn partial_eq() {
        assert_eq!(1.seconds(), 1.seconds());
        assert_eq!(0.seconds(), 0.seconds());
        assert_eq!((-1).seconds(), (-1).seconds());
        assert_ne!(1.minutes(), (-1).minutes());
        assert_ne!(40.seconds(), 1.minutes());
    }

    #[test]
    fn partial_eq_std() {
        assert_eq!(1.seconds(), 1.std_seconds());
        assert_eq!(0.seconds(), 0.std_seconds());
        assert_ne!((-1).seconds(), 1.std_seconds());
        assert_ne!((-1).minutes(), 1.std_minutes());
        assert_ne!(40.seconds(), 1.std_minutes());
    }

    #[test]
    fn std_partial_eq() {
        assert_eq!(1.std_seconds(), 1.seconds());
        assert_eq!(0.std_seconds(), 0.seconds());
        assert_ne!(1.std_seconds(), (-1).seconds());
        assert_ne!(1.std_minutes(), (-1).minutes());
        assert_ne!(40.std_seconds(), 1.minutes());
    }

    #[test]
    fn partial_ord() {
        use Ordering::*;
        assert_eq!(0.seconds().partial_cmp(&0.seconds()), Some(Equal));
        assert_eq!(1.seconds().partial_cmp(&0.seconds()), Some(Greater));
        assert_eq!(1.seconds().partial_cmp(&(-1).seconds()), Some(Greater));
        assert_eq!((-1).seconds().partial_cmp(&1.seconds()), Some(Less));
        assert_eq!(0.seconds().partial_cmp(&(-1).seconds()), Some(Greater));
        assert_eq!(0.seconds().partial_cmp(&1.seconds()), Some(Less));
        assert_eq!((-1).seconds().partial_cmp(&0.seconds()), Some(Less));
        assert_eq!(1.minutes().partial_cmp(&1.seconds()), Some(Greater));
        assert_eq!((-1).minutes().partial_cmp(&(-1).seconds()), Some(Less));
    }

    #[test]
    fn partial_ord_std() {
        use Ordering::*;
        assert_eq!(0.seconds().partial_cmp(&0.std_seconds()), Some(Equal));
        assert_eq!(1.seconds().partial_cmp(&0.std_seconds()), Some(Greater));
        assert_eq!((-1).seconds().partial_cmp(&1.std_seconds()), Some(Less));
        assert_eq!(0.seconds().partial_cmp(&1.std_seconds()), Some(Less));
        assert_eq!((-1).seconds().partial_cmp(&0.std_seconds()), Some(Less));
        assert_eq!(1.minutes().partial_cmp(&1.std_seconds()), Some(Greater));
    }

    #[test]
    fn std_partial_ord() {
        use Ordering::*;
        assert_eq!(0.std_seconds().partial_cmp(&0.seconds()), Some(Equal));
        assert_eq!(1.std_seconds().partial_cmp(&0.seconds()), Some(Greater));
        assert_eq!(1.std_seconds().partial_cmp(&(-1).seconds()), Some(Greater));
        assert_eq!(0.std_seconds().partial_cmp(&(-1).seconds()), Some(Greater));
        assert_eq!(0.std_seconds().partial_cmp(&1.seconds()), Some(Less));
        assert_eq!(1.std_minutes().partial_cmp(&1.seconds()), Some(Greater));
    }

    #[test]
    fn ord() {
        assert_eq!(0.seconds(), 0.seconds());
        assert!(1.seconds() > 0.seconds());
        assert!(1.seconds() > (-1).seconds());
        assert!((-1).seconds() < 1.seconds());
        assert!(0.seconds() > (-1).seconds());
        assert!(0.seconds() < 1.seconds());
        assert!((-1).seconds() < 0.seconds());
        assert!(1.minutes() > 1.seconds());
        assert!((-1).minutes() < (-1).seconds());
    }

    #[test]
    fn arithmetic_regression() {
        let added = 1.6.seconds() + 1.6.seconds();
        assert_eq!(added.whole_seconds(), 3);
        assert_eq!(added.subsec_milliseconds(), 200);

        let subtracted = 1.6.seconds() - (-1.6).seconds();
        assert_eq!(subtracted.whole_seconds(), 3);
        assert_eq!(subtracted.subsec_milliseconds(), 200);
    }

    #[test]
    fn saturating() {
        assert_eq!(
            StdDuration::new(u64::max_value(), 999_999_999) - Duration::min_value,
            Duration::max_value
        );
        assert_eq!(
            StdDuration::new(u64::max_value(), 999_999_999) + Duration::max_value,
            Duration::max_value
        );
        assert_eq!(
            Duration::max_value + StdDuration::new(u64::max_value(), 999_999_999),
            Duration::max_value
        );
        assert_eq!(
            Duration::min_value - StdDuration::new(u64::max_value(), 999_999_999),
            Duration::min_value
        );
    }
}
