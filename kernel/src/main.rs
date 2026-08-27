#![no_std]
#![no_main]
#![warn(unsafe_op_in_unsafe_fn)]

extern crate alloc;

#[macro_use]
mod print;

pub mod allocator;
pub mod arch;
pub mod boot;
pub mod drivers;
pub mod sync;
mod syscall;
pub mod threading;
pub mod timers;
pub mod vm;

use core::panic::PanicInfo;

use crate::arch::traits::TargetTimerQueue;
use crate::arch::{
    traits::{TargetInstant, TargetPlatform},
    Platform, PlatformInstant, TimerQueue,
};
use crate::boot::BootContext;
use crate::threading::init::setup_threads;
use core::time::Duration;
use sync::Mutex;
use timers::{TimerCallback, TimerCallbackContext, BENCH_RUNS, BENCH_SPIKE, BENCH_STEADY};

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

// Separate accumulators so burst impact on steady timers is visible
static BENCH_RUNNING: AtomicBool = AtomicBool::new(true);
static RNG_STATE: AtomicU64 = AtomicU64::new(0x9E3779B97F4A7C15); // any nonzero seed

fn setup_reschedule_timer() {
    TimerQueue::add_repeating_timer(Duration::from_secs(1).into(), TimerCallback::reschedule());

    const STEADY_POPULATION: usize = 50;
    const STEADY_MIN_DELAY_US: u64 = 500; // 0.5ms
    const STEADY_MAX_DELAY_US: u64 = 50_000; // 50ms
    const SPIKE_INTERVAL: Duration = Duration::from_millis(750);
    const SPIKE_SIZE: usize = 15; // extra timers landing together
    const BENCH_DURATION: Duration = Duration::from_secs(10);

    /// Small atomic xorshift64 — fine for jitter, not for anything crypto-adjacent.
    fn next_rand() -> u64 {
        let mut x = RNG_STATE.load(Ordering::Relaxed);
        loop {
            let mut y = x;
            y ^= y << 13;
            y ^= y >> 7;
            y ^= y << 17;
            match RNG_STATE.compare_exchange_weak(x, y, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => return y,
                Err(actual) => x = actual,
            }
        }
    }

    fn rand_range_us(min: u64, max: u64) -> u64 {
        min + (next_rand() % (max - min))
    }

    /// One steady-state timer: records its own fire latency, then re-arms
    /// itself at a new random deadline to keep the queue population stable.
    fn schedule_steady_timer() {
        let delay =
            Duration::from_micros(rand_range_us(STEADY_MIN_DELAY_US, STEADY_MAX_DELAY_US)).into();
        let scheduled_at = PlatformInstant::now();
        let deadline = scheduled_at + delay;

        let _ = TimerQueue::add_oneshot_timer(
            delay,
            TimerCallback::immediate(move |_| {
                let latency = PlatformInstant::now() - deadline;
                BENCH_STEADY.lock().record(latency.into());

                if BENCH_RUNNING.load(Ordering::Relaxed) {
                    schedule_steady_timer(); // re-arm from ISR context, as in real usage
                }
            }),
        );
    }

    /// A burst: N timers all targeting (approximately) the same deadline,
    /// to see how dispatch latency degrades under contention.
    fn schedule_spike_burst() {
        let deadline = PlatformInstant::now() + Duration::from_millis(200).into();
        let now = PlatformInstant::now();
        let delta = deadline - now;

        for _ in 0..SPIKE_SIZE {
            let _ = TimerQueue::add_oneshot_timer(
                delta.into(),
                TimerCallback::immediate(move |_| {
                    let latency = PlatformInstant::now() - deadline;
                    BENCH_SPIKE.lock().record(latency.into());
                }),
            );
        }
    }

    /// Recurring scheduler that fires the next spike burst until the bench window closes.
    fn schedule_next_spike() {
        let _ = TimerQueue::add_oneshot_timer(
            SPIKE_INTERVAL.into(),
            TimerCallback::immediate(move |_| {
                if BENCH_RUNNING.load(Ordering::Relaxed) {
                    schedule_spike_burst();
                    schedule_next_spike();
                }
            }),
        );
    }

    // --- Kick off the benchmark ---

    // Stop condition: flip the flag so in-flight callbacks stop re-arming,
    // then print both distributions.
    TimerQueue::add_oneshot_timer(
        BENCH_DURATION.into(),
        TimerCallback::immediate(|_| {
            BENCH_RUNNING.store(false, Ordering::Relaxed);

            let mut steady = BENCH_STEADY.lock();
            steady.average();
            info!(
                "-------- Steady-state latency (n~{}): {}",
                STEADY_POPULATION, steady
            );

            let mut spike = BENCH_SPIKE.lock();
            spike.average();
            info!(
                "-------- Burst latency ({} timers/burst): {}",
                SPIKE_SIZE, spike
            );

            let mut lock = BENCH_RUNS.lock();
            lock.average();
            info!("-------- Bench runs: {}", lock);
        }),
    );

    // Seed the steady population
    for _ in 0..STEADY_POPULATION {
        schedule_steady_timer();
    }

    // Kick off the recurring spike generator
    schedule_next_spike();
}

pub fn kernel_main(_ctx: BootContext) -> ! {
    info!("Starting kernel (kernel_main())");

    setup_threads();
    setup_reschedule_timer();
    unsafe {
        Platform::ei();
    }

    loop {
        Platform::wfi();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("Something went wrong.");
    error!("{}", info);
    error!("Shutting down...");
    riscv::asm::wfi();
    error!("After shutting down...");
    // sbi::system_reset::system_reset(ResetType::, ResetReason::SystemFailure).unwrap();

    loop {
        riscv::asm::wfi();
    }
}
