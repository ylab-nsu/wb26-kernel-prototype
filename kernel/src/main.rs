#![no_std]
#![no_main]

extern crate alloc;

#[macro_use]
mod print;

pub mod allocator;
pub mod arch;
pub mod boot;
pub mod sync;
pub mod vm;
pub mod threading;

use core::panic::PanicInfo;

use crate::arch::{traits::TargetPlatform, Platform};
use crate::boot::BootContext;
use crate::threading::init::setup_threads;

pub fn kernel_main(_ctx: BootContext) -> ! {
    info!("Starting kernel (kernel_main())");

    setup_threads();
    unsafe {
        Platform::ei();
    }

    loop {
        Platform::wfi();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Something went wrong.");
    println!("{}", info);
    println!("Shutting down...");
    riscv::asm::wfi();
    println!("After shutting down...");
    // sbi::system_reset::system_reset(ResetType::, ResetReason::SystemFailure).unwrap();

    loop {
        riscv::asm::wfi();
    }
}
