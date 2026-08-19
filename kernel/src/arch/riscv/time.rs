use crate::arch::traits::TargetInstant;
use core::{
    fmt::Debug,
    ops::{Add, Sub},
    time::Duration,
};

pub type Tick = u64;
const TICKS_PER_MICROSECOND: Tick = 10;

// Special type for duration in ticks
#[derive(Debug)]
pub struct TickDuration {
    ticks: Tick,
}

impl TickDuration {
    pub fn new(ticks: Tick) -> Self {
        Self { ticks }
    }
}

impl From<Duration> for TickDuration {
    fn from(value: Duration) -> Self {
        Self {
            ticks: value.as_micros() as u64 * TICKS_PER_MICROSECOND,
        }
    }
}

impl Into<Duration> for TickDuration {
    fn into(self) -> Duration {
        Duration::from_micros(self.ticks / TICKS_PER_MICROSECOND)
    }
}

// Instant at home
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct TickInstant {
    time_ticks: Tick,
}

impl TargetInstant for TickInstant {
    fn now() -> Self {
        Self {
            time_ticks: riscv::register::time::read64(),
        }
    }
}

impl TickInstant {
    pub fn get_ticks(&self) -> Tick {
        self.time_ticks
    }
}

impl Sub<Self> for TickInstant {
    type Output = TickDuration;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.time_ticks > rhs.time_ticks {
            return TickDuration::new(self.time_ticks - rhs.time_ticks);
        } else {
            return TickDuration::new(0);
        }
    }
}

impl Sub<TickDuration> for TickInstant {
    type Output = TickInstant;

    fn sub(self, rhs: TickDuration) -> Self::Output {
        Self {
            time_ticks: self.time_ticks - rhs.ticks,
        }
    }
}

impl Add<TickDuration> for TickInstant {
    type Output = TickInstant;

    fn add(self, rhs: TickDuration) -> Self::Output {
        Self {
            time_ticks: self.time_ticks + rhs.ticks,
        }
    }
}
