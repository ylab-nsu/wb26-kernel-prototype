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
mod drivers_;

use core::panic::PanicInfo;
use core::ptr::read;
use core::result;
use riscv::_export::critical_section;

use crate::arch::{traits::TargetPlatform, Platform};
use crate::drivers_::uart::UART;

use crate::boot::BootContext;
use crate::threading::init::setup_threads;

pub fn kernel_main(_ctx: BootContext) -> ! {
    info!("Starting kernel (kernel_main())");
	
    setup_threads();

	println!("=== im here===");
	critical_section::with(|cs|{
		let mut uart = UART.borrow(cs).borrow_mut();
		uart.init();
		uart.enable_rx_interrupt();
		uart.write(b"Hello world\n");
	});

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
