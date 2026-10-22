// Adapted from Embassy.
//
// Copyright (c) Embassy project contributors
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use crate::arch::traits::{TargetAlarm, TargetTimeConverter, TargetTimer};

pub type Ticks = u64;
pub type Temporal = u64;

const TICK_HZ: Ticks = 10_000_000;
const GCD_1K: Ticks = gcd(TICK_HZ, 1_000);
const GCD_1M: Ticks = gcd(TICK_HZ, 1_000_000);
const GCD_1G: Ticks = gcd(TICK_HZ, 1_000_000_000);

const fn gcd(a: Ticks, b: Ticks) -> Ticks {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

pub struct Timer(());

impl Timer {
    pub const fn get() -> &'static Self {
        static INSTANCE: Timer = Timer(());
        &INSTANCE
    }
}

impl TargetTimer for Timer {
    type Ticks = Ticks;

    fn now(&self) -> Self::Ticks {
        riscv::register::time::read64()
    }
}

pub struct TimeConverter(());

impl TimeConverter {
    pub const fn get() -> &'static Self {
        static INSTANCE: TimeConverter = TimeConverter(());
        &INSTANCE
    }
}

impl TargetTimeConverter for TimeConverter {
    type Ticks = Ticks;
    type Temporal = Temporal;

    fn try_from_nanos(&self, nanos: Self::Temporal) -> Option<Self::Ticks> {
        if let Some(value) = (nanos as Self::Ticks).checked_mul(TICK_HZ / GCD_1G) {
            Some(value.div_ceil(1_000_000_000 / GCD_1G))
        } else {
            None
        }
    }

    fn try_from_micros(&self, micros: Self::Temporal) -> Option<Self::Ticks> {
        if let Some(value) = (micros as Self::Ticks).checked_mul(TICK_HZ / GCD_1M) {
            Some(value.div_ceil(1_000_000 / GCD_1M))
        } else {
            None
        }
    }

    fn try_from_millis(&self, millis: Self::Temporal) -> Option<Self::Ticks> {
        if let Some(value) = (millis as Self::Ticks).checked_mul(TICK_HZ / GCD_1K) {
            Some(value.div_ceil(1_000 / GCD_1K))
        } else {
            None
        }
    }

    fn try_from_secs(&self, secs: Self::Temporal) -> Option<Self::Ticks> {
        if let Some(value) = (secs as Self::Ticks).checked_mul(TICK_HZ) {
            Some(value)
        } else {
            None
        }
    }

    fn try_from_nanos_floor(&self, nanos: Self::Temporal) -> Option<Self::Ticks> {
        if let Some(value) = (nanos as Self::Ticks).checked_mul(TICK_HZ / GCD_1G) {
            Some(value / (1_000_000_000 / GCD_1G))
        } else {
            None
        }
    }

    fn try_from_micros_floor(&self, micros: Self::Temporal) -> Option<Self::Ticks> {
        if let Some(value) = (micros as Self::Ticks).checked_mul(TICK_HZ / GCD_1M) {
            Some(value / (1_000_000 / GCD_1M))
        } else {
            None
        }
    }

    fn try_from_millis_floor(&self, millis: Self::Temporal) -> Option<Self::Ticks> {
        if let Some(value) = (millis as Self::Ticks).checked_mul(TICK_HZ / GCD_1K) {
            Some(value / (1_000 / GCD_1K))
        } else {
            None
        }
    }

    fn try_from_secs_floor(&self, secs: Self::Temporal) -> Option<Self::Ticks> {
        if let Some(value) = (secs as Self::Ticks).checked_mul(TICK_HZ) {
            Some(value)
        } else {
            None
        }
    }

    fn to_nanos(&self, ticks: Self::Ticks) -> Self::Temporal {
        (ticks * (1_000_000_000 / GCD_1G) / (TICK_HZ / GCD_1G)) as Self::Temporal
    }

    fn to_micros(&self, ticks: Self::Ticks) -> Self::Temporal {
        (ticks * (1_000_000 / GCD_1M) / (TICK_HZ / GCD_1M)) as Self::Temporal
    }

    fn to_millis(&self, ticks: Self::Ticks) -> Self::Temporal {
        (ticks * (1_000 / GCD_1K) / (TICK_HZ / GCD_1K)) as Self::Temporal
    }

    fn to_secs(&self, ticks: Self::Ticks) -> Self::Temporal {
        (ticks / TICK_HZ) as Self::Temporal
    }
}

pub struct Alarm(());

impl Alarm {
    pub const fn get() -> &'static Self {
        static INSTANCE: Alarm = Alarm(());
        &INSTANCE
    }
}

impl TargetAlarm for Alarm {
    type Ticks = Ticks;

    fn set_next(&self, ticks: Self::Ticks) -> Result<(), ()> {
        sbi::timer::set_timer(ticks).map_err(|_| ())
    }
}
