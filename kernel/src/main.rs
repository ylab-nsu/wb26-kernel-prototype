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
mod uart;
mod uart2;

use core::panic::PanicInfo;

use crate::arch::{traits::TargetPlatform, Platform};

use crate::boot::BootContext;
use crate::threading::init::setup_threads;
use crate::uart2::UART;

pub fn kernel_main(_ctx: BootContext) -> ! {
    info!("Starting kernel (kernel_main())");


	let mut uart = UART::new(0x10000000);
	uart.init();
	uart.enable_rx_interrupt();
	
	
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
