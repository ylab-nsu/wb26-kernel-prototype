use crate::arch::{PlatformDuration, PlatformInstant};
use crate::sync::{LazyLock, Mutex};
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt::Display;
use core::ops::{Add, AddAssign};
use core::sync::atomic::AtomicBool;
use core::time::Duration;

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

#[derive(Clone, Copy)]
pub struct LatencySample {
    pub seq: u64,
    pub t_ms: u64,       // time since bench start (ms) — for time-series plots
    pub latency_us: u64, // the measured value — for histograms
}

pub struct BenchLatency {
    samples: Vec<LatencySample>,
    total: Duration,
    count: u64,
    start: Option<PlatformInstant>,
}

impl BenchLatency {
    pub const fn new() -> Self {
        Self {
            samples: Vec::new(),
            total: Duration::from_micros(0),
            count: 0,
            start: None,
        }
    }

    /// Call from the ISR/callback. O(1) amortized (Vec push), no logging here.
    pub fn record(&mut self, latency: Duration, now: PlatformInstant) {
        if self.start.is_none() {
            self.start = Some(now);
        }
        let t_ms = Into::<Duration>::into(now.saturating_duration_since(self.start.unwrap()))
            .as_millis() as u64;

        self.samples.push(LatencySample {
            seq: self.count,
            t_ms,
            latency_us: latency.as_micros() as u64,
        });

        self.total += latency;
        self.count += 1;
    }

    pub fn average(&self) -> Duration {
        if self.count == 0 {
            Duration::from_micros(0)
        } else {
            self.total / self.count as u32
        }
    }

    pub fn max(&self) -> u64 {
        self.samples.iter().map(|s| s.latency_us).max().unwrap_or(0)
    }

    pub fn samples(&self) -> &[LatencySample] {
        &self.samples
    }

    /// Emit every sample as a CSV line over your logger, for a host-side
    /// script to capture from the serial console. Call this ONLY after
    /// the benchmark window has closed, not during.
    pub fn dump_csv(&self, label: &str) {
        for s in &self.samples {
            info!("BENCH_CSV,{},{},{},{}", label, s.seq, s.t_ms, s.latency_us);
        }
    }

    pub fn reset(&mut self) {
        self.samples.clear();
        self.total = Duration::from_micros(0);
        self.count = 0;
        self.start = None;
    }
}

impl Display for BenchLatency {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Latency: avg {} us, max {} us, across {} records;",
            self.average().as_micros(),
            self.max(),
            self.count,
        )
    }
}

pub static BENCH_STEADY: Mutex<BenchLatency> = Mutex::new(BenchLatency::new());
pub static BENCH_SPIKE: Mutex<BenchLatency> = Mutex::new(BenchLatency::new());

#[derive(Clone, Copy)]
pub struct RunSample {
    pub seq: u64,
    pub cycles: u64,
    pub time_us: u64,
}

pub struct BenchRun {
    samples: Vec<RunSample>,
    cycles: u64,
    time: Duration,
    count: u64,
}

impl BenchRun {
    pub const fn new() -> Self {
        Self {
            samples: Vec::new(),
            cycles: 0,
            time: Duration::from_micros(0),
            count: 0,
        }
    }

    pub fn record(&mut self, cycles: u64, time: Duration) {
        self.samples.push(RunSample {
            seq: self.count,
            cycles,
            time_us: time.as_micros() as u64,
        });
        self.cycles += cycles;
        self.time += time;
        self.count += 1;
    }

    pub fn average_cycles(&self) -> u64 {
        if self.count == 0 {
            0
        } else {
            self.cycles / self.count
        }
    }

    pub fn dump_csv(&self, label: &str) {
        for s in &self.samples {
            info!("BENCH_CSV,{},{},{},{}", label, s.seq, s.time_us, s.cycles);
        }
    }
}

impl AddAssign<BenchRun> for BenchRun {
    fn add_assign(&mut self, rhs: BenchRun) {
        self.samples.extend_from_slice(&rhs.samples);
        self.cycles += rhs.cycles;
        self.time += rhs.time;
        self.count += rhs.count;
    }
}

impl Display for BenchRun {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Cycles: {} (avg {}), Time: {} ms, across {} interrupts;",
            self.cycles,
            self.average_cycles(),
            self.time.as_millis(),
            self.count,
        )
    }
}

pub static BENCH_RUNS: Mutex<BenchRun> = Mutex::new(BenchRun::new());
