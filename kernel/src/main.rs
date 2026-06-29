#![no_std]
#![no_main]

#[macro_use]
extern crate alloc;

#[macro_use]
mod print;

pub mod allocator;
pub mod arch;
pub mod device_tree;
pub mod heap;
pub mod layout;
pub mod mmu;
pub mod paging;
pub mod sync;
pub mod thread;
pub mod vm;

use core::panic::PanicInfo;

use crate::arch::{traits::TargetPlatform, Platform};

pub fn kernel_main() -> ! {
    info!("Starting kernel (kernel_main())");

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
