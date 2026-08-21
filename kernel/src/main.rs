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
use crate::arch::{traits::TargetPlatform, Platform, PlatformInstant, TimerQueue};
use crate::boot::BootContext;
use crate::threading::init::setup_threads;
use core::time::Duration;
use sync::Mutex;
use timers::{TimerCallback, TimerCallbackContext};

fn setup_timers() {
    // simple repeating timer
    TimerQueue::add_repeating_timer(
        Duration::from_secs(1).into(),
        TimerCallback::immediate(|_| info!("----------------- 1 Second timer -----------------")),
    );
    // repeating timer with state
    TimerQueue::add_repeating_timer(
        Duration::from_secs(3).into(),
        TimerCallback::immediate(|_| {
            static COUNT: Mutex<u32> = Mutex::new(0);
            let mut count = COUNT.lock();
            *count += 1;
            info!(
                "----------------- 3 Second stateful timer {} -----------------",
                count
            )
        }),
    );
    // one shot timer
    TimerQueue::add_oneshot_timer(
        Duration::from_secs(10).into(),
        TimerCallback::immediate(|_| info!("-------------------------- 10 Second timer")),
    );
    // Reschedule timer
    TimerQueue::add_repeating_timer(Duration::from_secs(1).into(), TimerCallback::Reschedule);
    // One shot repeating timer
    fn oneshot_repeating_callback(_: TimerCallbackContext) {
        info!("-------------------------- One shot repeating timer");
        TimerQueue::add_oneshot_timer(
            Duration::from_secs(2).into(),
            TimerCallback::immediate(oneshot_repeating_callback),
        );
    }
    TimerQueue::add_oneshot_timer(
        Duration::from_secs(2).into(),
        TimerCallback::immediate(oneshot_repeating_callback),
    );
    let outer_state = 5;
    TimerQueue::add_oneshot_timer(
        Duration::from_secs(4).into(),
        TimerCallback::immediate(move |_| {
            info!(
                "-------------------------- One shot timer with state {}",
                outer_state
            );
        }),
    );
}

pub fn kernel_main(_ctx: BootContext) -> ! {
    info!("Starting kernel (kernel_main())");

    setup_threads();
    setup_timers();
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
