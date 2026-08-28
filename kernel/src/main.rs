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
pub mod exec;
pub mod sync;
mod syscall;
pub mod threading;
pub mod timers;
pub mod vm;

#[cfg(feature = "kernel-unit-tests")]
mod tests;

use core::panic::PanicInfo;

use crate::arch::traits::TargetTimerQueue;
use crate::arch::{traits::TargetPlatform, Platform, TimerQueue};
use crate::boot::BootContext;
use crate::threading::init::setup_threads;
use core::time::Duration;
use timers::TimerCallback;

fn setup_reschedule_timer() {
    TimerQueue::add_repeating_timer(Duration::from_secs(1).into(), TimerCallback::reschedule());
}

#[cfg(feature = "kernel-unit-tests")]
pub fn kernel_main(_ctx: BootContext) -> ! {
    info!("Starting kernel tests (kernel_main())");
    tests::run_kernel_tests();

    loop {}
}

#[cfg(not(feature = "kernel-unit-tests"))]
pub fn kernel_main(_ctx: BootContext) -> ! {
    info!("Starting kernel (kernel_main())");

    setup_threads();
    crate::exec::test::test_run_elf();
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
