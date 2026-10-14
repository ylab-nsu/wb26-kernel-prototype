use core::sync::atomic::{AtomicBool, Ordering};

use crate::arch::traits::{TargetAlarm, TargetTimer};

pub type Ticks = u64;
pub type Temporal = u64;
pub const TICK_HZ: Ticks = 10_000_000;

pub struct Timer;

impl TargetTimer for Timer {
    type Ticks = Ticks;

    fn now() -> Self::Ticks {
        riscv::register::time::read64()
    }
}

pub struct Alarm(());

impl TargetAlarm for Alarm {
    type Ticks = Ticks;

    fn take() -> Option<Self> {
        static ALARM_TAKEN: AtomicBool = AtomicBool::new(false);
        if ALARM_TAKEN.swap(true, Ordering::Relaxed) {
            Some(Alarm(()))
        } else {
            None
        }
    }

    fn schedule(&mut self, ticks: Self::Ticks) -> Result<(), ()> {
        sbi::timer::set_timer(ticks).map_err(|_| ())
    }

    fn cancel(&mut self) -> Result<(), ()> {
        sbi::timer::set_timer(Self::Ticks::MAX).map_err(|_| ())
    }
}
