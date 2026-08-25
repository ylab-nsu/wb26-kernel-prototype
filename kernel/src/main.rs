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
use crate::arch::{traits::{TargetPlatform, TargetInstant}, Platform, TimerQueue, PlatformInstant};
use crate::boot::BootContext;
use crate::threading::init::setup_threads;
use core::time::Duration;
use sync::Mutex;
use timers::{TimerCallback, TimerCallbackContext, BENCH_RUNS};

fn setup_reschedule_timer() {
    TimerQueue::add_repeating_timer(Duration::from_secs(1).into(), TimerCallback::reschedule());
    const BENCH_COUNT: u64 = 50;
    const BENCH_BASE: Duration = Duration::from_secs(1);

    // Print result once all bench timers have had a chance to fire
    TimerQueue::add_oneshot_timer(
        Duration::from_secs(10).into(),
        TimerCallback::immediate(|_| {
            let mut lock = BENCH_RUNS.lock();
            lock.average();
            info!("-------- Bench runs: {}", lock);
        }),
    );

    // Test: schedule N timers, each recording only its own fire-latency
    for i in 0..BENCH_COUNT {
        let requested_delay = BENCH_BASE + Duration::from_micros(i);
        let scheduled_at = PlatformInstant::now();
        let deadline = scheduled_at + requested_delay.into();

        let _ = TimerQueue::add_oneshot_timer(
            requested_delay.into(),
            TimerCallback::immediate(move |_| {
                let fired_at = PlatformInstant::now();
                let latency = fired_at - deadline;
                BENCH_RUNS.lock().record_lattency(latency.into());
            }),
        );
    }
    // TimerQueue::add_repeating_timer(
    //     Duration::from_secs(1).into(),
    //     TimerCallback::immediate(|_| info!("1 Second timer")),
    // );
    // // One shot timer
    // TimerQueue::add_oneshot_timer(
    //     Duration::from_secs(10).into(),
    //     TimerCallback::immediate(|_| info!("10 second oneshot timer")),
    // );
    // // Repeating timer with inner state
    // TimerQueue::add_repeating_timer(
    //     Duration::from_secs(3).into(),
    //     TimerCallback::immediate(|_| {
    //         static COUNT: Mutex<u32> = Mutex::new(0);
    //         let mut count = COUNT.lock();
    //         *count += 1;
    //         info!(
    //             "3 Second stateful timer {}",
    //             count
    //         )
    //     }),
    // );
    // // Reschedule timer
    // // TimerQueue::add_repeating_timer(Duration::from_secs(1).into(), TimerCallback::Reschedule);
    // // One shot repeating timer
    // fn oneshot_repeating_callback(_: TimerCallbackContext) {
    //     info!("One shot repeating timer");
    //     TimerQueue::add_oneshot_timer(
    //         Duration::from_secs(2).into(),
    //         TimerCallback::immediate(oneshot_repeating_callback),
    //     );
    // }
    // TimerQueue::add_oneshot_timer(
    //     Duration::from_secs(2).into(),
    //     TimerCallback::immediate(oneshot_repeating_callback),
    // );
    // // One shot timer with capture
    // let to_capture = 5;
    // TimerQueue::add_oneshot_timer(
    //     Duration::from_secs(4).into(),
    //     TimerCallback::immediate(move |_| {
    //         info!(
    //             "-------------------------- One shot timer with capture {}",
    //             to_capture
    //         );
    //     }),
    // );
    // // One shot timer with mutable capture
    // let mut to_capture_mutable = 10;
    // TimerQueue::add_repeating_timer(
    //     Duration::from_secs(4).into(),
    //     TimerCallback::immediate(move |_| {
    //         info!(
    //             "-------------------------- One shot timer with mut capture {}",
    //             to_capture_mutable
    //         );
    //         to_capture_mutable += 1;
    //     }),
    // );
    // // Stop timer with handle
    // let handle = TimerQueue::add_repeating_timer(Duration::from_secs(2).into(), TimerCallback::immediate(|_| {
    //     info!("1 second timer");
    // }));
    // TimerQueue::add_oneshot_timer(Duration::from_secs(5).into(), TimerCallback::immediate( move |_| {
    //     if let Some(handle) = handle.upgrade() {
    //         handle.stop();
    //         info!("timer stopped");
    //     } else {
    //         info!("timer already stopped");
    //     }
    // }));
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
