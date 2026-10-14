use core::{
    fmt::Debug,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

use crate::arch::{time::{PLATFORM_TICK_HZ, PlatformTemporal, PlatformTicks, PlatformTimer}, traits::TargetTimer};

const fn gcd(a: PlatformTicks, b: PlatformTicks) -> PlatformTicks {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

const GCD_1K: PlatformTicks = gcd(PLATFORM_TICK_HZ, 1_000);
const GCD_1M: PlatformTicks = gcd(PLATFORM_TICK_HZ, 1_000_000);
const GCD_1G: PlatformTicks = gcd(PLATFORM_TICK_HZ, 1_000_000_000);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Instant {
    ticks: PlatformTicks,
}

impl Instant {
    pub const MIN: Self = Instant {
        ticks: PlatformTicks::MIN,
    };
    pub const MAX: Self = Instant {
        ticks: PlatformTicks::MAX,
    };

    pub const fn from_ticks(ticks: PlatformTicks) -> Self {
        Self { ticks }
    }

    pub const fn as_ticks(&self) -> PlatformTicks {
        self.ticks
    }

    pub const fn duration_since(&self, earlier: Self) -> Duration {
        self.checked_duration_since(earlier).unwrap()
    }

    pub const fn checked_duration_since(&self, earlier: Self) -> Option<Duration> {
        if self.ticks < earlier.ticks {
            None
        } else {
            Some(Duration {
                ticks: self.ticks - earlier.ticks,
            })
        }
    }

    pub const fn saturating_duration_since(&self, earlier: Self) -> Duration {
        Duration {
            ticks: if self.ticks < earlier.ticks {
                0
            } else {
                self.ticks - earlier.ticks
            },
        }
    }

    pub const fn checked_add(&self, duration: Duration) -> Option<Self> {
        if let Some(ticks) = self.ticks.checked_add(duration.ticks) {
            Some(Self { ticks })
        } else {
            None
        }
    }

    pub const fn checked_sub(&self, duration: Duration) -> Option<Self> {
        if let Some(ticks) = self.ticks.checked_sub(duration.ticks) {
            Some(Self { ticks })
        } else {
            None
        }
    }

    pub const fn saturating_add(self, duration: Duration) -> Self {
        Self {
            ticks: self.ticks.saturating_add(duration.ticks),
        }
    }

    pub const fn saturating_sub(self, duration: Duration) -> Self {
        Self {
            ticks: self.ticks.saturating_sub(duration.ticks),
        }
    }

    pub fn elapsed(&self) -> Duration {
        Self::now() - *self
    }

    pub fn now() -> Self {
        Self {
            ticks: PlatformTimer::now(),
        }
    }
}

impl Sub<Self> for Instant {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        self.duration_since(rhs)
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;

    fn sub(self, rhs: Duration) -> Self::Output {
        self.checked_sub(rhs)
            .expect("underflow when subtracting duration from instant")
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, rhs: Duration) -> Self::Output {
        self.checked_add(rhs)
            .expect("overflow when adding duration to instant")
    }
}

impl AddAssign<Duration> for Instant {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct Duration {
    ticks: PlatformTicks,
}

impl Duration {
    pub const MIN: Self = Duration {
        ticks: PlatformTicks::MIN,
    };
    pub const MAX: Self = Duration {
        ticks: PlatformTicks::MAX,
    };

    pub const fn from_ticks(ticks: PlatformTicks) -> Self {
        Self { ticks }
    }

    pub fn from_nanos(nanos: PlatformTemporal) -> Self {
        Self::try_from_nanos(nanos).expect("overflow when creating duration from nanos")
    }
    pub fn from_millis(millis: PlatformTemporal) -> Self {
        Self::try_from_millis(millis).expect("overflow when creating duration from millis")
    }
    pub fn from_micros(micros: PlatformTemporal) -> Self {
        Self::try_from_micros(micros).expect("overflow when creating duration from micros")
    }
    pub fn from_secs(seconds: PlatformTemporal) -> Self {
        Self::try_from_secs(seconds).expect("overflow when creating duration from seconds")
    }

    pub fn from_millis_floor(millis: PlatformTemporal) -> Self {
        Self::try_from_millis_floor(millis).expect("overflow when creating duration from millis")
    }
    pub fn from_micros_floor(micros: PlatformTemporal) -> Self {
        Self::try_from_micros_floor(micros).expect("overflow when creating duration from micros")
    }
    pub fn from_secs_floor(seconds: PlatformTemporal) -> Self {
        Self::try_from_secs_floor(seconds).expect("overflow when creating duration from seconds")
    }

    pub fn try_from_nanos(nanos: PlatformTemporal) -> Option<Self> {
        if let Some(value) = (nanos as PlatformTicks).checked_mul(PLATFORM_TICK_HZ / GCD_1G) {
            Some(Duration {
                ticks: u64::div_ceil(value, 1_000_000_000 / GCD_1G),
            })
        } else {
            None
        }
    }

    pub fn try_from_micros(micros: PlatformTemporal) -> Option<Self> {
        if let Some(value) = (micros as PlatformTicks).checked_mul(PLATFORM_TICK_HZ / GCD_1M) {
            Some(Duration {
                ticks: u64::div_ceil(value, 1_000_000 / GCD_1M),
            })
        } else {
            None
        }
    }

    pub fn try_from_millis(millis: PlatformTemporal) -> Option<Self> {
        if let Some(value) = (millis as PlatformTicks).checked_mul(PLATFORM_TICK_HZ / GCD_1K) {
            Some(Duration {
                ticks: u64::div_ceil(value, 1000 / GCD_1K),
            })
        } else {
            None
        }
    }

    pub fn try_from_secs(seconds: PlatformTemporal) -> Option<Self> {
        if let Some(ticks) = (seconds as PlatformTicks).checked_mul(PLATFORM_TICK_HZ) {
            Some(Duration { ticks })
        } else {
            None
        }
    }

    pub fn try_from_micros_floor(micros: PlatformTemporal) -> Option<Self> {
        if let Some(value) = (micros as PlatformTicks).checked_mul(PLATFORM_TICK_HZ / GCD_1M) {
            Some(Duration {
                ticks: value / (1_000_000 / GCD_1M),
            })
        } else {
            None
        }
    }

    pub fn try_from_millis_floor(millis: PlatformTemporal) -> Option<Self> {
        if let Some(value) = (millis as PlatformTicks).checked_mul(PLATFORM_TICK_HZ / GCD_1K) {
            Some(Duration {
                ticks: value / (1000 / GCD_1K),
            })
        } else {
            None
        }
    }

    pub fn try_from_secs_floor(seconds: PlatformTemporal) -> Option<Self> {
        if let Some(ticks) = (seconds as PlatformTicks).checked_mul(PLATFORM_TICK_HZ) {
            Some(Duration { ticks })
        } else {
            None
        }
    }

    pub const fn as_ticks(&self) -> PlatformTicks {
        self.ticks
    }

    pub fn as_nanos(&self) -> PlatformTemporal {
        (self.ticks * (1_000_000_000 / GCD_1G) / (PLATFORM_TICK_HZ / GCD_1G)) as PlatformTemporal
    }

    pub fn as_micros(&self) -> PlatformTemporal {
        (self.ticks * (1_000_000 / GCD_1M) / (PLATFORM_TICK_HZ / GCD_1M)) as PlatformTemporal
    }

    pub fn as_millis(&self) -> PlatformTemporal {
        (self.ticks * (1000 / GCD_1K) / (PLATFORM_TICK_HZ / GCD_1K)) as PlatformTemporal
    }

    pub fn as_secs(&self) -> PlatformTemporal {
        (self.ticks / PLATFORM_TICK_HZ) as PlatformTemporal
    }

    pub const fn checked_add(&self, rhs: Self) -> Option<Self> {
        if let Some(ticks) = self.ticks.checked_add(rhs.ticks) {
            Some(Self { ticks })
        } else {
            None
        }
    }

    pub const fn checked_sub(&self, rhs: Self) -> Option<Self> {
        if let Some(ticks) = self.ticks.checked_sub(rhs.ticks) {
            Some(Self { ticks })
        } else {
            None
        }
    }

    pub const fn checked_mul(&self, rhs: u32) -> Option<Self> {
        if let Some(ticks) = self.ticks.checked_mul(rhs as PlatformTicks) {
            Some(Self { ticks })
        } else {
            None
        }
    }

    pub const fn checked_div(&self, rhs: u32) -> Option<Self> {
        if let Some(ticks) = self.ticks.checked_div(rhs as PlatformTicks) {
            Some(Self { ticks })
        } else {
            None
        }
    }
}

impl Add<Duration> for Duration {
    type Output = Duration;

    fn add(self, rhs: Duration) -> Self::Output {
        self.checked_add(rhs)
            .expect("overflow when adding durations")
    }
}

impl Sub<Self> for Duration {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        self.checked_sub(rhs)
            .expect("underflow when subtracting durations")
    }
}

impl Mul<u32> for Duration {
    type Output = Duration;

    fn mul(self, rhs: u32) -> Self::Output {
        self.checked_mul(rhs)
            .expect("overflow when multiplying duration by integer")
    }
}

impl Div<u32> for Duration {
    type Output = Duration;

    fn div(self, rhs: u32) -> Self::Output {
        self.checked_div(rhs).expect("division of duration by zero")
    }
}

impl AddAssign<Duration> for Duration {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl SubAssign<Duration> for Duration {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}

impl MulAssign<u32> for Duration {
    fn mul_assign(&mut self, rhs: u32) {
        *self = *self * rhs;
    }
}

impl DivAssign<u32> for Duration {
    fn div_assign(&mut self, rhs: u32) {
        *self = *self / rhs;
    }
}
