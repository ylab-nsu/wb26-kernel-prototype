use crate::arch::{PlatformDuration, PlatformInstant};
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::sync::atomic::AtomicBool;
use core::fmt::Display;
use core::ops::{Add, AddAssign};
use core::time::Duration;
use crate::sync::{Mutex, LazyLock};

pub struct TimerCallbackContext {
    pub handle_time: PlatformInstant,
    pub target_time: PlatformInstant,
}

pub trait TimerCallbackFunction: FnMut(TimerCallbackContext) + Send {}
impl<T: FnMut(TimerCallbackContext) + Send> TimerCallbackFunction for T {}
pub type TimerCallbackFunctionBoxed = Box<dyn TimerCallbackFunction>;

pub enum TimerCallback {
    Reschedule,
    // Inside interrupt
    Immediate {
        callback: TimerCallbackFunctionBoxed,
    },
    // TODO: Elsewhere
    Soft {
        callback: fn(),
    },
}

impl TimerCallback {
    pub fn immediate<T: TimerCallbackFunction + 'static>(callback: T) -> Self {
        TimerCallback::Immediate {
            callback: Box::new(callback),
        }
    }

    pub fn reschedule() -> Self {
        TimerCallback::Reschedule
    }

    pub fn soft(_: fn()) -> Self {
        todo!("implement timer soft callbacks");
    }
}

pub enum TimerKind {
    OneShot,
    Repeating { interval: PlatformDuration },
}

pub struct Timer {
    pub callback: TimerCallback,
    pub target_time: PlatformInstant,
    pub start_or_last_fire_time: PlatformInstant,
    pub kind: TimerKind,
    pub handle: Arc<TimerHandle>,
}

impl Ord for Timer {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        other.target_time.cmp(&self.target_time).then(
            other
                .start_or_last_fire_time
                .cmp(&self.start_or_last_fire_time),
        )
    }
}

impl PartialOrd for Timer {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Timer {
    fn eq(&self, other: &Self) -> bool {
        self.target_time == other.target_time
            && self.start_or_last_fire_time == other.start_or_last_fire_time
    }
}

impl Eq for Timer {}

pub struct TimerHandle {
    to_stop: AtomicBool,
}

impl TimerHandle {
    pub fn new() -> Self {
        TimerHandle {
            to_stop: AtomicBool::new(false),
        }
    }

    pub fn stop(&self) {
        self.to_stop
            .store(true, core::sync::atomic::Ordering::Release)
    }

    pub fn is_stoped(&self) -> bool {
        self.to_stop.load(core::sync::atomic::Ordering::Acquire)
    }
}

pub struct BenchRun {
    cycles: u64,
    time: Duration,
    count: u64,
    lattency: Duration,
    lattency_count: u64,
}

impl BenchRun {
    pub fn new(cycles: u64, time: Duration) -> Self {
        Self {
            cycles,
            time,
            count: 1,
            lattency: Duration::from_micros(0),
            lattency_count: 0,
        }
    }

    pub fn average(&mut self) -> BenchRun {
        BenchRun {
            cycles: self.cycles / self.count,
            time: self.time / self.count as u32,
            count: self.count,
            lattency: self.lattency / self.lattency_count as u32,
            lattency_count: self.lattency_count,
        }
    }

    pub fn record_lattency(&mut self, lattency: Duration) {
        self.lattency += lattency;
        self.lattency_count += 1;
    }
}

impl Display for BenchRun {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Cycles: {}, Time: {} ms, across {} interrupts; lattency {} ms across {} records",
            self.cycles,
            self.time.as_millis(),
            self.count,
            self.lattency.as_millis(),
            self.lattency_count
        )
    }
}

impl AddAssign<BenchRun> for BenchRun {

    fn add_assign(&mut self, rhs: BenchRun) {
        self.cycles += rhs.cycles;
        self.time += rhs.time;
        self.count += rhs.count;
    }
}

pub static BENCH_RUNS: Mutex<BenchRun> = Mutex::new(BenchRun {
    cycles: 0,
    time: Duration::from_micros(0),
    count: 0,
    lattency: Duration::from_micros(0),
    lattency_count: 0,
});