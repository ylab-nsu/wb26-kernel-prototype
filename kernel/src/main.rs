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

use crate::arch::{traits::TargetPlatform, traits::TargetTimerQueue, Platform, TimerQueue};
use crate::boot::BootContext;
use crate::threading::init::setup_threads;
use core::time::Duration;

fn setup_timers() {
    TimerQueue::add_timer(
        Duration::from_secs(1).into(),
        |_| println!("----------------- Second timer"),
        true,
    );
    TimerQueue::add_timer(
        Duration::from_secs(5).into(),
        |_| println!("5 TIMER ---------------------"),
        true,
    );
    TimerQueue::add_timer(
        Duration::from_secs(10).into(),
        |_| println!("-------------------------- 10 Second timer"),
        false,
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
