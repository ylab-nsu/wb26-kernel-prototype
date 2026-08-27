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
pub mod pci;
pub mod sdhci;

#[cfg(feature = "kernel-tests")]
mod tests;

use core::panic::PanicInfo;

use crate::arch::{traits::TargetPlatform, Platform};
use crate::boot::BootContext;
use crate::threading::init::setup_threads;

pub fn kernel_main(_ctx: BootContext) -> ! {
    cfg_if::cfg_if! {
        if #[cfg(feature = "kernel-tests")] {
            info!("Starting kernel tests (kernel_main())");
            tests::run_kernel_tests();

            loop { }
        } else {
            info!("Starting kernel (kernel_main())");

            setup_threads();
            unsafe {
                Platform::ei();
            }

            loop {
                Platform::wfi();
            }
        }
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
