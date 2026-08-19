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
pub mod vm;

use core::panic::PanicInfo;

use crate::arch::traits::TargetTimerQueue;
use crate::arch::{
    traits::TargetInstant, traits::TargetPlatform, traits::TargetTimerCallback, Platform,
    PlatformInstant, TimerQueue,
};
use crate::boot::BootContext;
use crate::threading::init::setup_threads;
use core::time::Duration;

fn setup_timers() {
    TimerQueue::add_repeating_timer(
        Duration::from_secs(1).into(),
        TargetTimerCallback::immediate(|_| {
            info!("----------------- 1 Second timer -----------------")
        }),
    );
    TimerQueue::add_repeating_timer(
        Duration::from_secs(3).into(),
        TargetTimerCallback::immediate(|_| {
            info!("----------------- 3 Second syncronized timer -----------------")
        }),
    );
    TimerQueue::add_oneshot_timer(
        Duration::from_secs(10).into(),
        TargetTimerCallback::immediate(|_| info!("-------------------------- 10 Second timer")),
    );
    TimerQueue::add_repeating_timer(
        Duration::from_secs(1).into(),
        TargetTimerCallback::Reschedule
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
